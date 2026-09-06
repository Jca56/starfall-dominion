use crate::actions::{self, Action};
use crate::camera::{Camera, View};
use crate::details;
use crate::economy::Resource;
use crate::fleet::{self, ShipVisual};
use crate::fog::Fog;
use crate::world::{Game, Side};
use lntrn_math::{Color, Rect, Vec2};
use lntrn_ui::{Response, Sense, Ui, WidgetId};

use crate::interface::region;

/// Logical height of the Command Strip across the top: resources and the menu.
const TOP_BAR: f64 = 75.0;
/// Logical diameter of the round End Turn button in the bottom-right corner.
const END_TURN_SIZE: f64 = 150.0;
const ALLOY_COLOR: Color = Color::hex(0xC9B79C);
const ELECTRONICS_COLOR: Color = Color::hex(0x8EDBE7);

pub(crate) struct Sector {
    /// The planet whose details popup is open.
    pub(crate) selected: Option<usize>,
    /// Which tab of the popup is showing: 0 Overview, 1 Shipyard.
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
    pub(crate) message: Option<String>,
    pub(crate) move_requests: Vec<Vec2>,
}

impl Default for Sector {
    fn default() -> Self {
        let game = Game::default();
        Self {
            selected: None,
            details_tab: 0,
            view: View::starting_at(game.fleets[0].position),
            chart: game.fog.clone(),
            synced: game.fog.version(),
            game,
            selected_fleet: None,
            ships: Vec::new(),
            message: None,
            move_requests: Vec::new(),
        }
    }
}

impl Sector {
    /// What lights the map on screen: the rules' view, except a gliding ship
    /// shines from where it is drawn.
    pub(crate) fn shown_vision_sources(&self, now: f64) -> Vec<(Vec2, f64)> {
        self.game.vision_sources_shown(Side::Player, |fleet| {
            fleet::shown_position(self, fleet, now)
        })
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
    ui.fill_square(bar, Color::hex(0x05070C));
    ui.hline(bar.max.y, 0.0, bounds.max.x, Color::hex(0x1E2633));
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
    // map's pan can, and is drawn after the map so it sits on top.
    let end_turn = Rect::from_center_size(
        bounds.max - Vec2::splat((25.0 + END_TURN_SIZE * 0.5) * scale),
        Vec2::splat(END_TURN_SIZE * scale),
    );
    let (end_turn_id, end_turn_response) = end_turn_interact(ui, end_turn);

    let map = map_region(bounds, scale);
    let camera = Camera::new(map, scale, sector.view.center, sector.view.zoom);
    let popup = sector
        .selected
        .map(|index| details::rect(map, &camera, index, scale));
    // Right-clicks on floating panels are not orders for the map underneath.
    sector.move_requests.retain(|point| {
        !end_turn.contains(*point) && !popup.is_some_and(|rect| rect.contains(*point))
    });
    region(ui, map, "sector-map", |ui| {
        crate::map::draw(ui, map, sector)
    });
    if let (Some(index), Some(rect)) = (sector.selected, popup) {
        details::draw(ui, rect, sector, index);
    }
    end_turn_draw(ui, end_turn, end_turn_id, &end_turn_response, sector);
    if let Some(message) = &sector.message {
        let rect = Rect::from_xywh(
            25.0 * scale,
            (TOP_BAR + 10.0) * scale,
            bounds.width() - 50.0 * scale,
            50.0 * scale,
        );
        ui.text_in_rect(message, &ui.text_style(), rect, Color::hex(0xF3BB8F));
    }
    menu
}

/// Stockpiles on the Command Strip, each with what arrives when the turn ends.
fn tracker(ui: &mut Ui, sector: &Sector, bar: Rect) {
    let scale = ui.m.scale;
    let income = sector.game.income();
    let style = ui.text_style();
    let mut x = 30.0 * scale;
    for resource in Resource::ALL {
        let glyph = Vec2::new(x + 14.0 * scale, bar.center().y);
        match resource {
            Resource::Alloys => {
                let corners: Vec<Vec2> = (0..6)
                    .map(|i| {
                        glyph
                            + Vec2::from_angle(f64::from(i) * std::f64::consts::FRAC_PI_3)
                                * 13.0
                                * scale
                    })
                    .collect();
                ui.draw.polyline(&corners, 2.5 * scale, ALLOY_COLOR, true);
            }
            Resource::Electronics => {
                let chip = Rect::from_center_size(glyph, Vec2::splat(22.0 * scale));
                ui.draw
                    .stroke_rect(chip, 2.5 * scale, 3.0 * scale, ELECTRONICS_COLOR);
                ui.draw.circle(glyph, 3.5 * scale, ELECTRONICS_COLOR);
            }
        }
        let text = format!(
            "{} {} (+{})",
            resource.name(),
            sector.game.stockpile.get(resource),
            income.get(resource)
        );
        let width = ui.measure(&text, &style);
        let rect = Rect::from_xywh(
            x + 38.0 * scale,
            bar.min.y,
            width + 10.0 * scale,
            bar.height(),
        );
        ui.text_in_rect(&text, &style, rect, ui.theme.text);
        x += 38.0 * scale + width + 50.0 * scale;
    }
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
    ui.draw
        .circle(center, radius, Color::hex(0x101925).fade(0.94));
    ui.draw.ring(
        center,
        radius - 1.5 * scale,
        3.0 * scale,
        ui.theme.accent.fade(strength),
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
