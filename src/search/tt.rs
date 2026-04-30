use crate::movegen::Move;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bound {
    Exact,
    Lower,
    Upper,
}

#[derive(Clone, Copy, Debug)]
pub struct TTEntry {
    pub key: u64,
    pub depth: u32,
    pub score: i32,
    pub bound: Bound,
    pub best_move: Option<Move>,
}

pub struct TranspositionTable {
    entries: Vec<Option<TTEntry>>,
}

impl TranspositionTable {
    pub fn new(size_mb: usize) -> Self {
        let bytes = size_mb.saturating_mul(1024).saturating_mul(1024);
        let entry_size = std::mem::size_of::<Option<TTEntry>>().max(1);
        let entry_count = bytes / entry_size;

        Self {
            entries: vec![None; entry_count],
        }
    }

    pub fn clear(&mut self) {
        self.entries.fill(None);
    }

    pub fn get(&self, key: u64) -> Option<TTEntry> {
        if self.entries.is_empty() {
            return None;
        }

        self.entries[(key as usize) % self.entries.len()].filter(|entry| entry.key == key)
    }

    pub fn store(&mut self, entry: TTEntry) {
        if self.entries.is_empty() {
            return;
        }

        let index = (entry.key as usize) % self.entries.len();
        let should_replace = self.entries[index].is_none_or(|old| entry.depth >= old.depth);

        if should_replace {
            self.entries[index] = Some(entry);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Square;

    #[test]
    fn tt_store_get_works() {
        let mut tt = TranspositionTable::new(1);
        let entry = entry(42, 3, 100);

        tt.store(entry);

        assert_eq!(tt.get(42).map(|found| found.score), Some(100));
    }

    #[test]
    fn deeper_entry_replaces_shallow_entry() {
        let mut tt = TranspositionTable::new(1);

        tt.store(entry(42, 1, 100));
        tt.store(entry(42, 3, 300));

        assert_eq!(tt.get(42).map(|found| found.depth), Some(3));
        assert_eq!(tt.get(42).map(|found| found.score), Some(300));
    }

    #[test]
    fn shallow_entry_does_not_replace_deeper_entry() {
        let mut tt = TranspositionTable::new(1);

        tt.store(entry(42, 3, 300));
        tt.store(entry(42, 1, 100));

        assert_eq!(tt.get(42).map(|found| found.depth), Some(3));
        assert_eq!(tt.get(42).map(|found| found.score), Some(300));
    }

    fn entry(key: u64, depth: u32, score: i32) -> TTEntry {
        TTEntry {
            key,
            depth,
            score,
            bound: Bound::Exact,
            best_move: Some(Move::new(square("e2"), square("e4"))),
        }
    }

    fn square(algebraic: &str) -> Square {
        Square::from_algebraic(algebraic).expect("test square is valid")
    }
}
