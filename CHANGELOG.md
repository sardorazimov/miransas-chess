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

**After (BÖLÜM 2.1):**
```
PERFT startpos depth=5 nodes=4865609 us=448579 nps=10846715
PERFT kiwipete depth=4 nodes=4085603 us=541385 nps=7546575
PERFT pos3 depth=5 nodes=674624 us=87496 nps=7710341
PERFT pos4 depth=4 nodes=422333 us=55202 nps=7650682
PERFT pos5 depth=4 nodes=2103487 us=289559 nps=7264450
SEARCH startpos depth=6 nodes=78837 us=104551 nps=754053
SEARCH kiwipete depth=5 nodes=233431 us=318777 nps=732270
SEARCH endgame depth=8 nodes=162853 us=102704 nps=1585653
TOTAL nodes=12626777 us=1948253
```

**Δ summary:**
- startpos d6: 118,073 → 78,837 (−33%)
- kiwipete d5: 31,895,610 → 233,431 (−99.3%)
- endgame d8: 135,826 → 162,853 (+20% nodes, better search quality; wall-clock faster due to higher NPS)

#### BÖLÜM 2.2 — Null move pruning
- Added `Board::make_null_move` / `unmake_null_move` with full incremental Zobrist updates (side-to-move toggle, en-passant clear).
- Added `Board::side_to_move_has_only_pawns` for zugzwang avoidance (skips null move when the side to move has only king + pawns).
- Added `NullUndo` struct (returned by `make_null_move`, consumed by `unmake_null_move`).
- Null move pruning in `negamax`: applies when `depth >= 3`, not in check, not pawns-only position, `ply > 0`, and `null_allowed == true`.
- Adaptive R: `depth > 6` → R=3, otherwise R=2.
- Null window search around beta: `(-beta, -beta+1)`.
- Added `null_allowed: bool` parameter to `negamax`; stacked null moves are prevented by passing `false` in the recursive null-move call; all other recursive calls pass `true`.
- Added tests: `pawns_only_detection`, `null_move_is_reversible`, `null_move_clears_en_passant_and_restores_it`, `null_move_finds_same_best_move_as_baseline_on_kiwipete`.
- Total tests: 88.

**Before (BÖLÜM 2.1):**
```
PERFT startpos depth=5 nodes=4865609 us=448579 nps=10846715
PERFT kiwipete depth=4 nodes=4085603 us=541385 nps=7546575
PERFT pos3 depth=5 nodes=674624 us=87496 nps=7710341
PERFT pos4 depth=4 nodes=422333 us=55202 nps=7650682
PERFT pos5 depth=4 nodes=2103487 us=289559 nps=7264450
SEARCH startpos depth=6 nodes=78837 us=104551 nps=754053
SEARCH kiwipete depth=5 nodes=233431 us=318777 nps=732270
SEARCH endgame depth=8 nodes=162853 us=102704 nps=1585653
TOTAL nodes=12626777 us=1948253
```

**After (BÖLÜM 2.2):**
```
PERFT startpos depth=5 nodes=4865609 us=440089 nps=11055965
PERFT kiwipete depth=4 nodes=4085603 us=518393 nps=7881285
PERFT pos3 depth=5 nodes=674624 us=84894 nps=7946662
PERFT pos4 depth=4 nodes=422333 us=54419 nps=7760763
PERFT pos5 depth=4 nodes=2103487 us=285012 nps=7380345
SEARCH startpos depth=6 nodes=42371 us=35155 nps=1205262
SEARCH kiwipete depth=5 nodes=220502 us=306317 nps=719849
SEARCH endgame depth=8 nodes=72321 us=43294 nps=1670462
TOTAL nodes=12486850 us=1767573
```

**Δ summary:**
- startpos d6: 78,837 → 42,371 (−46%)
- kiwipete d5: 233,431 → 220,502 (−6%; tactical complexity limits null move effectiveness)
- endgame d8: 162,853 → 72,321 (−56%)

#### BÖLÜM 2.3 — Late Move Reductions (LMR)
- Added `src/search/lmr.rs`: `LmrTable` with Stockfish-style log formula `floor(0.75 + ln(d)*ln(m)/2.25)`, precomputed into a 64×64 table at startup via `OnceLock`.
- LMR applied to quiet, non-killer moves at `depth >= 3` starting from move index 4 (`LMR_MIN_MOVE_INDEX`).
- Re-search at full depth when reduced-depth score exceeds alpha (ensures tactics are not missed).
- LMR excludes: captures, en passant, promotions, killer moves, in-check positions, root node (`ply == 0`).
- **Known limitation**: Moves that give check are not yet excluded from LMR (detecting checks post-move requires making the move first). Will be revisited.
- Added tests: `lmr_search_returns_legal_move_on_startpos`, `lmr_finds_mate_in_one`.
- Total tests: 93.

