use std::f64::consts::{FRAC_PI_2, TAU};

use lntrn_math::{Color, Rect, Vec2};
use lntrn_ui::{CursorIcon, Sense, Ui};

use crate::camera::{Camera, ZoomRange};
use crate::fleet;
use crate::layout;
use crate::planets::PLANETS;
use crate::sector::Sector;
use crate::world::{Side, WORLD_SIZE};

/// Zoom per wheel pixel; one notch is about 1.2×.
const WHEEL_ZOOM: f64 = 0.003;
/// Below this fraction of the starting zoom the map becomes a star chart: planets
/// shrink to dots and names hide, so the overview reads as space, not a board.
const CHART_ZOOM: f64 = 0.3;
const BORDER: Color = Color::hex(0x617A95);
pub(crate) const PLAYER_COLOR: Color = Color::hex(0x8EDBE7);
const DOMINION_COLOR: Color = Color::hex(0xF38F8F);
const FREE_COLOR: Color = Color::hex(0xD9D2A6);
/// A world charted from afar: position known, nothing else.
const CHARTED: Color = Color::hex(0x8A9BB3);

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
        if let Some(id) = sector.selected_fleet
            && ui.clip().contains(point)
        {
            fleet::order(sector, id, camera.to_world(point), now);
        }
    }
    // Orders commit first so a fresh glide charts its start and asks for frames.
    if fleet::advance(sector, now) {
        ui.state.request_redraw_after(0.0);
    }

    boundaries(ui, &camera);
    let hits = fleet::hit_rects(&camera, sector, now, scale);
    // A ship parked over a planet takes pointer selection priority.
    let on_ship = hits.iter().any(|(_, hit)| hit.contains(ui.state.press_pos));
    // Planets swell a little as the camera closes in, but never scale one-to-one;
    // pulled far back they become chart dots.
    let relative = sector.view.zoom / ZoomRange::for_region(region, scale).start;
    let chart = relative < CHART_ZOOM;
    let closeness = relative.sqrt().clamp(0.3, 1.5);
    let halo = 8.0 * scale * closeness.min(1.0);
    let line = scale * 2.0 * closeness.clamp(0.6, 1.0);
    let sources = sector.shown_vision_sources(now);
    for (index, planet) in PLANETS.iter().enumerate() {
        let point = camera.to_screen(planet.position);
        let radius = 26.0 * scale * closeness;
        let hit = Rect::from_center_size(
            point + Vec2::new(0.0, 15.0 * scale),
            Vec2::new(150.0 * scale, 110.0 * scale),
        );
        let id = ui.id(planet.name);
        let sense = if on_ship { Sense::NONE } else { Sense::CLICK };
        let mut response = ui.interact(id, hit, sense);
        if !hit.intersection(&ui.clip()).is_empty() {
            ui.focusable(id, hit);
            ui.key_click(id, &mut response);
        }
        if response.clicked {
            sector.selected = Some(index);
            sector.details_tab = 0;
            sector.selected_fleet = None;
            sector.message = None;
            ui.state.request_rebuild = true;
        }
        if response.hovered {
            ui.state.cursor_icon = CursorIcon::Pointer;
        }
        let focus = if sector.selected == Some(index) || response.hovered {
            1.0
        } else {
            0.45
        };
        let state = sector.game.planets[index].clone();
        let explored = sector.chart.explored_at(planet.position);
        // Surveyed worlds show what is there; in view they are lit, remembered they dim.
        let lit = if !explored {
            0.0
        } else if sources
            .iter()
            .any(|(center, radius)| (planet.position - *center).length() <= *radius)
        {
            1.0
        } else {
            0.55
        };
        if explored {
            let allegiance = allegiance(state.owner).fade(lit);
            ui.draw
                .circle(point, radius + halo * 1.5, allegiance.fade(0.07));
            ui.draw
                .ring(point, radius + halo, line, allegiance.fade(focus));
            ui.draw.circle_gradient(
                point,
                radius,
                planet.color.fade(lit),
                planet.color.scale_rgb(0.2).fade(lit),
            );
            if state.securing {
                // Influence builds clockwise from the top until the world is secured.
                let sweep =
                    TAU * f64::from(state.influence) / f64::from(planet.secure_turns.max(1));
                ui.draw.arc(
                    point,
                    radius + halo + 6.0 * scale,
                    -FRAC_PI_2,
                    -FRAC_PI_2 + sweep.max(0.08),
                    3.0 * scale,
                    PLAYER_COLOR.fade(0.9),
                );
            }
        } else {
            // Charted from afar: the locals know where it is, not what is there.
            ui.draw
                .ring(point, radius * 0.75, line, CHARTED.fade(focus * 0.7 + 0.15));
        }
        if !chart {
            let label = Rect::from_center_size(
                point + Vec2::new(0.0, radius + 35.0 * scale),
                Vec2::new(175.0 * scale, 45.0 * scale),
            );
            let text = ui
                .theme
                .text
                .fade(if explored { lit.max(0.7) } else { 0.5 });
            ui.text_centered(planet.name, &ui.text_style(), label, text);
        }
        ui.focus_ring(id, hit);
    }

    fleet::draw_all(ui, &camera, sector, chart, &hits);
    let response = ui.interact(ui.id("pan"), ui.clip(), Sense::DRAG);
    if response.dragging {
        sector.view.pan_by(response.drag_delta, scale);
        ui.state.cursor_icon = CursorIcon::Grabbing;
        if response.drag_delta != Vec2::ZERO {
            ui.state.request_rebuild = true;
        }
    }
}

fn allegiance(owner: Option<Side>) -> Color {
    match owner {
        Some(Side::Player) => PLAYER_COLOR,
        Some(Side::Dominion) => DOMINION_COLOR,
        None => FREE_COLOR,
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
