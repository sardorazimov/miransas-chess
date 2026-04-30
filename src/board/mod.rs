#![allow(clippy::module_inception)]

pub mod board;
pub mod color;
pub mod fen;
pub mod piece;
pub mod square;

pub use board::{Board, CastlingRights};
pub use color::Color;
pub use fen::FenError;
pub use piece::{Piece, PieceKind};
pub use square::Square;
