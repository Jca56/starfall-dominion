use lntrn_math::{Rect, Vec2};
use lntrn_render::DrawList;
use lntrn_text::TextEngine;
use lntrn_ui::{Event, Key, Modifiers, MouseButton, Theme, Ui, UiState, WidgetId};
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;

use crate::camera::Backdrop;
use crate::sector::{self, Sector};
use crate::theme;
use crate::world::Game;

/// A right press that moves less than this many logical pixels is a click,
/// which deselects, rather than a drag, which pans.
const DRAG_THRESHOLD: f64 = 6.0;

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
    /// Pinned by tests; otherwise every Start Game rolls a new map.
    fixed_seed: Option<u64>,
}

impl Default for Interface {
    fn default() -> Self {
        Self {
            text: TextEngine::new("Inter", "JetBrains Mono"),
            draw: DrawList::new(),
            state: UiState::new(),
            theme: theme::theme(),
            events: Vec::new(),
            modifiers: Modifiers::NONE,
            pointer: Vec2::new(-1.0, -1.0),
            screen: Screen::MainMenu,
            sector: Sector::default(),
            fixed_seed: None,
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
            self.track_right_button(&events, scale);
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
                && self.sector.selected_fleet.take().is_none()
                && self.sector.selected.take().is_none()
            {
                self.screen = Screen::MainMenu;
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
                        let seed = self.fixed_seed.unwrap_or_else(clock_seed);
                        self.sector = Sector::new(Game::new(seed));
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

    /// Lantern only tracks the left button through a drag, so the right button
    /// is followed here: a drag pans the map, a plain click deselects.
    fn track_right_button(&mut self, events: &[Event], scale: f64) {
        for event in events {
            match event {
                Event::Button {
                    button: MouseButton::Right,
                    pressed: true,
                    pos,
                    ..
                } => {
                    self.sector.right_press = Some(*pos);
                    self.sector.right_dragged = false;
                }
                Event::Button {
                    button: MouseButton::Right,
                    pressed: false,
                    ..
                } => {
                    if self.sector.right_press.take().is_some() && !self.sector.right_dragged {
                        self.sector.deselect();
                        self.state.request_rebuild = true;
                    }
                }
                Event::PointerMoved(pos) => {
                    if let Some(press) = self.sector.right_press
                        && (*pos - press).length() > DRAG_THRESHOLD * scale
                    {
                        self.sector.right_dragged = true;
                    }
                }
                _ => {}
            }
        }
    }

    /// The starfield uniforms for this frame: three `vec4<f32>` values.
    pub(crate) fn backdrop_uniform(&self, size: PhysicalSize<u32>, scale: f64) -> [f32; 12] {
        let viewport = Vec2::new(size.width as f64, size.height as f64);
        let backdrop = if self.screen == Screen::MainMenu {
            Backdrop::menu()
        } else {
            let bounds = Rect::from_min_size(Vec2::ZERO, viewport);
            Backdrop::sector(&self.sector.view, sector::map_region(bounds, scale), scale)
        };
        backdrop.uniform(viewport, scale)
    }

    /// Fog cells for the GPU whenever the chart changed since version `seen`.
    pub(crate) fn fog_texture(&self, seen: u64) -> Option<(u64, Vec<u8>)> {
        if self.screen != Screen::Sector {
            return None;
        }
        let sector = &self.sector;
        let version = sector.chart.version();
        (version != seen).then(|| {
            let sources = sector.shown_vision_sources(self.state.now);
            (version, sector.chart.texture(&sources))
        })
    }

    pub(crate) fn needs_rebuild(&self) -> bool {
        self.state.request_rebuild
    }

    /// Seconds until something on screen wants another frame; `None` when idle.
    pub(crate) fn wake_after(&self) -> Option<f64> {
        self.state.wake_after
    }
}

/// A fresh seed for every game, from the wall clock.
fn clock_seed() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_nanos() as u64)
}

fn main_menu(ui: &mut Ui, bounds: Rect) -> bool {
    let scale = ui.m.scale;
    let panel = Rect::from_xywh(
        120.0 * scale,
        (bounds.height() - 640.0 * scale) * 0.5,
        620.0 * scale,
        640.0 * scale,
    );
    panel_background(ui, panel);
    ui.draw.rect(
        Rect::from_xywh(
            panel.min.x,
            panel.min.y + 40.0 * scale,
            4.0 * scale,
            120.0 * scale,
        ),
        theme::GOLD,
    );
    let mut start = false;
    region(ui, panel.shrink(45.0 * scale), "main-menu", |ui| {
        ui.heading("STARFALL");
        ui.heading("DOMINION");
        ui.space(20.0 * scale);
        ui.label("The Farlight Expanse");
        ui.space(10.0 * scale);
        ui.paragraph("The last bastion of freedom in a sector claimed by war.");
        ui.space(40.0 * scale);
        start = ui.button_wide("Start Game").clicked;
    });
    start
}

pub(crate) fn panel_background(ui: &mut Ui, rect: Rect) {
    ui.floating_panel(rect, ui.theme.panel);
    ui.outline(rect, ui.m.px(1.0), theme::EDGE);
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
mod loop_tests;
#[cfg(test)]
mod tests;
