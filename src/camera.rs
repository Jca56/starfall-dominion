use lntrn_math::{Rect, Vec2};

use crate::world::WORLD_SIZE;

/// World units visible across the map region when a game starts: three scout moves,
/// so anything on screen is at most three turns away and a move is a visible jump.
pub(crate) const START_VIEW_WIDTH: f64 = 450.0;
/// How much closer than the starting view the camera can get: one move fills the screen.
const MAX_ZOOM_IN: f64 = 3.0;
/// Breathing room around the whole map at the farthest zoom.
const FIT_MARGIN: f64 = 0.92;
/// Seconds for the camera to close most of the gap to where the player sent it.
const EASE_SECONDS: f64 = 0.2;

pub(crate) struct Camera {
    screen_center: Vec2,
    world_center: Vec2,
    pub(crate) pixels_per_unit: f64,
}

impl Camera {
    /// At zoom 1, one world unit occupies one logical pixel, independent of world bounds.
    /// One uniform conversion factor preserves distances in both axes.
    pub(crate) fn new(region: Rect, scale: f64, world_center: Vec2, zoom: f64) -> Self {
        Self {
            screen_center: region.center(),
            world_center,
            pixels_per_unit: scale * zoom,
        }
    }

    pub(crate) fn to_screen(&self, world: Vec2) -> Vec2 {
        self.screen_center + (world - self.world_center) * self.pixels_per_unit
    }

    pub(crate) fn to_world(&self, screen: Vec2) -> Vec2 {
        (screen - self.screen_center) / self.pixels_per_unit + self.world_center
    }
}

/// Zoom limits in logical pixels per world unit, derived from the map region on screen.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ZoomRange {
    /// The whole map fits with a margin.
    pub(crate) min: f64,
    /// The home region fills the width.
    pub(crate) start: f64,
    pub(crate) max: f64,
}

impl ZoomRange {
    pub(crate) fn for_region(region: Rect, scale: f64) -> Self {
        let logical = region.size() / scale;
        let fit = (logical.x / WORLD_SIZE.x).min(logical.y / WORLD_SIZE.y) * FIT_MARGIN;
        let start = logical.x / START_VIEW_WIDTH;
        Self {
            min: fit.min(start),
            start,
            max: start * MAX_ZOOM_IN,
        }
    }

    pub(crate) fn clamp(self, zoom: f64) -> f64 {
        zoom.clamp(self.min, self.max)
    }
}

/// Where the player is looking. Input moves the target; the displayed camera eases after it.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct View {
    pub(crate) center: Vec2,
    /// Logical pixels per world unit. Zero until the first frame picks the starting zoom.
    pub(crate) zoom: f64,
    pub(crate) target_center: Vec2,
    pub(crate) target_zoom: f64,
    last_time: Option<f64>,
}

impl View {
    pub(crate) fn starting_at(center: Vec2) -> Self {
        Self {
            center,
            zoom: 0.0,
            target_center: center,
            target_zoom: 0.0,
            last_time: None,
        }
    }

    /// Jump the camera and its target together, with no easing.
    #[cfg(test)]
    pub(crate) fn snap(&mut self, center: Vec2, zoom: f64) {
        self.center = center;
        self.zoom = zoom;
        self.target_center = center;
        self.target_zoom = zoom;
    }

    /// Finish any easing in progress right now.
    #[cfg(test)]
    pub(crate) fn settle(&mut self) {
        self.center = self.target_center;
        self.zoom = self.target_zoom;
    }

    pub(crate) fn moving(&self) -> bool {
        self.center != self.target_center || self.zoom != self.target_zoom
    }

    /// Zoom the target by `factor`, keeping the world point under `anchor` (physical px) fixed.
    pub(crate) fn zoom_by(&mut self, factor: f64, anchor: Vec2, region: Rect, scale: f64) {
        let range = ZoomRange::for_region(region, scale);
        self.ensure_started(range);
        let before = Camera::new(region, scale, self.target_center, self.target_zoom);
        let world = before.to_world(anchor);
        self.target_zoom = range.clamp(self.target_zoom * factor);
        let after = Camera::new(region, scale, self.target_center, self.target_zoom);
        self.target_center += world - after.to_world(anchor);
        self.target_center = clamp_center(self.target_center, self.target_zoom, region, scale);
    }

