use super::*;
use crate::camera::{Camera, ZoomRange};
use crate::sector::Selection;
use crate::world::{SCOUT_SPEED, SCOUT_VISION, WORLD_SIZE};
use lntrn_ui::{MouseButton, WheelDelta};

pub(super) fn frame(ui: &mut Interface, scale: f64) {
    ui.rebuild(
        PhysicalSize::new((1280.0 * scale) as u32, (800.0 * scale) as u32),
        scale,
    );
}

pub(super) fn setup(scale: f64) -> Interface {
    let mut ui = Interface::default();
    assert!(ui.text.face_count() > 0, "The UI needs an installed font");
    ui.state.record_rects = true;
    ui.fixed_seed = Some(7);
    frame(&mut ui, scale);
    ui
}

pub(super) fn widget(ui: &Interface, region: &str, name: &str) -> Rect {
    ui.state.rects[&WidgetId::ROOT.with(region).with(name)]
}

pub(super) fn tab(ui: &Interface, region: &str, label: &str, index: usize) -> Rect {
    ui.state.rects[&WidgetId::ROOT.with(region).with(label).with_index(index)]
}

pub(super) fn planet(name: &str) -> &'static crate::planets::Planet {
    crate::planets::PLANETS
        .iter()
        .find(|planet| planet.name == name)
        .expect("a named planet")
}

pub(super) fn map_region(scale: f64) -> Rect {
    sector::map_region(
        Rect::from_xywh(0.0, 0.0, 1280.0 * scale, 800.0 * scale),
        scale,
    )
}

pub(super) fn button(button: MouseButton, pressed: bool, pos: Vec2) -> Event {
    Event::Button {
        button,
        pressed,
        pos,
        mods: Modifiers::NONE,
    }
}

/// A left click: press and release in place, a frame each.
pub(super) fn click(ui: &mut Interface, point: Vec2, scale: f64) {
    ui.events.push(Event::PointerMoved(point));
    ui.events.push(button(MouseButton::Left, true, point));
    frame(ui, scale);
    ui.events.push(button(MouseButton::Left, false, point));
    frame(ui, scale);
}

/// A right click without movement: deselects.
pub(super) fn right_click(ui: &mut Interface, point: Vec2, scale: f64) {
    ui.events.push(Event::PointerMoved(point));
    ui.events.push(button(MouseButton::Right, true, point));
    frame(ui, scale);
    ui.events.push(button(MouseButton::Right, false, point));
    frame(ui, scale);
}

/// A right drag from `from` to `to`: pans.
pub(super) fn right_drag(ui: &mut Interface, from: Vec2, to: Vec2, scale: f64) {
    ui.events.push(Event::PointerMoved(from));
    ui.events.push(button(MouseButton::Right, true, from));
    frame(ui, scale);
    ui.events.push(Event::PointerMoved(to));
    frame(ui, scale);
    ui.events.push(button(MouseButton::Right, false, to));
    frame(ui, scale);
}

pub(super) fn start_game(ui: &mut Interface, scale: f64) {
    let start = widget(ui, "main-menu", "Start Game");
    click(ui, start.center(), scale);
    assert_eq!(ui.screen, Screen::Sector);
}

pub(super) fn camera(ui: &Interface, scale: f64) -> Camera {
    let view = &ui.sector.view;
    Camera::new(map_region(scale), scale, view.center, view.zoom)
}

/// Run the clock forward by hand until every animation has stopped asking for frames.
pub(super) fn let_motion_finish(ui: &mut Interface, scale: f64) {
    for tick in 0..40 {
        ui.state.set_time(100.0 + tick as f64 * 0.1);
        frame(ui, scale);
    }
}

#[test]
fn start_planet_details_and_return_work_at_both_display_scales() {
    for scale in [1.0, 1.4] {
        let mut ui = setup(scale);
        let start = widget(&ui, "main-menu", "Start Game");
        assert!(start.height() >= 65.0 * scale);
        assert!(start.max.y < 800.0 * scale);
        start_game(&mut ui, scale);
        let zoom = ui.sector.view.zoom;
        ui.sector.view.snap(planet("Arcadia").position, zoom);
        frame(&mut ui, scale);
        let arcadia = widget(&ui, "sector-map", "Arcadia");
        click(&mut ui, arcadia.center(), scale);
        assert_eq!(ui.sector.selected, Some(Selection::Planet(2)));
        // Empty space, away from the panel down the left: closes it.
        click(&mut ui, Vec2::new(1000.0, 300.0) * scale, scale);
        assert_eq!(ui.sector.selected, None);
        let menu = widget(&ui, "top-bar", "Menu");
        click(&mut ui, menu.center(), scale);
        assert_eq!(ui.screen, Screen::MainMenu);
    }
}

