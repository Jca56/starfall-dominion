use lntrn_math::{Color, Rect, Vec2};
use lntrn_render::DrawList;
use lntrn_text::TextEngine;
use lntrn_ui::{CursorIcon, Event, Key, Modifiers, Theme, Ui, UiState, WidgetId};
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;

use crate::sector::{self, Sector};

#[derive(Default, Debug, PartialEq, Eq)]
enum Screen {
    #[default]
    MainMenu,
    Sector,
}

pub(crate) struct Interface {
    pub(crate) text: TextEngine,
    pub(crate) draw: DrawList,
    state: UiState,
    theme: Theme,
    events: Vec<Event>,
    modifiers: Modifiers,
    pointer: Vec2,
    screen: Screen,
    sector: Sector,
}

impl Default for Interface {
    fn default() -> Self {
        let mut state = UiState::new();
        state.reduce_motion = true;
        Self {
            text: TextEngine::new("Inter", "JetBrains Mono"),
            draw: DrawList::new(),
            state,
            theme: Theme {
                text_size: 30.0,
                heading_size: 45.0,
                widget_height: 65.0,
                padding: 15.0,
                gap: 10.0,
                text: Color::hex(0xEFF5FF),
                text_dim: Color::hex(0xB5C6DD),
                ..Theme::nightfall()
            },
            events: Vec::new(),
            modifiers: Modifiers::NONE,
            pointer: Vec2::new(-1.0, -1.0),
            screen: Screen::MainMenu,
            sector: Sector::default(),
        }
    }
}

impl Interface {
    pub(crate) fn window_event(&mut self, event: &WindowEvent) -> bool {
        if let Some(event) =
            lntrn_app::translate::window_event(event, &mut self.modifiers, &mut self.pointer)
        {
            self.events.push(event);
            true
        } else {
            false
        }
    }

    pub(crate) fn rebuild(&mut self, size: PhysicalSize<u32>, scale: f64) {
        let bounds = Rect::from_xywh(0.0, 0.0, size.width as f64, size.height as f64);
        let metrics = self.theme.metrics(scale);
        let events = std::mem::take(&mut self.events);
        if self.screen == Screen::Sector {
            for event in &events {
                if let Event::Button {
                    button: lntrn_ui::MouseButton::Right,
                    pressed: true,
                    pos,
                    ..
                } = event
                {
                    self.sector.move_requests.push(*pos);
                }
            }
        }
        for iteration in 0..4 {
            self.state
                .begin_frame(if iteration == 0 { &events } else { &[] }, metrics.widget_h);
            self.draw.clear();
            if self
                .state
                .take_key(|key| key.key == Key::Escape && !key.repeat)
                .is_some()
                && self.screen == Screen::Sector
            {
                if self.sector.fleet_selected {
                    self.sector.fleet_selected = false;
                } else if self.sector.selected.take().is_none() {
                    self.screen = Screen::MainMenu;
                }
                self.sector.message = None;
                self.state.focus = None;
            }
            let mut ui = Ui::new(
                &mut self.draw,
                &mut self.text,
                &self.theme,
                metrics,
                &mut self.state,
                bounds,
                bounds,
                WidgetId::ROOT,
                0,
            );
            ui.set_window_rect(bounds);
            let changed = match self.screen {
                Screen::MainMenu => {
                    if main_menu(&mut ui, bounds) {
                        self.sector = Sector::default();
                        self.screen = Screen::Sector;
                        true
                    } else {
                        false
                    }
                }
                Screen::Sector => {
                    if sector::draw(&mut ui, bounds, &mut self.sector) {
                        self.screen = Screen::MainMenu;
                        true
                    } else {
                        false
                    }
                }
            };
            ui.finish();
            self.state.end_frame();
            if changed {
                self.state.focus = None;
                self.state.request_rebuild = true;
            }
            if !self.state.request_rebuild {
                break;
            }
        }
    }

    pub(crate) fn needs_rebuild(&self) -> bool {
        self.state.request_rebuild
    }

    pub(crate) fn cursor(&self) -> winit::window::CursorIcon {
        match self.state.cursor_icon {
            CursorIcon::Pointer => winit::window::CursorIcon::Pointer,
            CursorIcon::Grabbing => winit::window::CursorIcon::Grabbing,
            _ => winit::window::CursorIcon::Default,
        }
    }
}

fn main_menu(ui: &mut Ui, bounds: Rect) -> bool {
    let scale = ui.m.scale;
    let panel = Rect::from_xywh(
        40.0 * scale,
        (bounds.height() - 530.0 * scale) * 0.5,
        460.0 * scale,
        530.0 * scale,
    );
    panel_background(ui, panel);
    ui.draw.rect(
        Rect::from_xywh(
            panel.min.x,
            panel.min.y + 30.0 * scale,
            3.0 * scale,
            95.0 * scale,
        ),
        ui.theme.accent,
    );
    let mut start = false;
    region(ui, panel.shrink(35.0 * scale), "main-menu", |ui| {
        ui.heading("STARFALL");
        ui.heading("DOMINION");
        ui.space(15.0 * scale);
        ui.label("The Farlight Expanse");
        ui.space(10.0 * scale);
        ui.paragraph("The last bastion of freedom in a sector claimed by war.");
        ui.space(30.0 * scale);
        start = ui.button_wide("Start Game").clicked;
    });
    start
}

pub(crate) fn panel_background(ui: &mut Ui, rect: Rect) {
    ui.floating_panel(rect, ui.theme.panel);
    ui.outline(rect, ui.m.px(1.0), Color::hex(0x43546E));
}

pub(crate) fn region(ui: &mut Ui, rect: Rect, name: &str, draw: impl FnOnce(&mut Ui)) {
    let clip = rect.intersection(&ui.clip());
    let window = ui.clip();
    let id = ui.id(name);
    let layer = ui.layer();
    let mut child = Ui::new(
        ui.draw, ui.text, ui.theme, ui.m, ui.state, rect, clip, id, layer,
    );
    child.set_window_rect(window);
    draw(&mut child);
    child.finish();
}

#[cfg(test)]
mod tests;
