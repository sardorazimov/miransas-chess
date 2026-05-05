// Standalone NNUE feature-index demo.
// This file is intentionally not imported by the engine.
#![allow(dead_code)]

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Color {
    White,
    Black,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

const PIECE_TYPES: usize = 6;
const COLORS: usize = 2;
const SQUARES: usize = 64;

fn piece_index(piece: Piece) -> usize {
    match piece {
        Piece::Pawn => 0,
        Piece::Knight => 1,
        Piece::Bishop => 2,
        Piece::Rook => 3,
        Piece::Queen => 4,
        Piece::King => 5,
    }
}

fn mirror_square_for_black(square: usize) -> usize {
    let file = square % 8;
    let rank = square / 8;
    (7 - rank) * 8 + file
}

fn orient_square(perspective: Color, square: usize) -> usize {
    match perspective {
        Color::White => square,
        Color::Black => mirror_square_for_black(square),
    }
}

fn feature_index(perspective: Color, piece_color: Color, piece: Piece, square: usize) -> usize {
    let oriented_square = orient_square(perspective, square);
    let oriented_color = if piece_color == perspective {
        0
    } else {
        1
    };

    ((oriented_color * PIECE_TYPES + piece_index(piece)) * SQUARES) + oriented_square
}

fn main() {
    let e2 = 12;
    let e7 = 52;

    let white_pawn_from_white = feature_index(Color::White, Color::White, Piece::Pawn, e2);
    let black_pawn_from_white = feature_index(Color::White, Color::Black, Piece::Pawn, e7);
    let black_pawn_from_black = feature_index(Color::Black, Color::Black, Piece::Pawn, e7);

    println!("white perspective, white pawn e2: {white_pawn_from_white}");
    println!("white perspective, black pawn e7: {black_pawn_from_white}");
    println!("black perspective, black pawn e7: {black_pawn_from_black}");

    assert_eq!(white_pawn_from_white, black_pawn_from_black);
    assert_eq!(COLORS * PIECE_TYPES * SQUARES, 768);
}
