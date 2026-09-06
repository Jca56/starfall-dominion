//! Points of interest: things worth a detour that the fog hides until a ship
//! charts them. Each sector holds one at a time; salvaging one starts a clock,
//! and a new one appears somewhere in that sector out of sight when it runs out.
use std::f64::consts::TAU;

use lntrn_math::Vec2;

use crate::economy::Resources;
use crate::layout::{self, REGIONS};
use crate::planets::PLANETS;
use crate::rng::Rng;
use crate::world::{Game, Side};

/// A ship must be this close to salvage a wreck.
pub(crate) const SALVAGE_RANGE: f64 = 100.0;
/// Room a point of interest keeps from planets and other points.
const CLEARANCE: f64 = 150.0;
/// Turns after a salvage before that sector gets a new point.
const RESPAWN_TURNS: (u32, u32) = (10, 20);
/// The starter point sits this far from the scout's start: just out of sight,
/// one move away.
const STARTER_BAND: (f64, f64) = (260.0, 330.0);
const PLACEMENT_TRIES: usize = 200;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PoiKind {
    DestroyedStation,
}

impl PoiKind {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::DestroyedStation => "Destroyed Space Station",
        }
    }

    /// Fits under a map marker.
    pub(crate) fn short_name(self) -> &'static str {
        match self {
            Self::DestroyedStation => "Station",
        }
    }

    pub(crate) fn description(self) -> &'static str {
        match self {
            Self::DestroyedStation => {
                "A hull cracked open long ago. Something in the wreckage is still worth hauling home."
            }
        }
    }

    /// Placeholder loot from the Roadmap: 6 to 12 Alloys, and a 35% chance of
    /// 2 to 4 Advanced Electronics.
    pub(crate) fn alloys(self) -> (u32, u32) {
        match self {
            Self::DestroyedStation => (6, 12),
        }
    }

    pub(crate) fn electronics(self) -> (f64, u32, u32) {
        match self {
            Self::DestroyedStation => (0.35, 2, 4),
        }
    }

    fn roll(self, rng: &mut Rng) -> Resources {
        let (low, high) = self.alloys();
        let (chance, few, many) = self.electronics();
        let alloys = rng.range(low, high);
        let electronics = if rng.chance(chance) {
            rng.range(few, many)
        } else {
            0
        };
        Resources::new(alloys, electronics)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Poi {
    pub(crate) id: u32,
    pub(crate) kind: PoiKind,
    pub(crate) position: Vec2,
    /// Index into `REGIONS`.
    pub(crate) sector: usize,
}

impl Game {
    pub(crate) fn poi(&self, id: u32) -> Option<&Poi> {
        self.pois.iter().find(|poi| poi.id == id)
    }

    /// One point per sector at the start. The scout's own sector gets the
    /// starter, placed a move away and just out of sight.
    pub(crate) fn seed_pois(&mut self) {
        let start = self.fleets[0].position;
        let home_sector = layout::region_index(start);
        for sector in 0..REGIONS.len() {
            let band = (Some(sector) == home_sector).then_some((start, STARTER_BAND));
            if let Some(position) = self.place(sector, band) {
                self.add_poi(sector, position);
            }
        }
    }

    /// Run every empty sector's respawn clock; at zero, try to place a new point.
    pub(crate) fn advance_pois(&mut self) {
        for sector in 0..REGIONS.len() {
            if self.pois.iter().any(|poi| poi.sector == sector) {
                continue;
            }
            if self.respawn[sector] > 0 {
                self.respawn[sector] -= 1;
            }
            if self.respawn[sector] == 0
                && let Some(position) = self.place(sector, None)
            {
                self.add_poi(sector, position);
            }
        }
    }

    /// Salvage `id`: roll its loot into the stockpile, remove it, and start the
    /// sector's clock. Validated by the action rules first.
    pub(crate) fn salvage(&mut self, id: u32) -> Resources {
        let index = self
            .pois
            .iter()
            .position(|poi| poi.id == id)
            .expect("validated point of interest");
        let poi = self.pois.remove(index);
        let reward = poi.kind.roll(&mut self.rng);
        self.stockpile = self.stockpile.plus(reward);
        self.respawn[poi.sector] = self.rng.range(RESPAWN_TURNS.0, RESPAWN_TURNS.1);
        reward
    }

    fn add_poi(&mut self, sector: usize, position: Vec2) {
        let id = self.next_poi_id;
        self.next_poi_id += 1;
        self.pois.push(Poi {
            id,
            kind: PoiKind::DestroyedStation,
            position,
            sector,
        });
    }

    /// A random legal spot in `sector`: inside it, clear of planets and other
    /// points, and out of the player's sight. `band` confines the search to a
    /// ring around a point instead of the whole sector.
    fn place(&mut self, sector: usize, band: Option<(Vec2, (f64, f64))>) -> Option<Vec2> {
        let bounds = REGIONS[sector].bounds();
        for _ in 0..PLACEMENT_TRIES {
            let point = match band {
                Some((center, (low, high))) => {
                    let angle = self.rng.unit() * TAU;
                    center + Vec2::from_angle(angle) * (low + self.rng.unit() * (high - low))
                }
                None => Vec2::new(
                    bounds.min.x + self.rng.unit() * bounds.width(),
                    bounds.min.y + self.rng.unit() * bounds.height(),
                ),
            };
            if self.placement_ok(point, sector) {
                return Some(point);
            }
        }
        None
    }

    fn placement_ok(&self, point: Vec2, sector: usize) -> bool {
        REGIONS[sector].contains(point)
            && PLANETS
                .iter()
                .all(|planet| (planet.position - point).length() >= CLEARANCE)
            && self
                .pois
                .iter()
                .all(|poi| (poi.position - point).length() >= CLEARANCE)
            && !self.can_see(Side::Player, point)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::{Action, execute};

    fn round(game: &mut Game) {
        execute(game, Side::Player, Action::EndTurn).unwrap();
        execute(game, Side::Dominion, Action::EndTurn).unwrap();
    }

    #[test]
    fn one_point_per_sector_clear_of_planets_and_out_of_sight() {
        let game = Game::default();
        assert_eq!(game.pois.len(), REGIONS.len());
        for (sector, region) in REGIONS.iter().enumerate() {
            let here: Vec<&Poi> = game
                .pois
                .iter()
                .filter(|poi| poi.sector == sector)
                .collect();
            assert_eq!(here.len(), 1, "sector {}", region.name);
            let poi = here[0];
            assert!(
                region.contains(poi.position),
                "{} sits outside its sector",
                region.name
            );
            assert!(
                PLANETS
                    .iter()
                    .all(|planet| (planet.position - poi.position).length() >= CLEARANCE)
            );
            assert!(
                !game.can_see(Side::Player, poi.position),
                "{} starts in sight",
                region.name
            );
        }
        let start = game.fleets[0].position;
        let home = layout::region_index(start).expect("the scout starts on the map");
        let starter = game.pois.iter().find(|poi| poi.sector == home).unwrap();
        let distance = (starter.position - start).length();
        assert!(
            (STARTER_BAND.0..=STARTER_BAND.1).contains(&distance),
            "the starter is {distance} from the scout"
        );
    }

    #[test]
    fn the_same_seed_makes_the_same_map_and_another_seed_does_not() {
        let a = Game::new(42);
        let b = Game::new(42);
        let c = Game::new(43);
        assert_eq!(a.pois, b.pois);
        assert_ne!(a.pois, c.pois);
    }

    #[test]
    fn a_salvaged_sector_respawns_after_its_clock_out_of_sight() {
        let mut game = Game::default();
        let poi = game.pois[5].clone();
        let reward = game.salvage(poi.id);
        assert_eq!(game.stockpile, reward);
        assert!(game.poi(poi.id).is_none());
        let clock = game.respawn[poi.sector];
        assert!((RESPAWN_TURNS.0..=RESPAWN_TURNS.1).contains(&clock));
        for turn in 0..clock {
            assert!(
                !game.pois.iter().any(|p| p.sector == poi.sector),
                "respawned early on turn {turn}"
            );
            round(&mut game);
        }
        let fresh = game
            .pois
            .iter()
            .find(|p| p.sector == poi.sector)
            .expect("respawned");
        assert_ne!(fresh.id, poi.id);
        assert!(REGIONS[poi.sector].contains(fresh.position));
        assert!(!game.can_see(Side::Player, fresh.position));
    }
}
