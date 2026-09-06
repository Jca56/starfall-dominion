use crate::actions::{self, Action};
use crate::camera::View;
use crate::details;
use crate::economy::Resource;
use crate::fleet::{self, ShipVisual};
use crate::fog::Fog;
use crate::icons::Icon;
use crate::theme;
use crate::world::{Game, Side};
use lntrn_math::{Rect, Vec2};
use lntrn_ui::{Response, Sense, Ui, WidgetId};

use crate::interface::region;

/// Logical height of the Command Strip across the top: resources and the menu.
const TOP_BAR: f64 = 75.0;
/// Logical diameter of the round End Turn button in the bottom-right corner.
const END_TURN_SIZE: f64 = 150.0;

/// What the details panel is showing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Selection {
    Planet(usize),
    Poi(u32),
}

pub(crate) struct Sector {
    /// What the details panel is open on.
    pub(crate) selected: Option<Selection>,
    /// Which tab of the panel is showing: 0 Overview, 1 Shipyard.
    pub(crate) details_tab: usize,
    pub(crate) view: View,
    pub(crate) game: Game,
    /// The fog as drawn. It trails the rules while a ship glides so the reveal
    /// follows the ship, then catches up once nothing is moving.
    pub(crate) chart: Fog,
    /// The rules' fog version the chart last copied.
    pub(crate) synced: u64,
    pub(crate) selected_fleet: Option<u32>,
    /// Headings and glides, one per ship the screen has drawn.
    pub(crate) ships: Vec<ShipVisual>,
    /// Where the right button went down, while it is held.
    pub(crate) right_press: Option<Vec2>,
    /// The held right button has moved far enough to count as a drag.
    pub(crate) right_dragged: bool,
    pub(crate) message: Option<String>,
}

impl Default for Sector {
    fn default() -> Self {
        Self::new(Game::default())
    }
}

impl Sector {
    pub(crate) fn new(game: Game) -> Self {
        Self {
            selected: None,
            details_tab: 0,
            view: View::starting_at(game.fleets[0].position),
            chart: game.fog.clone(),
            synced: game.fog.version(),
            game,
            selected_fleet: None,
            ships: Vec::new(),
            right_press: None,
            right_dragged: false,
            message: None,
        }
    }

    /// What lights the map on screen: the rules' view, except a gliding ship
    /// shines from where it is drawn.
    pub(crate) fn shown_vision_sources(&self, now: f64) -> Vec<(Vec2, f64)> {
        self.game.vision_sources_shown(Side::Player, |fleet| {
            fleet::shown_position(self, fleet, now)
        })
    }

    /// Drop the selected ship and close the planet panel.
    pub(crate) fn deselect(&mut self) {
        self.selected_fleet = None;
        self.selected = None;
        self.message = None;
    }

    #[cfg(test)]
    pub(crate) fn any_travel(&self) -> bool {
        self.ships.iter().any(|ship| ship.travel.is_some())
    }
}

/// The map below the Command Strip, in physical pixels. It runs to the bottom
/// edge; the turn controls float over it.
pub(crate) fn map_region(bounds: Rect, scale: f64) -> Rect {
    Rect::new(Vec2::new(0.0, TOP_BAR * scale), bounds.max)
}

