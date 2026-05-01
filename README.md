# miransas-chess
! <img src="https://github.com/sardorazimov/miransas-chess/blob/master/assets/logo.png" alt="Logo" width="400">
! <img src="https://github.com/sardorazimov/miransas-chess/blob/master/assets/miransas-logo.png" alt="Miransas" width="200"> 

A UCI chess engine written in Rust. Focused on correctness, clean architecture, and performance built in measured layers.

---

## Features

- **Board representation** — 64-square array with full FEN parsing (all six fields)
- **Legal move generation** — pseudo-legal generation + legality filter; handles castling, en passant, and promotions
- **Perft validation** — startpos depth 1–4 verified against known node counts
- **Negamax search** — alpha-beta pruning, iterative deepening, aspiration windows
- **Transposition table** — configurable size (MB), depth-preferred replacement
- **Move ordering** — TT move, promotions, MVV-LVA captures, killer moves, history heuristic
- **Quiescence search** — captures and promotions at leaf nodes
- **Zobrist hashing** — incremental updates via `make_move`/`unmake_move` (no per-node board clone)
- **Material evaluation** — piece values + piece-square tables
- **UCI protocol** — `position`, `go depth`, `go movetime`, `bestmove`
- **CLI interface** — demo, bench, perft, search commands with optional JSON output

---

## Build

Requires Rust stable (2026 edition).

```sh
cargo build --release
```

---

## Usage

### UCI mode (for GUI / arena)

```sh
cargo run --release -- uci
```

### Demo (quick sanity check)

```sh
cargo run --release -- demo
```

### Perft

```sh
cargo run --release -- perft 4
cargo run --release -- perft 4 --json
```

### Search

```sh
# Search startpos at depth 5
cargo run --release -- search 5

# Search a custom FEN
cargo run --release -- search 5 "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"

# JSON output
cargo run --release -- search 5 --json
```

### Bench

```sh
cargo run --release -- bench
cargo run --release -- bench 4
cargo run --release -- bench --json
```

---

## Tests

```sh
cargo test --release
```

83 tests cover move generation, perft node counts, search correctness, TT behavior, UCI parsing, Zobrist hashing, and make/unmake reversibility across 5 standard positions (including Kiwipete).

---

## Project structure

```
src/
├── board/        board representation, FEN parsing, make/unmake
├── movegen/      move generation, perft
├── search/       negamax, transposition table, Zobrist hashing
├── evaluation/   material + piece-square evaluation
├── bench/        benchmark runner
├── uci/          UCI protocol loop and command parser
└── main.rs       CLI entry point
```

---

## Roadmap

See [CHANGELOG.md](CHANGELOG.md) for completed work.

Planned improvements (FAZ 1 → FAZ 5):
- Bitboard representation (attack lookups, magic numbers)
- Null-move pruning, late-move reductions
- Static exchange evaluation
- Syzygy tablebase support
- NNUE evaluation
- 3-fold repetition and 50-move rule

---

## Author

Sardor Azimov — part of the Miransas project.
