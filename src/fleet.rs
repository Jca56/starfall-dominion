//! The player's fleet on the map: selection, order previews, and the glide that
//! plays a committed order out on screen after the rules have already resolved it.
use lntrn_math::{Color, Rect, Vec2};
use lntrn_ui::{CursorIcon, Sense, Ui};

use crate::actions::{self, Action, Preview};
use crate::camera::Camera;
use crate::sector::Sector;
use crate::world::Side;

/// World units per second a ship covers on screen; short hops still take a beat.
const GLIDE_SPEED: f64 = 240.0;
const GLIDE_SECONDS: (f64, f64) = (0.2, 0.9);
const ACCENT: Color = Color::hex(0xA2E7FF);

/// A committed order playing out on screen. The rules moved the fleet instantly.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Travel {
    from: Vec2,
    to: Vec2,
    start: f64,
    seconds: f64,
}

impl Travel {
    fn new(from: Vec2, to: Vec2, now: f64) -> Self {
        let seconds = ((to - from).length() / GLIDE_SPEED).clamp(GLIDE_SECONDS.0, GLIDE_SECONDS.1);
        Self {
            from,
            to,
            start: now,
            seconds,
        }
    }

    fn progress(&self, now: f64) -> f64 {
        ((now - self.start) / self.seconds).clamp(0.0, 1.0)
    }

    pub(crate) fn done(&self, now: f64) -> bool {
        self.progress(now) >= 1.0
    }

    fn position(&self, now: f64) -> Vec2 {
        // Ease out: the ship leaves briskly and settles gently.
        let t = 1.0 - (1.0 - self.progress(now)).powi(3);
        self.from + (self.to - self.from) * t
    }
}

/// Where the ship is drawn this frame: mid-glide, or parked where the rules put it.
pub(crate) fn shown_position(sector: &Sector, now: f64) -> Vec2 {
    match &sector.travel {
        Some(travel) => travel.position(now),
        None => sector.game.fleets[0].position,
    }
}

/// Play the glide forward: land finished orders, chart what the ship passes as it
/// flies, and let the screen's chart catch up with the rules once nothing moves.
/// Returns true while frames are still needed.
pub(crate) fn advance(sector: &mut Sector, now: f64) -> bool {
    if sector.travel.is_some_and(|travel| travel.done(now)) {
        sector.travel = None;
    }
    if sector.travel.is_some() {
        let vision = sector.game.fleets[0].vision;
        let position = shown_position(sector, now);
        sector.chart.reveal(position, vision);
        sector.chart.touch();
        return true;
    }
    if sector.synced != sector.game.fog.version() {
        sector.chart = sector.game.fog.clone();
        sector.synced = sector.game.fog.version();
    }
    false
}

pub(crate) fn hit_rect(camera: &Camera, sector: &Sector, now: f64, scale: f64) -> Rect {
    Rect::from_center_size(
        camera.to_screen(shown_position(sector, now)),
        Vec2::splat(80.0 * scale),
    )
}

/// Commit an order. On success the ship glides from wherever it is shown now.
pub(crate) fn order(sector: &mut Sector, destination: Vec2, now: f64) {
    let from = shown_position(sector, now);
    let action = Action::MoveFleet {
        fleet_id: 0,
        destination,
    };
    match actions::execute(&mut sector.game, Side::Player, action) {
        Ok(Preview::Move { destination, .. }) => {
            sector.heading = (destination - from).angle();
            sector.travel = Some(Travel::new(from, destination, now));
            sector.message = None;
        }
        Ok(Preview::EndTurn) => {}
        Err(error) => sector.message = Some(error.to_string()),
    }
}

/// `chart`: the map is zoomed far out, so names hide and only the ship stays.
pub(crate) fn draw(ui: &mut Ui, camera: &Camera, hit: Rect, sector: &mut Sector, chart: bool) {
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
    let point = hit.center();
    if sector.fleet_selected {
        // Range and preview start where the rules have the fleet: the next order begins there.
        let anchor = camera.to_screen(fleet.position);
        ui.draw.ring(
            anchor,
            fleet.remaining * camera.pixels_per_unit,
            scale,
            ACCENT.fade(0.35),
        );
        ui.draw.ring(point, 32.0 * scale, 2.0 * scale, ACCENT);
        if ui.state.pointer_in_window && ui.clip().contains(ui.state.pointer) {
            preview(ui, camera, sector, anchor);
        }
    }
    ship(ui, point, sector.heading, scale, sector.travel.is_some());
    if !chart {
        let label = Rect::from_center_size(
            point + Vec2::new(0.0, 52.0) * scale,
            Vec2::new(230.0, 45.0) * scale,
        );
        ui.text_centered(
            sector.game.fleets[0].name,
            &ui.text_style(),
            label,
            ui.theme.text,
        );
    }
    ui.focus_ring(id, hit);
}

fn preview(ui: &mut Ui, camera: &Camera, sector: &Sector, anchor: Vec2) {
    let scale = ui.m.scale;
    let fleet = &sector.game.fleets[0];
    let action = Action::MoveFleet {
        fleet_id: fleet.id,
        destination: camera.to_world(ui.state.pointer),
    };
    let Ok(Preview::Move { destination, cost }) =
        actions::preview(&sector.game, Side::Player, action)
    else {
        return;
    };
    let endpoint = camera.to_screen(destination);
    ui.draw.line(anchor, endpoint, 2.0 * scale, ACCENT);
    ui.draw.ring(endpoint, 7.0 * scale, 2.0 * scale, ACCENT);
    let label = Rect::from_min_size(
        endpoint + Vec2::new(15.0, -45.0) * scale,
        Vec2::new(200.0, 45.0) * scale,
    );
    ui.fill_square(label, Color::hex(0x101925).fade(0.9));
    ui.text_in_rect(
        &format!("{cost:.0} units"),
        &ui.text_style(),
        label,
        ui.theme.text,
    );
}

/// A simple silhouette from Lantern primitives, nose toward the heading.
fn ship(ui: &mut Ui, point: Vec2, heading: f64, scale: f64, under_way: bool) {
    let at = |x: f64, y: f64| point + Vec2::new(x, y).rotate(heading) * scale;
    if under_way {
        // Engine glow trails the tail while the ship is under way.
        ui.draw
            .circle(at(-16.0, 0.0), 9.0 * scale, ACCENT.fade(0.3));
    }
    ui.draw
        .triangle(at(24.0, 0.0), at(-18.0, -17.0), at(-10.0, 0.0), ACCENT);
    ui.draw.triangle(
        at(24.0, 0.0),
        at(-10.0, 0.0),
        at(-18.0, 17.0),
        Color::hex(0x4D91BC),
    );
}
