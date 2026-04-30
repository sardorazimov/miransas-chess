use std::time::Instant;

use crate::{
    board::Board,
    search::{IterativeSearchResult, search_iterative},
};

const BENCH_FENS: [&str; 5] = [
    Board::STARTPOS_FEN,
    "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",
    "4k3/8/8/3q4/8/2N5/8/4K3 w - - 0 1",
    "7k/4P3/8/8/8/8/8/4K3 w - - 0 1",
    "r2q1rk1/ppp2ppp/2n2n2/3pp3/2B1P3/2NP1N2/PPP2PPP/R1BQ1RK1 w - - 0 8",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BenchResult {
    pub positions: usize,
    pub depth: u32,
    pub total_nodes: u64,
    pub total_tt_hits: u64,
    pub elapsed_ms: u128,
    pub nps: u64,
}

pub fn run_bench(depth: u32) -> BenchResult {
    let start = Instant::now();
    let mut total_nodes = 0;
    let mut total_tt_hits = 0;
    let mut positions = 0;

    for fen in BENCH_FENS {
        let board = Board::from_fen(fen).expect("bench FEN is valid");
        let result: IterativeSearchResult = search_iterative(&board, depth, 16);
        total_nodes += result.nodes;
        total_tt_hits += result.tt_hits;
        positions += 1;
    }

    let elapsed_ms = start.elapsed().as_millis();
    let nps = if elapsed_ms == 0 {
        total_nodes
    } else {
        ((total_nodes as u128 * 1000) / elapsed_ms) as u64
    };

    BenchResult {
        positions,
        depth,
        total_nodes,
        total_tt_hits,
        elapsed_ms,
        nps,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bench_runs_and_has_positions() {
        let result = run_bench(1);

        assert!(result.positions > 0);
    }

    #[test]
    fn bench_returns_nodes() {
        let result = run_bench(1);

        assert!(result.total_nodes > 0);
    }
}
