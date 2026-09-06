//! The planet details panel: a card on the left of the map, in from the corner,
//! with a portrait beside the name. It floats a layer up so it takes its clicks
//! first; clicking anywhere outside it closes it.
use lntrn_math::{Color, Rect, Vec2};
use lntrn_ui::Ui;

use crate::actions::{self, Action};
use crate::economy::{Resource, ShipKind};
use crate::icons::Icon;
use crate::interface::panel_background;
use crate::layout;
use crate::planets::PLANETS;
use crate::sector::{Sector, Selection};
use crate::theme;
use crate::world::Side;

const WIDTH: f64 = 560.0;
const HEIGHT: f64 = 720.0;
/// In from the map's left edge and down from the Command Strip.
const OFFSET: Vec2 = Vec2::new(120.0, 90.0);
const PADDING: f64 = 25.0;
const PORTRAIT: f64 = 130.0;

pub(crate) fn rect(map: Rect, scale: f64) -> Rect {
    let offset = OFFSET * scale;
    let height = (HEIGHT * scale).min(map.height() - offset.y - 40.0 * scale);
    Rect::from_min_size(map.min + offset, Vec2::new(WIDTH * scale, height))
}

pub(crate) fn draw(ui: &mut Ui, rect: Rect, sector: &mut Sector, selection: Selection) {
    match selection {
        Selection::Planet(index) => planet(ui, rect, sector, index),
        Selection::Poi(id) => wreck(ui, rect, sector, id),
    }
}

/// A charted wreck: what it is, what it might hold, and the Salvage button.
fn wreck(ui: &mut Ui, rect: Rect, sector: &mut Sector, id: u32) {
    let Some(poi) = sector.game.poi(id).cloned() else {
        sector.selected = None;
        return;
    };
    let scale = ui.m.scale;
    let layer = ui.layer() + 1;
    ui.state.keep_popup(rect, layer);
    let widget_id = ui.id("planet-details");
    let window = ui.clip();
    let pad = PADDING * scale;
    let portrait = Rect::from_min_size(rect.min + Vec2::splat(pad), Vec2::splat(PORTRAIT * scale));
    let column = Rect::new(
        Vec2::new(portrait.max.x + 20.0 * scale, rect.min.y + pad),
        rect.max - Vec2::splat(pad),
    );
    let mut header = Ui::new(
        ui.draw, ui.text, ui.theme, ui.m, ui.state, column, rect, widget_id, layer,
    );
    header.set_window_rect(window);
    panel_background(&mut header, rect);
    header.draw.rect(portrait, Color::hex(0x030304));
    header
        .draw
        .stroke_rect(portrait, 2.0 * scale, 4.0 * scale, theme::EDGE);
    Icon::Wreck.draw(&mut header, portrait.center(), portrait.width() * 0.62);
    header.heading(poi.kind.short_name());
    header.label_dim(poi.kind.name());
    let sector_name = layout::region_of(poi.position).map_or("", |r| r.name);
    header.label_dim(&format!("Sector {sector_name}"));
    let reached = header.finish();

    let body = Rect::new(
        Vec2::new(rect.min.x + pad, reached.max(portrait.max.y) + 20.0 * scale),
        rect.max - Vec2::splat(pad),
    );
    let mut body = Ui::new(
        ui.draw, ui.text, ui.theme, ui.m, ui.state, body, rect, widget_id, layer,
    );
    body.set_window_rect(window);
    body.paragraph(poi.kind.description());
    body.space(10.0 * scale);
    body.label_dim("Salvage may hold");
    let (low, high) = poi.kind.alloys();
    let (chance, few, many) = poi.kind.electronics();
    body.row(|ui| {
        Icon::Alloys.badge(ui, &format!("{low} to {high}"));
        Icon::Electronics.badge(ui, &format!("{:.0}% for {few} to {many}", chance * 100.0));
    });
    body.space(15.0 * scale);
    let action = Action::Salvage { poi: id };
    let response = body.button_wide(action.definition().name);
    body.tooltip(&response, action.definition().description);
    if response.clicked {
        match actions::execute(&mut sector.game, Side::Player, action) {
            Ok(actions::Preview::Salvaged { reward }) => {
                sector.message = Some(format!("Salvaged {}.", reward.describe()));
                sector.selected = None;
            }
            Ok(_) => {}
            Err(error) => sector.message = Some(error.to_string()),
        }
        body.state.request_rebuild = true;
    }
    body.finish();
    ui.draw.set_layer(ui.layer());
}

