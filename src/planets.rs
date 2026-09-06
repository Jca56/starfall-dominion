//! Worlds on the map. Five have names; the rest are placeholders named by sector
//! code until Alva writes them into the lore. Positions are world units. Flavor
//! text belongs to the lore, so none lives here.
use lntrn_math::{Color, Vec2};

use crate::economy::Resources;
use crate::world::Side;

pub(crate) struct Planet {
    pub(crate) name: &'static str,
    pub(crate) position: Vec2,
    pub(crate) color: Color,
    /// Who holds it when the game starts. `None` is unclaimed.
    pub(crate) owner: Option<Side>,
    /// Farlight's shipyard is here.
    pub(crate) home: bool,
    /// Turns a ship must hold within range to secure it.
    pub(crate) secure_turns: u32,
    /// Influence lost per turn while no ship holds in range.
    pub(crate) decay: u32,
    /// Paid every turn while held and uncontested.
    pub(crate) yield_per_turn: Resources,
}

/// Placeholder security and production numbers, in the Roadmap's ranges:
/// `security` is (turns to secure, decay per turn).
const fn planet(
    name: &'static str,
    position: Vec2,
    color: u32,
    owner: Option<Side>,
    security: (u32, u32),
    yield_per_turn: Resources,
) -> Planet {
    Planet {
        name,
        position,
        color: Color::hex(color),
        owner,
        home: false,
        secure_turns: security.0,
        decay: security.1,
        yield_per_turn,
    }
}

const fn at(x: f64, y: f64) -> Vec2 {
    Vec2::new(x, y)
}

const fn pays(alloys: u32, electronics: u32) -> Resources {
    Resources::new(alloys, electronics)
}

const OURS: Option<Side> = Some(Side::Player);
const THEIRS: Option<Side> = Some(Side::Dominion);
const FREE: Option<Side> = None;

// Two worlds per sector, at least 300 units apart: two to three scout moves.
// Ownership is provisional: the west is free, the east is Dominion-held, the
// middle is contested.
#[rustfmt::skip]
pub(crate) const PLANETS: [Planet; 26] = [
    planet("L1-a", at(225.0, 188.0), 0x9DB7D5, FREE, (4, 1), pays(2, 0)),
    planet("L1-b", at(700.0, 175.0), 0xC9B79C, FREE, (3, 1), pays(1, 1)),
    Planet { home: true, ..planet("Arcadia", at(350.0, 955.0), 0x72C5EE, OURS, (3, 1), pays(3, 1)) },
    planet("L2-b", at(175.0, 1275.0), 0xA8C8B0, FREE, (3, 1), pays(2, 0)),
    planet("Haven", at(575.0, 750.0), 0x82D5AA, FREE, (4, 1), pays(2, 1)),
    planet("Verdant", at(950.0, 1025.0), 0xB7CE85, FREE, (5, 2), pays(3, 0)),
    planet("L4-a", at(200.0, 1700.0), 0xD0A9A9, FREE, (3, 1), pays(1, 0)),
    planet("L4-b", at(675.0, 1788.0), 0xB9A9D0, FREE, (4, 2), pays(2, 1)),
    planet("L5-a", at(650.0, 1450.0), 0xA9C4D0, FREE, (3, 1), pays(2, 0)),
    planet("L5-b", at(1175.0, 1600.0), 0xC9B79C, FREE, (4, 1), pays(3, 0)),
    planet("M1-a", at(1275.0, 188.0), 0x9DB7D5, FREE, (4, 1), pays(2, 0)),
    planet("M1-b", at(1750.0, 325.0), 0xA8C8B0, FREE, (5, 2), pays(3, 1)),
    planet("Cinder", at(1850.0, 825.0), 0xECAC73, THEIRS, (5, 2), pays(3, 1)),
    planet("M2-b", at(1300.0, 700.0), 0xD0A9A9, FREE, (4, 1), pays(2, 0)),
    planet("M3-a", at(1575.0, 1550.0), 0xB9A9D0, FREE, (3, 1), pays(1, 1)),
    planet("M3-b", at(1850.0, 1825.0), 0xA9C4D0, FREE, (4, 2), pays(2, 0)),
    planet("R1-a", at(2300.0, 150.0), 0xC9B79C, THEIRS, (4, 1), pays(2, 0)),
    planet("R1-b", at(2750.0, 225.0), 0x9DB7D5, THEIRS, (5, 2), pays(3, 1)),
    planet("R2-a", at(2225.0, 600.0), 0xA8C8B0, THEIRS, (4, 1), pays(2, 1)),
    planet("R2-b", at(2750.0, 700.0), 0xD0A9A9, THEIRS, (5, 2), pays(3, 0)),
    planet("Vesper", at(2500.0, 1400.0), 0xB496D7, THEIRS, (5, 2), pays(2, 1)),
    planet("R3-b", at(2125.0, 1075.0), 0xB9A9D0, THEIRS, (4, 1), pays(2, 0)),
    planet("R4-a", at(2500.0, 1000.0), 0xA9C4D0, THEIRS, (4, 2), pays(3, 1)),
    planet("R4-b", at(2825.0, 1375.0), 0xC9B79C, THEIRS, (5, 2), pays(3, 0)),
    planet("R5-a", at(2475.0, 1812.0), 0x9DB7D5, THEIRS, (4, 1), pays(2, 0)),
    planet("R5-b", at(2850.0, 1800.0), 0xA8C8B0, THEIRS, (5, 2), pays(2, 1)),
];
