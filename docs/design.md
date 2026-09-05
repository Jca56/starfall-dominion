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

Start Game opens a 1000 × 700 unit map with five selectable planets, three rough
sector divisions, and a player scout. The scout has 100 movement units per turn.
These dimensions and movement values are provisional. Three worlds are free and
two are occupied. These counts, planet names, positions,
and descriptions are placeholders for exploring the layout, not settled lore or
game rules. Planet details currently show a name, allegiance, and description.
Moves resolve immediately in this first version. End Turn hands control to the
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
