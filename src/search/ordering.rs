use crate::board::{Board, PieceKind};
use crate::movegen::Move;
use crate::search::context::piece_index;
use crate::search::{MAX_PLY, SearchContext};

pub const SCORE_TT_MOVE: i32 = 10_000_000;
pub const SCORE_PROMO_QUEEN: i32 = 9_000_000;
pub const SCORE_CAPTURE_BASE: i32 = 8_000_000;
pub const SCORE_PROMO_OTHER: i32 = 7_000_000;
pub const SCORE_KILLER_1: i32 = 6_000_000;
pub const SCORE_KILLER_2: i32 = 5_900_000;

fn piece_value(kind: PieceKind) -> i32 {
    match kind {
        PieceKind::Pawn => 100,
        PieceKind::Knight => 320,
        PieceKind::Bishop => 330,
        PieceKind::Rook => 500,
        PieceKind::Queen => 900,
        PieceKind::King => 20_000,
    }
}

pub fn score_move(
    board: &Board,
    mv: Move,
    tt_move: Option<Move>,
    ctx: &SearchContext,
    ply: usize,
) -> i32 {
    if Some(mv) == tt_move {
        return SCORE_TT_MOVE;
    }

    let target = board.squares[mv.to.index()];
    let attacker = board.squares[mv.from.index()].expect("from square must have a piece");

    let is_ep = attacker.kind == PieceKind::Pawn && Some(mv.to) == board.en_passant;

    if let Some(victim) = target {
        return SCORE_CAPTURE_BASE + piece_value(victim.kind) * 16 - piece_value(attacker.kind);
    }

    if is_ep {
        return SCORE_CAPTURE_BASE + piece_value(PieceKind::Pawn) * 16
            - piece_value(PieceKind::Pawn);
    }

    if let Some(promo) = mv.promotion {
        return if promo == PieceKind::Queen {
            SCORE_PROMO_QUEEN
        } else {
            SCORE_PROMO_OTHER
        };
    }

    if ctx.is_killer(ply, mv) {
        let is_first = ply < MAX_PLY && ctx.killers[ply][0] == Some(mv);
        return if is_first {
            SCORE_KILLER_1
        } else {
            SCORE_KILLER_2
        };
    }

    let pi = piece_index(attacker.color, attacker.kind);
    ctx.history_score(pi, mv.to.index())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{Board, Square};
    use crate::movegen::{Move, generate_legal_moves};
    use crate::search::context::SearchContext;

    fn sq(s: &str) -> Square {
        Square::from_algebraic(s).expect("valid square")
    }

    #[test]
    fn tt_move_scored_highest() {
        let board = Board::startpos();
        let ctx = SearchContext::new();
        let moves = generate_legal_moves(&board);
        let any_move = moves[0];
        let other_move = moves[1];

        let s1 = score_move(&board, any_move, Some(any_move), &ctx, 0);
        let s2 = score_move(&board, other_move, Some(any_move), &ctx, 0);
        assert!(s1 > s2);
        assert_eq!(s1, SCORE_TT_MOVE);
    }

    #[test]
    fn mvv_lva_orders_captures_correctly() {
        // e4=White pawn, d5=Black queen, c3=White knight, b5=Black pawn
        // Pawn takes Queen (e4xd5) should outscore Knight takes Pawn (c3xb5)
        let board = Board::from_fen("4k3/8/8/1p1q4/4P3/2N5/8/4K3 w - - 0 1").expect("valid FEN");
        let ctx = SearchContext::new();
        let pawn_takes_queen = Move::new(sq("e4"), sq("d5"));
        let knight_takes_pawn = Move::new(sq("c3"), sq("b5"));

        let s_pq = score_move(&board, pawn_takes_queen, None, &ctx, 0);
        let s_np = score_move(&board, knight_takes_pawn, None, &ctx, 0);
        assert!(
            s_pq > s_np,
            "Pawn-takes-Queen ({s_pq}) should outscore Knight-takes-Pawn ({s_np})"
        );
    }

    #[test]
    fn killer_outscores_quiet_with_zero_history() {
        let board = Board::startpos();
        let mut ctx = SearchContext::new();
        let moves = generate_legal_moves(&board);
        let killer = moves[0];
        let other_quiet = moves[1];
        ctx.record_killer(0, killer);

        let s_k = score_move(&board, killer, None, &ctx, 0);
        let s_q = score_move(&board, other_quiet, None, &ctx, 0);
        assert!(s_k > s_q);
    }

    #[test]
    fn history_increases_after_record() {
        let mut ctx = SearchContext::new();
        ctx.record_history(0, 16, 4);
        assert_eq!(ctx.history_score(0, 16), 16); // 4×4
        ctx.record_history(0, 16, 5);
        assert_eq!(ctx.history_score(0, 16), 41); // 16+25
    }

    #[test]
    fn capture_scores_above_quiet() {
        let board = Board::from_fen("4k3/8/8/3q4/8/2N5/8/4K3 w - - 0 1").expect("valid FEN");
        let ctx = SearchContext::new();
        let capture = Move::new(sq("c3"), sq("d5"));
        let quiet = Move::new(sq("c3"), sq("e4"));

        let s_cap = score_move(&board, capture, None, &ctx, 0);
        let s_qui = score_move(&board, quiet, None, &ctx, 0);
        assert!(s_cap > s_qui);
    }
}
