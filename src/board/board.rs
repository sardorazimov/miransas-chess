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
            let is_en_passant = piece.kind == PieceKind::Pawn
                && Some(mv.to) == self.en_passant
                && mv.from.file() != mv.to.file()
                && captured_piece.is_none();
            let captured_piece = if is_en_passant {
                let captured_rank = match piece.color {
                    Color::White => mv.to.rank() - 1,
                    Color::Black => mv.to.rank() + 1,
                };
                let captured_square = Square::from_file_rank(mv.to.file(), captured_rank)
                    .expect("en passant captured pawn square is on the board");
                let captured_piece = self.piece_at(captured_square);
                next.set_piece(captured_square, None);
                captured_piece
            } else {
                captured_piece
            };

            update_castling_rights_for_move(&mut next.castling_rights, piece, mv.from);
            if let Some(captured_piece) = captured_piece {
                update_castling_rights_for_capture(
                    &mut next.castling_rights,
                    captured_piece,
                    mv.to,
                );
            }

            if piece.kind == PieceKind::King && mv.from.file().abs_diff(mv.to.file()) == 2 {
                move_castling_rook(&mut next, piece.color, mv.to);
            }

            let was_pawn = piece.kind == PieceKind::Pawn;
            if let Some(promotion) = mv.promotion {
                piece.kind = promotion;
            }
            next.set_piece(mv.to, Some(piece));

            if was_pawn || captured_piece.is_some() {
                next.halfmove_clock = 0;
            } else {
                next.halfmove_clock += 1;
            }

            next.en_passant = double_pawn_push_en_passant_square(mv.from, mv.to, was_pawn);
        }

        next.side_to_move = self.side_to_move.opposite();
        if self.side_to_move == Color::Black {
            next.fullmove_number += 1;
        }

        next
    }
}

fn double_pawn_push_en_passant_square(from: Square, to: Square, was_pawn: bool) -> Option<Square> {
    if was_pawn && from.file() == to.file() && from.rank().abs_diff(to.rank()) == 2 {
        let rank = (from.rank() + to.rank()) / 2;
        Square::from_file_rank(from.file(), rank)
    } else {
        None
    }
}

fn update_castling_rights_for_move(rights: &mut CastlingRights, piece: Piece, from: Square) {
    match (piece.color, piece.kind, from) {
        (Color::White, PieceKind::King, _) => {
            rights.white_kingside = false;
            rights.white_queenside = false;
        }
        (Color::Black, PieceKind::King, _) => {
            rights.black_kingside = false;
            rights.black_queenside = false;
        }
        (Color::White, PieceKind::Rook, square) if square == algebraic("h1") => {
            rights.white_kingside = false;
        }
        (Color::White, PieceKind::Rook, square) if square == algebraic("a1") => {
            rights.white_queenside = false;
        }
        (Color::Black, PieceKind::Rook, square) if square == algebraic("h8") => {
            rights.black_kingside = false;
        }
        (Color::Black, PieceKind::Rook, square) if square == algebraic("a8") => {
            rights.black_queenside = false;
        }
        _ => {}
    }
}

fn update_castling_rights_for_capture(
    rights: &mut CastlingRights,
    captured_piece: Piece,
    captured_square: Square,
) {
    if captured_piece.kind != PieceKind::Rook {
        return;
    }

    match (captured_piece.color, captured_square) {
        (Color::White, square) if square == algebraic("h1") => rights.white_kingside = false,
        (Color::White, square) if square == algebraic("a1") => rights.white_queenside = false,
        (Color::Black, square) if square == algebraic("h8") => rights.black_kingside = false,
        (Color::Black, square) if square == algebraic("a8") => rights.black_queenside = false,
        _ => {}
    }
}

fn move_castling_rook(board: &mut Board, color: Color, king_to: Square) {
    let (rook_from, rook_to) = match (color, king_to) {
        (Color::White, square) if square == algebraic("g1") => (algebraic("h1"), algebraic("f1")),
        (Color::White, square) if square == algebraic("c1") => (algebraic("a1"), algebraic("d1")),
        (Color::Black, square) if square == algebraic("g8") => (algebraic("h8"), algebraic("f8")),
        (Color::Black, square) if square == algebraic("c8") => (algebraic("a8"), algebraic("d8")),
        _ => return,
    };

    let rook = board.piece_at(rook_from);
    board.set_piece(rook_from, None);
    board.set_piece(rook_to, rook);
}

fn algebraic(square: &str) -> Square {
    Square::from_algebraic(square).expect("hard-coded square is valid")
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

    #[test]
    fn king_move_removes_both_castling_rights_for_that_color() {
        let board = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").expect("valid FEN");
        let next =
            board.make_move_unchecked(crate::movegen::Move::new(algebraic("e1"), algebraic("e2")));

        assert!(!next.castling_rights.white_kingside);
        assert!(!next.castling_rights.white_queenside);
        assert!(next.castling_rights.black_kingside);
        assert!(next.castling_rights.black_queenside);
    }

    #[test]
    fn rook_move_and_capture_remove_matching_castling_right() {
        let board = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").expect("valid FEN");
        let rook_move =
            board.make_move_unchecked(crate::movegen::Move::new(algebraic("h1"), algebraic("h2")));

        assert!(!rook_move.castling_rights.white_kingside);
        assert!(rook_move.castling_rights.white_queenside);

        let capture = Board::from_fen("r3k2r/8/8/8/8/8/6b1/R3K2R b KQkq - 0 1")
            .expect("valid FEN")
            .make_move_unchecked(crate::movegen::Move::new(algebraic("g2"), algebraic("h1")));

        assert!(!capture.castling_rights.white_kingside);
        assert!(capture.castling_rights.white_queenside);
    }
}
