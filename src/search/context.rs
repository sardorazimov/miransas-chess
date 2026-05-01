use crate::board::{Color, PieceKind};
use crate::movegen::Move;

pub const MAX_PLY: usize = 64;

pub struct SearchContext {
    pub killers: [[Option<Move>; 2]; MAX_PLY],
    pub history: [[i32; 64]; 12],
}

impl SearchContext {
    pub fn new() -> Self {
        Self {
            killers: [[None; 2]; MAX_PLY],
            history: [[0; 64]; 12],
        }
    }

    pub fn record_killer(&mut self, ply: usize, mv: Move) {
        if ply >= MAX_PLY {
            return;
        }
        if self.killers[ply][0] == Some(mv) {
            return;
        }
        self.killers[ply][1] = self.killers[ply][0];
        self.killers[ply][0] = Some(mv);
    }

    pub fn is_killer(&self, ply: usize, mv: Move) -> bool {
        if ply >= MAX_PLY {
            return false;
        }
        self.killers[ply][0] == Some(mv) || self.killers[ply][1] == Some(mv)
    }

    pub fn record_history(&mut self, piece_idx: usize, to: usize, depth: i32) {
        let bonus = depth * depth;
        let entry = &mut self.history[piece_idx][to];
        *entry = entry.saturating_add(bonus);
        if *entry > 1_000_000 {
            *entry = 1_000_000;
        }
    }

    pub fn history_score(&self, piece_idx: usize, to: usize) -> i32 {
        self.history[piece_idx][to]
    }
}

impl Default for SearchContext {
    fn default() -> Self {
        Self::new()
    }
}

pub fn piece_index(color: Color, kind: PieceKind) -> usize {
    let c = match color {
        Color::White => 0,
        Color::Black => 1,
    };
    let k = match kind {
        PieceKind::Pawn => 0,
        PieceKind::Knight => 1,
        PieceKind::Bishop => 2,
        PieceKind::Rook => 3,
        PieceKind::Queen => 4,
        PieceKind::King => 5,
    };
    c * 6 + k
}
