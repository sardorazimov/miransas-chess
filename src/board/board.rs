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
    pub zobrist_key: u64,
}

#[derive(Clone, Copy, Debug)]
pub struct NullUndo {
    pub prev_en_passant: Option<Square>,
    pub prev_zobrist_key: u64,
}

#[derive(Clone, Copy, Debug)]
pub struct Undo {
    pub mv: crate::movegen::Move,
    pub captured: Option<Piece>,
    pub captured_square: Square,
    pub prev_castling_rights: CastlingRights,
    pub prev_en_passant: Option<Square>,
    pub prev_halfmove_clock: u32,
    pub prev_zobrist_key: u64,
    pub was_en_passant: bool,
    pub was_castling: bool,
    pub was_promotion: bool,
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
            zobrist_key: 0,
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

    pub fn make_move(&mut self, mv: crate::movegen::Move) -> Undo {
        let z = crate::search::zobrist();
        let mut hash = self.zobrist_key;

        let moving_piece = self
            .piece_at(mv.from)
            .expect("make_move called with no piece at mv.from");
        let captured_at_to = self.piece_at(mv.to);

        let prev_castling_rights = self.castling_rights;
        let prev_en_passant = self.en_passant;
        let prev_halfmove_clock = self.halfmove_clock;
        let prev_zobrist_key = hash;

        if let Some(ep) = prev_en_passant {
            hash = z.toggle_en_passant(hash, ep);
        }
        hash = z.toggle_castling(hash, &prev_castling_rights);
        hash = z.toggle_piece(hash, moving_piece.color, moving_piece.kind, mv.from);

        let is_en_passant = moving_piece.kind == PieceKind::Pawn
            && Some(mv.to) == self.en_passant
            && mv.from.file() != mv.to.file()
            && captured_at_to.is_none();

        let is_castling =
            moving_piece.kind == PieceKind::King && mv.from.file().abs_diff(mv.to.file()) == 2;

        let is_promotion = mv.promotion.is_some();

        let (captured, captured_square) = if is_en_passant {
            let captured_rank = match moving_piece.color {
                Color::White => mv.to.rank() - 1,
                Color::Black => mv.to.rank() + 1,
            };
            let csq = Square::from_file_rank(mv.to.file(), captured_rank)
                .expect("en passant captured pawn square is valid");
            let cap = self.piece_at(csq);
            if let Some(cp) = cap {
                hash = z.toggle_piece(hash, cp.color, cp.kind, csq);
            }
            self.set_piece(csq, None);
            (cap, csq)
        } else {
            if let Some(cp) = captured_at_to {
                hash = z.toggle_piece(hash, cp.color, cp.kind, mv.to);
            }
            (captured_at_to, mv.to)
        };

        if is_castling
            && let Some((rook_from, rook_to)) = castling_rook_squares(moving_piece.color, mv.to)
        {
            let rook = self.piece_at(rook_from);
            if let Some(r) = rook {
                hash = z.toggle_piece(hash, r.color, r.kind, rook_from);
                hash = z.toggle_piece(hash, r.color, r.kind, rook_to);
            }
            self.set_piece(rook_from, None);
            self.set_piece(rook_to, rook);
        }

        let was_pawn = moving_piece.kind == PieceKind::Pawn;
        let mut landing_piece = moving_piece;
        if let Some(promo_kind) = mv.promotion {
            landing_piece.kind = promo_kind;
        }
        self.set_piece(mv.from, None);
        self.set_piece(mv.to, Some(landing_piece));
        hash = z.toggle_piece(hash, landing_piece.color, landing_piece.kind, mv.to);

        if was_pawn || captured.is_some() {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }

        update_castling_rights_for_move(&mut self.castling_rights, moving_piece, mv.from);
        if let Some(cap) = captured {
            update_castling_rights_for_capture(&mut self.castling_rights, cap, captured_square);
        }

        self.en_passant = double_pawn_push_en_passant_square(mv.from, mv.to, was_pawn);
        if let Some(ep) = self.en_passant {
            hash = z.toggle_en_passant(hash, ep);
        }
        hash = z.toggle_castling(hash, &self.castling_rights);

        if self.side_to_move == Color::Black {
            self.fullmove_number += 1;
        }
        self.side_to_move = self.side_to_move.opposite();
        hash = z.toggle_side_to_move(hash);

        self.zobrist_key = hash;

        Undo {
            mv,
            captured,
            captured_square,
            prev_castling_rights,
            prev_en_passant,
            prev_halfmove_clock,
            prev_zobrist_key,
            was_en_passant: is_en_passant,
            was_castling: is_castling,
            was_promotion: is_promotion,
        }
    }

    /// Returns true if the side to move has only king and pawns (zugzwang-prone).
    pub fn side_to_move_has_only_pawns(&self) -> bool {
        for piece in self.squares.iter().flatten() {
            if piece.color != self.side_to_move {
                continue;
            }
            match piece.kind {
                PieceKind::King | PieceKind::Pawn => continue,
                _ => return false,
            }
        }
        true
    }

    pub fn make_null_move(&mut self) -> NullUndo {
        let undo = NullUndo {
            prev_en_passant: self.en_passant,
            prev_zobrist_key: self.zobrist_key,
        };
        let z = crate::search::zobrist();
        self.zobrist_key = z.toggle_side_to_move(self.zobrist_key);
        self.side_to_move = self.side_to_move.opposite();
        if let Some(ep) = self.en_passant {
            self.zobrist_key = z.toggle_en_passant(self.zobrist_key, ep);
            self.en_passant = None;
        }
        undo
    }

