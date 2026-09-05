use crate::actions::{self, Action};
use crate::camera::View;
use crate::fleet::{self, Travel};
use crate::fog::Fog;
use crate::layout;
use crate::planets::PLANETS;
use crate::world::{Game, Side};
use lntrn_math::{Color, Rect, Vec2};
use lntrn_ui::{Response, Sense, Ui, WidgetId};

use crate::interface::{panel_background, region};

/// Logical height of the slim bar across the top that holds the menu button.
const TOP_BAR: f64 = 75.0;
/// Logical diameter of the round End Turn button in the bottom-right corner.
const END_TURN_SIZE: f64 = 150.0;

pub(crate) struct Sector {
    pub(crate) selected: Option<usize>,
    pub(crate) view: View,
    pub(crate) game: Game,
    /// The fog as drawn. It trails the rules while a ship glides so the reveal
    /// follows the ship, then catches up once nothing is moving.
    pub(crate) chart: Fog,
    /// The rules' fog version the chart last copied.
    pub(crate) synced: u64,
    pub(crate) fleet_selected: bool,
    /// The order being played out on screen, if any.
    pub(crate) travel: Option<Travel>,
    /// Radians; the ship's nose points along its last order.
    pub(crate) heading: f64,
    pub(crate) message: Option<String>,
    pub(crate) move_requests: Vec<Vec2>,
}

impl Default for Sector {
    fn default() -> Self {
        let game = Game::default();
        Self {
            selected: None,
            view: View::starting_at(game.fleets[0].position),
            chart: game.fog.clone(),
            synced: game.fog.version(),
            game,
            fleet_selected: false,
            travel: None,
            heading: 0.0,
            message: None,
            move_requests: Vec::new(),
        }
    }
}

impl Sector {
    /// What lights the map on screen: the rules' view, except a gliding ship
    /// shines from where it is drawn.
    pub(crate) fn shown_vision_sources(&self, now: f64) -> Vec<(Vec2, f64)> {
        let shown = fleet::shown_position(self, now);
        self.game.vision_sources_shown(Side::Player, |fleet| {
            if fleet.id == 0 { shown } else { fleet.position }
        })
    }
}

/// The map below the top bar, in physical pixels. It runs to the bottom edge;
/// the turn controls float over it.
pub(crate) fn map_region(bounds: Rect, scale: f64) -> Rect {
    Rect::new(Vec2::new(0.0, TOP_BAR * scale), bounds.max)
}

pub(crate) fn draw(ui: &mut Ui, bounds: Rect, sector: &mut Sector) -> bool {
    let scale = ui.m.scale;
    let bar = Rect::from_xywh(0.0, 0.0, bounds.width(), TOP_BAR * scale);
    ui.fill_square(bar, Color::hex(0x05070C));
    ui.hline(bar.max.y, 0.0, bounds.max.x, Color::hex(0x1E2633));
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
    // A right-click on the button is not an order for the map underneath.
    sector
        .move_requests
        .retain(|point| !end_turn.contains(*point));

    let map = map_region(bounds, scale);
    let details = Rect::from_xywh(
        bounds.max.x - 380.0 * scale,
        bar.max.y + 25.0 * scale,
        355.0 * scale,
        480.0 * scale,
    );
    let visible = if sector.selected.is_some() {
        Rect::new(map.min, Vec2::new(details.min.x - 15.0 * scale, map.max.y))
    } else {
        map
    };
    region(ui, visible, "sector-map", |ui| {
        crate::map::draw(ui, map, sector)
    });
    if let Some(index) = sector.selected {
        let planet = &PLANETS[index];
        panel_background(ui, details);
        region(ui, details.shrink(25.0 * scale), "planet-details", |ui| {
            ui.heading(planet.name);
            let explored = sector.chart.explored_at(planet.position);
            ui.label_dim(match (explored, planet.owner) {
                (false, _) => "Uncharted",
                (true, Some(Side::Player)) => "Your world",
                (true, Some(Side::Dominion)) => "Dominion occupied",
                (true, None) => "Free world",
            });
            if let Some(region) = layout::region_of(planet.position) {
                ui.label_dim(&format!("Sector {}", region.name));
            }
            ui.space(15.0 * scale);
            ui.paragraph(if explored {
                planet.description
            } else {
                "Charted from afar. Send a ship to survey it."
            });
            ui.space(20.0 * scale);
            if ui.button_wide("Close").clicked {
                sector.selected = None;
                ui.state.request_rebuild = true;
            }
        });
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
