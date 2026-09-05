# Starfall Dominion

A menu-driven 4X space war game. The native Rust window draws a procedural
GPU starfield with Lantern UI 2 widgets over it. The only direct external
dependencies are `wgpu` and `winit`; Lantern crates are local path dependencies.

## Run

```sh
cargo run --offline
```

The initial window requests 1280 × 800 logical pixels, with a minimum of
900 × 640, and has no OS titlebar or replacement window-control bar. Winit reads
the display scale from the compositor; the renderer uses physical surface sizes
and keeps stars sized in logical pixels when the scale changes.

- **Start Game:** enter the rough sector map.
- **Click a planet:** open its information panel.
- **Left-click the scout:** select it and preview movement toward the cursor.
- **Right-click the map:** commit the previewed move, capped at remaining range.
- **End Turn:** advance through the enemy's placeholder pass and refresh movement.
- **Drag empty map space / scroll:** pan / zoom.
- **Tab / Shift+Tab, Enter / Space:** navigate and activate widgets.
- **Escape:** deselect a fleet or close planet details, then return to the menu.
- **Menu:** return to the main menu. Starting again resets the rough map.
- **F11:** toggle borderless fullscreen. Use the window manager to close the game.

The backdrop stays still and redraws on demand. There are no downloaded art
assets. Widgets, input translation, 2D rendering, and text use the unmodified
crates in `../lantern-ui-2`. This includes that framework's bundled `lntrn-text`
0.2 engine, rather than the separate Lantern-DE 0.1 engine. Body text is 30 logical
pixels and buttons are 65 logical pixels high, scaled by the compositor.

The map is 1000 × 700 world units; the scout gets 100 units of movement per turn.
Distances are independent of display size, zoom, and compositor scale. Movement
can be split across orders; committed moves resolve immediately and cannot be
cancelled. The turn counter persists during play, but this build has no save/load.

Planet names, positions, sector boundaries, and ownership are provisional layout data. There is no
resource economy, combat, or AI simulation yet. See [the design notes](docs/design.md).
The initial [Action Registry](ActionRegistry.md) describes shared movement and turn rules.

## Development

```sh
cargo fmt --check
cargo clippy --offline --all-targets -- -D warnings
cargo test --offline
cargo build --offline
```

Keep `Cargo.lock` in version control. Build output lives in `target/`.
The interaction tests run headlessly and do not capture the screen.
