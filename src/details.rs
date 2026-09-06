//! The planet details popup: anchored above the planet and drawn a layer up, so
//! it floats over the map and takes its clicks first.
use lntrn_math::{Rect, Vec2};
use lntrn_ui::Ui;

use crate::actions::{self, Action};
use crate::camera::Camera;
use crate::economy::{Resource, ShipKind};
use crate::interface::panel_background;
use crate::layout;
use crate::planets::PLANETS;
use crate::sector::Sector;
use crate::world::Side;

const WIDTH: f64 = 470.0;
const HEIGHT: f64 = 500.0;
/// The Shipyard tab strip adds a row on the home world.
const TAB_ROW: f64 = 75.0;
/// Space between the planet and the popup's edge.
const GAP: f64 = 60.0;

/// Where the popup sits this frame: above the planet, or below when there is
/// no room, and never off the sides of the map.
pub(crate) fn rect(map: Rect, camera: &Camera, index: usize, scale: f64) -> Rect {
    let planet = &PLANETS[index];
    let point = camera.to_screen(planet.position);
    let height = HEIGHT + if planet.home { TAB_ROW } else { 0.0 };
    let size = Vec2::new(WIDTH, height) * scale;
    let margin = 10.0 * scale;
    let above = point.y - GAP * scale - size.y;
    let y = if above >= map.min.y + margin {
        above
    } else {
        point.y + GAP * scale
    };
    let x = (point.x - size.x * 0.5).clamp(
        map.min.x + margin,
        (map.max.x - margin - size.x).max(map.min.x + margin),
    );
    Rect::from_min_size(Vec2::new(x, y), size)
}

pub(crate) fn draw(ui: &mut Ui, rect: Rect, sector: &mut Sector, index: usize) {
    let scale = ui.m.scale;
    let layer = ui.layer() + 1;
    ui.state.keep_popup(rect, layer);
    let id = ui.id("planet-details");
    let window = ui.clip();
    let mut child = Ui::new(
        ui.draw,
        ui.text,
        ui.theme,
        ui.m,
        ui.state,
        rect.shrink(25.0 * scale),
        rect,
        id,
        layer,
    );
    child.set_window_rect(window);
    panel_background(&mut child, rect);
    contents(&mut child, sector, index);
    child.finish();
    ui.draw.set_layer(ui.layer());
}

fn contents(ui: &mut Ui, sector: &mut Sector, index: usize) {
    let planet = &PLANETS[index];
    let scale = ui.m.scale;
    ui.heading(planet.name);
    let explored = sector.chart.explored_at(planet.position);
    let owner = sector.game.planets[index].owner;
    let sector_name = layout::region_of(planet.position).map_or("", |region| region.name);
    let holder = match (explored, owner) {
        (false, _) => "Uncharted",
        (true, Some(side)) => side.name(),
        (true, None) => "Unclaimed",
    };
    ui.label_dim(&format!("{holder} · Sector {sector_name}"));
    if !explored {
        ui.space(15.0 * scale);
        ui.paragraph("Charted from afar. Send a ship to see what is there.");
    } else {
        let mut tab = sector.details_tab;
        if planet.home && ui.tabs(&mut tab, &["Overview", "Shipyard"]) {
            sector.details_tab = tab;
            ui.state.request_rebuild = true;
        }
        if planet.home && tab == 1 {
            shipyard(ui, sector);
        } else {
            overview(ui, sector, index);
        }
    }
    ui.space(15.0 * scale);
    if ui.button_wide("Close").clicked {
        sector.selected = None;
        ui.state.request_rebuild = true;
    }
}

fn overview(ui: &mut Ui, sector: &mut Sector, index: usize) {
    let planet = &PLANETS[index];
    let scale = ui.m.scale;
    let state = sector.game.planets[index].clone();
    ui.space(10.0 * scale);
    ui.label(&format!(
        "Secure: {} turns · decay {}",
        planet.secure_turns, planet.decay
    ));
    let production: Vec<String> = Resource::ALL
        .iter()
        .filter(|resource| planet.yield_per_turn.get(**resource) > 0)
        .map(|resource| {
            format!(
                "{} +{}",
                short(*resource),
                planet.yield_per_turn.get(*resource)
            )
        })
        .collect();
    ui.label(&if production.is_empty() {
        "Produces nothing".to_string()
    } else {
        format!("Per turn: {}", production.join(" · "))
    });
    if state.owner == Some(Side::Player) {
        if sector.game.contested(index) {
            ui.label_dim("Contested: an enemy ship is in range");
        }
        return;
    }
    ui.space(10.0 * scale);
    if state.securing {
        ui.progress(
            &format!("{} / {} turns", state.influence, planet.secure_turns),
            f64::from(state.influence) / f64::from(planet.secure_turns.max(1)),
        );
    } else {
        let action = Action::SecurePlanet { planet: index };
        let response = ui.button_wide(action.definition().name);
        ui.tooltip(&response, action.definition().description);
        if response.clicked {
            sector.message = actions::execute(&mut sector.game, Side::Player, action)
                .err()
                .map(|error| error.to_string());
            ui.state.request_rebuild = true;
        }
    }
}

fn shipyard(ui: &mut Ui, sector: &mut Sector) {
    let scale = ui.m.scale;
    ui.space(10.0 * scale);
    if let Some(build) = &sector.game.shipyard {
        let total = build.kind.build_turns().max(1);
        ui.label(&format!("Building {}", build.kind.name()));
        ui.progress(
            &format!("{} turns left", build.turns_left),
            1.0 - f64::from(build.turns_left) / f64::from(total),
        );
        return;
    }
    let kind = ShipKind::Scout;
    let cost = kind.cost();
    ui.label(&format!(
        "{}: {} Alloys · {} Electronics",
        kind.name(),
        cost.alloys,
        cost.electronics
    ));
    let action = Action::BuildShip { kind };
    let response = ui.button_wide(&format!("Build {}", kind.name()));
    ui.tooltip(&response, action.definition().description);
    if response.clicked {
        sector.message = actions::execute(&mut sector.game, Side::Player, action)
            .err()
            .map(|error| error.to_string());
        ui.state.request_rebuild = true;
    }
}

/// Short names fit the popup; the Command Strip spells them out.
fn short(resource: Resource) -> &'static str {
    match resource {
        Resource::Alloys => "Alloys",
        Resource::Electronics => "Electronics",
    }
}
