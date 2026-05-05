# Working.md - MVP Status Notes

Date: 2026-05-05

## Short Verdict

The project is progressing better than expected. It is no longer just a toy engine: it already has a UCI loop, legal move generation, FEN support, make/unmake, incremental Zobrist hashing, a transposition table, iterative search, aspiration windows, null move pruning, LMR, benchmarking, and a serious test surface.

For MVP, the main missing pieces are not about whether the engine can play chess. The bigger gaps are reliability in real GUI games, better time-control handling, draw-rule awareness, and clearer regression checks.

## Current Strengths

- Legal move generation covers special moves: castling, en passant, promotion, check, and pin scenarios.
- `make_move` / `unmake_move` reversibility tests exist across standard positions.
- Zobrist hashing is maintained incrementally and is integrated with the search flow.
- Search is already strong for an MVP: negamax, alpha-beta, quiescence, TT, move ordering, killers, history heuristic, null move pruning, LMR, and aspiration windows.
- CLI is usable: `demo`, `uci`, `bench`, `perft`, `search`, and JSON output.
- `cargo test` passed cleanly during review. In that run, the `bench` binary had 87 passing tests and the main binary had 110 passing tests.

## MVP Target

MVP definition:

"A Rust chess engine that can be added to a UCI-compatible GUI, does not generate illegal moves, responds under basic time controls, can be validated with perft, and can be played from a release build."

By that definition, the project is already close to MVP. The remaining work is listed below.

## Missing Before MVP

### 1. UCI Time Controls

The engine currently supports `go depth` and `go movetime`. For smoother GUI use, it should also support:

- Parse `go wtime btime winc binc`.
- Choose the correct remaining time based on side to move.
- Use simple time allocation: a small fraction of remaining time plus part of increment.
- Search currently checks time between completed depths, but not inside the search tree. A single deep iteration can still overshoot `movetime`.

MVP priority: high.

### 2. Stop Behavior

The `stop` command is parsed, but search is synchronous, so the engine cannot interrupt an active search immediately.

Two possible MVP levels:

- Basic MVP: acceptable if `go movetime` becomes reliable enough.
- Stronger MVP: add a search thread or shared stop flag.

MVP priority: medium-high.

### 3. Fifty-Move Rule And Repetition

`Board` has `halfmove_clock`, but search terminal scoring does not yet handle 50-move draws or repetition.

Missing pieces:

- Treat the 50-move rule as a terminal draw score.
- Track position history for threefold repetition.
- Preserve history while applying UCI `position ... moves ...`.

Not strictly required for a short MVP, but it prevents strange game endings in GUIs.

MVP priority: medium.

### 4. Search Deadline Model

`go movetime` currently checks time between depths. If one depth takes too long, the engine can exceed its time budget.

Needed:

- Add a deadline or stop flag to search context.
- Check it periodically inside `negamax` and `quiescence`.
- On timeout, return the last fully completed iterative result.

MVP priority: high.

### 5. UCI Options

The engine is not yet configurable from GUI settings.

Useful minimum:

- `option name Hash type spin default 16 min 1 max ...`
- `setoption name Hash value N`
- Optional: `Clear Hash`

MVP priority: medium.

### 6. Perft FEN Support

CLI `perft <depth>` currently runs only from startpos. Debugging would be easier with:

- `perft <depth> [fen]`
- Optional divide mode later.

MVP priority: medium.

### 7. Quality Gate

`cargo test` is clean, which is good. MVP release should have a fixed checklist:

- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`
- `cargo run --release --bin bench -- --quick`
- At least one GUI smoke test: from startpos, `go movetime 1000` should return a legal `bestmove`.

MVP priority: high.

### 8. README / SETUP Test Count

README and SETUP mention "96 tests". The latest observed run showed 87 tests for the `bench` binary and 110 tests for the main binary, which can confuse readers.

MVP priority: low.

### 9. Playing Strength Tracking

Bench output tracks nodes and NPS, but there is no Elo or self-play regression loop yet.

Post-MVP ideas:

- CuteChess self-play script.
- Old build vs new build mini match.
- SPRT or a simpler win/draw/loss report.

MVP priority: low-medium.

## Suggested Work Order Before MVP

1. Parse UCI `go wtime/btime/winc/binc`.
2. Add deadline / stop checks to search.
3. Return the last completed depth for `go movetime` and normal time controls.
4. Add `Hash` UCI option and `setoption`.
5. Run and document the MVP quality gate.
6. Update README/SETUP test counts and supported UCI commands.
7. Run a short GUI smoke test.

## Can Stay Out Of MVP

- Full bitboard rewrite / the rest of Phase 3 optimization.
- SEE.
- NNUE.
- MultiPV.
- Opening book.
- Ponder.
- Syzygy tablebases.
- Full repetition support, if the first MVP is only meant for short GUI games.

## General Feeling

The project is in good shape. The main risk is no longer "can the engine produce moves?" It is "can the engine answer what a real GUI asks during real games?" Once time management and deadline/stop behavior are handled, this can comfortably wear an MVP label.