#[test]
fn keyboard_can_start_and_escape_returns_to_menu() {
    let mut ui = setup(1.4);
    ui.events.push(Event::Key {
        key: Key::Tab,
        pressed: true,
        repeat: false,
        mods: Modifiers::NONE,
    });
    frame(&mut ui, 1.4);
    ui.events.push(Event::Key {
        key: Key::Enter,
        pressed: true,
        repeat: false,
        mods: Modifiers::NONE,
    });
    frame(&mut ui, 1.4);
    assert_eq!(ui.screen, Screen::Sector);
    ui.events.push(Event::Key {
        key: Key::Escape,
        pressed: true,
        repeat: false,
        mods: Modifiers::NONE,
    });
    frame(&mut ui, 1.4);
    assert_eq!(ui.screen, Screen::MainMenu);
}

#[test]
fn releasing_outside_start_does_not_launch_and_small_window_fits() {
    let mut ui = setup(1.0);
    ui.rebuild(PhysicalSize::new(900, 640), 1.0);
    let start = widget(&ui, "main-menu", "Start Game");
    assert!(start.max.y < 640.0 && start.max.x < 900.0);
    ui.events.push(Event::PointerMoved(start.center()));
    ui.events
        .push(button(MouseButton::Left, true, start.center()));
    ui.rebuild(PhysicalSize::new(900, 640), 1.0);
    ui.events.push(Event::PointerMoved(Vec2::new(850.0, 600.0)));
    ui.events
        .push(button(MouseButton::Left, false, Vec2::new(850.0, 600.0)));
    ui.rebuild(PhysicalSize::new(900, 640), 1.0);
    assert_eq!(ui.screen, Screen::MainMenu);
}

#[test]
fn map_zoom_ignores_top_bar_eases_and_right_drag_pans_then_settles() {
    let mut ui = setup(1.0);
    start_game(&mut ui, 1.0);
    let start_zoom = ZoomRange::for_region(map_region(1.0), 1.0).start;
    assert_eq!(ui.sector.view.zoom, start_zoom);
    assert!(
        ui.wake_after().is_none(),
        "A fresh map has nothing to animate"
    );
    ui.events.push(Event::PointerMoved(Vec2::new(200.0, 45.0)));
    ui.events.push(Event::Wheel {
        delta: WheelDelta::Lines(Vec2::new(0.0, 1.0)),
        pos: Vec2::new(200.0, 45.0),
        mods: Modifiers::NONE,
    });
    frame(&mut ui, 1.0);
    assert_eq!(ui.sector.view.target_zoom, start_zoom);
    let point = Vec2::new(100.0, 140.0);
    ui.events.push(Event::PointerMoved(point));
    ui.events.push(Event::Wheel {
        delta: WheelDelta::Lines(Vec2::new(0.0, 1.0)),
        pos: point,
        mods: Modifiers::NONE,
    });
    frame(&mut ui, 1.0);
    assert!(ui.sector.view.target_zoom > start_zoom);
    assert!(ui.sector.view.moving(), "Zoom eases toward its target");
    assert!(ui.wake_after().is_some(), "Easing asks for another frame");
    let_motion_finish(&mut ui, 1.0);
    assert!(!ui.sector.view.moving());
    let before = ui.sector.view.center;
    ui.events.push(button(MouseButton::Right, true, point));
    frame(&mut ui, 1.0);
    ui.events
        .push(Event::PointerMoved(point + Vec2::new(30.0, 20.0)));
    frame(&mut ui, 1.0);
    let pan = ui.sector.view.center;
    assert_ne!(pan, before, "A right drag pans the map");
    frame(&mut ui, 1.0);
    assert_eq!(ui.sector.view.center, pan);
    assert!(
        !ui.needs_rebuild() && ui.wake_after().is_none(),
        "Holding the mouse still must not cause a redraw loop"
    );
    ui.events.push(button(
        MouseButton::Right,
        false,
        point + Vec2::new(30.0, 20.0),
    ));
    frame(&mut ui, 1.0);
    assert_eq!(ui.sector.view.center, pan, "Releasing a drag does not jump");
}

