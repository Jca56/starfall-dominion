use lntrn_math::{Rect, Vec2};

use crate::world::WORLD_SIZE;

pub(crate) struct Camera {
    center: Vec2,
    pub(crate) pixels_per_unit: f64,
}

impl Camera {
    pub(crate) fn new(map: Rect, scale: f64, pan: Vec2, zoom: f64) -> Self {
        let fit = (map.width() / (WORLD_SIZE.x + 100.0)).min(map.height() / (WORLD_SIZE.y + 100.0));
        Self {
            center: map.center() + pan * scale,
            pixels_per_unit: fit * zoom,
        }
    }

    pub(crate) fn to_screen(&self, world: Vec2) -> Vec2 {
        self.center + (world - WORLD_SIZE * 0.5) * self.pixels_per_unit
    }

    pub(crate) fn to_world(&self, screen: Vec2) -> Vec2 {
        (screen - self.center) / self.pixels_per_unit + WORLD_SIZE * 0.5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_positions_survive_scale_zoom_pan_and_resize() {
        for (width, height, scale, zoom) in [
            (1280.0, 800.0, 1.0, 1.0),
            (1792.0, 1120.0, 1.4, 2.3),
            (900.0, 640.0, 1.0, 0.6),
        ] {
            let camera = Camera::new(
                Rect::from_xywh(0.0, 95.0 * scale, width, height - 195.0 * scale),
                scale,
                Vec2::new(80.0, -20.0),
                zoom,
            );
            let world = Vec2::new(287.0, 432.0);
            assert!((camera.to_world(camera.to_screen(world)) - world).length() < 1e-8);
            let distance = (camera.to_screen(world + Vec2::new(100.0, 0.0))
                - camera.to_screen(world))
            .length();
            assert!((distance / camera.pixels_per_unit - 100.0).abs() < 1e-8);
        }
    }
}
