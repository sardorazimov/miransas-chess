use std::fmt;

use crate::board::{Board, Color, PieceKind, Square};

const PROMOTION_PIECES: [PieceKind; 4] = [
    PieceKind::Queen,
    PieceKind::Rook,
    PieceKind::Bishop,
    PieceKind::Knight,
];
const ROOK_DIRECTIONS: [(i8, i8); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
const BISHOP_DIRECTIONS: [(i8, i8); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
const QUEEN_DIRECTIONS: [(i8, i8); 8] = [
    (1, 0),
    (-1, 0),
    (0, 1),
    (0, -1),
    (1, 1),
    (1, -1),
    (-1, 1),
    (-1, -1),
];
const KNIGHT_ATTACK_DELTAS: [(i8, i8); 8] = [
    (1, 2),
    (2, 1),
    (2, -1),
    (1, -2),
    (-1, -2),
    (-2, -1),
    (-2, 1),
    (-1, 2),
];
const KING_ATTACK_DELTAS: [(i8, i8); 8] = [
    (0, 1),
    (1, 1),
    (1, 0),
    (1, -1),
    (0, -1),
    (-1, -1),
    (-1, 0),
    (-1, 1),
];

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub promotion: Option<PieceKind>,
}

impl Move {
    pub const fn new(from: Square, to: Square) -> Self {
        Self {
            from,
            to,
            promotion: None,
        }
    }

    pub const fn promotion(from: Square, to: Square, promotion: PieceKind) -> Self {
        Self {
            from,
            to,
            promotion: Some(promotion),
        }
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.promotion {
            Some(kind) => write!(f, "{}{}{}", self.from, self.to, promotion_char(kind)),
            None => write!(f, "{}{}", self.from, self.to),
        }
    }
}

fn promotion_char(kind: PieceKind) -> char {
    match kind {
        PieceKind::Queen => 'q',
        PieceKind::Rook => 'r',
        PieceKind::Bishop => 'b',
        PieceKind::Knight => 'n',
        PieceKind::Pawn | PieceKind::King => '?',
    }
}

pub fn generate_pseudo_legal_moves(board: &Board) -> Vec<Move> {
    let mut moves = Vec::new();

    for index in 0..64 {
        let from = Square::from_index(index);
        moves.extend(generate_pseudo_legal_moves_from_square(board, from));
    }

    moves
}

pub fn generate_legal_moves(board: &Board) -> Vec<Move> {
    let us = board.side_to_move;
    let mut board_copy = board.clone();

    generate_pseudo_legal_moves(board)
        .into_iter()
        .filter(|&mv| {
            let undo = board_copy.make_move(mv);
            let legal = !is_in_check(&board_copy, us);
            board_copy.unmake_move(&undo);
            legal
        })
        .collect()
}

pub fn generate_pseudo_legal_moves_from_square(board: &Board, from: Square) -> Vec<Move> {
    let mut moves = Vec::new();
    let Some(piece) = board.piece_at(from) else {
        return moves;
    };
    if piece.color != board.side_to_move {
        return moves;
    }

    match piece.kind {
        PieceKind::Pawn => generate_pawn_moves(board, from, piece.color, &mut moves),
        PieceKind::Knight => generate_knight_moves(board, from, piece.color, &mut moves),
        PieceKind::Bishop => moves.extend(generate_sliding_moves(board, from, &BISHOP_DIRECTIONS)),
        PieceKind::Rook => moves.extend(generate_sliding_moves(board, from, &ROOK_DIRECTIONS)),
        PieceKind::Queen => moves.extend(generate_sliding_moves(board, from, &QUEEN_DIRECTIONS)),
        PieceKind::King => generate_king_moves(board, from, piece.color, &mut moves),
    }

    moves
}

pub fn print_moves_for_square(board: &Board, from: Square) {
    for mv in generate_pseudo_legal_moves_from_square(board, from) {
        println!("{mv}");
    }
}

pub fn generate_sliding_moves(board: &Board, from: Square, directions: &[(i8, i8)]) -> Vec<Move> {
    let mut moves = Vec::new();
    let Some(piece) = board.piece_at(from) else {
        return moves;
    };

    for &(file_delta, rank_delta) in directions {
        let mut current = from;

        while let Some(to) = offset_square(current, file_delta, rank_delta) {
            match board.piece_at(to) {
                Some(target) if target.color == piece.color => break,
                Some(_) => {
                    moves.push(Move::new(from, to));
                    break;
                }
                None => {
                    moves.push(Move::new(from, to));
                    current = to;
                }
            }
        }
    }

    moves
}

pub fn king_square(board: &Board, color: Color) -> Option<Square> {
    for index in 0..64 {
        let square = Square::from_index(index);
        if let Some(piece) = board.piece_at(square)
            && piece.color == color
            && piece.kind == PieceKind::King
        {
            return Some(square);
        }
    }

    None
}

pub fn is_in_check(board: &Board, color: Color) -> bool {
    king_square(board, color)
        .is_some_and(|square| is_square_attacked(board, square, color.opposite()))
}

pub fn is_square_attacked(board: &Board, square: Square, by_color: Color) -> bool {
    is_attacked_by_pawn(board, square, by_color)
        || is_attacked_by_leaper(
            board,
            square,
            by_color,
            PieceKind::Knight,
            &KNIGHT_ATTACK_DELTAS,
        )
        || is_attacked_by_leaper(
            board,
            square,
            by_color,
            PieceKind::King,
            &KING_ATTACK_DELTAS,
        )
        || is_attacked_by_slider(
            board,
            square,
            by_color,
            &BISHOP_DIRECTIONS,
            &[PieceKind::Bishop, PieceKind::Queen],
        )
        || is_attacked_by_slider(
            board,
            square,
            by_color,
            &ROOK_DIRECTIONS,
            &[PieceKind::Rook, PieceKind::Queen],
        )
}

fn is_attacked_by_pawn(board: &Board, square: Square, by_color: Color) -> bool {
    // Work backward from the attacked square to the squares where an attacking pawn would sit.
    let pawn_rank_delta = match by_color {
        Color::White => -1,
        Color::Black => 1,
    };

    [-1, 1].into_iter().any(|file_delta| {
        offset_square(square, file_delta, pawn_rank_delta).is_some_and(|from| {
            board
                .piece_at(from)
                .is_some_and(|piece| piece.color == by_color && piece.kind == PieceKind::Pawn)
        })
    })
}

fn is_attacked_by_leaper(
    board: &Board,
    square: Square,
    by_color: Color,
    piece_kind: PieceKind,
    deltas: &[(i8, i8)],
) -> bool {
    deltas.iter().any(|&(file_delta, rank_delta)| {
        offset_square(square, file_delta, rank_delta).is_some_and(|from| {
            board
                .piece_at(from)
                .is_some_and(|piece| piece.color == by_color && piece.kind == piece_kind)
        })
    })
}

fn is_attacked_by_slider(
    board: &Board,
    square: Square,
    by_color: Color,
    directions: &[(i8, i8)],
    attackers: &[PieceKind],
) -> bool {
    for &(file_delta, rank_delta) in directions {
        let mut current = square;

        while let Some(from) = offset_square(current, file_delta, rank_delta) {
            let Some(piece) = board.piece_at(from) else {
                current = from;
                continue;
            };

            if piece.color == by_color && attackers.contains(&piece.kind) {
                return true;
            }

            break;
        }
    }

    false
}

fn generate_pawn_moves(board: &Board, from: Square, color: Color, moves: &mut Vec<Move>) {
    let direction: i8 = match color {
        Color::White => 1,
        Color::Black => -1,
    };
    let start_rank = match color {
        Color::White => 1,
        Color::Black => 6,
    };

    if let Some(one_forward) = offset_square(from, 0, direction)
        && board.piece_at(one_forward).is_none()
    {
        push_pawn_move(from, one_forward, color, moves);

        if from.rank() == start_rank
            && let Some(two_forward) = offset_square(from, 0, direction * 2)
            && board.piece_at(two_forward).is_none()
        {
            moves.push(Move::new(from, two_forward));
        }
    }

    for file_delta in [-1, 1] {
        if let Some(to) = offset_square(from, file_delta, direction)
            && let Some(target) = board.piece_at(to)
            && target.color != color
        {
            push_pawn_move(from, to, color, moves);
        }
    }

    if let Some(en_passant) = board.en_passant {
        let (pawn_rank, en_passant_rank) = match color {
            Color::White => (4, 5),
            Color::Black => (3, 2),
        };

        if from.rank() == pawn_rank
            && (from.file() as i8 - en_passant.file() as i8).abs() == 1
            && en_passant.rank() == en_passant_rank
        {
            moves.push(Move::new(from, en_passant));
        }
    }
}

fn generate_knight_moves(board: &Board, from: Square, color: Color, moves: &mut Vec<Move>) {
    generate_leaper_moves(board, from, color, &KNIGHT_ATTACK_DELTAS, moves);
}

fn generate_king_moves(board: &Board, from: Square, color: Color, moves: &mut Vec<Move>) {
    generate_leaper_moves(board, from, color, &KING_ATTACK_DELTAS, moves);
    generate_castling_moves(board, from, color, moves);
}

fn generate_castling_moves(board: &Board, from: Square, color: Color, moves: &mut Vec<Move>) {
    let opponent = color.opposite();
    if is_square_attacked(board, from, opponent) {
        return;
    }

    match color {
        Color::White if from == square("e1") => {
            if board.castling_rights.white_kingside
                && can_castle(board, color, square("h1"), &[square("f1"), square("g1")])
                && !is_square_attacked(board, square("f1"), opponent)
                && !is_square_attacked(board, square("g1"), opponent)
            {
                moves.push(Move::new(from, square("g1")));
            }
            if board.castling_rights.white_queenside
                && can_castle(
                    board,
                    color,
                    square("a1"),
                    &[square("b1"), square("c1"), square("d1")],
                )
                && !is_square_attacked(board, square("d1"), opponent)
                && !is_square_attacked(board, square("c1"), opponent)
            {
                moves.push(Move::new(from, square("c1")));
            }
        }
        Color::Black if from == square("e8") => {
            if board.castling_rights.black_kingside
                && can_castle(board, color, square("h8"), &[square("f8"), square("g8")])
                && !is_square_attacked(board, square("f8"), opponent)
                && !is_square_attacked(board, square("g8"), opponent)
            {
                moves.push(Move::new(from, square("g8")));
            }
            if board.castling_rights.black_queenside
                && can_castle(
                    board,
                    color,
                    square("a8"),
                    &[square("b8"), square("c8"), square("d8")],
                )
                && !is_square_attacked(board, square("d8"), opponent)
                && !is_square_attacked(board, square("c8"), opponent)
            {
                moves.push(Move::new(from, square("c8")));
            }
        }
        _ => {}
    }
}

fn can_castle(board: &Board, color: Color, rook_square: Square, empty_squares: &[Square]) -> bool {
    board
        .piece_at(rook_square)
        .is_some_and(|piece| piece.color == color && piece.kind == PieceKind::Rook)
        && empty_squares
            .iter()
            .all(|&square| board.piece_at(square).is_none())
}

fn generate_leaper_moves(
    board: &Board,
    from: Square,
    color: Color,
    deltas: &[(i8, i8)],
    moves: &mut Vec<Move>,
) {
    for &(file_delta, rank_delta) in deltas {
        if let Some(to) = offset_square(from, file_delta, rank_delta)
            && board.piece_at(to).is_none_or(|piece| piece.color != color)
        {
            moves.push(Move::new(from, to));
        }
    }
}

fn push_pawn_move(from: Square, to: Square, color: Color, moves: &mut Vec<Move>) {
    let promotion_rank = match color {
        Color::White => 7,
        Color::Black => 0,
    };

    if to.rank() == promotion_rank {
        for promotion in PROMOTION_PIECES {
            moves.push(Move::promotion(from, to, promotion));
        }
    } else {
        moves.push(Move::new(from, to));
    }
}

fn square(algebraic: &str) -> Square {
    Square::from_algebraic(algebraic).expect("hard-coded square is valid")
}

fn offset_square(square: Square, file_delta: i8, rank_delta: i8) -> Option<Square> {
    let file = square.file() as i8 + file_delta;
    let rank = square.rank() as i8 + rank_delta;

    if (0..8).contains(&file) && (0..8).contains(&rank) {
        Square::from_file_rank(file as u8, rank as u8)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startpos_generates_pawns_and_knights() {
        let board = Board::startpos();
        let moves = generate_pseudo_legal_moves(&board);

        let pawn_double_pushes = moves
            .iter()
            .filter(|mv| mv.from.rank() == 1 && mv.to.rank() == 3)
            .count();
        let knight_moves = moves
            .iter()
            .filter(|mv| {
                matches!(
                    board.piece_at(mv.from).map(|piece| piece.kind),
                    Some(PieceKind::Knight)
                )
            })
            .count();

        assert_eq!(pawn_double_pushes, 8);
        assert_eq!(knight_moves, 4);
        assert_eq!(moves.len(), 20);
    }

    #[test]
    fn startpos_sliding_pieces_are_blocked() {
        let board = Board::startpos();
        let moves = generate_pseudo_legal_moves(&board);

        assert_eq!(moves.len(), 20);
    }

    #[test]
    fn queen_in_center_generates_all_rays() {
        let board = Board::from_fen("8/8/8/3q4/8/8/8/8 b - - 0 1").expect("valid FEN");
        let moves = generate_pseudo_legal_moves(&board);

        assert_eq!(moves.len(), 27);
    }

    #[test]
    fn startpos_kings_are_not_in_check() {
        let board = Board::startpos();

        assert!(!is_in_check(&board, Color::White));
        assert!(!is_in_check(&board, Color::Black));
    }

    #[test]
    fn rook_gives_check() {
        let board = Board::from_fen("4k3/8/8/8/8/8/8/4R3 b - - 0 1").expect("valid FEN");

        assert!(is_in_check(&board, Color::Black));
    }

    #[test]
    fn bishop_gives_check() {
        let board = Board::from_fen("4k3/8/8/1B6/8/8/8/8 b - - 0 1").expect("valid FEN");

        assert!(is_in_check(&board, Color::Black));
    }

    #[test]
    fn knight_gives_check() {
        let board = Board::from_fen("4k3/8/3N4/8/8/8/8/8 b - - 0 1").expect("valid FEN");

        assert!(is_in_check(&board, Color::Black));
    }

    #[test]
    fn pawn_gives_check() {
        let board = Board::from_fen("4k3/3P4/8/8/8/8/8/8 b - - 0 1").expect("valid FEN");

        assert!(is_in_check(&board, Color::Black));
    }

    #[test]
    fn blocked_rook_does_not_give_check() {
        let board = Board::from_fen("4k3/8/8/8/8/8/4p3/4R3 b - - 0 1").expect("valid FEN");

        assert!(!is_in_check(&board, Color::Black));
    }

    #[test]
    fn startpos_has_twenty_legal_moves() {
        let board = Board::startpos();

        assert_eq!(generate_legal_moves(&board).len(), 20);
    }

    #[test]
    fn checked_king_legal_moves_all_resolve_check() {
        let board = Board::from_fen("4k3/8/8/8/8/8/4r3/4K3 w - - 0 1").expect("valid FEN");
        let moves = generate_legal_moves(&board);

        assert!(!moves.is_empty());
        assert!(moves.into_iter().all(|mv| {
            let mut b = board.clone();
            b.make_move(mv);
            !is_in_check(&b, Color::White)
        }));
    }

    #[test]
    fn pinned_rook_cannot_move_off_file() {
        let board = Board::from_fen("4r3/8/8/8/8/8/4R3/4K3 w - - 0 1").expect("valid FEN");
        let rook_square = Square::from_algebraic("e2").expect("valid square");
        let legal_rook_moves: Vec<_> = generate_legal_moves(&board)
            .into_iter()
            .filter(|mv| mv.from == rook_square)
            .collect();

        assert!(!legal_rook_moves.is_empty());
        assert!(
            legal_rook_moves
                .iter()
                .all(|mv| mv.to.file() == rook_square.file())
        );
        assert!(legal_rook_moves.into_iter().all(|mv| {
            let mut b = board.clone();
            b.make_move(mv);
            !is_in_check(&b, Color::White)
        }));
    }

    #[test]
    fn white_pawn_promotes_to_all_piece_types() {
        let board = Board::from_fen("8/P7/8/8/8/8/8/4K3 w - - 0 1").expect("valid FEN");
        let from = square("a7");
        let to = square("a8");

        assert_promotions(&generate_pseudo_legal_moves(&board), from, to);
    }

    #[test]
    fn black_pawn_promotes_to_all_piece_types() {
        let board = Board::from_fen("4k3/8/8/8/8/8/7p/8 b - - 0 1").expect("valid FEN");
        let from = square("h2");
        let to = square("h1");

        assert_promotions(&generate_pseudo_legal_moves(&board), from, to);
    }

    #[test]
    fn promotion_capture_generates_all_piece_types() {
        let board = Board::from_fen("1r6/P7/8/8/8/8/8/4K3 w - - 0 1").expect("valid FEN");
        let from = square("a7");
        let to = square("b8");

        assert_promotions(&generate_pseudo_legal_moves(&board), from, to);
    }

    #[test]
    fn promotion_move_replaces_pawn_with_promoted_piece() {
        let board = Board::from_fen("8/P7/8/8/8/8/8/4K3 w - - 0 1").expect("valid FEN");
        let mut next = board.clone();
        next.make_move(Move::promotion(
            square("a7"),
            square("a8"),
            PieceKind::Knight,
        ));

        assert_eq!(next.piece_at(square("a7")), None);
        assert_eq!(
            next.piece_at(square("a8")).map(|piece| piece.kind),
            Some(PieceKind::Knight)
        );
    }

    #[test]
    fn white_en_passant_is_generated_and_removes_captured_pawn() {
        let board = Board::from_fen("8/8/8/3pP3/8/8/8/4K3 w - d6 0 1").expect("valid FEN");
        let mv = Move::new(square("e5"), square("d6"));

        assert!(generate_pseudo_legal_moves(&board).contains(&mv));

        let mut next = board.clone();
        next.make_move(mv);
        assert_eq!(next.piece_at(square("e5")), None);
        assert_eq!(next.piece_at(square("d5")), None);
        assert_eq!(
            next.piece_at(square("d6")).map(|piece| piece.kind),
            Some(PieceKind::Pawn)
        );
    }

    #[test]
    fn en_passant_exposing_king_to_rook_is_not_legal() {
        let board = Board::from_fen("8/8/8/r2pP2K/8/8/8/8 w - d6 0 1").expect("valid FEN");
        let mv = Move::new(square("e5"), square("d6"));

        assert!(generate_pseudo_legal_moves(&board).contains(&mv));
        assert!(!generate_legal_moves(&board).contains(&mv));
    }

    #[test]
    fn empty_castling_position_generates_both_white_castles() {
        let board = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").expect("valid FEN");
        let moves = generate_legal_moves(&board);

        assert!(moves.contains(&Move::new(square("e1"), square("g1"))));
        assert!(moves.contains(&Move::new(square("e1"), square("c1"))));
    }

    #[test]
    fn empty_castling_position_generates_both_black_castles() {
        let board = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1").expect("valid FEN");
        let moves = generate_legal_moves(&board);

        assert!(moves.contains(&Move::new(square("e8"), square("g8"))));
        assert!(moves.contains(&Move::new(square("e8"), square("c8"))));
    }

    #[test]
    fn blocked_castling_is_not_generated() {
        let board = Board::from_fen("r3k2r/8/8/8/8/8/8/R2BK2R w KQkq - 0 1").expect("valid FEN");
        let moves = generate_legal_moves(&board);

        assert!(moves.contains(&Move::new(square("e1"), square("g1"))));
        assert!(!moves.contains(&Move::new(square("e1"), square("c1"))));
    }

    #[test]
    fn castling_through_attacked_square_is_not_generated() {
        let board = Board::from_fen("r3k2r/8/8/8/8/5r2/8/R3K2R w KQkq - 0 1").expect("valid FEN");
        let moves = generate_legal_moves(&board);

        assert!(is_square_attacked(&board, square("f1"), Color::Black));
        assert!(!moves.contains(&Move::new(square("e1"), square("g1"))));
    }

    #[test]
    fn castling_while_in_check_is_not_generated() {
        let board = Board::from_fen("r3k2r/8/8/8/8/4r3/8/R3K2R w KQkq - 0 1").expect("valid FEN");
        let moves = generate_legal_moves(&board);

        assert!(!moves.contains(&Move::new(square("e1"), square("g1"))));
        assert!(!moves.contains(&Move::new(square("e1"), square("c1"))));
    }

    fn assert_promotions(moves: &[Move], from: Square, to: Square) {
        for promotion in PROMOTION_PIECES {
            assert!(moves.contains(&Move::promotion(from, to, promotion)));
        }
    }
}
