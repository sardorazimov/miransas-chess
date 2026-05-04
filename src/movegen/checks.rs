#![allow(dead_code)]

use crate::board::{Board, Color, PieceKind, Square};

/// Information about who, if anyone, is giving check to a king.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CheckerInfo {
    /// Number of pieces giving check (0, 1, or 2).
    pub count: u8,
    /// Squares of the checking pieces. Only the first `count` entries are valid.
    pub squares: [Option<Square>; 2],
}

impl CheckerInfo {
    pub fn none() -> Self {
        Self {
            count: 0,
            squares: [None, None],
        }
    }

    pub fn is_in_check(&self) -> bool {
        self.count > 0
    }

    pub fn is_double_check(&self) -> bool {
        self.count >= 2
    }

    pub fn single_checker(&self) -> Option<Square> {
        if self.count == 1 {
            self.squares[0]
        } else {
            None
        }
    }
}

/// Pin information for one side. For each square on the board, if a friendly
/// piece is there AND it is pinned, stores the direction of the pin ray.
/// Otherwise stores None.
///
/// The pin ray is represented as a (file_delta, rank_delta) unit vector,
/// e.g. (1, 0) for a horizontal pin, (0, 1) for vertical, (1, 1) for diagonal.
#[derive(Clone, Copy, Debug)]
pub struct PinMask {
    pub rays: [Option<(i8, i8)>; 64],
}

impl PinMask {
    pub fn empty() -> Self {
        Self { rays: [None; 64] }
    }

    pub fn is_pinned(&self, square: Square) -> bool {
        self.rays[square.index()].is_some()
    }

    /// Returns the pin ray direction for a pinned piece, or None if not pinned.
    /// Direction is normalized to a unit vector with components in {-1, 0, 1}.
    pub fn pin_ray(&self, square: Square) -> Option<(i8, i8)> {
        self.rays[square.index()]
    }

    fn set(&mut self, square: Square, dir: (i8, i8)) {
        self.rays[square.index()] = Some(dir);
    }
}

impl Default for PinMask {
    fn default() -> Self {
        Self::empty()
    }
}

// ── direction tables ──────────────────────────────────────────────────────────

