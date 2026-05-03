# SETUP — miransas-chess

## Prerequisites

| Tool | Minimum version |
|------|----------------|
| Rust toolchain | 1.85 (edition 2024) |
| Cargo | ships with Rust |

Install Rust via [rustup.rs](https://rustup.rs):

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

---

## Build

```sh
git clone https://github.com/sardorazimov/miransas-chess.git
cd miransas-chess

cargo build --release
```

Binary lands at `target/release/miransas-chess`.

---

## CLI reference

| Command | What it does |
|---------|-------------|
| `cargo run --release -- uci` | Start UCI loop (connect a GUI to this) |
| `cargo run --release -- demo` | Search starting position, print best move |
| `cargo run --release -- perft <depth>` | Perft node count from start position |
| `cargo run --release -- perft <depth> --json` | Same, JSON output |
| `cargo run --release -- search <depth>` | Search start position to given depth |
| `cargo run --release -- search <depth> "<FEN>"` | Search a specific FEN |
| `cargo run --release --bin bench` | Run benchmark suite |

---

## UCI GUI setup

### Arena (Windows/Linux/Wine)

1. **Engines → Install New Engine** → select `miransas-chess.exe` (or binary)
2. Set engine type to **UCI**
3. Click **OK** — engine is ready

### CuteChess (cross-platform)

1. **Tools → Settings → Engines → Add**
2. Command: path to `miransas-chess` binary
3. Protocol: **UCI**
4. Save and create a new game against the engine

### En Croissant (cross-platform, modern)

1. **Engines → Add engine → Custom**
2. Path: `target/release/miransas-chess`
3. Protocol: UCI — save

---

## Tests

```sh
cargo test
```

96 tests covering: move generation, perft node counts, make/unmake reversibility,
Zobrist hashing, UCI parsing, TT behavior, search correctness, mate detection,
aspiration windows, null move pruning, LMR.

---

## Project structure

```
src/
├── board/       board representation, FEN parsing, make/unmake
├── movegen/     move generation, perft
├── search/      negamax, TT, Zobrist, move ordering, LMR
├── evaluation/  material + piece-square tables
├── bench/       benchmark runner
├── uci/         UCI protocol loop and command parser
└── main.rs      CLI entry point
```

---

## UCI protocol — quick reference

```
uci                         → engine info + uciok
isready                     → readyok
position startpos
position fen <FEN>
position startpos moves e2e4 e7e5
go depth 10
go movetime 3000
```
