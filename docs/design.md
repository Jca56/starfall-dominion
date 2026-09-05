# Starfall Dominion — design notes

## Confirmed direction

- A 4X-style game: the player versus one enemy AI, the Starfall Dominion.
- Turn-based play: the player acts, ends their turn, the enemy acts, and play returns to the player.
- Fleets move freely through space, rather than along fixed planet connections.
- Left-clicking a fleet selects it and shows a movement line toward the cursor.
  The endpoint is capped at the fleet's available movement distance; destinations
  beyond that range move the fleet as far as it can go in that direction.
- Right-click commits the move. Distance traveled is deducted from a per-turn
  movement budget that may be split across several orders.
- Committed moves finish without cancellation or interruption, even if enemies
  would be revealed en route. This is a provisional simplicity rule.
- The Dominion has conquered nearly the entire sector.
- The Farlight Expanse is the last bastion of freedom in the galaxy's outer
  regions: a few midsized, mostly peaceful planets.
- Gameplay centers on collecting and managing resources, building armies,
  defending planets, and securing new planets to expand influence.
- The interface is primarily menu-based: a large navigable map, contextual
  panels when selecting objects, and a persistent information bar along the top.
- Main menu: a left-side panel with Start Game.
- No OS titlebar or custom window controls; Alva uses compositor shortcuts.

## Current scaffold

Start Game opens a 3,000 × 2,000 unit map with thirteen sectors traced from
Alva's sketch (five west, three middle, five east), twenty-six selectable
planets, and a player scout. Units are abstract; what matters is the ratios.
The scout moves 150 units per turn (an average ship will move about 100), so
neighbouring planets are two or three turns apart, a sector takes about seven
turns to cross, and the map about twenty. The camera starts three scout moves
wide so a move is a visible jump, and pulls back into a star chart when zoomed
far out. These dimensions and movement values are provisional. Five planets are named: Arcadia is the home world; Farlight is
the region, not a planet. The other twenty-one are placeholders named by sector
code, two worlds per sector. Western worlds are free, eastern worlds are
Dominion-held, and the middle is contested. These counts, names, positions, and
descriptions are placeholders for exploring the layout, not settled lore or game
rules. Planet details currently show a name, allegiance, sector, and description.
Fog of war follows Alva's Age of Wonders reading: planet positions are always
charted, but what is there stays unknown until surveyed; charted space out of
view dims and remembers; future points of interest hide entirely until found.
The scout sees 225 units and charts a corridor along each order; held worlds
see 150. The game starts knowing only what Arcadia and the scout can see.
Moves resolve immediately in this first version; the ship glides to its
destination on screen while the rules have already placed it there. The camera
starts on the home region, eases toward zoom and pan targets, and never leaves
the map. The starfield is a screen-space parallax backdrop rather than a
world-anchored texture. End Turn hands control to the
Dominion, which currently passes, then increments the counter and refills player
movement. There is no enemy planner, fog of war, or save/load yet.

## Still open

- What actions a turn permits and how much in-universe time it represents.
- Resource types, production, and upkeep.
- Armies, fleets, travel, and how battles resolve.
- Exploration, visibility, and victory/defeat conditions.
- How the Dominion pressures the player as the campaign progresses.

## Working proposals

Alva's [Mechanics.md](../Mechanics.md) and [Roadmap.md](../Roadmap.md) describe
initial ideas to discuss and evolve, not binding rules. They propose west/east
starting positions, sector influence from majority planet ownership, fog of war,
fixed key locations mixed with procedural planets and points of interest, fleets,
and a small resource economy. Planet acquisition is still undecided.

The revised roadmap introduces an Action Registry alongside the first turn loop
and movable fleet, ahead of fog of war and procedural generation. The proposed
registry is implemented in `src/actions.rs` with typed actions, metadata, shared
read-only previews/validation, and execution. See [ActionRegistry.md](../ActionRegistry.md).
