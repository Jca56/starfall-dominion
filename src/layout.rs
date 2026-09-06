//! The thirteen sectors of the Farlight Expanse, traced from Alva's sketch: five
//! in the west, three in the middle, five in the east. An odd count means no ties
//! for control of the map. Outlines are authored in `LAYOUT_SIZE` units and
//! stretch with `WORLD_SIZE`, so nudging a corner here moves it on the map.
use lntrn_math::{Rect, Vec2};

use crate::world::WORLD_SIZE;

/// The map size the outlines below were authored against.
const LAYOUT_SIZE: Vec2 = Vec2::new(3000.0, 2000.0);

/// One map sector: a closed polygon. Neighbours share their corner points exactly,
/// so borders dedupe cleanly and the sectors tile the map without gaps.
pub(crate) struct Region {
    /// Placeholder code until the sectors are named: L, M, R for west, middle, east.
    pub(crate) name: &'static str,
    outline: &'static [Vec2],
}

const fn v(x: f64, y: f64) -> Vec2 {
    Vec2::new(x, y)
}

#[rustfmt::skip]
pub(crate) const REGIONS: [Region; 13] = [
    // West: the free worlds.
    Region { name: "L1", outline: &[v(0.0, 0.0), v(1000.0, 0.0), v(1000.0, 295.0), v(398.0, 530.0), v(0.0, 685.0)] },
    Region { name: "L2", outline: &[v(0.0, 685.0), v(398.0, 530.0), v(662.0, 1082.0), v(330.0, 1302.0), v(0.0, 1520.0)] },
    Region { name: "L3", outline: &[v(398.0, 530.0), v(1000.0, 295.0), v(1000.0, 395.0), v(942.0, 708.0), v(1322.0, 1030.0), v(755.0, 1275.0), v(662.0, 1082.0)] },
    Region { name: "L4", outline: &[v(0.0, 1520.0), v(330.0, 1302.0), v(408.0, 1665.0), v(965.0, 1555.0), v(1000.0, 2000.0), v(0.0, 2000.0)] },
    Region { name: "L5", outline: &[v(662.0, 1082.0), v(755.0, 1275.0), v(1322.0, 1030.0), v(1378.0, 1375.0), v(1412.0, 2000.0), v(1000.0, 2000.0), v(965.0, 1555.0), v(408.0, 1665.0), v(330.0, 1302.0)] },
    // Middle: contested space.
    Region { name: "M1", outline: &[v(1000.0, 0.0), v(2000.0, 0.0), v(2000.0, 518.0), v(1545.0, 618.0), v(1000.0, 395.0), v(1000.0, 295.0)] },
    Region { name: "M2", outline: &[v(1000.0, 395.0), v(1545.0, 618.0), v(2000.0, 518.0), v(2000.0, 850.0), v(2000.0, 1185.0), v(1378.0, 1375.0), v(1322.0, 1030.0), v(942.0, 708.0)] },
    Region { name: "M3", outline: &[v(1378.0, 1375.0), v(2000.0, 1185.0), v(2000.0, 2000.0), v(1412.0, 2000.0)] },
    // East: the Dominion front.
    Region { name: "R1", outline: &[v(2000.0, 0.0), v(3000.0, 0.0), v(3000.0, 450.0), v(2572.0, 572.0)] },
    Region { name: "R2", outline: &[v(2000.0, 0.0), v(2572.0, 572.0), v(3000.0, 450.0), v(3000.0, 850.0), v(2305.0, 850.0), v(2000.0, 850.0), v(2000.0, 518.0)] },
    Region { name: "R3", outline: &[v(2000.0, 850.0), v(2305.0, 850.0), v(2292.0, 1165.0), v(2638.0, 1160.0), v(2638.0, 1610.0), v(2305.0, 1575.0), v(2325.0, 2000.0), v(2000.0, 2000.0), v(2000.0, 1185.0)] },
    Region { name: "R4", outline: &[v(2305.0, 850.0), v(3000.0, 850.0), v(3000.0, 1610.0), v(2638.0, 1610.0), v(2638.0, 1160.0), v(2292.0, 1165.0)] },
    Region { name: "R5", outline: &[v(2305.0, 1575.0), v(2638.0, 1610.0), v(3000.0, 1610.0), v(3000.0, 2000.0), v(2325.0, 2000.0)] },
];

fn to_world(point: Vec2) -> Vec2 {
    Vec2::new(
        point.x * WORLD_SIZE.x / LAYOUT_SIZE.x,
        point.y * WORLD_SIZE.y / LAYOUT_SIZE.y,
    )
}

impl Region {
    fn corners(&self) -> Vec<Vec2> {
        self.outline.iter().map(|point| to_world(*point)).collect()
    }

