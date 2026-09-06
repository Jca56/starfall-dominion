//! Fleets on the map: selection, order previews, and the glide that plays a
//! committed order out on screen after the rules have already resolved it.
use lntrn_math::{Color, Rect, Vec2};
use lntrn_ui::{CursorIcon, Sense, Ui};

use crate::actions::{self, Action, Preview};
use crate::camera::Camera;
use crate::sector::Sector;
use crate::world::{Fleet, Side};

/// World units per second a ship covers on screen: a full scout move takes well
/// over a second, so it reads as flying rather than flashing across.
const GLIDE_SPEED: f64 = 110.0;
const GLIDE_SECONDS: (f64, f64) = (0.3, 1.8);
const ACCENT: Color = Color::hex(0xA2E7FF);
const HULL: Color = Color::hex(0x4D91BC);
const ENEMY: Color = Color::hex(0xF38F8F);
const ENEMY_HULL: Color = Color::hex(0xA85A5A);

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
        // Ease in and out: the ship gathers way, cruises, and settles.
        let p = self.progress(now);
        let t = p * p * (3.0 - 2.0 * p);
        self.from + (self.to - self.from) * t
    }
}

/// What the screen remembers about a ship that the rules do not.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ShipVisual {
    pub(crate) id: u32,
    /// Radians; the nose points along the last order.
    pub(crate) heading: f64,
    pub(crate) travel: Option<Travel>,
}

fn visual(sector: &Sector, id: u32) -> Option<&ShipVisual> {
    sector.ships.iter().find(|ship| ship.id == id)
}

fn visual_mut(sector: &mut Sector, id: u32) -> &mut ShipVisual {
    if let Some(index) = sector.ships.iter().position(|ship| ship.id == id) {
        return &mut sector.ships[index];
    }
    sector.ships.push(ShipVisual {
        id,
        heading: 0.0,
        travel: None,
    });
    sector.ships.last_mut().expect("just pushed")
}

/// Where a ship is drawn this frame: mid-glide, or parked where the rules put it.
pub(crate) fn shown_position(sector: &Sector, fleet: &Fleet, now: f64) -> Vec2 {
    visual(sector, fleet.id)
        .and_then(|ship| ship.travel)
        .map_or(fleet.position, |travel| travel.position(now))
}

/// Play every glide forward: land finished orders, chart what a ship passes as
/// it flies, and let the screen's chart catch up with the rules once nothing
/// moves. Returns true while frames are still needed.
pub(crate) fn advance(sector: &mut Sector, now: f64) -> bool {
    let mut animating = false;
    for index in 0..sector.ships.len() {
        let Some(travel) = sector.ships[index].travel else {
            continue;
        };
        if travel.done(now) {
            sector.ships[index].travel = None;
            continue;
        }
        let id = sector.ships[index].id;
        if let Some(fleet) = sector.game.fleets.iter().find(|fleet| fleet.id == id)
            && fleet.owner == Side::Player
        {
            let vision = fleet.vision;
            sector.chart.reveal(travel.position(now), vision);
        }
        animating = true;
    }
    if animating {
        sector.chart.touch();
        return true;
    }
    if sector.synced != sector.game.fog.version() {
        sector.chart = sector.game.fog.clone();
        sector.synced = sector.game.fog.version();
    }
    false
}

/// Commit an order. On success the ship glides from wherever it is shown now.
pub(crate) fn order(sector: &mut Sector, fleet_id: u32, destination: Vec2, now: f64) {
    let Some(from) = sector
        .game
        .fleets
        .iter()
        .find(|fleet| fleet.id == fleet_id)
        .map(|fleet| shown_position(sector, fleet, now))
    else {
        return;
    };
    let action = Action::MoveFleet {
        fleet_id,
        destination,
    };
    match actions::execute(&mut sector.game, Side::Player, action) {
        Ok(Preview::Move { destination, .. }) => {
            let ship = visual_mut(sector, fleet_id);
            ship.heading = (destination - from).angle();
            ship.travel = Some(Travel::new(from, destination, now));
            sector.message = None;
        }
        Ok(_) => {}
        Err(error) => sector.message = Some(error.to_string()),
    }
}

