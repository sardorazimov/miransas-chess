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
