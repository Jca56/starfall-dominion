use lntrn_math::Vec2;

use crate::fog::Fog;
use crate::planets::PLANETS;

pub(crate) const WORLD_SIZE: Vec2 = Vec2::new(3000.0, 2000.0);
/// A scout is fast: an average ship moves 100 per turn. It crosses the map in about
/// twenty turns and a sector in about seven.
pub(crate) const SCOUT_SPEED: f64 = 150.0;
/// A scout sees a little past where it can reach this turn.
pub(crate) const SCOUT_VISION: f64 = 225.0;
/// A world you hold keeps one move of space around it in view.
pub(crate) const PLANET_VISION: f64 = 150.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Side {
    Player,
    Dominion,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Fleet {
    pub(crate) id: u32,
    pub(crate) name: &'static str,
    pub(crate) owner: Side,
    pub(crate) position: Vec2,
    pub(crate) speed: f64,
    pub(crate) remaining: f64,
    /// How far it sees, in world units.
    pub(crate) vision: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Game {
    pub(crate) turn: u64,
    pub(crate) active_side: Side,
    pub(crate) fleets: Vec<Fleet>,
    /// The player's chart. The Dominion has no fog of its own yet.
    pub(crate) fog: Fog,
}

impl Game {
    /// Everything `side` can see right now: its fleets and the worlds it holds.
    pub(crate) fn vision_sources(&self, side: Side) -> Vec<(Vec2, f64)> {
        self.vision_sources_shown(side, |fleet| fleet.position)
    }

    /// Vision with each fleet where `shown` puts it, so the screen can light a
    /// ship from where it is drawn mid-glide.
    pub(crate) fn vision_sources_shown(
        &self,
        side: Side,
        shown: impl Fn(&Fleet) -> Vec2,
    ) -> Vec<(Vec2, f64)> {
        self.fleets
            .iter()
            .filter(|fleet| fleet.owner == side)
            .map(|fleet| (shown(fleet), fleet.vision))
            .chain(
                PLANETS
                    .iter()
                    .filter(|planet| planet.owner == Some(side))
                    .map(|planet| (planet.position, PLANET_VISION)),
            )
            .collect()
    }

    #[cfg(test)]
    pub(crate) fn can_see(&self, side: Side, point: Vec2) -> bool {
        self.vision_sources(side)
            .iter()
            .any(|(center, radius)| (point - *center).length() <= *radius)
    }
}

impl Default for Game {
    fn default() -> Self {
        let mut game = Self {
            turn: 1,
            active_side: Side::Player,
            fleets: vec![Fleet {
                id: 0,
                name: "Farlight Scout",
                owner: Side::Player,
                position: Vec2::new(300.0, 1000.0),
                speed: SCOUT_SPEED,
                remaining: SCOUT_SPEED,
                vision: SCOUT_VISION,
            }],
            fog: Fog::new(),
        };
        // You start knowing only what your home and your scout can see.
        for (center, radius) in game.vision_sources(Side::Player) {
            game.fog.reveal(center, radius);
        }
        game
    }
}
