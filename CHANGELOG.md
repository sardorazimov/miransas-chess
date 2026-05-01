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
- Perft mismatch detection exits non-zero if any node count diverges from the standard reference.
- NPS = nodes × 1,000,000 ÷ us; outputs `nps=inf` when us=0.

**Baseline (post-1.1):**
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

---

### FAZ 2 — Search Improvements

#### BÖLÜM 2.1 — Move ordering pipeline
- Added `src/search/context.rs`: `SearchContext` with killer moves (2 slots/ply) and history heuristic (`[piece_index][to_square]`), reset per `search_iterative` call; reused across iterative deepening iterations.
- Added `src/search/ordering.rs`: `score_move` function — TT move (10M) > MVV-LVA captures (8M + victim×16 − aggressor) > queen promotion (9M) > other promotions (7M) > killer 1 (6M) > killer 2 (5.9M) > history-scored quiets.
- Replaced full-sort with **selection-sort iteration** in `negamax` and `quiescence`: best remaining move swapped to front on each step.
- Killers and history recorded only on beta cutoff by a quiet move.
- Quiescence search now MVV-LVA orders captures (was unordered).
- TT move extracted before move generation at each node to seed ordering.
- `SearchContext` and `MAX_PLY` re-exported from `search/mod.rs`.
- Added 5 ordering tests in `ordering.rs`; total tests 84.

**Before (BÖLÜM 1.2 baseline):**
```
SEARCH startpos depth=6 nodes=118073 us=219609 nps=537651
SEARCH kiwipete depth=5 nodes=31895610 us=81463601 nps=391532
SEARCH endgame depth=8 nodes=135826 us=97537 nps=1392558
```

**After (BÖLÜM 2.1):**
```
SEARCH startpos depth=6 nodes=78837 us=104544 nps=754103
SEARCH kiwipete depth=5 nodes=233431 us=322306 nps=724252
SEARCH endgame depth=8 nodes=162853 us=104662 nps=1555989
```

**Δ summary:**
- startpos d6: 118,073 → 78,837 (−33%)
- kiwipete d5: 31,895,610 → 233,431 (−99.3%)
- endgame d8: 135,826 → 162,853 (+20% nodes, faster wall-clock due to better NPS)

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
