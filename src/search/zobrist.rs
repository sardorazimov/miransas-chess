use crate::board::{Board, Color, PieceKind};

const ZOBRIST_SEED: u64 = 0x9e37_79b9_7f4a_7c15;

pub struct Zobrist {
    piece_square: [[[u64; 64]; 6]; 2],
    side_to_move: u64,
    castling: [u64; 16],
    en_passant_file: [u64; 8],
}

impl Zobrist {
    pub fn new() -> Self {
        let mut state = ZOBRIST_SEED;
        let mut piece_square = [[[0; 64]; 6]; 2];

        for color in &mut piece_square {
            for piece in color {
                for square in piece {
                    *square = splitmix64(&mut state);
                }
            }
        }

        let side_to_move = splitmix64(&mut state);
        let mut castling = [0; 16];
        for value in &mut castling {
            *value = splitmix64(&mut state);
        }

        let mut en_passant_file = [0; 8];
        for value in &mut en_passant_file {
            *value = splitmix64(&mut state);
        }

        Self {
            piece_square,
            side_to_move,
            castling,
            en_passant_file,
        }
    }

    pub fn hash_board(&self, board: &Board) -> u64 {
        let mut hash = 0;

        for (index, piece) in board.squares.iter().enumerate() {
            let Some(piece) = piece else {
                continue;
            };
            hash ^=
                self.piece_square[color_index(piece.color)][piece_kind_index(piece.kind)][index];
        }

        if board.side_to_move == Color::Black {
            hash ^= self.side_to_move;
        }

        hash ^= self.castling[castling_index(board)];

        if let Some(en_passant) = board.en_passant {
            hash ^= self.en_passant_file[en_passant.file() as usize];
        }

        hash
    }
}

impl Default for Zobrist {
    fn default() -> Self {
        Self::new()
    }
}

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut value = *state;
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

fn color_index(color: Color) -> usize {
    match color {
        Color::White => 0,
        Color::Black => 1,
    }
}

fn piece_kind_index(kind: PieceKind) -> usize {
    match kind {
        PieceKind::Pawn => 0,
        PieceKind::Knight => 1,
        PieceKind::Bishop => 2,
        PieceKind::Rook => 3,
        PieceKind::Queen => 4,
        PieceKind::King => 5,
    }
}

fn castling_index(board: &Board) -> usize {
    let mut index = 0;
    if board.castling_rights.white_kingside {
        index |= 1;
    }
    if board.castling_rights.white_queenside {
        index |= 2;
    }
    if board.castling_rights.black_kingside {
        index |= 4;
    }
    if board.castling_rights.black_queenside {
        index |= 8;
    }
    index
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Square;
    use crate::movegen::Move;

    #[test]
    fn same_board_hashes_the_same() {
        let zobrist = Zobrist::new();
        let first = Board::startpos();
        let second = Board::startpos();

        assert_eq!(zobrist.hash_board(&first), zobrist.hash_board(&second));
    }

    #[test]
    fn different_side_to_move_hashes_differently() {
        let zobrist = Zobrist::new();
        let white = Board::from_fen("8/8/8/8/8/8/8/4K3 w - - 0 1").expect("valid FEN");
        let black = Board::from_fen("8/8/8/8/8/8/8/4K3 b - - 0 1").expect("valid FEN");

        assert_ne!(zobrist.hash_board(&white), zobrist.hash_board(&black));
    }

    #[test]
    fn moving_piece_changes_hash() {
        let zobrist = Zobrist::new();
        let board = Board::startpos();
        let next = board.make_move_unchecked(Move::new(square("e2"), square("e4")));

        assert_ne!(zobrist.hash_board(&board), zobrist.hash_board(&next));
    }

    #[test]
    fn castling_rights_affect_hash() {
        let zobrist = Zobrist::new();
        let with_rights =
            Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").expect("valid FEN");
        let without_rights =
            Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w - - 0 1").expect("valid FEN");

        assert_ne!(
            zobrist.hash_board(&with_rights),
            zobrist.hash_board(&without_rights)
        );
    }

    #[test]
    fn en_passant_file_affects_hash() {
        let zobrist = Zobrist::new();
        let d_file = Board::from_fen("8/8/8/3pP3/8/8/8/4K3 w - d6 0 1").expect("valid FEN");
        let e_file = Board::from_fen("8/8/8/4pP2/8/8/8/4K3 w - e6 0 1").expect("valid FEN");

        assert_ne!(zobrist.hash_board(&d_file), zobrist.hash_board(&e_file));
    }

    fn square(algebraic: &str) -> Square {
        Square::from_algebraic(algebraic).expect("test square is valid")
    }
}
