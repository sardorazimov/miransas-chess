# Changelog

All notable changes to miransas-chess will be documented here.

## [Unreleased]

### FAZ 1 — Performance Foundation

#### BÖLÜM 1.1 — make_move / unmake_move + incremental zobrist
- Added `Board::zobrist_key: u64`, kept correct incrementally via `make_move`/`unmake_move`.
- Added `Undo` struct and `Board::make_move(&mut self, Move) -> Undo` / `Board::unmake_move(&mut self, &Undo)`.
- Removed `Board::make_move_unchecked` (clone-based). All callers migrated.
- `Zobrist` gained incremental toggle methods (`toggle_piece`, `toggle_side_to_move`, `toggle_castling`, `toggle_en_passant`) and a global `OnceLock` accessor `crate::search::zobrist()`.
- `FEN` parsing now sets `zobrist_key` via `hash_board` after all fields are populated.
- Search, perft, and legal-move generation now use in-place make/unmake on a single mutable board, eliminating per-node board clones in the hot path.
- `generate_legal_moves` keeps `&Board` signature; clones the board once internally for legality checking (N clones → 1 clone per call).
- `perft`/`perft_legal` signatures changed to `&mut Board`; all callers updated.
- Internal search functions (`search_root`, `negamax_alpha_beta_with_ply`, `quiescence`) now take `&mut Board`. Public API (`search_best_move`, `search_iterative`) keeps `&Board`, cloning once per search call.
- Removed `Zobrist` from `SearchContext`; TT lookups now use `board.zobrist_key` directly.
- `extract_pv` keeps clone-based forward traversal (no unmake needed; noted here for transparency).
- Added `make_unmake_restores_board_exactly` reversibility test across 5 standard positions including Kiwipete.
- All 83 tests pass; perft depth 3 from startpos confirmed at 8902 nodes.

#### BÖLÜM 1.2 — Bench harness + CI comparison
- Added `cargo run --release --bin bench` with deterministic perft (5 positions) and search (3 positions) measurements, machine-readable output (`us=` microsecond timing, `nps=` throughput).
- Added `--quick` mode (depths −2) for fast local iteration.
- Added `.github/workflows/bench.yml` that runs comparative bench on every PR (main vs PR head, sticky comment with Δ% table).
- Added `scripts/compare_bench.sh` for the NPS comparison report.
- Perft mismatch detection: if any node count diverges from the standard reference, the bench binary exits non-zero.
- NPS = nodes × 1,000,000 ÷ us; outputs `nps=inf` when us=0 instead of dividing by zero.
- Documented in README.

**Baseline (post-1.1, full bench, local — CI numbers will differ due to shared runner noise):**

```
PERFT startpos depth=5 nodes=4865609 us=435759 nps=11165825
PERFT kiwipete depth=4 nodes=4085603 us=532326 nps=7675001
PERFT pos3 depth=5 nodes=674624 us=85996 nps=7844829
PERFT pos4 depth=4 nodes=422333 us=56059 nps=7533723
PERFT pos5 depth=4 nodes=2103487 us=295699 nps=7113608
SEARCH startpos depth=6 nodes=118073 us=217383 nps=543156
SEARCH kiwipete depth=5 nodes=31895610 us=81543169 nps=391149
SEARCH endgame depth=8 nodes=135826 us=96081 nps=1413661
TOTAL nodes=44301165 us=83262472
```

Timing is in microseconds (us). NPS = nodes × 1,000,000 ÷ us.

#### Setup — CI + tooling
- Created `.github/workflows/ci.yml`: fmt, clippy `-D warnings`, build release, test (two profiles).
- Rewrote `README.md` with current feature list, usage examples, and project structure.

---

## [0.1.0] - 2026-05-01

### Added
- Initial engine release
- Legal move generation with perft validation
- Negamax search
- CI workflow (rust checks)
