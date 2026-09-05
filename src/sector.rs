use crate::actions::{self, Action};
use crate::world::{Game, Side};
use lntrn_math::{Color, Rect, Vec2};
use lntrn_ui::Ui;

use crate::interface::{panel_background, region};

pub(crate) struct Planet {
    pub(crate) name: &'static str,
    pub(crate) position: Vec2,
    pub(crate) color: Color,
    pub(crate) occupied: bool,
    description: &'static str,
}

// Provisional names and positions for exploring the interface, not simulation data.
pub(crate) const PLANETS: [Planet; 5] = [
    Planet {
        name: "Farlight",
        position: Vec2::new(190.0, 320.0),
        color: Color::hex(0x72C5EE),
        occupied: false,
        description: "A temperate world at the heart of the Expanse.",
    },
    Planet {
        name: "Haven",
        position: Vec2::new(350.0, 140.0),
        color: Color::hex(0x82D5AA),
        occupied: false,
        description: "A peaceful ocean world beneath wide, open skies.",
    },
    Planet {
        name: "Verdant",
        position: Vec2::new(500.0, 530.0),
        color: Color::hex(0xB7CE85),
        occupied: false,
        description: "A fertile world on the edge of the free planets.",
    },
    Planet {
        name: "Cinder",
        position: Vec2::new(740.0, 290.0),
        color: Color::hex(0xECAC73),
        occupied: true,
        description: "An industrial world under Dominion occupation.",
    },
    Planet {
        name: "Vesper",
        position: Vec2::new(880.0, 520.0),
        color: Color::hex(0xB496D7),
        occupied: true,
        description: "A distant world beyond the Dominion frontier.",
    },
];

pub(crate) struct Sector {
    pub(crate) selected: Option<usize>,
    pub(crate) pan: Vec2,
    pub(crate) zoom: f64,
    pub(crate) game: Game,
    pub(crate) fleet_selected: bool,
    pub(crate) message: Option<String>,
    pub(crate) move_requests: Vec<Vec2>,
}

impl Default for Sector {
    fn default() -> Self {
        Self {
            selected: None,
            pan: Vec2::ZERO,
            zoom: 1.0,
            game: Game::default(),
            fleet_selected: false,
            message: None,
            move_requests: Vec::new(),
        }
    }
}

pub(crate) fn draw(ui: &mut Ui, bounds: Rect, sector: &mut Sector) -> bool {
    let scale = ui.m.scale;
    let bar = Rect::from_xywh(0.0, 0.0, bounds.width(), 95.0 * scale);
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
    let map = Rect::new(
        Vec2::new(0.0, bar.max.y),
        bounds.max - Vec2::new(0.0, 110.0 * scale),
    );
    let details = Rect::from_xywh(
        bounds.max.x - 380.0 * scale,
        bar.max.y + 25.0 * scale,
        355.0 * scale,
        420.0 * scale,
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
            "Movement: {:.1} / {:.0} units",
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
