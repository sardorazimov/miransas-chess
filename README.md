<div align="center">
  <img src="https://github.com/sardorazimov/miransas-chess/blob/master/assets/logo.png" alt="miransas-chess" width="340">
</div>

<br>

<div align="center">
  A UCI chess engine written in Rust — built in measured layers, correctness first.
</div>

<br>

---

**Search:** Negamax · Alpha-Beta · Iterative Deepening · Aspiration Windows · Null Move · LMR  
**Ordering:** TT move → MVV-LVA → Killers → History  
**Eval:** Material + Piece-Square Tables  
**Protocol:** UCI — works with Arena, CuteChess, En Croissant and any UCI-compatible GUI

---

```sh
cargo build --release
cargo run --release -- uci     # UCI mode
cargo run --release -- demo    # quick sanity check
cargo test                     # 96 tests
```

Full setup, GUI integration, and CLI reference → [SETUP.md](SETUP.md)  
Development history → [CHANGELOG.md](CHANGELOG.md)

---

**Roadmap:** FAZ 3 (bitboards) · FAZ 4 (50-move / repetition / SEE) · FAZ 5 (NNUE)

*Sardor Azimov — [Miransas](https://github.com/sardorazimov) project*