    /// Dragging moves the view directly under the pointer; the target follows so nothing springs back.
    pub(crate) fn pan_by(&mut self, screen_delta: Vec2, scale: f64) {
        if self.zoom <= 0.0 {
            return;
        }
        let delta = screen_delta / (self.zoom * scale);
        self.center -= delta;
        self.target_center -= delta;
    }

    /// Ease toward the target and keep both inside the map. Returns true while still moving.
    pub(crate) fn step(&mut self, now: f64, region: Rect, scale: f64) -> bool {
        let range = ZoomRange::for_region(region, scale);
        self.ensure_started(range);
        self.target_zoom = range.clamp(self.target_zoom);
        self.target_center = clamp_center(self.target_center, self.target_zoom, region, scale);
        // A long idle must not turn into one huge step.
        let dt = self
            .last_time
            .map_or(0.0, |last| (now - last).clamp(0.0, 0.1));
        self.last_time = Some(now);
        let k = 1.0 - (-dt * 4.0 / EASE_SECONDS).exp();
        // Zoom eases in log space so every step feels the same at any magnification.
        // Each axis snaps once it is within a rounding error of its target.
        if self.zoom != self.target_zoom {
            let log_zoom = self.zoom.ln() + (self.target_zoom.ln() - self.zoom.ln()) * k;
            self.zoom = log_zoom.exp();
            if (self.zoom / self.target_zoom - 1.0).abs() < 1e-4 {
                self.zoom = self.target_zoom;
            }
        }
        if self.center != self.target_center {
            self.center += (self.target_center - self.center) * k;
            if (self.center - self.target_center).length() * self.zoom * scale < 0.05 {
                self.center = self.target_center;
            }
        }
        self.zoom = range.clamp(self.zoom);
        self.center = clamp_center(self.center, self.zoom, region, scale);
        self.moving()
    }

    fn ensure_started(&mut self, range: ZoomRange) {
        if self.zoom <= 0.0 {
            self.zoom = range.start;
            self.target_zoom = range.start;
        }
    }
}

fn clamp_center(center: Vec2, zoom: f64, region: Rect, scale: f64) -> Vec2 {
    let half = region.size() / (zoom * scale) * 0.5;
    Vec2::new(
        clamp_axis(center.x, half.x, WORLD_SIZE.x),
        clamp_axis(center.y, half.y, WORLD_SIZE.y),
    )
}

/// The viewport stays inside the map; a viewport wider than the map centers it.
fn clamp_axis(center: f64, half: f64, size: f64) -> f64 {
    if half >= size * 0.5 {
        size * 0.5
    } else {
        center.clamp(half, size - half)
    }
}

/// What the starfield shader needs: parallax scroll, zoom, and the lit map area.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Backdrop {
    /// How far the map center has scrolled from the middle of the region, physical px.
    pub(crate) scroll: Vec2,
    /// Physical pixels per world unit.
    pub(crate) pixels_per_unit: f64,
    /// Zoom relative to the starting zoom: 1 when a game begins.
    pub(crate) zoom_relative: f64,
    /// The map rectangle on screen, physical px.
    pub(crate) bounds: Rect,
    pub(crate) gameplay: bool,
}

impl Backdrop {
    pub(crate) fn menu() -> Self {
        Self {
            scroll: Vec2::ZERO,
            pixels_per_unit: 1.0,
            zoom_relative: 1.0,
            bounds: Rect::new(Vec2::ZERO, Vec2::ZERO),
            gameplay: false,
        }
    }

    pub(crate) fn sector(view: &View, region: Rect, scale: f64) -> Self {
        let range = ZoomRange::for_region(region, scale);
        let zoom = if view.zoom > 0.0 {
            view.zoom
        } else {
            range.start
        };
        let camera = Camera::new(region, scale, view.center, zoom);
        let zoom_relative = zoom / range.start;
        // Deep layers drift with the square root of zoom, so zooming reads as a gentle dolly.
        let scroll =
            (view.center - WORLD_SIZE * 0.5) * camera.pixels_per_unit / zoom_relative.sqrt();
        Self {
            scroll,
            pixels_per_unit: camera.pixels_per_unit,
            zoom_relative,
            bounds: Rect::new(camera.to_screen(Vec2::ZERO), camera.to_screen(WORLD_SIZE)),
            gameplay: true,
        }
    }

