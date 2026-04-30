mod board;
mod movegen;

use board::{Board, Square};
use movegen::{
    generate_pseudo_legal_moves, is_in_check, is_square_attacked, king_square, perft,
    print_moves_for_square,
};

fn main() {
    let board = Board::startpos();
    let pseudo_moves = generate_pseudo_legal_moves(&board);

    println!("MIRANSAS-CHESS");
    println!("startpos: {}", board.to_fen_piece_placement());
    println!("pieces: {}", board.piece_count());
    println!("pseudo moves from startpos: {}", pseudo_moves.len());
    println!("perft depth 1: {}", perft(&board, 1));
    println!(
        "white in check: {}",
        is_in_check(&board, board::Color::White)
    );
    if let Some(white_king) = king_square(&board, board::Color::White) {
        println!(
            "white king attacked: {}",
            is_square_attacked(&board, white_king, board::Color::Black)
        );
    }

    if std::env::var_os("MIRANSAS_DEBUG_MOVES").is_some() {
        let square = Square::from_algebraic("b1").expect("debug square is valid");
        print_moves_for_square(&board, square);
    }
}