    fn edges(&self) -> Vec<(Vec2, Vec2)> {
        let corners = self.corners();
        (0..corners.len())
            .map(|i| (corners[i], corners[(i + 1) % corners.len()]))
            .collect()
    }

    /// Ray casting: a point is inside when a ray to the right crosses an odd number of edges.
    pub(crate) fn contains(&self, point: Vec2) -> bool {
        let mut inside = false;
        for (a, b) in self.edges() {
            if (a.y > point.y) != (b.y > point.y) {
                let x = a.x + (point.y - a.y) / (b.y - a.y) * (b.x - a.x);
                if point.x < x {
                    inside = !inside;
                }
            }
        }
        inside
    }

    /// The corners' bounding box, for sampling points inside the sector.
    pub(crate) fn bounds(&self) -> Rect {
        let corners = self.corners();
        let mut bounds = Rect::new(corners[0], corners[0]);
        for corner in corners {
            bounds = bounds.union(&Rect::new(corner, corner));
        }
        bounds
    }

    /// Area-weighted centre: where the sector's label sits.
    pub(crate) fn centroid(&self) -> Vec2 {
        let mut area = 0.0;
        let mut sum = Vec2::ZERO;
        for (a, b) in self.edges() {
            let cross = a.x * b.y - b.x * a.y;
            area += cross;
            sum += (a + b) * cross;
        }
        sum / (3.0 * area)
    }
}

pub(crate) fn region_index(point: Vec2) -> Option<usize> {
    REGIONS.iter().position(|region| region.contains(point))
}

pub(crate) fn region_of(point: Vec2) -> Option<&'static Region> {
    REGIONS.iter().find(|region| region.contains(point))
}

/// Every border once, in world units, even where two sectors share it.
pub(crate) fn borders() -> Vec<(Vec2, Vec2)> {
    let mut borders: Vec<(Vec2, Vec2)> = Vec::new();
    for region in &REGIONS {
        for (a, b) in region.edges() {
            let edge = if (a.x, a.y) <= (b.x, b.y) {
                (a, b)
            } else {
                (b, a)
            };
            if !borders.contains(&edge) {
                borders.push(edge);
            }
        }
    }
    borders
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planets::PLANETS;

    fn distance_to_segment(point: Vec2, a: Vec2, b: Vec2) -> f64 {
        let along = b - a;
        let length_squared = along.x * along.x + along.y * along.y;
        let t = if length_squared == 0.0 {
            0.0
        } else {
            (((point - a).x * along.x + (point - a).y * along.y) / length_squared).clamp(0.0, 1.0)
        };
        (point - (a + along * t)).length()
    }

    fn near_a_border(borders: &[(Vec2, Vec2)], point: Vec2) -> bool {
        borders
            .iter()
            .any(|(a, b)| distance_to_segment(point, *a, *b) < 3.0)
    }

    #[test]
    fn thirteen_sectors_tile_the_map_exactly_once() {
        let counts =
            ["L", "M", "R"].map(|side| REGIONS.iter().filter(|r| r.name.starts_with(side)).count());
        assert_eq!(counts, [5, 3, 5]);
        let borders = borders();
        let mut y = 12.0;
        while y < WORLD_SIZE.y {
            let mut x = 12.0;
            while x < WORLD_SIZE.x {
                let point = Vec2::new(x, y);
                if !near_a_border(&borders, point) {
                    let hits = REGIONS.iter().filter(|r| r.contains(point)).count();
                    assert_eq!(hits, 1, "{point:?} lies in {hits} sectors");
                }
                x += 25.0;
            }
            y += 25.0;
        }
    }

    #[test]
    fn every_sector_holds_two_planets_and_its_own_label() {
        let borders = borders();
        for region in &REGIONS {
            let planets: Vec<_> = PLANETS
                .iter()
                .filter(|planet| region.contains(planet.position))
                .map(|planet| planet.name)
                .collect();
            assert_eq!(planets.len(), 2, "sector {} holds {planets:?}", region.name);
            assert!(
                region.contains(region.centroid()),
                "sector {} label falls outside it",
                region.name
            );
        }
        for planet in &PLANETS {
            assert!(
                region_of(planet.position).is_some(),
                "{} is off the map",
                planet.name
            );
            assert!(
                !near_a_border(&borders, planet.position),
                "{} sits on a border",
                planet.name
            );
        }
    }

    #[test]
    fn shared_corners_dedupe_into_one_border_each() {
        let total: usize = REGIONS.iter().map(|region| region.outline.len()).sum();
        let unique = borders().len();
        assert!(
            unique < total,
            "{unique} borders from {total} outline edges"
        );
    }
}
