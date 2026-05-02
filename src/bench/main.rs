#![allow(clippy::module_inception, dead_code, unused_imports)]

#[path = "../board/mod.rs"]
mod board;
#[path = "../evaluation/mod.rs"]
mod evaluation;
#[path = "../movegen/mod.rs"]
mod movegen;
#[path = "../search/mod.rs"]
mod search;

use std::process::ExitCode;
use std::time::Instant;

use board::Board;
use movegen::perft_legal;
use search::search_iterative;

const PERFT_POSITIONS: &[(&str, &str, u32, u64)] = &[
    (
        "startpos",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        5,
        4_865_609,
    ),
    (
        "kiwipete",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        4,
        4_085_603,
    ),
    (
        "pos3",
        "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        5,
        674_624,
    ),
    (
        "pos4",
        "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
        4,
        422_333,
    ),
    (
        "pos5",
        "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
        4,
        2_103_487,
    ),
];

const SEARCH_POSITIONS: &[(&str, &str, u32)] = &[
    (
        "startpos",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        6,
    ),
    (
        "kiwipete",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        5,
    ),
    ("endgame", "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", 8),
];

fn nps(nodes: u64, us: u128) -> String {
    (nodes as u128 * 1_000_000)
        .checked_div(us)
        .map(|n| (n as u64).to_string())
        .unwrap_or_else(|| "inf".to_string())
}

fn main() -> ExitCode {
    let quick = std::env::args().any(|a| a == "--quick");
    let depth_reduce = if quick { 2 } else { 0 };

    let mut total_nodes: u64 = 0;
    let mut total_us: u128 = 0;
    let mut had_mismatch = false;

    for &(name, fen, depth, expected) in PERFT_POSITIONS {
        let actual_depth = depth.saturating_sub(depth_reduce);
        let mut board = Board::from_fen(fen).expect("valid bench FEN");
        let start = Instant::now();
        let nodes = perft_legal(&mut board, actual_depth);
        let us = start.elapsed().as_micros();
        let speed = nps(nodes, us);

        if !quick && nodes != expected {
            eprintln!("PERFT MISMATCH {name} expected={expected} got={nodes}");
            had_mismatch = true;
        }

        println!("PERFT {name} depth={actual_depth} nodes={nodes} us={us} nps={speed}");
        total_nodes += nodes;
        total_us += us;
    }

    for &(name, fen, depth) in SEARCH_POSITIONS {
        let actual_depth = depth.saturating_sub(depth_reduce);
        let board = Board::from_fen(fen).expect("valid bench FEN");
        let start = Instant::now();
        let result = search_iterative(&board, actual_depth, 16);
        let us = start.elapsed().as_micros();
        let speed = nps(result.nodes, us);

        println!(
            "SEARCH {name} depth={actual_depth} nodes={} us={us} nps={speed}",
            result.nodes
        );
        total_nodes += result.nodes;
        total_us += us;
    }

    println!("TOTAL nodes={total_nodes} us={total_us}");

    if had_mismatch {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