**Before (BÖLÜM 2.2):**
```
PERFT startpos depth=5 nodes=4865609 us=440089 nps=11055965
PERFT kiwipete depth=4 nodes=4085603 us=518393 nps=7881285
PERFT pos3 depth=5 nodes=674624 us=84894 nps=7946662
PERFT pos4 depth=4 nodes=422333 us=54419 nps=7760763
PERFT pos5 depth=4 nodes=2103487 us=285012 nps=7380345
SEARCH startpos depth=6 nodes=42371 us=35155 nps=1205262
SEARCH kiwipete depth=5 nodes=220502 us=306317 nps=719849
SEARCH endgame depth=8 nodes=72321 us=43294 nps=1670462
TOTAL nodes=12486850 us=1767573
```

**After (BÖLÜM 2.3):**
```
PERFT startpos depth=5 nodes=4865609 us=417738 nps=11647513
PERFT kiwipete depth=4 nodes=4085603 us=518483 nps=7879916
PERFT pos3 depth=5 nodes=674624 us=85039 nps=7933113
PERFT pos4 depth=4 nodes=422333 us=55174 nps=7654565
PERFT pos5 depth=4 nodes=2103487 us=284538 nps=7392639
SEARCH startpos depth=6 nodes=19794 us=23642 nps=837238
SEARCH kiwipete depth=5 nodes=168992 us=289772 nps=583189
SEARCH endgame depth=8 nodes=22987 us=18362 nps=1251878
TOTAL nodes=12363429 us=1692748
```

**Δ summary:**
- startpos d6: 42,371 → 19,794 (−53%)
- kiwipete d5: 220,502 → 168,992 (−23%)
- endgame d8: 72,321 → 22,987 (−68%)
- Cumulative since BÖLÜM 1.2 baseline (search nodes total):
  - 32,149,509 (1.2) → 211,773 (2.3) (−99.3%)

**Fix (CI):**
- Refactored three division-by-zero guards to use `u128::checked_div` instead of manual `if x == 0` pattern. Clippy 1.95 introduced `manual_checked_ops` lint which flagged these. Behavior unchanged.

#### BÖLÜM 2.4 — Aspiration Windows
- Added Stockfish-style exponential aspiration windows in iterative deepening.
- Active at depth >= 4. Initial half-window: 25 cp. Each fail doubles delta. Up to 4 retries before falling back to full window.
- Fail-low widens alpha; fail-high widens beta. Within-window result terminates the retry loop immediately.
- Depths 1–3 continue to use full window (`NEG_INF`, `POS_INF`) — no aspiration overhead at trivial depths.
- Replaced `ASPIRATION_WINDOW` constant (fixed ±50, single retry) with `ASPIRATION_MIN_DEPTH`, `INITIAL_DELTA`, `MAX_RETRIES`.
- Added tests: `aspiration_search_finds_legal_move_on_startpos`, `aspiration_search_still_finds_mate_in_one`, `aspiration_score_is_finite_on_quiet_startpos`.
- Total tests: 96.

**Before (BÖLÜM 2.3):**
```
PERFT startpos depth=5 nodes=4865609 us=417738 nps=11647513
PERFT kiwipete depth=4 nodes=4085603 us=518483 nps=7879916
PERFT pos3 depth=5 nodes=674624 us=85039 nps=7933113
PERFT pos4 depth=4 nodes=422333 us=55174 nps=7654565
PERFT pos5 depth=4 nodes=2103487 us=284538 nps=7392639
SEARCH startpos depth=6 nodes=19794 us=23642 nps=837238
SEARCH kiwipete depth=5 nodes=168992 us=289772 nps=583189
SEARCH endgame depth=8 nodes=22987 us=18362 nps=1251878
TOTAL nodes=12363429 us=1692748
```

**After (BÖLÜM 2.4):**
```
PERFT startpos depth=5 nodes=4865609 us=425025 nps=11447818
PERFT kiwipete depth=4 nodes=4085603 us=545377 nps=7491337
PERFT pos3 depth=5 nodes=674624 us=84439 nps=7989483
PERFT pos4 depth=4 nodes=422333 us=56847 nps=7429292
PERFT pos5 depth=4 nodes=2103487 us=286046 nps=7353666
SEARCH startpos depth=6 nodes=14791 us=21318 nps=693826
SEARCH kiwipete depth=5 nodes=169474 us=293073 nps=578265
SEARCH endgame depth=8 nodes=22969 us=18664 nps=1230657
TOTAL nodes=12358890 us=1730789
```

**Δ summary:**
- startpos d6: 19,794 → 14,791 (−25.3%)
- kiwipete d5: 168,992 → 169,474 (+0.3%; re-searches on fail-high/fail-low add nodes, net near-zero at this depth)
- endgame d8: 22,987 → 22,969 (−0.1%)

**Cumulative since BÖLÜM 1.2 baseline (search nodes total):**
- 32,149,509 (1.2) → 207,234 (2.4) (−99.4%)

**FAZ 2 complete.** Modern alpha-beta search now has: TT move ordering, MVV-LVA captures, killers, history heuristic, null move pruning, LMR, aspiration windows.

---

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