#[test]
fn fleet_orders_glide_and_end_turn_work_through_the_ui_at_both_scales() {
    for scale in [1.0, 1.4] {
        let mut ui = setup(scale);
        start_game(&mut ui, scale);
        let ship = widget(&ui, "sector-map", "Farlight Scout");
        click(&mut ui, ship.center(), scale);
        assert!(ui.sector.selected_fleet.is_some());
        let camera = camera(&ui, scale);
        let origin = ui.sector.game.fleets[0].position;
        let destination = origin + Vec2::new(30.0, 0.0);
        let point = camera.to_screen(destination);
        ui.events.push(Event::PointerMoved(point));
        frame(&mut ui, scale);
        assert_eq!(
            ui.sector.game.fleets[0].position, origin,
            "Preview must not move the ship"
        );
        click(&mut ui, point, scale);
        assert!((ui.sector.game.fleets[0].position - destination).length() < 1e-8);
        assert!((ui.sector.game.fleets[0].remaining - (SCOUT_SPEED - 30.0)).abs() < 1e-8);
        assert!(ui.sector.any_travel(), "A committed order glides on screen");
        assert!(ui.wake_after().is_some(), "The glide keeps frames coming");
        let far = camera.to_screen(destination + Vec2::new(SCOUT_SPEED, 0.0));
        click(&mut ui, far, scale);
        assert_eq!(ui.sector.game.fleets[0].remaining, 0.0);
        assert!(
            (ui.sector.game.fleets[0].position - (origin + Vec2::new(SCOUT_SPEED, 0.0))).length()
                < 1e-8
        );
        let end = widget(&ui, "turn-controls", "End Turn");
        assert!(end.max.x <= 1280.0 * scale && end.max.y <= 800.0 * scale);
        click(&mut ui, end.center(), scale);
        assert_eq!(ui.sector.game.turn, 2);
        assert_eq!(ui.sector.game.fleets[0].remaining, SCOUT_SPEED);
        // Once the glide lands the loop goes back to sleep.
        ui.events.push(Event::PointerMoved(point));
        let_motion_finish(&mut ui, scale);
        assert!(!ui.sector.any_travel());
        assert!(ui.wake_after().is_none(), "Nothing moving means no frames");
    }
}

#[test]
fn clicks_on_ui_are_not_orders_and_a_right_click_deselects() {
    let mut ui = setup(1.0);
    start_game(&mut ui, 1.0);
    let before = ui.sector.game.clone();
    click(&mut ui, Vec2::new(900.0, 400.0), 1.0);
    assert_eq!(ui.sector.game, before, "Nothing selected, nothing moves");
    let ship = widget(&ui, "sector-map", "Farlight Scout");
    click(&mut ui, ship.center(), 1.0);
    assert!(ui.sector.selected_fleet.is_some());
    let end = widget(&ui, "turn-controls", "End Turn");
    click(&mut ui, end.center(), 1.0);
    assert_eq!(ui.sector.game.turn, 2, "The button wins over the map");
    assert_eq!(ui.sector.game.fleets[0].position, before.fleets[0].position);
    click(&mut ui, Vec2::new(200.0, 45.0), 1.0);
    assert_eq!(
        ui.sector.game.fleets[0].position, before.fleets[0].position,
        "The Command Strip is not the map"
    );
    assert!(ui.sector.selected_fleet.is_some());
    right_drag(
        &mut ui,
        Vec2::new(700.0, 300.0),
        Vec2::new(760.0, 340.0),
        1.0,
    );
    assert!(
        ui.sector.selected_fleet.is_some(),
        "A drag keeps the selection"
    );
    right_click(&mut ui, Vec2::new(700.0, 300.0), 1.0);
    assert!(
        ui.sector.selected_fleet.is_none(),
        "A plain right click drops it"
    );
    assert_eq!(ui.screen, Screen::Sector);
}

