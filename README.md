# Starfall Dominion

A menu-driven 4X space war game. The native Rust window draws a procedural
GPU starfield with Lantern UI 2 widgets over it. The only direct external
dependencies are `wgpu` and `winit`; Lantern crates are local path dependencies.

## Run

```sh
cargo run --offline
```

The game launches borderless fullscreen with no OS titlebar or window controls;
F11 drops to a 1280 × 800 logical window (minimum 900 × 640) and back. Winit
reads the display scale from the compositor; the renderer uses physical surface
sizes. The pointer is Alva's prism from `assets/cursor.png`, decoded with
Lantern's image crate and scaled to 96 logical pixels tall for the display.

- **Start Game:** enter the rough sector map, looking at the home region.
- **Left-click a planet:** open its details panel on the left of the map: a
  portrait beside the name, turns-to-secure and decay as icons with numbers,
  the Secure Planet button when it isn't yours, what it pays per turn as icons,
  then who holds it and its sector. Arcadia's panel has a Shipyard tab. Every
  icon names itself on hover. Clicking anywhere outside closes the panel.
- **Left-click a ship:** select it and preview movement toward the cursor.
- **Left-click again:** commit the previewed move, capped at remaining range.
  Clicking a planet with a ship selected sends the ship toward it. The ship
  glides to its destination; the rules resolve the move instantly.
- **Right-click:** deselect the ship or close the panel.
- **End Turn:** the round button in the bottom-right corner, with the turn
  counter beside it. Advances through the enemy's placeholder pass and refreshes
  movement.
- **Right-drag / scroll:** pan / zoom. Zoom eases toward the cursor.
- **Tab / Shift+Tab, Enter / Space:** navigate and activate widgets.
- **Escape:** deselect a fleet or close planet details, then return to the menu.
- **Menu:** the only thing in the slim bar across the top. Returns to the main
  menu; starting again resets the rough map.
- **F11:** toggle between fullscreen and a window. Use the window manager to close the game.

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

## The first gameplay loop

Secure, earn, build. A ship that holds within 200 units of a planet you don't
own can start a securing effort from the planet's popup. Each End Turn with a
ship still in range and no enemy in range adds a turn of influence; the planet
is yours once that reaches its secure count (3 to 5). With no ship in range the
influence decays (1 or 2 a turn) and the effort ends when it hits zero. A
securing planet shows its progress as an arc on the map.

Every planet you hold pays its yield of Alloys and Advanced Electronics when
you end your turn, unless an enemy ship sits within 200 units, in which case it
pays nothing that turn. The Command Strip at the top shows each resource as an
icon, what you hold, and what arrives next turn in parentheses; hover an icon
for its name. Arcadia's Shipyard tab builds a Scout for
10 Alloys and 5 Electronics, paid up front; it launches beside Arcadia two turns
later with full movement. All of these numbers are placeholders in
`src/planets.rs` and `src/economy.rs`. The rules live in the Action Registry so
previews, buttons, and the future AI share one set of checks.

## Points of interest

Each sector holds one point of interest at a time, placed by a seeded generator
of our own (`src/rng.rs`) so the same seed always makes the same map; Start Game
seeds from the clock. A point spawns inside its sector, at least 150 units from
any planet or other point, and never inside your current sight, so a respawn is
something you find. The scout's own sector gets the starter, 260 to 330 units
from the scout: just out of sight, one move away. Points hide until a ship has
charted their cell, then stay on the map. The rules are in `src/poi.rs`.

The first kind is the Destroyed Space Station. Click one for its panel, and
press Salvage while one of your ships is within 100 units: it pays 6 to 12
Alloys and, 35% of the time, 2 to 4 Advanced Electronics, then it is gone and
the sector's clock starts. 10 to 20 turns later a new point appears somewhere
in that sector out of sight. Numbers are placeholders in `src/poi.rs`.

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
hidden entirely until discovered, and they are: see points of interest above.

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
