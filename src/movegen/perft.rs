use crate::board::Board;

use super::generate_pseudo_legal_moves;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perft_depth_one_matches_pseudo_move_count() {
        let board = Board::startpos();

        assert_eq!(perft(&board, 1), 20);
    }
}
