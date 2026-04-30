use crate::board::{Board, Color, PieceKind, Square};

const PAWN_TABLE: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, //
    5, 10, 10, -10, -10, 10, 10, 5, //
    4, 8, 8, 12, 12, 8, 8, 4, //
    2, 4, 6, 15, 15, 6, 4, 2, //
    1, 2, 4, 12, 12, 4, 2, 1, //
    1, 1, 2, -5, -5, 2, 1, 1, //
    0, 0, 0, -8, -8, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0,
];

const KNIGHT_TABLE: [i32; 64] = [
    -30, -20, -10, -10, -10, -10, -20, -30, //
    -20, -5, 0, 5, 5, 0, -5, -20, //
    -10, 5, 15, 20, 20, 15, 5, -10, //
    -10, 10, 20, 30, 30, 20, 10, -10, //
    -10, 5, 20, 30, 30, 20, 5, -10, //
    -10, 0, 10, 15, 15, 10, 0, -10, //
    -20, -5, 0, 0, 0, 0, -5, -20, //
    -30, -20, -10, -10, -10, -10, -20, -30,
];

const BISHOP_TABLE: [i32; 64] = [
    -15, -10, -10, -10, -10, -10, -10, -15, //
    -10, 5, 0, 0, 0, 0, 5, -10, //
    -10, 10, 10, 12, 12, 10, 10, -10, //
    -10, 0, 12, 15, 15, 12, 0, -10, //
    -10, 5, 10, 15, 15, 10, 5, -10, //
    -10, 0, 10, 10, 10, 10, 0, -10, //
    -10, 0, 0, 0, 0, 0, 0, -10, //
    -15, -10, -10, -10, -10, -10, -10, -15,
];

const ROOK_TABLE: [i32; 64] = [
    0, 0, 5, 10, 10, 5, 0, 0, //
    -5, 0, 0, 0, 0, 0, 0, -5, //
    -5, 0, 0, 0, 0, 0, 0, -5, //
    -5, 0, 0, 0, 0, 0, 0, -5, //
    -5, 0, 0, 0, 0, 0, 0, -5, //
    -5, 0, 0, 0, 0, 0, 0, -5, //
    5, 10, 10, 10, 10, 10, 10, 5, //
    0, 0, 0, 5, 5, 0, 0, 0,
];

const QUEEN_TABLE: [i32; 64] = [
    -10, -5, -5, -2, -2, -5, -5, -10, //
    -5, 0, 0, 0, 0, 0, 0, -5, //
    -5, 0, 5, 5, 5, 5, 0, -5, //
    -2, 0, 5, 8, 8, 5, 0, -2, //
    -2, 0, 5, 8, 8, 5, 0, -2, //
    -5, 0, 5, 5, 5, 5, 0, -5, //
    -5, 0, 0, 0, 0, 0, 0, -5, //
    -10, -5, -5, -2, -2, -5, -5, -10,
];

const KING_TABLE: [i32; 64] = [
    20, 25, 10, 0, 0, 10, 25, 20, //
    15, 15, 0, 0, 0, 0, 15, 15, //
    -10, -15, -15, -20, -20, -15, -15, -10, //
    -20, -25, -25, -30, -30, -25, -25, -20, //
    -30, -35, -35, -40, -40, -35, -35, -30, //
    -35, -40, -40, -45, -45, -40, -40, -35, //
    -40, -45, -45, -50, -50, -45, -45, -40, //
    -40, -45, -45, -50, -50, -45, -45, -40,
];

pub fn evaluate(board: &Board) -> i32 {
    let mut white_score = 0;
    let mut black_score = 0;

    for (index, piece) in board.squares.iter().enumerate() {
        let Some(piece) = piece else {
            continue;
        };
        let square = Square::from_index(index as u8);
        let value = piece_value(piece.kind) + piece_square_value(piece.kind, piece.color, square);

        match piece.color {
            Color::White => white_score += value,
            Color::Black => black_score += value,
        }
    }

    match board.side_to_move {
        Color::White => white_score - black_score,
        Color::Black => black_score - white_score,
    }
}

fn piece_square_value(kind: PieceKind, color: Color, square: Square) -> i32 {
    let index = match color {
        Color::White => square.index(),
        Color::Black => mirror_square_vertically(square).index(),
    };

    piece_square_table(kind)[index]
}

fn piece_square_table(kind: PieceKind) -> &'static [i32; 64] {
    match kind {
        PieceKind::Pawn => &PAWN_TABLE,
        PieceKind::Knight => &KNIGHT_TABLE,
        PieceKind::Bishop => &BISHOP_TABLE,
        PieceKind::Rook => &ROOK_TABLE,
        PieceKind::Queen => &QUEEN_TABLE,
        PieceKind::King => &KING_TABLE,
    }
}

fn mirror_square_vertically(square: Square) -> Square {
    Square::from_file_rank(square.file(), 7 - square.rank()).expect("mirrored square is valid")
}

fn piece_value(kind: PieceKind) -> i32 {
    match kind {
        PieceKind::Pawn => 100,
        PieceKind::Knight => 320,
        PieceKind::Bishop => 330,
        PieceKind::Rook => 500,
        PieceKind::Queen => 900,
        PieceKind::King => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startpos_evaluates_to_zero() {
        let board = Board::startpos();

        assert_eq!(evaluate(&board), 0);
    }

    #[test]
    fn missing_black_queen_is_positive_for_white_to_move() {
        let board = Board::from_fen("rnb1kbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .expect("valid FEN");

        assert!(evaluate(&board) > 0);
    }

    #[test]
    fn white_knight_in_center_evaluates_better_than_corner() {
        let center = Board::from_fen("4k3/8/8/8/3N4/8/8/4K3 w - - 0 1").expect("valid FEN");
        let corner = Board::from_fen("4k3/8/8/8/8/8/8/N3K3 w - - 0 1").expect("valid FEN");

        assert!(evaluate(&center) > evaluate(&corner));
    }

    #[test]
    fn mirrored_black_and_white_positions_evaluate_symmetrically() {
        let white = Board::from_fen("4k3/8/8/8/3N4/8/8/4K3 w - - 0 1").expect("valid FEN");
        let black = Board::from_fen("4k3/8/8/8/3n4/8/8/4K3 b - - 0 1").expect("valid FEN");

        assert_eq!(evaluate(&white), evaluate(&black));
    }
}