#[test]
fn resizing_preserves_world_scale_and_backdrop_tracks_pan_and_zoom() {
    let mut ui = setup(1.0);
    ui.screen = Screen::Sector;
    for scale in [1.0, 1.4] {
        for width in [900, 1280, 2400] {
            let size = PhysicalSize::new(width, 1000);
            let bounds = Rect::from_xywh(0.0, 0.0, width as f64, 1000.0);
            let map = sector::map_region(bounds, scale);
            let zoom = ZoomRange::for_region(map, scale).start * 1.5;
            ui.sector.view.snap(Vec2::new(1000.0, 1000.0), zoom);
            ui.rebuild(size, scale);
            assert_eq!(
                ui.sector.view.zoom, zoom,
                "Resizing must not rescale distances"
            );
            let camera = Camera::new(map, scale, ui.sector.view.center, ui.sector.view.zoom);
            assert!((camera.pixels_per_unit / scale - zoom).abs() < 1e-8);
            let uniform = ui.backdrop_uniform(size, scale);
            assert_eq!(uniform[3], 1.0);
            assert!((uniform[6] as f64 - camera.pixels_per_unit).abs() < 1e-3);
            let min = camera.to_screen(Vec2::ZERO);
            let max = camera.to_screen(WORLD_SIZE);
            for (actual, expected) in uniform[8..].iter().zip([min.x, min.y, max.x, max.y]) {
                assert!((*actual as f64 - expected).abs() < 0.01);
            }
            // The map scrolls the backdrop west of center; a centered row leaves it at rest.
            assert!(uniform[4] < 0.0 && uniform[5].abs() < 1e-3);
        }
    }
    ui.screen = Screen::MainMenu;
    assert_eq!(
        ui.backdrop_uniform(PhysicalSize::new(1280, 800), 1.0)[3],
        0.0
    );
}

#[test]
fn fog_uploads_once_per_change_and_lights_only_what_you_can_see() {
    use crate::fog::{CELL, COLUMNS, ROWS};
    let mut ui = setup(1.0);
    assert!(ui.fog_texture(0).is_none(), "The menu has no fog");
    start_game(&mut ui, 1.0);
    let (version, bytes) = ui.fog_texture(0).expect("A fresh map uploads its fog");
    assert_eq!(bytes.len(), COLUMNS * ROWS * 2);
    let cell = |point: Vec2| {
        let index = ((point.y / CELL) as usize * COLUMNS + (point.x / CELL) as usize) * 2;
        (bytes[index], bytes[index + 1])
    };
    assert_eq!(
        cell(planet("Arcadia").position),
        (255, 255),
        "Home is charted and lit"
    );
    assert_eq!(
        cell(planet("Vesper").position),
        (0, 0),
        "The far east is dark"
    );
    assert!(
        ui.fog_texture(version).is_none(),
        "Nothing changed, so nothing re-uploads"
    );
    let ship = widget(&ui, "sector-map", "Farlight Scout");
    click(&mut ui, ship.center(), 1.0);
    let target =
        camera(&ui, 1.0).to_screen(ui.sector.game.fleets[0].position + Vec2::new(100.0, 0.0));
    click(&mut ui, target, 1.0);
    let (moved, _) = ui
        .fog_texture(version)
        .expect("A move changes what is in view");
    assert_ne!(moved, version);
}

#[test]
fn the_chart_reveals_as_the_ship_glides_and_catches_up_when_it_lands() {
    let mut ui = setup(1.0);
    start_game(&mut ui, 1.0);
    let ship = widget(&ui, "sector-map", "Farlight Scout");
    click(&mut ui, ship.center(), 1.0);
    let origin = ui.sector.game.fleets[0].position;
    // In range only from the landing point, so it is charted by the rules at
    // commit but shown only once the ship gets there.
    let ahead = origin + Vec2::new(SCOUT_SPEED + SCOUT_VISION - 20.0, 0.0);
    let target = camera(&ui, 1.0).to_screen(origin + Vec2::new(SCOUT_SPEED, 0.0));
    click(&mut ui, target, 1.0);
    assert!(
        ui.sector.game.fog.explored_at(ahead),
        "The rules chart the whole corridor at once"
    );
    assert!(
        !ui.sector.chart.explored_at(ahead),
        "The screen reveals only as far as the ship has flown"
    );
    let_motion_finish(&mut ui, 1.0);
    assert!(ui.sector.chart.explored_at(ahead));
    assert_eq!(
        ui.sector.chart, ui.sector.game.fog,
        "Landing syncs the chart to the rules"
    );
    assert!(
        ui.wake_after().is_none(),
        "A landed ship stops asking for frames"
    );
}
