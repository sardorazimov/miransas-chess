use crate::board::Board;

use super::{generate_legal_moves, generate_pseudo_legal_moves};

pub fn perft(board: &Board, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }

    let moves = generate_pseudo_legal_moves(board);
    if depth == 1 {
        return moves.len() as u64;
    }

    moves
        .into_iter()
        .map(|mv| perft(&board.make_move_unchecked(mv), depth - 1))
        .sum()
}

pub fn perft_legal(board: &Board, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }

    let moves = generate_legal_moves(board);
    if depth == 1 {
        return moves.len() as u64;
    }

    moves
        .into_iter()
        .map(|mv| perft_legal(&board.make_move_unchecked(mv), depth - 1))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perft_depth_one_matches_pseudo_move_count() {
        let board = Board::startpos();

        assert_eq!(perft(&board, 1), 20);
    }

    #[test]
    fn legal_perft_startpos_matches_known_counts() {
        let board = Board::startpos();

        assert_eq!(perft_legal(&board, 1), 20);
        assert_eq!(perft_legal(&board, 2), 400);
        assert_eq!(perft_legal(&board, 3), 8902);
        assert_eq!(perft_legal(&board, 4), 197281);
    }
}
