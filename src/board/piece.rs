use super::Color;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Piece {
    pub color: Color,
    pub kind: PieceKind,
}

impl Piece {
    pub fn from_fen_char(ch: char) -> Option<Self> {
        let color = if ch.is_ascii_uppercase() {
            Color::White
        } else {
            Color::Black
        };

        let kind = match ch.to_ascii_lowercase() {
            'p' => PieceKind::Pawn,
            'n' => PieceKind::Knight,
            'b' => PieceKind::Bishop,
            'r' => PieceKind::Rook,
            'q' => PieceKind::Queen,
            'k' => PieceKind::King,
            _ => return None,
        };

        Some(Self { color, kind })
    }

    pub fn to_fen_char(self) -> char {
        let ch = match self.kind {
            PieceKind::Pawn => 'p',
            PieceKind::Knight => 'n',
            PieceKind::Bishop => 'b',
            PieceKind::Rook => 'r',
            PieceKind::Queen => 'q',
            PieceKind::King => 'k',
        };

        match self.color {
            Color::White => ch.to_ascii_uppercase(),
            Color::Black => ch,
        }
    }
}
