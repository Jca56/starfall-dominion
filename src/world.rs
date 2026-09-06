use lntrn_math::Vec2;

use crate::economy::{Build, PlanetState, Resources};
use crate::fog::Fog;
use crate::layout::REGIONS;
use crate::planets::PLANETS;
use crate::poi::Poi;
use crate::rng::Rng;

pub(crate) const WORLD_SIZE: Vec2 = Vec2::new(3000.0, 2000.0);
/// A scout is fast: an average ship moves 100 per turn. It crosses the map in about
/// twenty turns and a sector in about seven.
pub(crate) const SCOUT_SPEED: f64 = 150.0;
/// A scout sees a little past where it can reach this turn.
pub(crate) const SCOUT_VISION: f64 = 225.0;
/// A world you hold keeps one move of space around it in view.
pub(crate) const PLANET_VISION: f64 = 150.0;
/// The seed `Game::default` uses, so tests and the menu's placeholder agree.
const FIXED_SEED: u64 = 0x5EED_5EED;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Side {
    Player,
    Dominion,
}

impl Side {
    /// The faction name as the player sees it.
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Player => "Farlight",
            Self::Dominion => "Starfall",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Fleet {
    pub(crate) id: u32,
    pub(crate) name: String,
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
    /// One entry per planet in `PLANETS`.
    pub(crate) planets: Vec<PlanetState>,
    pub(crate) stockpile: Resources,
    /// The ship under construction at the home world.
    pub(crate) shipyard: Option<Build>,
    /// Ids for ships launched during play; the first scout is 0.
    pub(crate) next_fleet_id: u32,
    pub(crate) pois: Vec<Poi>,
    /// Per sector: turns until a salvaged sector may get a new point.
    pub(crate) respawn: Vec<u32>,
    pub(crate) rng: Rng,
    pub(crate) next_poi_id: u32,
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
                    .zip(&self.planets)
                    .filter(|(_, state)| state.owner == Some(side))
                    .map(|(planet, _)| (planet.position, PLANET_VISION)),
            )
            .collect()
    }

    pub(crate) fn can_see(&self, side: Side, point: Vec2) -> bool {
        self.vision_sources(side)
            .iter()
            .any(|(center, radius)| (point - *center).length() <= *radius)
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new(FIXED_SEED)
    }
}

impl Game {
    /// A fresh game. The seed decides where every point of interest lies.
    pub(crate) fn new(seed: u64) -> Self {
        let mut game = Self {
            turn: 1,
            active_side: Side::Player,
            fleets: vec![Fleet {
                id: 0,
                name: "Farlight Scout".to_string(),
                owner: Side::Player,
                position: Vec2::new(300.0, 1000.0),
                speed: SCOUT_SPEED,
                remaining: SCOUT_SPEED,
                vision: SCOUT_VISION,
            }],
            fog: Fog::new(),
            planets: PLANETS
                .iter()
                .map(|planet| PlanetState {
                    owner: planet.owner,
                    influence: 0,
                    securing: false,
                })
                .collect(),
            stockpile: Resources::default(),
            shipyard: None,
            next_fleet_id: 1,
            pois: Vec::new(),
            respawn: vec![0; REGIONS.len()],
            rng: Rng::new(seed),
            next_poi_id: 1,
        };
        // You start knowing only what your home and your scout can see.
        for (center, radius) in game.vision_sources(Side::Player) {
            game.fog.reveal(center, radius);
        }
        game.seed_pois();
        game
    }
}
