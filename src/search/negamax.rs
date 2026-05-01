use crate::{
    board::{Board, Color, PieceKind, Undo},
    evaluation::evaluate,
    movegen::{Move, generate_legal_moves, is_in_check},
    search::tt::{Bound, TTEntry, TranspositionTable},
};

const CHECKMATE_SCORE: i32 = 100_000;
const ASPIRATION_WINDOW: i32 = 50;
const NEG_INF: i32 = i32::MIN + 1;
const POS_INF: i32 = i32::MAX;
const MAX_DEPTH: usize = 64;

pub struct KillerMoves {
    moves: [[Option<Move>; 2]; MAX_DEPTH],
}

impl KillerMoves {
    pub const fn new() -> Self {
        Self {
            moves: [[None; 2]; MAX_DEPTH],
        }
    }

    fn store(&mut self, ply: usize, mv: Move) {
        if ply >= MAX_DEPTH || self.moves[ply][0] == Some(mv) {
            return;
        }

        self.moves[ply][1] = self.moves[ply][0];
        self.moves[ply][0] = Some(mv);
    }

    fn is_killer(&self, ply: usize, mv: Move) -> bool {
        ply < MAX_DEPTH && self.moves[ply].contains(&Some(mv))
    }
}

impl Default for KillerMoves {
    fn default() -> Self {
        Self::new()
    }
}

pub struct HistoryHeuristic {
    table: [[[i32; 64]; 64]; 2],
}

impl HistoryHeuristic {
    pub const fn new() -> Self {
        Self {
            table: [[[0; 64]; 64]; 2],
        }
    }

    fn add(&mut self, color: Color, mv: Move, depth: u32) {
        let bonus = (depth * depth) as i32;
        self.table[color_index(color)][mv.from.index()][mv.to.index()] += bonus;
    }

    fn score(&self, color: Color, mv: Move) -> i32 {
        self.table[color_index(color)][mv.from.index()][mv.to.index()]
    }
}

impl Default for HistoryHeuristic {
    fn default() -> Self {
        Self::new()
    }
}

struct SearchContext<'a> {
    tt: &'a mut TranspositionTable,
    killer_moves: KillerMoves,
    history: HistoryHeuristic,
    tt_hits: u64,
}

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
    let mut nodes = 0;
    let mut tt = TranspositionTable::new(tt_size_mb);
    tt.clear();
    let mut context = SearchContext {
        tt: &mut tt,
        killer_moves: KillerMoves::new(),
        history: HistoryHeuristic::new(),
        tt_hits: 0,
    };
    let result = search_root(
        &mut board,
        depth,
        NEG_INF,
        POS_INF,
        &mut nodes,
        &mut context,
    );

    SearchResult {
        best_move: result.best_move,
        score: result.score,
        nodes,
        tt_hits: context.tt_hits,
    }
}

pub fn search_iterative(board: &Board, max_depth: u32, tt_size_mb: usize) -> IterativeSearchResult {
    let mut board = board.clone();
    let mut tt = TranspositionTable::new(tt_size_mb);
    tt.clear();
    let mut context = SearchContext {
        tt: &mut tt,
        killer_moves: KillerMoves::new(),
        history: HistoryHeuristic::new(),
        tt_hits: 0,
    };
    let mut nodes = 0;
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
        latest = search_root(&mut board, depth, alpha, beta, &mut nodes, &mut context);

        if latest.score <= alpha {
            latest = search_root(&mut board, depth, NEG_INF, beta, &mut nodes, &mut context);
        } else if latest.score >= beta {
            latest = search_root(&mut board, depth, alpha, POS_INF, &mut nodes, &mut context);
        }

        previous_score = latest.score;
        reached_depth = depth;
    }

    let principal_variation = extract_pv(&board, context.tt, reached_depth);

    IterativeSearchResult {
        best_move: latest.best_move,
        score: latest.score,
        depth: reached_depth,
        nodes,
        tt_hits: context.tt_hits,
        principal_variation,
    }
}

pub fn format_pv(pv: &[Move]) -> String {
    pv.iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(" ")
}

