# Roadmap

## The Map
Everything revolves around the map so it gets locked in first.

### Milestone 1

- [x] Distance sacle/units
- [x] Map size / borders
- [x] Rough sector layout
- [x] A handful of planets

### Milestone 2

- [x] Turn system started
        - Start the "Action Registery" where every action a player can take on their turn documented in order to provide things like tips, warnings, reminders, and for the AI to reference for making decisions and ending their turn.
                1. Move units up to their Movement Speed.
- [x] Fleet that can move up to a certain distance.
- [x] Persistant Turn Counter and "End Turn" button in the bottom right corner.

Initial implementation: 1000 × 700 world units, three rough sector divisions,
five placeholder planets, and one scout with 100 movement units per turn.
These are provisional values. The Dominion currently passes its turn; see
[ActionRegistry.md](ActionRegistry.md) for the shared action rules.


### Milestone 3

- [ ] Fog of War
- [ ] Minimap
- [ ] Basic Proc Gen

## Turn System
- [ ] Determine what can be done in a turn
- [ ] Determine in universe time scale