fn planet(ui: &mut Ui, rect: Rect, sector: &mut Sector, index: usize) {
    let scale = ui.m.scale;
    let layer = ui.layer() + 1;
    ui.state.keep_popup(rect, layer);
    let id = ui.id("planet-details");
    let window = ui.clip();
    let pad = PADDING * scale;
    let portrait = Rect::from_min_size(rect.min + Vec2::splat(pad), Vec2::splat(PORTRAIT * scale));
    let column = Rect::new(
        Vec2::new(portrait.max.x + 20.0 * scale, rect.min.y + pad),
        rect.max - Vec2::splat(pad),
    );
    let explored = sector.chart.explored_at(PLANETS[index].position);

    // The column beside the portrait: name, security, the secure button, yields.
    let mut header = Ui::new(
        ui.draw, ui.text, ui.theme, ui.m, ui.state, column, rect, id, layer,
    );
    header.set_window_rect(window);
    panel_background(&mut header, rect);
    picture(&mut header, portrait, index, explored);
    heading(&mut header, index, explored);
    let reached = header.finish();

    // Below both: who holds it, and the shipyard on the home world.
    let body = Rect::new(
        Vec2::new(rect.min.x + pad, reached.max(portrait.max.y) + 20.0 * scale),
        rect.max - Vec2::splat(pad),
    );
    let mut body = Ui::new(
        ui.draw, ui.text, ui.theme, ui.m, ui.state, body, rect, id, layer,
    );
    body.set_window_rect(window);
    let holder = match (explored, sector.game.planets[index].owner) {
        (false, _) => "Uncharted",
        (true, Some(side)) => side.name(),
        (true, None) => "Unclaimed",
    };
    let sector_name = layout::region_of(PLANETS[index].position).map_or("", |r| r.name);
    body.label_dim(&format!("{holder} · Sector {sector_name}"));
    if explored && sector.game.contested(index) {
        body.label_dim("Contested: an enemy ship is in range");
    }
    if explored {
        body.space(6.0 * scale);
        secure_controls(&mut body, sector, index);
    }
    if explored && PLANETS[index].home {
        let mut tab = sector.details_tab;
        if body.tabs(&mut tab, &["Overview", "Shipyard"]) {
            sector.details_tab = tab;
            body.state.request_rebuild = true;
        }
        if tab == 1 {
            shipyard(&mut body, sector);
        }
    }
    body.finish();
    ui.draw.set_layer(ui.layer());
}

/// A framed portrait: the world lit from the upper left, or a question mark
/// until a ship has seen it.
fn picture(ui: &mut Ui, frame: Rect, index: usize, explored: bool) {
    let scale = ui.m.scale;
    ui.draw.rect(frame, Color::hex(0x030304));
    ui.draw
        .stroke_rect(frame, 2.0 * scale, 4.0 * scale, theme::EDGE);
    let center = frame.center();
    let radius = frame.width() * 0.32;
    if !explored {
        ui.draw
            .ring(center, radius, 2.0 * scale, theme::CHARTED.fade(0.6));
        ui.text_centered("?", &ui.text_style().bold(), frame, theme::TEXT_DIM);
        return;
    }
    let color = PLANETS[index].color;
    ui.draw.circle(center, radius * 1.35, color.fade(0.12));
    ui.draw
        .circle_gradient(center, radius, color, color.scale_rgb(0.22));
    ui.draw.circle(
        center + Vec2::new(-radius * 0.35, -radius * 0.38),
        radius * 0.24,
        Color::WHITE.fade(0.22),
    );
    ui.draw.ring(
        center,
        radius + 1.0 * scale,
        1.5 * scale,
        Color::WHITE.fade(0.08),
    );
}

/// The secure button, or the effort's progress once it is under way.
fn secure_controls(ui: &mut Ui, sector: &mut Sector, index: usize) {
    let planet = &PLANETS[index];
    let state = sector.game.planets[index].clone();
    if state.owner == Some(Side::Player) {
        return;
    }
    if state.securing {
        ui.progress(
            &format!("{} / {} turns", state.influence, planet.secure_turns),
            f64::from(state.influence) / f64::from(planet.secure_turns.max(1)),
        );
        return;
    }
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

/// Name with the security icons beside it, then yields.
fn heading(ui: &mut Ui, index: usize, explored: bool) {
    let planet = &PLANETS[index];
    let scale = ui.m.scale;
    ui.row(|ui| {
        ui.heading(planet.name);
        if explored {
            Icon::Secure.badge(ui, &planet.secure_turns.to_string());
            Icon::Decay.badge(ui, &planet.decay.to_string());
        }
    });
    if !explored {
        ui.paragraph("Charted from afar. Send a ship to see what is there.");
        return;
    }
    ui.space(6.0 * scale);
    let yields: Vec<(Icon, u32)> = [
        (Icon::Alloys, Resource::Alloys),
        (Icon::Electronics, Resource::Electronics),
    ]
    .into_iter()
    .map(|(icon, resource)| (icon, planet.yield_per_turn.get(resource)))
    .filter(|(_, amount)| *amount > 0)
    .collect();
    if yields.is_empty() {
        ui.label_dim("Produces nothing");
    } else {
        ui.row(|ui| {
            for (icon, amount) in yields {
                icon.badge(ui, &format!("+{amount}"));
            }
        });
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
    ui.row(|ui| {
        ui.label(&format!("{} costs", kind.name()));
        Icon::Alloys.badge(ui, &cost.alloys.to_string());
        Icon::Electronics.badge(ui, &cost.electronics.to_string());
    });
    ui.space(10.0 * scale);
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