fn search_root(
    board: &mut Board,
    depth: u32,
    mut alpha: i32,
    beta: i32,
    nodes: &mut u64,
    context: &mut SearchContext<'_>,
) -> SearchResult {
    let key = board.zobrist_key;
    let tt_best_move = context.tt.get(key).and_then(|entry| entry.best_move);
    let moves = ordered_legal_moves(board, tt_best_move, context, 0);

    if moves.is_empty() {
        return SearchResult {
            best_move: None,
            score: terminal_score(board, 0),
            nodes: *nodes,
            tt_hits: context.tt_hits,
        };
    }

    let mut best_move = None;
    let mut best_score = i32::MIN;
    let original_alpha = alpha;

    for mv in moves {
        let undo: Undo = board.make_move(mv);
        let score = -negamax_alpha_beta(
            board,
            depth.saturating_sub(1),
            -beta,
            -alpha,
            nodes,
            context,
        );
        board.unmake_move(&undo);

        if best_move.is_none() || score > best_score {
            best_move = Some(mv);
            best_score = score;
        }
        alpha = alpha.max(score);
    }

    context.tt.store(TTEntry {
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
        tt_hits: context.tt_hits,
    }
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

fn ordered_legal_moves(
    board: &Board,
    tt_best_move: Option<Move>,
    context: &SearchContext<'_>,
    ply: usize,
) -> Vec<Move> {
    let mut moves = generate_legal_moves(board);
    moves.sort_by_key(|&mv| {
        std::cmp::Reverse(move_order_score(board, mv, tt_best_move, context, ply))
    });
    moves
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

fn move_order_score(
    board: &Board,
    mv: Move,
    tt_best_move: Option<Move>,
    context: &SearchContext<'_>,
    ply: usize,
) -> i32 {
    if Some(mv) == tt_best_move {
        return 10_000_000;
    }

    let promotion = promotion_score(mv);
    if promotion > 0 {
        return 9_000_000 + promotion;
    }

    if let Some(capture_score) = capture_score(board, mv) {
        return 8_000_000 + capture_score;
    }

    if context.killer_moves.is_killer(ply, mv) {
        return 7_000_000;
    }

    context.history.score(board.side_to_move, mv)
}

fn negamax_alpha_beta(
    board: &mut Board,
    depth: u32,
    alpha: i32,
    beta: i32,
    nodes: &mut u64,
    context: &mut SearchContext<'_>,
) -> i32 {
    negamax_alpha_beta_with_ply(board, depth, alpha, beta, nodes, context, 1)
}

fn negamax_alpha_beta_with_ply(
    board: &mut Board,
    depth: u32,
    mut alpha: i32,
    mut beta: i32,
    nodes: &mut u64,
    context: &mut SearchContext<'_>,
    ply: i32,
) -> i32 {
    *nodes += 1;

    if depth == 0 {
        return quiescence(board, alpha, beta, nodes);
    }

    let key = board.zobrist_key;
    let original_alpha = alpha;
    let tt_best_move = if let Some(entry) = context.tt.get(key) {
        if entry.depth >= depth {
            context.tt_hits += 1;
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

    let moves = ordered_legal_moves(board, tt_best_move, context, ply as usize);
    if moves.is_empty() {
        return terminal_score(board, ply);
    }

    let mut best_move = None;
    let mut best_score = i32::MIN;
    for mv in moves {
        let undo = board.make_move(mv);
        let score =
            -negamax_alpha_beta_with_ply(board, depth - 1, -beta, -alpha, nodes, context, ply + 1);
        board.unmake_move(&undo);

        if score > best_score {
            best_score = score;
            best_move = Some(mv);
        }
        alpha = alpha.max(score);

        if alpha >= beta {
            if is_quiet(board, &mv) {
                context.killer_moves.store(ply as usize, mv);
                context.history.add(board.side_to_move, mv, depth);
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

    context.tt.store(TTEntry {
        key,
        depth,
        score: best_score,
        bound,
        best_move,
    });

    best_score
}

fn quiescence(board: &mut Board, mut alpha: i32, beta: i32, nodes: &mut u64) -> i32 {
    *nodes += 1;

    let stand_pat = evaluate(board);
    if stand_pat >= beta {
        return beta;
    }

    if alpha < stand_pat {
        alpha = stand_pat;
    }

    let noisy_moves: Vec<_> = generate_legal_moves(board)
        .into_iter()
        .filter(|mv| is_capture(board, mv) || mv.promotion.is_some())
        .collect();

    for mv in noisy_moves {
        let undo = board.make_move(mv);
        let score = -quiescence(board, -beta, -alpha, nodes);
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

fn terminal_score(board: &Board, ply: i32) -> i32 {
    if is_in_check(board, board.side_to_move) {
        -CHECKMATE_SCORE + ply
    } else {
        0
    }
}

fn promotion_score(mv: Move) -> i32 {
    match mv.promotion {
        Some(PieceKind::Queen) => 9000,
        Some(_) => 8000,
        None => 0,
    }
}

fn is_capture(board: &Board, mv: &Move) -> bool {
    captured_piece_kind(board, mv).is_some()
}

fn is_quiet(board: &Board, mv: &Move) -> bool {
    !is_capture(board, mv) && mv.promotion.is_none()
}

fn capture_score(board: &Board, mv: Move) -> Option<i32> {
    let victim = captured_piece_kind(board, &mv)?;
    let attacker = board.piece_at(mv.from)?;

    Some(piece_value(victim) * 10 - piece_value(attacker.kind))
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

fn color_index(color: Color) -> usize {
    match color {
        Color::White => 0,
        Color::Black => 1,
    }
}

fn piece_value(kind: PieceKind) -> i32 {
    match kind {
        PieceKind::Pawn => 100,
        PieceKind::Knight => 320,
        PieceKind::Bishop => 330,
        PieceKind::Rook => 500,
        PieceKind::Queen => 900,
        PieceKind::King => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_returns_a_move_from_startpos() {
        let board = Board::startpos();
        let result = search_best_move(&board, 1);

        assert!(result.best_move.is_some());
        assert!(result.nodes > 0);
    }

    #[test]
    fn capture_move_is_ordered_before_quiet_move() {
        let board = Board::from_fen("4k3/8/8/3q4/8/2N5/8/4K3 w - - 0 1").expect("valid FEN");
        let mut tt = TranspositionTable::new(0);
        let context = test_context(&mut tt);
        let moves = ordered_legal_moves(&board, None, &context, 0);

        assert_eq!(moves.first(), Some(&Move::new(square("c3"), square("d5"))));
    }

    #[test]
    fn promotion_move_is_ordered_before_quiet_move() {
        let board = Board::from_fen("4k3/P7/8/8/8/8/8/4K3 w - - 0 1").expect("valid FEN");
        let mut tt = TranspositionTable::new(0);
        let context = test_context(&mut tt);
        let moves = ordered_legal_moves(&board, None, &context, 0);

        assert_eq!(
            moves.first(),
            Some(&Move::promotion(
                square("a7"),
                square("a8"),
                PieceKind::Queen
            ))
        );
    }

    #[test]
    fn killer_move_is_prioritized() {
        let board = Board::startpos();
        let mut tt = TranspositionTable::new(0);
        let mut context = test_context(&mut tt);
        let killer = Move::new(square("g1"), square("f3"));

        context.killer_moves.store(0, killer);
        let moves = ordered_legal_moves(&board, None, &context, 0);

        assert_eq!(moves.first(), Some(&killer));
    }

    #[test]
    fn history_increases_for_good_moves() {
        let mut history = HistoryHeuristic::new();
        let mv = Move::new(square("b1"), square("c3"));

        history.add(Color::White, mv, 3);

        assert_eq!(history.score(Color::White, mv), 9);
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
        let score = quiescence(&mut board.clone(), i32::MIN + 1, i32::MAX, &mut nodes);

        assert!(nodes > 0);
        assert!(score > i32::MIN);
    }

    #[test]
    fn quiescence_increases_nodes_compared_to_static_leaf() {
        let board = Board::from_fen("4k3/8/8/3q4/8/2N5/8/4K3 w - - 0 1").expect("valid FEN");
        let mut quiescence_nodes = 0;
        let mut static_nodes = 0;
        let mut tt = TranspositionTable::new(0);
        let mut context = SearchContext {
            tt: &mut tt,
            killer_moves: KillerMoves::new(),
            history: HistoryHeuristic::new(),
            tt_hits: 0,
        };

        let _ = negamax_alpha_beta(
            &mut board.clone(),
            0,
            i32::MIN + 1,
            i32::MAX,
            &mut quiescence_nodes,
            &mut context,
        );
        let _ = static_leaf_alpha_beta(
            &mut board.clone(),
            0,
            i32::MIN + 1,
            i32::MAX,
            &mut static_nodes,
            1,
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
        assert_eq!(with_tt.best_move, without_tt.best_move);
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

    fn square(algebraic: &str) -> crate::board::Square {
        crate::board::Square::from_algebraic(algebraic).expect("test square is valid")
    }

    fn test_context(tt: &mut TranspositionTable) -> SearchContext<'_> {
        SearchContext {
            tt,
            killer_moves: KillerMoves::new(),
            history: HistoryHeuristic::new(),
            tt_hits: 0,
        }
    }
}