    pub fn unmake_null_move(&mut self, undo: NullUndo) {
        self.side_to_move = self.side_to_move.opposite();
        self.en_passant = undo.prev_en_passant;
        self.zobrist_key = undo.prev_zobrist_key;
    }

    pub fn unmake_move(&mut self, undo: &Undo) {
        let mv = undo.mv;

        self.side_to_move = self.side_to_move.opposite();
        if self.side_to_move == Color::Black {
            self.fullmove_number -= 1;
        }

        self.castling_rights = undo.prev_castling_rights;
        self.en_passant = undo.prev_en_passant;
        self.halfmove_clock = undo.prev_halfmove_clock;
        self.zobrist_key = undo.prev_zobrist_key;

        let color = self.side_to_move;

        if undo.was_castling
            && let Some((rook_from, rook_to)) = castling_rook_squares(color, mv.to)
        {
            let rook = self.piece_at(rook_to);
            self.set_piece(rook_to, None);
            self.set_piece(rook_from, rook);
        }

        let mut restore_piece = self
            .piece_at(mv.to)
            .expect("piece must be at mv.to during unmake");
        if undo.was_promotion {
            restore_piece.kind = PieceKind::Pawn;
        }
        self.set_piece(mv.from, Some(restore_piece));

        if undo.was_en_passant {
            self.set_piece(mv.to, None);
            self.set_piece(undo.captured_square, undo.captured);
        } else {
            self.set_piece(mv.to, undo.captured);
        }
    }
}

fn castling_rook_squares(color: Color, king_to: Square) -> Option<(Square, Square)> {
    match (color, king_to) {
        (Color::White, sq) if sq == algebraic("g1") => Some((algebraic("h1"), algebraic("f1"))),
        (Color::White, sq) if sq == algebraic("c1") => Some((algebraic("a1"), algebraic("d1"))),
        (Color::Black, sq) if sq == algebraic("g8") => Some((algebraic("h8"), algebraic("f8"))),
        (Color::Black, sq) if sq == algebraic("c8") => Some((algebraic("a8"), algebraic("d8"))),
        _ => None,
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

fn algebraic(square: &str) -> Square {
    Square::from_algebraic(square).expect("hard-coded square is valid")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::movegen::{Move, generate_legal_moves};

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
        let mut next = board.clone();
        next.make_move(Move::new(algebraic("e1"), algebraic("e2")));

        assert!(!next.castling_rights.white_kingside);
        assert!(!next.castling_rights.white_queenside);
        assert!(next.castling_rights.black_kingside);
        assert!(next.castling_rights.black_queenside);
    }

    #[test]
    fn rook_move_and_capture_remove_matching_castling_right() {
        let board = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").expect("valid FEN");
        let mut rook_move = board.clone();
        rook_move.make_move(Move::new(algebraic("h1"), algebraic("h2")));

        assert!(!rook_move.castling_rights.white_kingside);
        assert!(rook_move.castling_rights.white_queenside);

        let mut capture =
            Board::from_fen("r3k2r/8/8/8/8/8/6b1/R3K2R b KQkq - 0 1").expect("valid FEN");
        capture.make_move(Move::new(algebraic("g2"), algebraic("h1")));

        assert!(!capture.castling_rights.white_kingside);
        assert!(capture.castling_rights.white_queenside);
    }

    #[test]
    fn pawns_only_detection() {
        let kp = Board::from_fen("4k3/pppppppp/8/8/8/8/PPPPPPPP/4K3 w - - 0 1").expect("valid FEN");
        assert!(kp.side_to_move_has_only_pawns());

        let kn =
            Board::from_fen("4k3/pppppppp/8/8/8/8/PPPPPPPP/N3K3 w - - 0 1").expect("valid FEN");
        assert!(!kn.side_to_move_has_only_pawns());

        let start = Board::startpos();
        assert!(!start.side_to_move_has_only_pawns());
    }

    #[test]
    fn null_move_is_reversible() {
        let mut board = Board::startpos();
        let snapshot = board.clone();
        let undo = board.make_null_move();
        assert_ne!(board.zobrist_key, snapshot.zobrist_key);
        assert_ne!(board.side_to_move, snapshot.side_to_move);
        board.unmake_null_move(undo);
        assert_eq!(board, snapshot);
        assert_eq!(board.zobrist_key, snapshot.zobrist_key);
    }

    #[test]
    fn null_move_clears_en_passant_and_restores_it() {
        let mut board =
            Board::from_fen("rnbqkbnr/pppp1ppp/8/4pP2/8/8/PPPPP1PP/RNBQKBNR w KQkq e6 0 2")
                .expect("valid FEN with en passant");
        let snapshot = board.clone();
        assert!(snapshot.en_passant.is_some());
        let undo = board.make_null_move();
        assert!(
            board.en_passant.is_none(),
            "en passant must be cleared after null move"
        );
        board.unmake_null_move(undo);
        assert_eq!(board.en_passant, snapshot.en_passant);
        assert_eq!(board.zobrist_key, snapshot.zobrist_key);
    }

    #[test]
    fn make_unmake_restores_board_exactly() {
        let positions = [
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
            "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
        ];
        for fen in positions {
            let original = Board::from_fen(fen).expect("valid FEN");
            let mut board = original.clone();
            for mv in generate_legal_moves(&board) {
                let snapshot = board.clone();
                let undo = board.make_move(mv);
                board.unmake_move(&undo);
                assert_eq!(
                    board, snapshot,
                    "make/unmake mismatch on move {mv} in fen {fen}"
                );
                assert_eq!(
                    board.zobrist_key, snapshot.zobrist_key,
                    "zobrist_key drift on move {mv} in fen {fen}"
                );
            }
            assert_eq!(board, original);
        }
    }
}
