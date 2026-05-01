use crate::board::Board;

use super::{generate_legal_moves, generate_pseudo_legal_moves};

pub fn perft(board: &mut Board, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }

    let moves = generate_pseudo_legal_moves(board);
    if depth == 1 {
        return moves.len() as u64;
    }

    let mut total = 0;
    for mv in moves {
        let undo = board.make_move(mv);
        total += perft(board, depth - 1);
        board.unmake_move(&undo);
    }
    total
}

pub fn perft_legal(board: &mut Board, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }

    let moves = generate_legal_moves(board);
    if depth == 1 {
        return moves.len() as u64;
    }

    let mut total = 0;
    for mv in moves {
        let undo = board.make_move(mv);
        total += perft_legal(board, depth - 1);
        board.unmake_move(&undo);
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perft_depth_one_matches_pseudo_move_count() {
        let mut board = Board::startpos();

        assert_eq!(perft(&mut board, 1), 20);
    }

    #[test]
    fn legal_perft_startpos_matches_known_counts() {
        let mut board = Board::startpos();

        assert_eq!(perft_legal(&mut board, 1), 20);
        assert_eq!(perft_legal(&mut board, 2), 400);
        assert_eq!(perft_legal(&mut board, 3), 8902);
        assert_eq!(perft_legal(&mut board, 4), 197281);
    }
}
