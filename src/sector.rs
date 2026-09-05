use crate::actions::{self, Action};
use crate::camera::View;
use crate::fleet::Travel;
use crate::layout;
use crate::planets::PLANETS;
use crate::world::{Game, Side};
use lntrn_math::{Color, Rect, Vec2};
use lntrn_ui::Ui;

use crate::interface::{panel_background, region};

/// Logical heights of the title bar and the turn controls that frame the map.
const TOP_BAR: f64 = 95.0;
const BOTTOM_BAR: f64 = 110.0;

pub(crate) struct Sector {
    pub(crate) selected: Option<usize>,
    pub(crate) view: View,
    pub(crate) game: Game,
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
            game,
            fleet_selected: false,
            travel: None,
            heading: 0.0,
            message: None,
            move_requests: Vec::new(),
        }
    }
}

/// The map between the title bar and the turn controls, in physical pixels.
pub(crate) fn map_region(bounds: Rect, scale: f64) -> Rect {
    Rect::new(
        Vec2::new(0.0, TOP_BAR * scale),
        bounds.max - Vec2::new(0.0, BOTTOM_BAR * scale),
    )
}

pub(crate) fn draw(ui: &mut Ui, bounds: Rect, sector: &mut Sector) -> bool {
    let scale = ui.m.scale;
    let bar = Rect::from_xywh(0.0, 0.0, bounds.width(), TOP_BAR * scale);
    ui.fill_square(bar, Color::hex(0x101925));
    ui.hline(bar.max.y, 0.0, bounds.max.x, Color::hex(0x43546E));
    let title = Rect::from_xywh(
        30.0 * scale,
        10.0 * scale,
        bounds.width() - 195.0 * scale,
        70.0 * scale,
    );
    let style = ui.text_style().bold();
    ui.text_in_rect("FARLIGHT EXPANSE", &style, title, ui.theme.text);
    let mut menu = false;
    region(
        ui,
        Rect::from_xywh(
            bounds.max.x - 145.0 * scale,
            15.0 * scale,
            115.0 * scale,
            65.0 * scale,
        ),
        "top-bar",
        |ui| {
            menu = ui.button_wide("Menu").clicked;
        },
    );
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
            ui.label_dim(if planet.occupied {
                "Dominion occupied"
            } else {
                "Free world"
            });
            if let Some(region) = layout::region_of(planet.position) {
                ui.label_dim(&format!("Sector {}", region.name));
            }
            ui.space(15.0 * scale);
            ui.paragraph(planet.description);
            ui.space(20.0 * scale);
            if ui.button_wide("Close").clicked {
                sector.selected = None;
                ui.state.request_rebuild = true;
            }
        });
    }
    if sector.fleet_selected {
        let fleet = &sector.game.fleets[0];
        let label = format!(
            "Movement: {:.0} / {:.0} units",
            fleet.remaining, fleet.speed
        );
        let text_rect = Rect::from_xywh(
            25.0 * scale,
            bounds.max.y - 100.0 * scale,
            bounds.width() - 390.0 * scale,
            40.0 * scale,
        );
        ui.text_in_rect(
            fleet.name,
            &ui.text_style().bold(),
            text_rect,
            ui.theme.text,
        );
        ui.text_in_rect(
            &label,
            &ui.text_style(),
            text_rect.translate(Vec2::new(0.0, 40.0 * scale)),
            ui.theme.text_dim,
        );
    }
    let turn_rect = Rect::from_xywh(
        bounds.max.x - 350.0 * scale,
        bounds.max.y - 100.0 * scale,
        325.0 * scale,
        85.0 * scale,
    );
    region(ui, turn_rect, "turn-controls", |ui| {
        ui.row(|ui| {
            ui.label(&format!("Turn {}", sector.game.turn));
            let action = Action::EndTurn;
            let response = ui.button(action.definition().name);
            ui.tooltip(&response, action.definition().description);
            if response.clicked {
                let result = actions::execute(&mut sector.game, Side::Player, action)
                    // Until an enemy planner exists, the Dominion passes through the same rules.
                    .and_then(|_| {
                        actions::execute(&mut sector.game, Side::Dominion, Action::EndTurn)
                    });
                sector.message = result.err().map(|error| error.to_string());
                ui.state.request_rebuild = true;
            }
        });
    });
    if let Some(message) = &sector.message {
        let rect = Rect::from_xywh(
            25.0 * scale,
            100.0 * scale,
            bounds.width() - 50.0 * scale,
            50.0 * scale,
        );
        ui.text_in_rect(message, &ui.text_style(), rect, Color::hex(0xF3BB8F));
    }
    menu
}
