//! The first gameplay loop: hold a ship near a planet to secure it, secured
//! planets pay resources each turn, and resources build ships at the home world.
use lntrn_math::Vec2;

use crate::planets::PLANETS;
use crate::world::{Fleet, Game, SCOUT_SPEED, SCOUT_VISION, Side};

/// A ship must hold within this many units of a planet to secure it, and an
/// enemy within it contests the planet.
pub(crate) const SECURE_RANGE: f64 = 200.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Resource {
    Alloys,
    Electronics,
}

impl Resource {
    pub(crate) const ALL: [Resource; 2] = [Resource::Alloys, Resource::Electronics];

    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Alloys => "Alloys",
            Self::Electronics => "Advanced Electronics",
        }
    }
}

/// Amounts of each resource: a stockpile, a cost, or a per-turn yield.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Resources {
    pub(crate) alloys: u32,
    pub(crate) electronics: u32,
}

impl Resources {
    pub(crate) const fn new(alloys: u32, electronics: u32) -> Self {
        Self {
            alloys,
            electronics,
        }
    }

    pub(crate) fn get(self, resource: Resource) -> u32 {
        match resource {
            Resource::Alloys => self.alloys,
            Resource::Electronics => self.electronics,
        }
    }

    pub(crate) fn plus(self, other: Self) -> Self {
        Self::new(
            self.alloys + other.alloys,
            self.electronics + other.electronics,
        )
    }

    pub(crate) fn covers(self, cost: Self) -> bool {
        self.alloys >= cost.alloys && self.electronics >= cost.electronics
    }

    pub(crate) fn minus(self, cost: Self) -> Self {
        Self::new(
            self.alloys.saturating_sub(cost.alloys),
            self.electronics.saturating_sub(cost.electronics),
        )
    }

    #[cfg(test)]
    pub(crate) fn is_empty(self) -> bool {
        self == Self::default()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ShipKind {
    Scout,
}

impl ShipKind {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Scout => "Scout",
        }
    }

    /// Placeholder numbers from the Roadmap; Alva will tune them.
    pub(crate) fn cost(self) -> Resources {
        match self {
            Self::Scout => Resources::new(10, 5),
        }
    }

    pub(crate) fn build_turns(self) -> u32 {
        match self {
            Self::Scout => 2,
        }
    }
}

/// A ship under construction at the home world.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Build {
    pub(crate) kind: ShipKind,
    pub(crate) turns_left: u32,
}

/// What changes about a planet during play; the rest lives in `PLANETS`.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PlanetState {
    pub(crate) owner: Option<Side>,
    /// Turns of influence built up toward securing it for Farlight.
    pub(crate) influence: u32,
    /// A securing effort is under way.
    pub(crate) securing: bool,
}

impl Game {
    pub(crate) fn home_planet() -> usize {
        PLANETS
            .iter()
            .position(|planet| planet.home)
            .expect("a home world")
    }

    pub(crate) fn fleet_in_range(&self, side: Side, position: Vec2) -> bool {
        self.fleets
            .iter()
            .any(|f| f.owner == side && (f.position - position).length() <= SECURE_RANGE)
    }

    /// An enemy of `side` is close enough to contest the planet.
    pub(crate) fn enemy_in_range(&self, side: Side, position: Vec2) -> bool {
        self.fleets
            .iter()
            .any(|f| f.owner != side && (f.position - position).length() <= SECURE_RANGE)
    }

    /// A held planet pays nothing while an enemy ship sits within range.
    pub(crate) fn contested(&self, planet: usize) -> bool {
        match self.planets[planet].owner {
            Some(owner) => self.enemy_in_range(owner, PLANETS[planet].position),
            None => false,
        }
    }

    /// What Farlight will receive when this turn ends.
    pub(crate) fn income(&self) -> Resources {
        PLANETS
            .iter()
            .enumerate()
            .filter(|(index, _)| {
                self.planets[*index].owner == Some(Side::Player) && !self.contested(*index)
            })
            .fold(Resources::default(), |sum, (_, planet)| {
                sum.plus(planet.yield_per_turn)
            })
    }

    /// Everything that resolves when the player ends a turn: securing efforts
    /// tick, held worlds pay out, and the shipyard works.
    pub(crate) fn end_player_turn(&mut self) {
        for index in 0..PLANETS.len() {
            self.advance_securing(index);
        }
        self.stockpile = self.stockpile.plus(self.income());
        let launched = match &mut self.shipyard {
            Some(build) => {
                build.turns_left -= 1;
                (build.turns_left == 0).then_some(build.kind)
            }
            None => None,
        };
        if let Some(kind) = launched {
            self.shipyard = None;
            self.launch(kind);
        }
    }

    fn advance_securing(&mut self, index: usize) {
        let planet = &PLANETS[index];
        if !self.planets[index].securing {
            return;
        }
        let held = self.fleet_in_range(Side::Player, planet.position)
            && !self.enemy_in_range(Side::Player, planet.position);
        let state = &mut self.planets[index];
        if held {
            state.influence += 1;
            if state.influence >= planet.secure_turns {
                state.owner = Some(Side::Player);
                state.influence = 0;
                state.securing = false;
            }
        } else {
            state.influence = state.influence.saturating_sub(planet.decay);
            if state.influence == 0 {
                state.securing = false;
            }
        }
    }

    /// A finished ship appears beside the home world, ready next turn.
    fn launch(&mut self, kind: ShipKind) {
        let home = PLANETS[Self::home_planet()].position;
        let count = self
            .fleets
            .iter()
            .filter(|fleet| fleet.owner == Side::Player)
            .count()
            + 1;
        let id = self.next_fleet_id;
        self.next_fleet_id += 1;
        self.fleets.push(Fleet {
            id,
            name: format!("{} {count}", kind.name()),
            owner: Side::Player,
            position: home + Vec2::new(40.0, 30.0),
            speed: SCOUT_SPEED,
            remaining: 0.0,
            vision: SCOUT_VISION,
        });
    }
}
