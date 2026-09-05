use lntrn_math::{Color, Rect, Vec2};
use lntrn_ui::{CursorIcon, Sense, Ui};

use crate::actions::{self, Action, Preview};
use crate::camera::Camera;
use crate::sector::{PLANETS, Sector};
use crate::world::{Side, WORLD_SIZE};

pub(crate) fn draw(ui: &mut Ui, map: Rect, sector: &mut Sector) {
    let scale = ui.m.scale;
    if ui.clip().contains(ui.state.pointer) && ui.state.wheel.y != 0.0 {
        let old = sector.zoom;
        sector.zoom = (old * (ui.state.wheel.y * 0.002).exp()).clamp(0.6, 3.5);
        let anchor = (ui.state.pointer - map.center()) / scale;
        sector.pan = anchor - (anchor - sector.pan) * (sector.zoom / old);
    }
    let camera = Camera::new(map, scale, sector.pan, sector.zoom);
    for point in std::mem::take(&mut sector.move_requests) {
        if sector.fleet_selected && ui.clip().contains(point) {
            let action = Action::MoveFleet {
                fleet_id: 0,
                destination: camera.to_world(point),
            };
            sector.message = actions::execute(&mut sector.game, Side::Player, action)
                .err()
                .map(|error| error.to_string());
        }
    }

    boundaries(ui, &camera);
    let ship_point = camera.to_screen(sector.game.fleets[0].position);
    let ship_hit = Rect::from_center_size(ship_point, Vec2::splat(80.0 * scale));
    for (index, planet) in PLANETS.iter().enumerate() {
        let point = camera.to_screen(planet.position);
        let radius = 25.0 * scale * sector.zoom.sqrt();
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
            .circle(point, radius + 12.0 * scale, allegiance.fade(0.07));
        ui.draw.ring(
            point,
            radius + 8.0 * scale,
            scale * 2.0,
            allegiance.fade(if sector.selected == Some(index) || response.hovered {
                1.0
            } else {
                0.45
            }),
        );
        ui.draw
            .circle_gradient(point, radius, planet.color, planet.color.scale_rgb(0.2));
        let label = Rect::from_center_size(
            point + Vec2::new(0.0, radius + 35.0 * scale),
            Vec2::new(175.0 * scale, 45.0 * scale),
        );
        ui.text_centered(planet.name, &ui.text_style(), label, ui.theme.text);
        ui.focus_ring(id, hit);
    }

    fleet(ui, &camera, ship_hit, sector);
    let response = ui.interact(ui.id("pan"), ui.clip(), Sense::DRAG);
    if response.dragging {
        sector.pan += response.drag_delta / scale;
        ui.state.cursor_icon = CursorIcon::Grabbing;
        if response.drag_delta != Vec2::ZERO {
            ui.state.request_rebuild = true;
        }
    }
}

fn fleet(ui: &mut Ui, camera: &Camera, hit: Rect, sector: &mut Sector) {
    let scale = ui.m.scale;
    let id = ui.id("Farlight Scout");
    let mut response = ui.interact(id, hit, Sense::CLICK);
    if !hit.intersection(&ui.clip()).is_empty() {
        ui.focusable(id, hit);
        ui.key_click(id, &mut response);
    }
    if response.clicked {
        sector.fleet_selected = true;
        sector.selected = None;
        sector.message = None;
        ui.state.request_rebuild = true;
    }
    if response.hovered {
        ui.state.cursor_icon = CursorIcon::Pointer;
    }
    let fleet = &sector.game.fleets[0];
    let point = camera.to_screen(fleet.position);
    let accent = Color::hex(0xA2E7FF);
    if sector.fleet_selected {
        ui.draw.ring(
            point,
            fleet.remaining * camera.pixels_per_unit,
            scale,
            accent.fade(0.35),
        );
        ui.draw.ring(point, 32.0 * scale, 2.0 * scale, accent);
        if ui.state.pointer_in_window && ui.clip().contains(ui.state.pointer) {
            let action = Action::MoveFleet {
                fleet_id: fleet.id,
                destination: camera.to_world(ui.state.pointer),
            };
            if let Ok(Preview::Move { destination, cost }) =
                actions::preview(&sector.game, Side::Player, action)
            {
                let endpoint = camera.to_screen(destination);
                ui.draw.line(point, endpoint, 2.0 * scale, accent);
                ui.draw.ring(endpoint, 7.0 * scale, 2.0 * scale, accent);
                let label = Rect::from_min_size(
                    endpoint + Vec2::new(15.0, -45.0) * scale,
                    Vec2::new(200.0, 45.0) * scale,
                );
                ui.fill_square(label, Color::hex(0x101925).fade(0.9));
                ui.text_in_rect(
                    &format!("{cost:.1} units"),
                    &ui.text_style(),
                    label,
                    ui.theme.text,
                );
            }
        }
    }
    // A simple ship silhouette made from Lantern draw primitives.
    ui.draw.triangle(
        point + Vec2::new(24.0, 0.0) * scale,
        point + Vec2::new(-18.0, -17.0) * scale,
        point + Vec2::new(-10.0, 0.0) * scale,
        accent,
    );
    ui.draw.triangle(
        point + Vec2::new(24.0, 0.0) * scale,
        point + Vec2::new(-10.0, 0.0) * scale,
        point + Vec2::new(-18.0, 17.0) * scale,
        Color::hex(0x4D91BC),
    );
    let label = Rect::from_center_size(
        point + Vec2::new(0.0, 52.0) * scale,
        Vec2::new(230.0, 45.0) * scale,
    );
    ui.text_centered(fleet.name, &ui.text_style(), label, ui.theme.text);
    ui.focus_ring(id, hit);
}

fn boundaries(ui: &mut Ui, camera: &Camera) {
    let bounds = Rect::new(camera.to_screen(Vec2::ZERO), camera.to_screen(WORLD_SIZE));
    ui.draw
        .stroke_rect(bounds, ui.m.px(1.0), 0.0, Color::hex(0x617A95).fade(0.65));
    for x in [WORLD_SIZE.x / 3.0, WORLD_SIZE.x * 2.0 / 3.0] {
        ui.draw.line(
            camera.to_screen(Vec2::new(x, 0.0)),
            camera.to_screen(Vec2::new(x, WORLD_SIZE.y)),
            ui.m.px(1.0),
            Color::hex(0x617A95).fade(0.25),
        );
    }
}