const ROOK_DIRS: [(i8, i8); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
const BISHOP_DIRS: [(i8, i8); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
const KNIGHT_DELTAS: [(i8, i8); 8] = [
    (1, 2),
    (2, 1),
    (2, -1),
    (1, -2),
    (-1, -2),
    (-2, -1),
    (-2, 1),
    (-1, 2),
];
const KING_DELTAS: [(i8, i8); 8] = [
    (0, 1),
    (1, 1),
    (1, 0),
    (1, -1),
    (0, -1),
    (-1, -1),
    (-1, 0),
    (-1, 1),
];

// ── helpers ───────────────────────────────────────────────────────────────────

fn offset_sq(sq: Square, df: i8, dr: i8) -> Option<Square> {
    let f = sq.file() as i8 + df;
    let r = sq.rank() as i8 + dr;
    if (0..8).contains(&f) && (0..8).contains(&r) {
        Square::from_file_rank(f as u8, r as u8)
    } else {
        None
    }
}

fn slider_attacks(board: &Board, from: Square, dirs: &[(i8, i8)], target: Square) -> bool {
    for &(df, dr) in dirs {
        let mut cur = from;
        while let Some(sq) = offset_sq(cur, df, dr) {
            if sq == target {
                return true;
            }
            if board.squares[sq.index()].is_some() {
                break;
            }
            cur = sq;
        }
    }
    false
}

/// Returns true if the piece of `color` and `kind` at `from` geometrically
/// attacks `target`, respecting board blocking for sliders.
fn piece_attacks_square(
    board: &Board,
    from: Square,
    color: Color,
    kind: PieceKind,
    target: Square,
) -> bool {
    match kind {
        PieceKind::Pawn => {
            let rank_dir: i8 = match color {
                Color::White => 1,
                Color::Black => -1,
            };
            [-1i8, 1i8]
                .iter()
                .any(|&fd| offset_sq(from, fd, rank_dir) == Some(target))
        }
        PieceKind::Knight => KNIGHT_DELTAS
            .iter()
            .any(|&(df, dr)| offset_sq(from, df, dr) == Some(target)),
        PieceKind::King => KING_DELTAS
            .iter()
            .any(|&(df, dr)| offset_sq(from, df, dr) == Some(target)),
        PieceKind::Bishop => slider_attacks(board, from, &BISHOP_DIRS, target),
        PieceKind::Rook => slider_attacks(board, from, &ROOK_DIRS, target),
        PieceKind::Queen => {
            slider_attacks(board, from, &BISHOP_DIRS, target)
                || slider_attacks(board, from, &ROOK_DIRS, target)
        }
    }
}

// ── public API ────────────────────────────────────────────────────────────────

/// Return information about pieces of `attacker_color` giving check to the
/// king of the opposite color located at `king_square`.
pub fn find_checkers(board: &Board, king_square: Square, attacker_color: Color) -> CheckerInfo {
    let mut info = CheckerInfo::none();

    for index in 0u8..64 {
        let sq = Square::from_index(index);
        let Some(piece) = board.squares[index as usize] else {
            continue;
        };
        if piece.color != attacker_color {
            continue;
        }
        if piece_attacks_square(board, sq, piece.color, piece.kind, king_square) {
            if info.count < 2 {
                info.squares[info.count as usize] = Some(sq);
            }
            info.count = info.count.saturating_add(1);
        }
    }

    if info.count > 2 {
        info.count = 2;
    }
    info
}

/// Compute the pin mask for the side defending `king_square`. Pinning pieces
/// belong to `attacker_color`. Pinned pieces (which we mark) are friendly to
/// the king (i.e., NOT `attacker_color`).
pub fn compute_pin_mask(board: &Board, king_square: Square, attacker_color: Color) -> PinMask {
    let mut mask = PinMask::empty();
    let king_file = king_square.file() as i8;
    let king_rank = king_square.rank() as i8;

    for index in 0u8..64 {
        let slider_sq = Square::from_index(index);
        let Some(slider) = board.squares[index as usize] else {
            continue;
        };
        if slider.color != attacker_color {
            continue;
        }
        let is_diag_slider = matches!(slider.kind, PieceKind::Bishop | PieceKind::Queen);
        let is_orth_slider = matches!(slider.kind, PieceKind::Rook | PieceKind::Queen);
        if !is_diag_slider && !is_orth_slider {
            continue;
        }

        let df = king_file - slider_sq.file() as i8;
        let dr = king_rank - slider_sq.rank() as i8;

        // Direction from slider toward king — must be orthogonal or diagonal.
        let (step_f, step_r) = if df == 0 && dr != 0 {
            (0i8, dr.signum())
        } else if dr == 0 && df != 0 {
            (df.signum(), 0i8)
        } else if df.abs() == dr.abs() && df != 0 {
            (df.signum(), dr.signum())
        } else {
            continue; // not on a straight line to the king
        };

        let is_diag_step = step_f != 0 && step_r != 0;
        if is_diag_step && !is_diag_slider {
            continue;
        }
        if !is_diag_step && !is_orth_slider {
            continue;
        }

        // Walk from slider toward king. If exactly one friendly piece is
        // encountered before the king, that piece is pinned.
        let mut pinned_candidate: Option<Square> = None;
        let mut cur = slider_sq;
        while let Some(walk_sq) = offset_sq(cur, step_f, step_r) {
            if walk_sq == king_square {
                if let Some(pinned_sq) = pinned_candidate {
                    // Store direction from king toward pinner (opposite of step).
                    mask.set(pinned_sq, (-step_f, -step_r));
                }
                break;
            }
            match board.squares[walk_sq.index()] {
                None => {}
                Some(piece) if piece.color == attacker_color => break, // enemy blocker
                Some(_) => {
                    if pinned_candidate.is_some() {
                        break; // two friendly pieces — no pin
                    }
                    pinned_candidate = Some(walk_sq);
                }
            }
            cur = walk_sq;
        }
    }

    mask
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Board;

    fn sq(algebraic: &str) -> Square {
        Square::from_algebraic(algebraic).expect("valid square")
    }

    #[test]
    fn no_checks_in_quiet_position() {
        let board = Board::startpos();
        let king_sq = sq("e1");
        let checkers = find_checkers(&board, king_sq, Color::Black);
        assert_eq!(checkers.count, 0);
        assert!(!checkers.is_in_check());
    }

    #[test]
    fn single_check_by_rook() {
        let board = Board::from_fen("4r3/8/8/8/8/8/8/4K3 w - - 0 1").expect("valid FEN");
        let king_sq = sq("e1");
        let checkers = find_checkers(&board, king_sq, Color::Black);
        assert_eq!(checkers.count, 1);
        assert_eq!(checkers.single_checker(), Some(sq("e8")));
    }

    #[test]
    fn no_check_when_blocker_on_ray() {
        let board = Board::from_fen("4r3/8/8/8/4P3/8/8/4K3 w - - 0 1").expect("valid FEN");
        let king_sq = sq("e1");
        let checkers = find_checkers(&board, king_sq, Color::Black);
        assert_eq!(checkers.count, 0);
    }

    #[test]
    fn double_check_by_discovered_attack_position() {
        let board = Board::from_fen("8/8/8/8/8/5n2/8/r3K3 w - - 0 1").expect("valid FEN");
        let king_sq = sq("e1");
        let checkers = find_checkers(&board, king_sq, Color::Black);
        assert_eq!(checkers.count, 2);
        assert!(checkers.is_double_check());
    }

    #[test]
    fn knight_checks_correctly() {
        let board = Board::from_fen("8/8/8/8/8/5n2/8/4K3 w - - 0 1").expect("valid FEN");
        let king_sq = sq("e1");
        let checkers = find_checkers(&board, king_sq, Color::Black);
        assert_eq!(checkers.count, 1);
        assert_eq!(checkers.single_checker(), Some(sq("f3")));
    }

    #[test]
    fn pawn_checks_diagonally() {
        let board = Board::from_fen("8/8/8/8/8/8/3p4/4K3 w - - 0 1").expect("valid FEN");
        let king_sq = sq("e1");
        let checkers = find_checkers(&board, king_sq, Color::Black);
        assert_eq!(checkers.count, 1);
        assert_eq!(checkers.single_checker(), Some(sq("d2")));
    }

    #[test]
    fn no_pins_in_quiet_startpos() {
        let board = Board::startpos();
        let king_sq = sq("e1");
        let pins = compute_pin_mask(&board, king_sq, Color::Black);
        for i in 0..64 {
            assert!(pins.rays[i].is_none(), "square {i} should not be pinned");
        }
    }

    #[test]
    fn rook_pins_friendly_piece_horizontally() {
        let board = Board::from_fen("8/8/8/8/8/8/8/r2NK3 w - - 0 1").expect("valid FEN");
        let king_sq = sq("e1");
        let pins = compute_pin_mask(&board, king_sq, Color::Black);
        assert!(pins.is_pinned(sq("d1")), "knight on d1 should be pinned");
        assert!(
            !pins.is_pinned(sq("e1")),
            "king should not be marked pinned"
        );
    }

    #[test]
    fn bishop_pins_friendly_piece_diagonally() {
        let board = Board::from_fen("8/8/8/8/7b/8/5N2/4K3 w - - 0 1").expect("valid FEN");
        let king_sq = sq("e1");
        let pins = compute_pin_mask(&board, king_sq, Color::Black);
        assert!(
            pins.is_pinned(sq("f2")),
            "knight on f2 should be pinned diagonally"
        );
    }

    #[test]
    fn queen_pins_along_orthogonal_and_diagonal() {
        let orth = Board::from_fen("8/8/8/8/8/8/8/q2NK3 w - - 0 1").expect("valid FEN");
        let pins_orth = compute_pin_mask(&orth, sq("e1"), Color::Black);
        assert!(pins_orth.is_pinned(sq("d1")));

        let diag = Board::from_fen("8/8/8/8/7q/8/5N2/4K3 w - - 0 1").expect("valid FEN");
        let pins_diag = compute_pin_mask(&diag, sq("e1"), Color::Black);
        assert!(pins_diag.is_pinned(sq("f2")));
    }

    #[test]
    fn knight_does_not_pin() {
        let board = Board::from_fen("8/4n3/8/8/4P3/8/8/4K3 w - - 0 1").expect("valid FEN");
        let pins = compute_pin_mask(&board, sq("e1"), Color::Black);
        assert!(!pins.is_pinned(sq("e4")), "pawn cannot be pinned by knight");
    }

    #[test]
    fn two_friendly_pieces_between_means_no_pin() {
        let board = Board::from_fen("4r3/8/8/8/4P3/8/4P3/4K3 w - - 0 1").expect("valid FEN");
        let pins = compute_pin_mask(&board, sq("e1"), Color::Black);
        assert!(!pins.is_pinned(sq("e2")));
        assert!(!pins.is_pinned(sq("e4")));
    }

    #[test]
    fn enemy_blocker_breaks_pin() {
        let board = Board::from_fen("4r3/8/8/8/4n3/8/8/4K3 w - - 0 1").expect("valid FEN");
        let pins = compute_pin_mask(&board, sq("e1"), Color::Black);
        for i in 0..64 {
            assert!(pins.rays[i].is_none());
        }
    }

    #[test]
    fn pin_ray_direction_is_normalized() {
        let board = Board::from_fen("8/8/8/8/8/8/8/r2NK3 w - - 0 1").expect("valid FEN");
        let pins = compute_pin_mask(&board, sq("e1"), Color::Black);
        let ray = pins.pin_ray(sq("d1")).expect("pinned");
        assert_eq!(
            ray.0.abs() + ray.1.abs(),
            1,
            "horizontal/vertical pin: one component is ±1, the other is 0"
        );
    }
}