/// Where each ship is on screen this frame, for clicks and priority over planets.
pub(crate) fn hit_rects(
    camera: &Camera,
    sector: &Sector,
    now: f64,
    scale: f64,
) -> Vec<(u32, Rect)> {
    sector
        .game
        .fleets
        .iter()
        .map(|fleet| {
            let point = camera.to_screen(shown_position(sector, fleet, now));
            (
                fleet.id,
                Rect::from_center_size(point, Vec2::splat(80.0 * scale)),
            )
        })
        .collect()
}

/// `chart`: the map is zoomed far out, so names hide and only the ships stay.
pub(crate) fn draw_all(
    ui: &mut Ui,
    camera: &Camera,
    sector: &mut Sector,
    chart: bool,
    hits: &[(u32, Rect)],
) {
    let scale = ui.m.scale;
    for (id, hit) in hits {
        let Some(fleet) = sector.game.fleets.iter().find(|fleet| fleet.id == *id) else {
            continue;
        };
        let (name, owner, position, remaining) = (
            fleet.name.clone(),
            fleet.owner,
            fleet.position,
            fleet.remaining,
        );
        let heading = visual(sector, *id).map_or(0.0, |ship| ship.heading);
        let under_way = visual(sector, *id).is_some_and(|ship| ship.travel.is_some());
        let widget = ui.id(&name);
        let mut response = ui.interact(widget, *hit, Sense::CLICK);
        if !hit.intersection(&ui.clip()).is_empty() {
            ui.focusable(widget, *hit);
            ui.key_click(widget, &mut response);
        }
        if response.clicked && owner == Side::Player {
            sector.selected_fleet = Some(*id);
            sector.selected = None;
            sector.message = None;
            ui.state.request_rebuild = true;
        }
        if response.hovered {
            ui.state.cursor_icon = CursorIcon::Pointer;
        }
        let point = hit.center();
        let (accent, hull) = if owner == Side::Player {
            (ACCENT, HULL)
        } else {
            (ENEMY, ENEMY_HULL)
        };
        if sector.selected_fleet == Some(*id) {
            // Range and preview start where the rules have the fleet: the next order begins there.
            let anchor = camera.to_screen(position);
            ui.draw.ring(
                anchor,
                remaining * camera.pixels_per_unit,
                scale,
                accent.fade(0.35),
            );
            ui.draw.ring(point, 32.0 * scale, 2.0 * scale, accent);
            if ui.state.pointer_in_window && ui.clip().contains(ui.state.pointer) {
                preview(ui, camera, sector, *id, anchor);
            }
        }
        ship(ui, point, heading, scale, under_way, accent, hull);
        if !chart {
            let label = Rect::from_center_size(
                point + Vec2::new(0.0, 52.0) * scale,
                Vec2::new(230.0, 45.0) * scale,
            );
            ui.text_centered(&name, &ui.text_style(), label, ui.theme.text);
        }
        ui.focus_ring(widget, *hit);
    }
}

fn preview(ui: &mut Ui, camera: &Camera, sector: &Sector, fleet_id: u32, anchor: Vec2) {
    let scale = ui.m.scale;
    let action = Action::MoveFleet {
        fleet_id,
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
fn ship(
    ui: &mut Ui,
    point: Vec2,
    heading: f64,
    scale: f64,
    under_way: bool,
    accent: Color,
    hull: Color,
) {
    let at = |x: f64, y: f64| point + Vec2::new(x, y).rotate(heading) * scale;
    if under_way {
        // Engine glow trails the tail while the ship is under way.
        ui.draw
            .circle(at(-16.0, 0.0), 9.0 * scale, accent.fade(0.3));
    }
    ui.draw
        .triangle(at(24.0, 0.0), at(-18.0, -17.0), at(-10.0, 0.0), accent);
    ui.draw
        .triangle(at(24.0, 0.0), at(-10.0, 0.0), at(-18.0, 17.0), hull);
}
