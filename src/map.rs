use lntrn_math::{Color, Rect, Vec2};
use lntrn_ui::{CursorIcon, Sense, Ui};

use crate::camera::{Camera, ZoomRange};
use crate::fleet;
use crate::layout;
use crate::planets::PLANETS;
use crate::sector::Sector;
use crate::world::WORLD_SIZE;

/// Zoom per wheel pixel; one notch is about 1.2×.
const WHEEL_ZOOM: f64 = 0.003;
/// Below this fraction of the starting zoom the map becomes a star chart: planets
/// shrink to dots and names hide, so the overview reads as space, not a board.
const CHART_ZOOM: f64 = 0.3;
const BORDER: Color = Color::hex(0x617A95);

pub(crate) fn draw(ui: &mut Ui, region: Rect, sector: &mut Sector) {
    let scale = ui.m.scale;
    let now = ui.now();
    if ui.clip().contains(ui.state.pointer) && ui.state.wheel.y != 0.0 {
        let factor = (ui.state.wheel.y * WHEEL_ZOOM).exp();
        sector.view.zoom_by(factor, ui.state.pointer, region, scale);
    }
    if sector.view.step(now, region, scale) {
        ui.state.request_redraw_after(0.0);
    }
    let camera = Camera::new(region, scale, sector.view.center, sector.view.zoom);
    for point in std::mem::take(&mut sector.move_requests) {
        if sector.fleet_selected && ui.clip().contains(point) {
            fleet::order(sector, camera.to_world(point), now);
        }
    }

    boundaries(ui, &camera);
    let ship_hit = fleet::hit_rect(&camera, sector, now, scale);
    // Planets swell a little as the camera closes in, but never scale one-to-one;
    // pulled far back they become chart dots.
    let relative = sector.view.zoom / ZoomRange::for_region(region, scale).start;
    let chart = relative < CHART_ZOOM;
    let closeness = relative.sqrt().clamp(0.3, 1.5);
    let halo = 8.0 * scale * closeness.min(1.0);
    for (index, planet) in PLANETS.iter().enumerate() {
        let point = camera.to_screen(planet.position);
        let radius = 26.0 * scale * closeness;
        let hit = Rect::from_center_size(
            point + Vec2::new(0.0, 15.0 * scale),
            Vec2::new(150.0 * scale, 110.0 * scale),
        );
        let id = ui.id(planet.name);
        // A ship parked over a planet takes pointer selection priority.
        let sense = if ship_hit.contains(ui.state.press_pos) {
            Sense::NONE
        } else {
            Sense::CLICK
        };
        let mut response = ui.interact(id, hit, sense);
        if !hit.intersection(&ui.clip()).is_empty() {
            ui.focusable(id, hit);
            ui.key_click(id, &mut response);
        }
        if response.clicked {
            sector.selected = Some(index);
            sector.fleet_selected = false;
            sector.message = None;
            ui.state.request_rebuild = true;
        }
        if response.hovered {
            ui.state.cursor_icon = CursorIcon::Pointer;
        }
        let allegiance = if planet.occupied {
            Color::hex(0xF38F8F)
        } else {
            Color::hex(0x8EDBE7)
        };
        ui.draw
            .circle(point, radius + halo * 1.5, allegiance.fade(0.07));
        ui.draw.ring(
            point,
            radius + halo,
            scale * 2.0 * closeness.clamp(0.6, 1.0),
            allegiance.fade(if sector.selected == Some(index) || response.hovered {
                1.0
            } else {
                0.45
            }),
        );
        ui.draw
            .circle_gradient(point, radius, planet.color, planet.color.scale_rgb(0.2));
        if !chart {
            let label = Rect::from_center_size(
                point + Vec2::new(0.0, radius + 35.0 * scale),
                Vec2::new(175.0 * scale, 45.0 * scale),
            );
            ui.text_centered(planet.name, &ui.text_style(), label, ui.theme.text);
        }
        ui.focus_ring(id, hit);
    }

    fleet::draw(ui, &camera, ship_hit, sector, chart);
    let response = ui.interact(ui.id("pan"), ui.clip(), Sense::DRAG);
    if response.dragging {
        sector.view.pan_by(response.drag_delta, scale);
        ui.state.cursor_icon = CursorIcon::Grabbing;
        if response.drag_delta != Vec2::ZERO {
            ui.state.request_rebuild = true;
        }
    }
}

/// Sector borders and their placeholder codes, plus the map edge.
fn boundaries(ui: &mut Ui, camera: &Camera) {
    let scale = ui.m.scale;
    for (a, b) in layout::borders() {
        ui.draw.line(
            camera.to_screen(a),
            camera.to_screen(b),
            ui.m.px(1.0),
            BORDER.fade(0.5),
        );
    }
    let bounds = Rect::new(camera.to_screen(Vec2::ZERO), camera.to_screen(WORLD_SIZE));
    ui.draw
        .stroke_rect(bounds, ui.m.px(1.0), 0.0, BORDER.fade(0.8));
    let style = ui.text_style();
    for region in &layout::REGIONS {
        let label = Rect::from_center_size(
            camera.to_screen(region.centroid()),
            Vec2::new(120.0, 45.0) * scale,
        );
        if label.intersects(&ui.clip()) {
            ui.text_centered(region.name, &style, label, ui.theme.text_dim.fade(0.5));
        }
    }
}
