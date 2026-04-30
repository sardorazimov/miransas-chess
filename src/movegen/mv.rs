use std::fmt;

use crate::board::{Board, Color, PieceKind, Square};

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
            Some(kind) => write!(f, "{}{}{:?}", self.from, self.to, kind),
            None => write!(f, "{}{}", self.from, self.to),
        }
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

            return piece.color == by_color && attackers.contains(&piece.kind);
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

    // Pawn captures only exist when an opposing piece is present. En passant is intentionally
    // left out of this foundational generator.
    for file_delta in [-1, 1] {
        if let Some(to) = offset_square(from, file_delta, direction)
            && let Some(target) = board.piece_at(to)
            && target.color != color
        {
            push_pawn_move(from, to, color, moves);
        }
    }
}

fn generate_knight_moves(board: &Board, from: Square, color: Color, moves: &mut Vec<Move>) {
    generate_leaper_moves(board, from, color, &KNIGHT_ATTACK_DELTAS, moves);
}

fn generate_king_moves(board: &Board, from: Square, color: Color, moves: &mut Vec<Move>) {
    generate_leaper_moves(board, from, color, &KING_ATTACK_DELTAS, moves);
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
        moves.push(Move::promotion(from, to, PieceKind::Queen));
    } else {
        moves.push(Move::new(from, to));
    }
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
}
