# NNUE Demo Notes

This folder is not connected to the real engine code. Its purpose is to explore how NNUE could be written for Miransas Chess using small, readable examples first.

## What Is NNUE?

NNUE is a fast neural-network evaluation method used by chess engines instead of, or alongside, handcrafted evaluation. The core idea:

- Convert a board position into many simple features.
- Add the active features into the first layer.
- When a move is made, avoid recomputing the whole network. Update only the changed features in an accumulator.
- Search visits millions of nodes, so incremental evaluation can save a lot of time.

## Suggested MVP Architecture For Miransas

A first NNUE version can be built in this order:

1. Keep the current handcrafted evaluation unchanged.
2. Add NNUE as a separate module later, for example `src/evaluation/nnue.rs`.
3. Implement inference first; training can come later.
4. Start with a simple feature set such as `(perspective, piece, square)` or a HalfKP-like `(perspective, piece, square, king_square)` layout.
5. Use fixed demo arrays for weights at first.
6. Later, load weights from `.nnue` or a small custom binary format.
7. Only after correctness is proven, connect accumulator updates to make/unmake.

## First Demo Goal

Before touching the real engine, this folder demonstrates two things:

- How feature indexes can be generated.
- How an accumulator can be built with full refresh and then updated incrementally.

The examples here are standalone Rust files. They should not be imported by the engine.

## Files

- `feature_index_demo.rs`: Example feature index generation from color, piece, square, and perspective.
- `accumulator_demo.rs`: Tiny weight table showing full refresh and add/remove accumulator updates.

## Real Implementation Notes

- NNUE evaluation must be fast and allocation-free in the search hot path.
- Do not embed the accumulator into `Board` immediately. Test it separately first.
- Model move effects clearly: add piece, remove piece, move piece.
- Promotion, capture, en passant, and castling need dedicated tests.
- First correctness test: incremental accumulator result must always match full refresh.
- NNUE score should return from the side-to-move perspective, matching the current `evaluate` behavior.

## Post-MVP Training Idea

Training should be a separate pipeline:

- Collect a position dataset: FEN plus score or game result.
- Train with Python/PyTorch rather than Rust at first.
- Quantize weights and export them into the format the engine can read.
- Keep the engine focused on inference only.

A custom trainer is not required at the beginning. First, get inference architecture and accumulator correctness right.
