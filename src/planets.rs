//! Worlds on the map. Five have names; the rest are placeholders named by sector
//! code until Alva writes them into the lore. Positions are world units.
use lntrn_math::{Color, Vec2};

use crate::world::Side;

pub(crate) struct Planet {
    pub(crate) name: &'static str,
    pub(crate) position: Vec2,
    pub(crate) color: Color,
    /// Who holds it. `None` is a free world nobody has claimed.
    pub(crate) owner: Option<Side>,
    pub(crate) description: &'static str,
}

const fn planet(
    name: &'static str,
    x: f64,
    y: f64,
    color: u32,
    owner: Option<Side>,
    description: &'static str,
) -> Planet {
    Planet {
        name,
        position: Vec2::new(x, y),
        color: Color::hex(color),
        owner,
        description,
    }
}

const OURS: Option<Side> = Some(Side::Player);
const THEIRS: Option<Side> = Some(Side::Dominion);
const FREE: Option<Side> = None;
const PLACEHOLDER: &str = "A placeholder world. Its name and story are not written yet.";

// Two worlds per sector, at least 300 units apart: two to three scout moves.
// Ownership is provisional: the west is free, the east is Dominion-held, the
// middle is contested.
#[rustfmt::skip]
pub(crate) const PLANETS: [Planet; 26] = [
    planet("L1-a", 225.0, 188.0, 0x9DB7D5, FREE, PLACEHOLDER),
    planet("L1-b", 700.0, 175.0, 0xC9B79C, FREE, PLACEHOLDER),
    planet("Arcadia", 350.0, 955.0, 0x72C5EE, OURS, "A temperate world at the heart of the Expanse."),
    planet("L2-b", 175.0, 1275.0, 0xA8C8B0, FREE, PLACEHOLDER),
    planet("Haven", 575.0, 750.0, 0x82D5AA, FREE, "A peaceful ocean world beneath wide, open skies."),
    planet("Verdant", 950.0, 1025.0, 0xB7CE85, FREE, "A fertile world on the edge of the free planets."),
    planet("L4-a", 200.0, 1700.0, 0xD0A9A9, FREE, PLACEHOLDER),
    planet("L4-b", 675.0, 1788.0, 0xB9A9D0, FREE, PLACEHOLDER),
    planet("L5-a", 650.0, 1450.0, 0xA9C4D0, FREE, PLACEHOLDER),
    planet("L5-b", 1175.0, 1600.0, 0xC9B79C, FREE, PLACEHOLDER),
    planet("M1-a", 1275.0, 188.0, 0x9DB7D5, FREE, PLACEHOLDER),
    planet("M1-b", 1750.0, 325.0, 0xA8C8B0, FREE, PLACEHOLDER),
    planet("Cinder", 1850.0, 825.0, 0xECAC73, THEIRS, "An industrial world under Dominion occupation."),
    planet("M2-b", 1300.0, 700.0, 0xD0A9A9, FREE, PLACEHOLDER),
    planet("M3-a", 1575.0, 1550.0, 0xB9A9D0, FREE, PLACEHOLDER),
    planet("M3-b", 1850.0, 1825.0, 0xA9C4D0, FREE, PLACEHOLDER),
    planet("R1-a", 2300.0, 150.0, 0xC9B79C, THEIRS, PLACEHOLDER),
    planet("R1-b", 2750.0, 225.0, 0x9DB7D5, THEIRS, PLACEHOLDER),
    planet("R2-a", 2225.0, 600.0, 0xA8C8B0, THEIRS, PLACEHOLDER),
    planet("R2-b", 2750.0, 700.0, 0xD0A9A9, THEIRS, PLACEHOLDER),
    planet("Vesper", 2500.0, 1400.0, 0xB496D7, THEIRS, "A distant world beyond the Dominion frontier."),
    planet("R3-b", 2125.0, 1075.0, 0xB9A9D0, THEIRS, PLACEHOLDER),
    planet("R4-a", 2500.0, 1000.0, 0xA9C4D0, THEIRS, PLACEHOLDER),
    planet("R4-b", 2825.0, 1375.0, 0xC9B79C, THEIRS, PLACEHOLDER),
    planet("R5-a", 2475.0, 1812.0, 0x9DB7D5, THEIRS, PLACEHOLDER),
    planet("R5-b", 2850.0, 1800.0, 0xA8C8B0, THEIRS, PLACEHOLDER),
];