pub(crate) fn draw(ui: &mut Ui, bounds: Rect, sector: &mut Sector) -> bool {
    let scale = ui.m.scale;
    let bar = Rect::from_xywh(0.0, 0.0, bounds.width(), TOP_BAR * scale);
    ui.fill_square(bar, theme::INK);
    ui.hline(bar.max.y, 0.0, bounds.max.x, theme::EDGE);
    tracker(ui, sector, bar);
    let mut menu = false;
    region(
        ui,
        Rect::from_xywh(
            bounds.max.x - 145.0 * scale,
            5.0 * scale,
            115.0 * scale,
            65.0 * scale,
        ),
        "top-bar",
        |ui| {
            menu = ui.button_wide("Menu").clicked;
        },
    );

    // The End Turn button floats over the map. It takes its click before the
    // map can, and is drawn after the map so it sits on top.
    let end_turn = Rect::from_center_size(
        bounds.max - Vec2::splat((25.0 + END_TURN_SIZE * 0.5) * scale),
        Vec2::splat(END_TURN_SIZE * scale),
    );
    let (end_turn_id, end_turn_response) = end_turn_interact(ui, end_turn);

    let map = map_region(bounds, scale);
    region(ui, map, "sector-map", |ui| {
        crate::map::draw(ui, map, sector)
    });
    if let Some(selection) = sector.selected {
        details::draw(ui, details::rect(map, scale), sector, selection);
    }
    end_turn_draw(ui, end_turn, end_turn_id, &end_turn_response, sector);
    if let Some(message) = &sector.message {
        let rect = Rect::from_xywh(
            25.0 * scale,
            (TOP_BAR + 10.0) * scale,
            bounds.width() - 50.0 * scale,
            50.0 * scale,
        );
        ui.text_in_rect(message, &ui.text_style(), rect, theme::WARNING);
    }
    menu
}

/// Stockpiles on the Command Strip: each icon, what you hold, and what arrives
/// when the turn ends. Hovering an icon names it.
fn tracker(ui: &mut Ui, sector: &Sector, bar: Rect) {
    let scale = ui.m.scale;
    let income = sector.game.income();
    let strip = Rect::from_xywh(
        20.0 * scale,
        (bar.height() - ui.m.widget_h) * 0.5,
        bar.width() - 200.0 * scale,
        ui.m.widget_h,
    );
    region(ui, strip, "resources", |ui| {
        ui.row(|ui| {
            for (icon, resource) in [
                (Icon::Alloys, Resource::Alloys),
                (Icon::Electronics, Resource::Electronics),
            ] {
                let text = format!(
                    "{} (+{})",
                    sector.game.stockpile.get(resource),
                    income.get(resource)
                );
                icon.badge(ui, &text);
            }
        });
    });
}

/// Claim input for the End Turn button before the map is drawn.
fn end_turn_interact(ui: &mut Ui, rect: Rect) -> (WidgetId, Response) {
    let mut result = None;
    region(ui, rect, "turn-controls", |ui| {
        let id = ui.id(Action::EndTurn.definition().name);
        let mut response = ui.interact(id, rect, Sense::CLICK);
        ui.focusable(id, rect);
        ui.key_click(id, &mut response);
        result = Some((id, response));
    });
    result.expect("the turn controls region ran")
}

/// A large round button with the turn counter beside it, drawn over the map.
fn end_turn_draw(ui: &mut Ui, rect: Rect, id: WidgetId, response: &Response, sector: &mut Sector) {
    let scale = ui.m.scale;
    let action = Action::EndTurn;
    let center = rect.center();
    let radius = rect.width() * 0.5;
    let strength = if response.hovered || response.held {
        1.0
    } else {
        0.6
    };
    ui.draw.circle(center, radius, theme::PANEL_FILL.fade(0.94));
    ui.draw.ring(
        center,
        radius - 1.5 * scale,
        3.0 * scale,
        theme::BLUE.fade(strength),
    );
    ui.text_centered(
        action.definition().name,
        &ui.text_style().bold(),
        rect,
        ui.theme.text,
    );
    let counter = Rect::from_xywh(
        rect.min.x - 170.0 * scale,
        center.y - 25.0 * scale,
        160.0 * scale,
        50.0 * scale,
    );
    ui.text_centered(
        &format!("Turn {}", sector.game.turn),
        &ui.text_style(),
        counter,
        ui.theme.text_dim,
    );
    ui.tooltip(response, action.definition().description);
    ui.focus_ring(id, rect);
    if response.clicked {
        let result = actions::execute(&mut sector.game, Side::Player, action)
            // Until an enemy planner exists, the Dominion passes through the same rules.
            .and_then(|_| actions::execute(&mut sector.game, Side::Dominion, Action::EndTurn));
        sector.message = result.err().map(|error| error.to_string());
        ui.state.request_rebuild = true;
    }
}
