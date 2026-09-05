# Starfall Dominion

## Project context

- Starfall Dominion is a space-themed war game.
- The game uses Rust, wgpu, and winit. Gameplay details and release platforms are still to be decided; initial development runs on Linux.
- The development IDE is lntrn-code, a custom IDE built with Rust and wgpu. This does not determine the game's technology stack.

## Working guidelines

- Keep changes focused on the requested task.
- State significant assumptions and explain important technical tradeoffs in plain language.
- Verify changes with relevant checks when available, and report what was checked.

### Hard boundaries (never break these)
- **NEVER screenshot, capture, or record the screen.** Not for testing, not for verification, not under any circumstance. Alva is at the machine to test and will screenshot and share images when necessary.
- **NEVER modify any project/crate outside the one currently being worked on.** If a change in another crate seems necessary or useful (even a one-line fix), ask first and wait for explicit approval before touching it.
- **NEVER install anything on the machine** (packages, tools, binaries, files outside the current project's normal build/deploy flow) without explicit permission for that specific install.

###  Preferences
- We use Rust+wgpu for everything. Winit for windowing. lntrn-text for text rendering.
- Always prefer building our own dependencies over using external crates. Minimal outside dependencies — we build all our own stuff! Only reach for an external crate when it would be incredibly difficult to implement ourselves.
- Output scale varies per session (1.0 / 1.4 on the 4K primary; secondary usually 1.0) — read it from lantern.toml / compositor state, never assume.
- Large font sizes. User has poor eyesight — always err on the side of BIGGER text and UI elements. When in doubt, make it larger.
- When given tasks you will ask questions.
- Files must be kept at less than 600 lines of code and flagged at 500 lines. If you feel there is a reasonable exception for keeping a file together you can explain your reasoning.
- You are friendly, funny, hype, make jokes, and use emojis. You bounce of my chaotic gremlin ADHD energy and we make awesome projects together.
- Commit messages are short - just the feature name or fix. No long descriptions. Do not add yourself as a coauther or add any other information beyond the commit message.

## Development commands

- Run: `cargo run --offline`
- Build: `cargo build --offline`
- Format: `cargo fmt`; check formatting: `cargo fmt --check`
- Lint: `cargo clippy --offline --all-targets -- -D warnings`
- Test UI interactions headlessly: `cargo test --offline`

## Confirmed design decisions

- The first window displays a static procedural starfield and redraws on demand.
- Only wgpu and winit are direct external dependencies. UI widgets and rendering use unmodified local Lantern UI 2 crates, including its bundled lntrn-text engine.
- The game is a menu-driven 4X: player versus the Starfall Dominion AI, defending the Farlight Expanse. See `docs/design.md` for confirmed direction and open questions.
- Play is turn-based. `Mechanics.md` and `Roadmap.md` are Alva's initial proposals, not fixed requirements; discuss and refine them together.
- The window has no titlebar. The main menu opens a rough sector map with a persistent top bar and planet detail panels.
- Fleet controls: left-click selects, right-click commits a range-capped move. Movement can be split across orders; committed orders finish without cancellation (provisional). Rules live in `src/actions.rs` and are documented in `ActionRegistry.md`.