    /// Three `vec4<f32>` uniforms: viewport, camera, bounds.
    pub(crate) fn uniform(&self, viewport: Vec2, scale: f64) -> [f32; 12] {
        [
            viewport.x as f32,
            viewport.y as f32,
            scale as f32,
            if self.gameplay { 1.0 } else { 0.0 },
            self.scroll.x as f32,
            self.scroll.y as f32,
            self.pixels_per_unit as f32,
            self.zoom_relative as f32,
            self.bounds.min.x as f32,
            self.bounds.min.y as f32,
            self.bounds.max.x as f32,
            self.bounds.max.y as f32,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn region(width: f64, height: f64, scale: f64) -> Rect {
        Rect::from_xywh(0.0, 95.0 * scale, width * scale, (height - 205.0) * scale)
    }

    #[test]
    fn world_positions_survive_scale_zoom_pan_and_resize() {
        for (width, height, scale, zoom) in [
            (1280.0, 800.0, 1.0, 1.0),
            (1792.0, 1120.0, 1.4, 2.3),
            (900.0, 640.0, 1.0, 0.6),
        ] {
            let camera = Camera::new(
                region(width, height, scale),
                scale,
                Vec2::new(280.0, 420.0),
                zoom,
            );
            let world = Vec2::new(287.0, 432.0);
            assert!((camera.to_world(camera.to_screen(world)) - world).length() < 1e-8);
            let distance = (camera.to_screen(world + Vec2::new(100.0, 0.0))
                - camera.to_screen(world))
            .length();
            assert!((distance / scale - 100.0 * zoom).abs() < 1e-8);
        }
    }

    #[test]
    fn zoom_range_fits_the_map_and_starts_on_the_home_region() {
        for (width, height, scale) in [
            (1280.0, 800.0, 1.0),
            (2600.0, 1400.0, 1.4),
            (900.0, 640.0, 1.0),
        ] {
            let region = region(width, height, scale);
            let range = ZoomRange::for_region(region, scale);
            assert!(range.min < range.start && range.start < range.max);
            assert!(WORLD_SIZE.x * range.min * scale <= region.width());
            assert!(WORLD_SIZE.y * range.min * scale <= region.height());
            assert!((region.width() / scale / range.start - START_VIEW_WIDTH).abs() < 1e-8);
        }
    }

    #[test]
    fn view_starts_inside_the_map_and_eases_to_a_zoom_anchor() {
        let scale = 1.4;
        let region = region(1280.0, 800.0, scale);
        let range = ZoomRange::for_region(region, scale);
        let mut view = View::starting_at(Vec2::new(300.0, 1000.0));
        assert!(!view.step(0.0, region, scale));
        assert_eq!(view.zoom, range.start);
        // Nothing left of the map edge is visible.
        assert!(view.center.x * view.zoom * scale >= region.width() * 0.5 - 1e-6);

        let anchor = Vec2::new(500.0 * scale, 300.0 * scale);
        let world = Camera::new(region, scale, view.center, view.zoom).to_world(anchor);
        view.zoom_by(2.0, anchor, region, scale);
        assert!(view.moving());
        let mut time = 0.0;
        while view.step(time, region, scale) {
            time += 1.0 / 60.0;
            assert!(time < 3.0, "The camera must settle");
        }
        assert!((view.zoom - range.start * 2.0).abs() < 1e-9);
        let after = Camera::new(region, scale, view.center, view.zoom).to_world(anchor);
        assert!((after - world).length() < 1e-6);

        // Zooming out past the whole map stops at the fit zoom, centered on the map.
        view.zoom_by(0.001, anchor, region, scale);
        view.settle();
        assert!(!view.step(time, region, scale));
        assert_eq!(view.zoom, range.min);
        assert!((view.center - WORLD_SIZE * 0.5).length() < 1e-9);
    }

    #[test]
    fn backdrop_scroll_rests_at_the_map_center() {
        let scale = 1.0;
        let region = region(1280.0, 800.0, scale);
        let mut view = View::starting_at(WORLD_SIZE * 0.5);
        view.step(0.0, region, scale);
        let backdrop = Backdrop::sector(&view, region, scale);
        assert_eq!(backdrop.scroll, Vec2::ZERO);
        assert!((backdrop.zoom_relative - 1.0).abs() < 1e-9);
        assert!(backdrop.gameplay);
        assert_eq!(Backdrop::menu().uniform(Vec2::new(10.0, 20.0), 1.4)[3], 0.0);
    }
}
