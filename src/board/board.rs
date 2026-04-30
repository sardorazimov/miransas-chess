use super::{Color, FenError, Piece, PieceKind, Square};

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct CastlingRights {
    pub white_kingside: bool,
    pub white_queenside: bool,
    pub black_kingside: bool,
    pub black_queenside: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Board {
    pub squares: [Option<Piece>; 64],
    pub side_to_move: Color,
    pub castling_rights: CastlingRights,
    pub en_passant: Option<Square>,
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
}

impl Board {
    pub const STARTPOS_FEN: &'static str =
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    pub fn empty() -> Self {
        Self {
            squares: [None; 64],
            side_to_move: Color::White,
            castling_rights: CastlingRights::default(),
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    pub fn startpos() -> Self {
        Self::from_fen(Self::STARTPOS_FEN).expect("built-in start position FEN must be valid")
    }

    pub fn from_fen(fen: &str) -> Result<Self, FenError> {
        crate::board::fen::parse_fen(fen)
    }

    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        self.squares[square.index()]
    }

    pub fn set_piece(&mut self, square: Square, piece: Option<Piece>) {
        self.squares[square.index()] = piece;
    }

    pub fn piece_count(&self) -> usize {
        self.squares.iter().flatten().count()
    }

    pub fn to_fen_piece_placement(&self) -> String {
        let mut result = String::new();

        for rank in (0..8).rev() {
            let mut empty = 0;

            for file in 0..8 {
                let square = Square::from_file_rank(file, rank).expect("rank/file are in range");
                match self.piece_at(square) {
                    Some(piece) => {
                        if empty > 0 {
                            result.push(char::from_digit(empty, 10).expect("empty run is <= 8"));
                            empty = 0;
                        }
                        result.push(piece.to_fen_char());
                    }
                    None => empty += 1,
                }
            }

            if empty > 0 {
                result.push(char::from_digit(empty, 10).expect("empty run is <= 8"));
            }

            if rank > 0 {
                result.push('/');
            }
        }

        result
    }

    pub fn make_move_unchecked(&self, mv: crate::movegen::Move) -> Self {
        let mut next = self.clone();
        let moving_piece = self.piece_at(mv.from);
        let captured_piece = self.piece_at(mv.to);

        next.set_piece(mv.from, None);
        if let Some(mut piece) = moving_piece {
            if let Some(promotion) = mv.promotion {
                piece.kind = promotion;
            }
            next.set_piece(mv.to, Some(piece));

            if piece.kind == PieceKind::Pawn || captured_piece.is_some() {
                next.halfmove_clock = 0;
            } else {
                next.halfmove_clock += 1;
            }
        }

        next.en_passant = None;
        next.side_to_move = self.side_to_move.opposite();
        if self.side_to_move == Color::Black {
            next.fullmove_number += 1;
        }

        next
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_position_fen_parses_correctly() {
        let board = Board::startpos();

        assert_eq!(board.side_to_move, Color::White);
        assert_eq!(board.piece_count(), 32);
        assert_eq!(
            board.to_fen_piece_placement(),
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR"
        );
        assert!(board.castling_rights.white_kingside);
        assert!(board.castling_rights.white_queenside);
        assert!(board.castling_rights.black_kingside);
        assert!(board.castling_rights.black_queenside);
        assert_eq!(board.en_passant, None);
        assert_eq!(board.halfmove_clock, 0);
        assert_eq!(board.fullmove_number, 1);
    }
}
