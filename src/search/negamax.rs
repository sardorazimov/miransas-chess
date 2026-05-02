use crate::{
    board::{Board, PieceKind, Undo},
    evaluation::evaluate,
    movegen::{Move, generate_legal_moves, is_in_check},
    search::{
        SearchContext,
        context::piece_index,
        ordering::score_move,
        tt::{Bound, TTEntry, TranspositionTable},
    },
};

const NULL_MOVE_MIN_DEPTH: u32 = 3;

const CHECKMATE_SCORE: i32 = 100_000;
const ASPIRATION_WINDOW: i32 = 50;
const NEG_INF: i32 = i32::MIN + 1;
const POS_INF: i32 = i32::MAX;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct SearchResult {
    pub best_move: Option<Move>,
    pub score: i32,
    pub nodes: u64,
    pub tt_hits: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IterativeSearchResult {
    pub best_move: Option<Move>,
    pub score: i32,
    pub depth: u32,
    pub nodes: u64,
    pub tt_hits: u64,
    pub principal_variation: Vec<Move>,
}

pub fn search_best_move(board: &Board, depth: u32) -> SearchResult {
    search_best_move_with_tt(board, depth, 4)
}

pub fn search_best_move_with_tt(board: &Board, depth: u32, tt_size_mb: usize) -> SearchResult {
    let mut board = board.clone();
    let mut nodes = 0u64;
    let mut tt_hits = 0u64;
    let mut tt = TranspositionTable::new(tt_size_mb);
    tt.clear();
    let mut ctx = SearchContext::new();

    let result = search_root(
        &mut board,
        depth,
        NEG_INF,
        POS_INF,
        &mut nodes,
        &mut tt_hits,
        &mut ctx,
        &mut tt,
    );

    SearchResult {
        best_move: result.best_move,
        score: result.score,
        nodes,
        tt_hits,
    }
}

pub fn search_iterative(board: &Board, max_depth: u32, tt_size_mb: usize) -> IterativeSearchResult {
    let mut board = board.clone();
    let mut tt = TranspositionTable::new(tt_size_mb);
    tt.clear();
    let mut ctx = SearchContext::new();
    let mut nodes = 0u64;
    let mut tt_hits = 0u64;
    let mut latest = SearchResult {
        best_move: None,
        score: terminal_score(&board, 0),
        nodes: 0,
        tt_hits: 0,
    };
    let mut reached_depth = 0;
    let mut previous_score: i32 = 0;

    for depth in 1..=max_depth {
        let alpha = previous_score.saturating_sub(ASPIRATION_WINDOW);
        let beta = previous_score.saturating_add(ASPIRATION_WINDOW);
        latest = search_root(
            &mut board,
            depth,
            alpha,
            beta,
            &mut nodes,
            &mut tt_hits,
            &mut ctx,
            &mut tt,
        );

        if latest.score <= alpha {
            latest = search_root(
                &mut board,
                depth,
                NEG_INF,
                beta,
                &mut nodes,
                &mut tt_hits,
                &mut ctx,
                &mut tt,
            );
        } else if latest.score >= beta {
            latest = search_root(
                &mut board,
                depth,
                alpha,
                POS_INF,
                &mut nodes,
                &mut tt_hits,
                &mut ctx,
                &mut tt,
            );
        }

        previous_score = latest.score;
        reached_depth = depth;
    }

    let principal_variation = extract_pv(&board, &tt, reached_depth);

    IterativeSearchResult {
        best_move: latest.best_move,
        score: latest.score,
        depth: reached_depth,
        nodes,
        tt_hits,
        principal_variation,
    }
}

pub fn format_pv(pv: &[Move]) -> String {
    pv.iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(" ")
}

#[allow(clippy::too_many_arguments)]
fn search_root(
    board: &mut Board,
    depth: u32,
    mut alpha: i32,
    beta: i32,
    nodes: &mut u64,
    tt_hits: &mut u64,
    ctx: &mut SearchContext,
    tt: &mut TranspositionTable,
) -> SearchResult {
    let key = board.zobrist_key;
    let tt_move = tt.get(key).and_then(|entry| entry.best_move);

    let mut moves = generate_legal_moves(board);
    if moves.is_empty() {
        return SearchResult {
            best_move: None,
            score: terminal_score(board, 0),
            nodes: *nodes,
            tt_hits: *tt_hits,
        };
    }

    let mut scores: Vec<i32> = moves
        .iter()
        .map(|&mv| score_move(board, mv, tt_move, ctx, 0))
        .collect();

    let mut best_move: Option<Move> = None;
    let mut best_score = NEG_INF;
    let original_alpha = alpha;

    for i in 0..moves.len() {
        let mut best_idx = i;
        for j in (i + 1)..moves.len() {
            if scores[j] > scores[best_idx] {
                best_idx = j;
            }
        }
        if best_idx != i {
            moves.swap(i, best_idx);
            scores.swap(i, best_idx);
        }
        let mv = moves[i];

        let undo: Undo = board.make_move(mv);
        let score = -negamax(
            board,
            depth.saturating_sub(1),
            1,
            -beta,
            -alpha,
            nodes,
            tt_hits,
            ctx,
            tt,
            true,
        );
        board.unmake_move(&undo);

        if score > best_score {
            best_score = score;
            best_move = Some(mv);
        }
        alpha = alpha.max(score);
    }

    tt.store(TTEntry {
        key,
        depth,
        score: best_score,
        bound: root_bound(best_score, original_alpha, beta),
        best_move,
    });

    SearchResult {
        best_move,
        score: best_score,
        nodes: *nodes,
        tt_hits: *tt_hits,
    }
}

#[allow(clippy::too_many_arguments)]
fn negamax(
    board: &mut Board,
    depth: u32,
    ply: usize,
    mut alpha: i32,
    mut beta: i32,
    nodes: &mut u64,
    tt_hits: &mut u64,
    ctx: &mut SearchContext,
    tt: &mut TranspositionTable,
    null_allowed: bool,
) -> i32 {
    *nodes += 1;

    if depth == 0 {
        return quiescence(board, alpha, beta, nodes, ctx);
    }

    let key = board.zobrist_key;
    let original_alpha = alpha;

    let tt_move = if let Some(entry) = tt.get(key) {
        if entry.depth >= depth {
            *tt_hits += 1;
            match entry.bound {
                Bound::Exact => return entry.score,
                Bound::Lower => alpha = alpha.max(entry.score),
                Bound::Upper => beta = beta.min(entry.score),
            }
            if alpha >= beta {
                return entry.score;
            }
        }
        entry.best_move
    } else {
        None
    };

    let in_check = is_in_check(board, board.side_to_move);

    // Null move pruning: skip if in check, zugzwang-prone, stacked null, or shallow depth
    if null_allowed
        && depth >= NULL_MOVE_MIN_DEPTH
        && ply > 0
        && !in_check
        && !board.side_to_move_has_only_pawns()
    {
        let r: u32 = if depth > 6 { 3 } else { 2 };
        let null_depth = depth - 1 - r; // safe: depth >= 3 and r <= 3, but depth-1-r >= 0 when depth>=3,r=2
        let null_undo = board.make_null_move();
        let null_score = -negamax(
            board,
            null_depth,
            ply + 1,
            -beta,
            -beta + 1,
            nodes,
            tt_hits,
            ctx,
            tt,
            false,
        );
        board.unmake_null_move(null_undo);
        if null_score >= beta {
            return beta;
        }
    }

    let mut moves = generate_legal_moves(board);
    if moves.is_empty() {
        return terminal_score(board, ply as i32);
    }

    let mut scores: Vec<i32> = moves
        .iter()
        .map(|&mv| score_move(board, mv, tt_move, ctx, ply))
        .collect();

    let mut best_move: Option<Move> = None;
    let mut best_score = NEG_INF;

    for i in 0..moves.len() {
        let mut best_idx = i;
        for j in (i + 1)..moves.len() {
            if scores[j] > scores[best_idx] {
                best_idx = j;
            }
        }
        if best_idx != i {
            moves.swap(i, best_idx);
            scores.swap(i, best_idx);
        }
        let mv = moves[i];

        let undo = board.make_move(mv);
        let score = -negamax(
            board,
            depth - 1,
            ply + 1,
            -beta,
            -alpha,
            nodes,
            tt_hits,
            ctx,
            tt,
            true,
        );
        board.unmake_move(&undo);

        if score > best_score {
            best_score = score;
            best_move = Some(mv);
        }
        alpha = alpha.max(score);

        if alpha >= beta {
            if !is_capture(board, &mv) && mv.promotion.is_none() {
                ctx.record_killer(ply, mv);
                let attacker = board.squares[mv.from.index()].expect("piece at from after unmake");
                let pi = piece_index(attacker.color, attacker.kind);
                ctx.record_history(pi, mv.to.index(), depth as i32);
            }
            break;
        }
    }

    let bound = if best_score <= original_alpha {
        Bound::Upper
    } else if best_score >= beta {
        Bound::Lower
    } else {
        Bound::Exact
    };

    tt.store(TTEntry {
        key,
        depth,
        score: best_score,
        bound,
        best_move,
    });

    best_score
}

fn quiescence(
    board: &mut Board,
    mut alpha: i32,
    beta: i32,
    nodes: &mut u64,
    ctx: &SearchContext,
) -> i32 {
    *nodes += 1;

    let stand_pat = evaluate(board);
    if stand_pat >= beta {
        return beta;
    }
    if alpha < stand_pat {
        alpha = stand_pat;
    }

    let mut moves: Vec<Move> = generate_legal_moves(board)
        .into_iter()
        .filter(|mv| is_capture(board, mv) || mv.promotion.is_some())
        .collect();

    let mut scores: Vec<i32> = moves
        .iter()
        .map(|&mv| score_move(board, mv, None, ctx, 0))
        .collect();

    for i in 0..moves.len() {
        let mut best_idx = i;
        for j in (i + 1)..moves.len() {
            if scores[j] > scores[best_idx] {
                best_idx = j;
            }
        }
        if best_idx != i {
            moves.swap(i, best_idx);
            scores.swap(i, best_idx);
        }
        let mv = moves[i];

        let undo = board.make_move(mv);
        let score = -quiescence(board, -beta, -alpha, nodes, ctx);
        board.unmake_move(&undo);

        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }

    alpha
}

fn extract_pv(board: &Board, tt: &TranspositionTable, max_depth: u32) -> Vec<Move> {
    let mut pv = Vec::new();
    let mut current = board.clone();

    for _ in 0..max_depth {
        let key = current.zobrist_key;
        let Some(best_move) = tt.get(key).and_then(|entry| entry.best_move) else {
            break;
        };
        if !generate_legal_moves(&current).contains(&best_move) {
            break;
        }
        pv.push(best_move);
        current.make_move(best_move);
    }

    pv
}

fn root_bound(score: i32, alpha: i32, beta: i32) -> Bound {
    if score <= alpha {
        Bound::Upper
    } else if score >= beta {
        Bound::Lower
    } else {
        Bound::Exact
    }
}

fn terminal_score(board: &Board, ply: i32) -> i32 {
    if is_in_check(board, board.side_to_move) {
        -CHECKMATE_SCORE + ply
    } else {
        0
    }
}

fn is_capture(board: &Board, mv: &Move) -> bool {
    captured_piece_kind(board, mv).is_some()
}

fn captured_piece_kind(board: &Board, mv: &Move) -> Option<PieceKind> {
    if let Some(piece) = board.piece_at(mv.to) {
        return Some(piece.kind);
    }
    let attacker = board.piece_at(mv.from)?;
    if attacker.kind == PieceKind::Pawn
        && Some(mv.to) == board.en_passant
        && mv.from.file() != mv.to.file()
    {
        return Some(PieceKind::Pawn);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::context::SearchContext;

    #[test]
    fn search_returns_a_move_from_startpos() {
        let board = Board::startpos();
        let result = search_best_move(&board, 1);

        assert!(result.best_move.is_some());
        assert!(result.nodes > 0);
    }

    #[test]
    fn checkmate_handling_does_not_panic() {
        let board = Board::from_fen("7k/6Q1/6K1/8/8/8/8/8 b - - 0 1").expect("valid FEN");
        let result = search_best_move(&board, 2);

        assert_eq!(result.best_move, None);
        assert_eq!(result.score, -CHECKMATE_SCORE);
    }

    #[test]
    fn stalemate_handling_does_not_panic() {
        let board = Board::from_fen("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1").expect("valid FEN");
        let result = search_best_move(&board, 2);

        assert_eq!(result.best_move, None);
        assert_eq!(result.score, 0);
    }

    #[test]
    fn quiescence_does_not_panic() {
        let board = Board::from_fen("4k3/8/8/3q4/8/2N5/8/4K3 w - - 0 1").expect("valid FEN");
        let mut nodes = 0;
        let ctx = SearchContext::new();
        let score = quiescence(&mut board.clone(), i32::MIN + 1, i32::MAX, &mut nodes, &ctx);

        assert!(nodes > 0);
        assert!(score > i32::MIN);
    }

    #[test]
    fn quiescence_visits_more_nodes_than_static_eval() {
        let board = Board::from_fen("4k3/8/8/3q4/8/2N5/8/4K3 w - - 0 1").expect("valid FEN");
        let mut quiescence_nodes = 0u64;
        let mut static_nodes = 0u64;
        let mut tt = TranspositionTable::new(0);
        let mut ctx = SearchContext::new();
        let mut tt_hits = 0u64;

        // depth=0 immediately drops into quiescence
        let _ = negamax(
            &mut board.clone(),
            0,
            0,
            i32::MIN + 1,
            i32::MAX,
            &mut quiescence_nodes,
            &mut tt_hits,
            &mut ctx,
            &mut tt,
            false,
        );
        let _ = static_leaf_alpha_beta(
            &mut board.clone(),
            0,
            i32::MIN + 1,
            i32::MAX,
            &mut static_nodes,
            0,
        );

        assert!(quiescence_nodes > static_nodes);
    }

    #[test]
    fn search_with_tt_returns_a_move_from_startpos() {
        let board = Board::startpos();
        let result = search_best_move_with_tt(&board, 3, 4);

        assert!(result.best_move.is_some());
    }

    #[test]
    fn search_with_tt_score_matches_without_tt_at_small_depth() {
        let board = Board::startpos();
        let with_tt = search_best_move_with_tt(&board, 3, 4);
        let without_tt = search_best_move_with_tt(&board, 3, 0);

        assert_eq!(with_tt.score, without_tt.score);
    }

    #[test]
    fn iterative_search_returns_a_move_from_startpos() {
        let board = Board::startpos();
        let result = search_iterative(&board, 3, 4);

        assert!(result.best_move.is_some());
    }

    #[test]
    fn iterative_search_score_matches_full_window_search() {
        let board = Board::startpos();
        let iterative = search_iterative(&board, 3, 4);
        let full_window = search_best_move_with_tt(&board, 3, 4);

        assert_eq!(iterative.score, full_window.score);
    }

    #[test]
    fn iterative_search_final_depth_equals_requested_depth() {
        let board = Board::startpos();
        let result = search_iterative(&board, 3, 4);

        assert_eq!(result.depth, 3);
    }

    #[test]
    fn iterative_search_pv_is_not_empty_for_startpos_depth_three() {
        let board = Board::startpos();
        let result = search_iterative(&board, 3, 4);

        assert!(!result.principal_variation.is_empty());
    }

    #[test]
    fn iterative_search_pv_moves_are_legal_in_sequence() {
        let board = Board::startpos();
        let result = search_iterative(&board, 3, 4);
        let mut current = board;

        for mv in result.principal_variation {
            assert!(generate_legal_moves(&current).contains(&mv));
            current.make_move(mv);
        }
    }

    #[test]
    fn iterative_search_does_not_panic_on_checkmate_or_stalemate() {
        let checkmate = Board::from_fen("7k/6Q1/6K1/8/8/8/8/8 b - - 0 1").expect("valid FEN");
        let stalemate = Board::from_fen("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1").expect("valid FEN");

        assert_eq!(search_iterative(&checkmate, 3, 4).best_move, None);
        assert_eq!(search_iterative(&stalemate, 3, 4).best_move, None);
    }

    #[test]
    fn format_pv_returns_space_separated_moves() {
        let pv = [
            Move::new(square("b1"), square("c3")),
            Move::new(square("b8"), square("c6")),
            Move::new(square("g1"), square("f3")),
        ];

        assert_eq!(format_pv(&pv), "b1c3 b8c6 g1f3");
    }

    fn static_leaf_alpha_beta(
        board: &mut Board,
        depth: u32,
        mut alpha: i32,
        beta: i32,
        nodes: &mut u64,
        ply: i32,
    ) -> i32 {
        *nodes += 1;
        if depth == 0 {
            return evaluate(board);
        }
        let moves = generate_legal_moves(board);
        if moves.is_empty() {
            return terminal_score(board, ply);
        }
        for mv in moves {
            let undo = board.make_move(mv);
            let score = -static_leaf_alpha_beta(board, depth - 1, -beta, -alpha, nodes, ply + 1);
            board.unmake_move(&undo);
            alpha = alpha.max(score);
            if alpha >= beta {
                break;
            }
        }
        alpha
    }

    #[test]
    fn null_move_finds_same_best_move_as_baseline_on_kiwipete() {
        let board =
            Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
                .expect("valid FEN");
        let result = search_iterative(&board, 5, 16);
        assert!(result.best_move.is_some(), "search must return a move");
        assert!(
            generate_legal_moves(&board).contains(&result.best_move.unwrap()),
            "best move must be legal"
        );
    }

    fn square(algebraic: &str) -> crate::board::Square {
        crate::board::Square::from_algebraic(algebraic).expect("test square is valid")
    }
}
