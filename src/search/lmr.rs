use std::sync::OnceLock;

const LMR_MAX_DEPTH: usize = 64;
const LMR_MAX_MOVES: usize = 64;

pub struct LmrTable {
    table: [[i32; LMR_MAX_MOVES]; LMR_MAX_DEPTH],
}

static LMR_TABLE: OnceLock<LmrTable> = OnceLock::new();

pub fn lmr_table() -> &'static LmrTable {
    LMR_TABLE.get_or_init(LmrTable::new)
}

impl LmrTable {
    #[allow(clippy::needless_range_loop)]
    pub fn new() -> Self {
        let mut table = [[0i32; LMR_MAX_MOVES]; LMR_MAX_DEPTH];
        for depth in 1..LMR_MAX_DEPTH {
            for move_idx in 1..LMR_MAX_MOVES {
                let d = depth as f64;
                let m = move_idx as f64;
                let reduction = 0.75 + (d.ln() * m.ln()) / 2.25;
                table[depth][move_idx] = reduction.floor().max(0.0) as i32;
            }
        }
        Self { table }
    }

    pub fn reduction(&self, depth: i32, move_index: usize) -> i32 {
        let d = (depth as usize).min(LMR_MAX_DEPTH - 1);
        let m = move_index.min(LMR_MAX_MOVES - 1);
        self.table[d][m]
    }
}

impl Default for LmrTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reduction_is_zero_at_low_depth_and_low_move_index() {
        let lmr = LmrTable::new();
        assert_eq!(lmr.reduction(1, 1), 0);
        assert_eq!(lmr.reduction(2, 1), 0);
    }

    #[test]
    fn reduction_grows_with_depth_and_move_index() {
        let lmr = LmrTable::new();
        let r_small = lmr.reduction(4, 4);
        let r_big = lmr.reduction(20, 30);
        assert!(
            r_big > r_small,
            "reduction should grow with depth and move_index"
        );
    }

    #[test]
    fn reduction_is_bounded_and_sane() {
        let lmr = LmrTable::new();
        // At depth 20, move 20: roughly 0.75 + log(20)*log(20)/2.25 ≈ 4.74 -> floor = 4
        let r = lmr.reduction(20, 20);
        assert!(
            (3..=6).contains(&r),
            "reduction at d=20 m=20 should be reasonable, got {r}"
        );
    }
}
