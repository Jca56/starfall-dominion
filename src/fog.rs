//! What the player has charted. Planet positions are known from the start; fog
//! hides what is there. Explored cells stay known, and what is lit right now comes
//! from the fleets and worlds that can see it.
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

use lntrn_math::Vec2;

use crate::world::WORLD_SIZE;

/// World units per fog cell. Edges soften over one cell on screen.
pub(crate) const CELL: f64 = 10.0;
pub(crate) const COLUMNS: usize = (WORLD_SIZE.x / CELL) as usize;
pub(crate) const ROWS: usize = (WORLD_SIZE.y / CELL) as usize;

#[derive(Clone, PartialEq)]
pub(crate) struct Fog {
    explored: Vec<bool>,
    /// Changes whenever knowledge or lighting changes; the renderer re-uploads then.
    version: u64,
}

/// Sixty thousand cells would drown a test failure; summarise instead.
impl fmt::Debug for Fog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let charted = self.explored.iter().filter(|cell| **cell).count();
        write!(
            f,
            "Fog {{ charted: {charted}/{}, version: {} }}",
            self.explored.len(),
            self.version
        )
    }
}

fn next_version() -> u64 {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
}

impl Fog {
    pub(crate) fn new() -> Self {
        Self {
            explored: vec![false; COLUMNS * ROWS],
            version: next_version(),
        }
    }

    pub(crate) fn version(&self) -> u64 {
        self.version
    }

    /// Note that what is in view changed, even if nothing new was charted.
    pub(crate) fn touch(&mut self) {
        self.version = next_version();
    }

    fn cell(point: Vec2) -> Option<usize> {
        if point.x < 0.0 || point.y < 0.0 {
            return None;
        }
        let (column, row) = ((point.x / CELL) as usize, (point.y / CELL) as usize);
        (column < COLUMNS && row < ROWS).then_some(row * COLUMNS + column)
    }

    pub(crate) fn explored_at(&self, point: Vec2) -> bool {
        Self::cell(point).is_some_and(|index| self.explored[index])
    }

    /// Chart every cell whose centre lies within `radius` of `center`.
    pub(crate) fn reveal(&mut self, center: Vec2, radius: f64) {
        let span = |low: f64, high: f64, limit: usize| {
            let first = (low / CELL).floor().max(0.0) as usize;
            let last = ((high / CELL).ceil().max(0.0) as usize).min(limit);
            first..last
        };
        for row in span(center.y - radius, center.y + radius, ROWS) {
            for column in span(center.x - radius, center.x + radius, COLUMNS) {
                if (Self::middle(column, row) - center).length() <= radius {
                    self.explored[row * COLUMNS + column] = true;
                }
            }
        }
    }

    /// A ship charts the whole way along its order, not just where it stops.
    pub(crate) fn reveal_path(&mut self, from: Vec2, to: Vec2, radius: f64) {
        let steps = ((to - from).length() / (CELL * 2.0)).ceil().max(1.0) as usize;
        for step in 0..=steps {
            self.reveal(from + (to - from) * (step as f64 / steps as f64), radius);
        }
    }

    /// Two bytes per cell for the GPU: charted, and lit by `sources` right now.
    pub(crate) fn texture(&self, sources: &[(Vec2, f64)]) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(COLUMNS * ROWS * 2);
        for row in 0..ROWS {
            for column in 0..COLUMNS {
                let explored = self.explored[row * COLUMNS + column];
                let middle = Self::middle(column, row);
                let lit = explored
                    && sources
                        .iter()
                        .any(|(center, radius)| (middle - *center).length() <= *radius);
                bytes.push(if explored { 255 } else { 0 });
                bytes.push(if lit { 255 } else { 0 });
            }
        }
        bytes
    }

    fn middle(column: usize, row: usize) -> Vec2 {
        Vec2::new((column as f64 + 0.5) * CELL, (row as f64 + 0.5) * CELL)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reveal_marks_a_disc_and_nothing_outside_the_map() {
        let mut fog = Fog::new();
        assert!(!fog.explored_at(Vec2::new(500.0, 500.0)));
        fog.reveal(Vec2::new(500.0, 500.0), 100.0);
        assert!(fog.explored_at(Vec2::new(500.0, 500.0)));
        assert!(fog.explored_at(Vec2::new(590.0, 500.0)));
        assert!(!fog.explored_at(Vec2::new(620.0, 500.0)));
        assert!(!fog.explored_at(Vec2::new(-5.0, 500.0)));
        assert!(!fog.explored_at(Vec2::new(500.0, WORLD_SIZE.y + 5.0)));
        // Discs at the edge clip cleanly instead of wrapping to the next row.
        fog.reveal(Vec2::new(WORLD_SIZE.x, 1000.0), 50.0);
        assert!(!fog.explored_at(Vec2::new(5.0, 1000.0)));
        assert!(!fog.explored_at(Vec2::new(5.0, 1010.0)));
    }

    #[test]
    fn texture_lights_explored_cells_only_within_sources() {
        let mut fog = Fog::new();
        fog.reveal(Vec2::new(100.0, 100.0), 50.0);
        let bytes = fog.texture(&[(Vec2::new(100.0, 100.0), 20.0)]);
        assert_eq!(bytes.len(), COLUMNS * ROWS * 2);
        let at = |x: f64, y: f64| {
            let index = ((y / CELL) as usize * COLUMNS + (x / CELL) as usize) * 2;
            (bytes[index], bytes[index + 1])
        };
        assert_eq!(at(105.0, 105.0), (255, 255));
        assert_eq!(at(140.0, 100.0), (255, 0));
        assert_eq!(at(200.0, 200.0), (0, 0));
        // Light never leaks into uncharted space.
        let dark = Fog::new().texture(&[(Vec2::new(100.0, 100.0), 500.0)]);
        assert!(dark.iter().all(|byte| *byte == 0));
    }
}
