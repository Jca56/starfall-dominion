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
the display scale from the compositor; the renderer uses physical surface sizes.

- **Start Game:** enter the rough sector map, looking at the home region.
- **Click a planet:** open its information panel.
- **Left-click the scout:** select it and preview movement toward the cursor.
- **Right-click the map:** commit the previewed move, capped at remaining range.
  The scout glides to its destination; the rules resolve the move instantly.
- **End Turn:** the round button in the bottom-right corner, with the turn
  counter beside it. Advances through the enemy's placeholder pass and refreshes
  movement.
- **Drag empty map space / scroll:** pan / zoom. Zoom eases toward the cursor.
- **Tab / Shift+Tab, Enter / Space:** navigate and activate widgets.
- **Escape:** deselect a fleet or close planet details, then return to the menu.
- **Menu:** the only thing in the slim bar across the top. Returns to the main
  menu; starting again resets the rough map.
- **F11:** toggle borderless fullscreen. Use the window manager to close the game.

The window redraws on input and keeps drawing while the camera or a ship is
moving, then sleeps. There are no downloaded art assets. Widgets, input
translation, 2D rendering, and text use the unmodified crates in
`../lantern-ui-2`, including that framework's bundled `lntrn-text` 0.2 engine.
Body text is 30 logical pixels and buttons are 65 logical pixels high, scaled
by the compositor.

## Map scale

A world unit is abstract: the screen always shows a fixed number of units
across, so what a player feels is ratios, not numbers. The map is 3,000 × 2,000
units. The scout moves 150 units per turn and an average ship will move about
100, so the scout crosses a sector in about seven turns and the map in about
twenty. Neighbouring planets are two or three scout moves apart. Distances are
independent of display size, zoom, and compositor scale. Movement can be split
across orders; committed moves resolve immediately and cannot be cancelled. The
turn counter persists during play, but this build has no save/load.

The camera starts centered on the scout with 450 units across the map area:
three scout moves, so a move is a visible jump and anything on screen is at most
three turns away. Zooming in stops when one move fills the screen. Zooming out
stops when the whole map fits, and below 30% of the starting zoom the map turns
into a star chart: planets shrink to dots and names hide while sector codes
stay. The camera's centre stays on the map, so it can look half a screen past
the edge and still centre a ship at the border; space beyond the border is dimmed.
Resizing the window reveals more or less space without rescaling distances.

The starfield is drawn in screen space as parallax layers. They drift as the
map pans, at a fraction of its speed, and only gently rescale with zoom, so
zooming out never turns them into noise. Menus and HUD stay anchored to the window.

## Fog of war

The map is charted but not surveyed. Every planet's position is known from the
start, since the people of the Expanse mapped their own home, but nothing else
about a world is known until a ship has seen it: unsurveyed planets draw as
faint hollow markers and their panel says "Uncharted". Space you have never seen
is dark, space you have seen but cannot see right now is dim and remembers what
was there, and space in view is lit. The scout sees 225 units, a little past one
move, and charts a corridor along every order it flies; on screen the reveal
follows the ship as it glides. A world you hold keeps
150 units around it in view. You start knowing only what Arcadia and the scout
can see. The chart lives in `src/fog.rs` as a grid of 10-unit cells and is
uploaded to the GPU as a small texture whenever it changes; the starfield shader
darkens the backdrop from it. Points of interest, once they exist, will be
hidden entirely until discovered.

## Sectors and planets

The map is divided into thirteen sectors traced from Alva's sketch: five in the
west, three in the middle, five in the east, so votes for control can never tie.
Their outlines live in `src/layout.rs` as polygons that share corner points, and
a test checks that they tile the map exactly once. Until they are named they
carry placeholder codes: L1 to L5, M1 to M3, R1 to R5.

There are twenty-six planets, two per sector, listed in `src/planets.rs` with
an owner: yours, the Dominion's, or free. Five
have names: Arcadia, the home world next to the scout, plus Haven, Verdant,
Cinder, and Vesper. The rest are placeholders named by sector code, such as
L3-b. Farlight is the name of the whole region, not a planet.

Planet names, positions, sector boundaries, and ownership are provisional
layout data. There is no resource economy, combat, or AI simulation yet. See
[the design notes](docs/design.md). The initial
[Action Registry](ActionRegistry.md) describes shared movement and turn rules.

## Development

```sh
cargo fmt --check
cargo clippy --offline --all-targets -- -D warnings
cargo test --offline
cargo build --offline
```

Keep `Cargo.lock` in version control. Build output lives in `target/`.
The interaction tests run headlessly and do not capture the screen.
