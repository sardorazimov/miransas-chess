# Web And Mobile Optimization Notes

This folder is separate from the chess engine code. It is a planning area for future website and mobile app work, so optimization ideas can be tracked without mixing them into `src/`.

## Goal

Build a website or mobile app around Miransas Chess that feels fast on low-end phones, does not waste battery, and keeps the chess engine responsive. The UI should stay smooth even while the engine is thinking.

## Website Optimization Checklist

### 1. Load Strategy

- Keep the first screen small: board, basic controls, and engine status.
- Lazy-load non-critical pages such as docs, benchmarks, changelog, settings, and analysis history.
- Split large JavaScript bundles by route or feature.
- Avoid loading the engine/WASM before the user actually needs to analyze or play.
- Cache static assets with long-lived cache headers.

### 2. Rendering Performance

- Keep the chessboard layout stable: fixed square grid, fixed control sizes, no layout shifts.
- Avoid re-rendering the whole board for every clock tick or engine info update.
- Update only changed squares after a move.
- Use requestAnimationFrame for board animations.
- Keep animations short and disable expensive effects on low-power devices.

### 3. Engine Isolation

- Run the chess engine in a Web Worker, not on the main UI thread.
- Send compact messages between UI and worker: FEN, UCI command, bestmove, info lines.
- Do not send large board objects every frame.
- Stop or throttle analysis when the tab is hidden.

### 4. WASM Notes

- Compile release WASM with size optimization for web delivery.
- Load WASM asynchronously and show a clear ready state.
- Cache the WASM file after first load.
- Keep a fallback path for browsers where WASM fails.

### 5. Images And Assets

- Use responsive images for logos and marketing screens.
- Prefer modern formats such as WebP or AVIF when supported.
- Keep piece assets small and cached.
- Avoid heavy backgrounds on the board/play screen.

### 6. Network

- Make the play screen mostly offline-capable.
- Use a service worker only when there is a real caching/offline goal.
- Avoid polling for local-only engine features.
- Batch analytics or disable them during engine search.

### 7. Measurement

- Track Lighthouse performance, accessibility, best practices, and SEO.
- Measure Core Web Vitals: LCP, CLS, INP.
- Test on a real mid-range Android phone, not only desktop Chrome.
- Add a small manual smoke test: open app, make move, engine replies, start/stop analysis, rotate screen.

## Mobile App Optimization Checklist

### 1. Engine Threading

- Run engine search off the UI thread.
- Use a cancellation token or stop flag for analysis.
- Keep the latest completed search result so the app can respond quickly.
- Avoid blocking navigation while the engine is thinking.

### 2. Battery And Heat

- Lower default analysis depth on mobile.
- Add a strength or speed setting.
- Pause analysis when the app goes to background.
- Reduce engine work when battery saver mode is active, if the platform exposes it.

### 3. Memory

- Make Hash size configurable.
- Use a smaller default Hash on mobile than desktop.
- Free large analysis buffers when leaving analysis mode.
- Avoid storing unlimited PGN/history/analysis lines in memory.

### 4. Touch UX

- Board squares must be large enough for fingers.
- Support tap-tap moves and drag moves.
- Highlight legal moves without expensive redraws.
- Keep clocks, status, and controls readable on small screens.

### 5. Offline Behavior

- Local engine play should work without network.
- Opening books, cloud analysis, accounts, or sync can be optional online features.
- Store user settings locally.

### 6. Release Build Settings

- Build native engine code in release mode.
- Strip symbols for production builds where appropriate.
- Keep debug logs disabled by default.
- Add crash logs or error reporting only if privacy and consent are clear.

## Suggested Work Order

1. Build the simplest playable UI: board, move input, engine response.
2. Move engine work into a worker/background thread.
3. Add strict start/stop/deadline handling.
4. Add performance measurement on a real phone.
5. Optimize bundle size and WASM/native engine loading.
6. Add offline caching only after the core app is stable.

## Things To Avoid Early

- Heavy dashboards before the play screen is smooth.
- Large animation systems around the board.
- Always-on infinite analysis on mobile.
- Sending full board state through the UI bridge every small update.
- Optimizing visual polish before search cancellation and UI responsiveness are reliable.

## MVP Definition For Web/Mobile

"A user can open the app on a phone, play a legal game against the engine, see the engine thinking state, stop or restart analysis, and keep a smooth UI without battery-draining defaults."
