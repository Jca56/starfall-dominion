use super::*;
use crate::camera::{Camera, ZoomRange};
use crate::world::{SCOUT_SPEED, WORLD_SIZE};
use lntrn_ui::{MouseButton, WheelDelta};

fn frame(ui: &mut Interface, scale: f64) {
    ui.rebuild(
        PhysicalSize::new((1280.0 * scale) as u32, (800.0 * scale) as u32),
        scale,
    );
}

fn setup(scale: f64) -> Interface {
    let mut ui = Interface::default();
    assert!(ui.text.face_count() > 0, "The UI needs an installed font");
    ui.state.record_rects = true;
    frame(&mut ui, scale);
    ui
}

fn widget(ui: &Interface, region: &str, name: &str) -> Rect {
    ui.state.rects[&WidgetId::ROOT.with(region).with(name)]
}

fn map_region(scale: f64) -> Rect {
    sector::map_region(
        Rect::from_xywh(0.0, 0.0, 1280.0 * scale, 800.0 * scale),
        scale,
    )
}

fn click(ui: &mut Interface, point: Vec2, scale: f64) {
    ui.events.push(Event::PointerMoved(point));
    ui.events.push(Event::Button {
        button: MouseButton::Left,
        pressed: true,
        pos: point,
        mods: Modifiers::NONE,
    });
    frame(ui, scale);
    ui.events.push(Event::Button {
        button: MouseButton::Left,
        pressed: false,
        pos: point,
        mods: Modifiers::NONE,
    });
    frame(ui, scale);
}

fn right_click(ui: &mut Interface, point: Vec2) {
    ui.events.push(Event::Button {
        button: MouseButton::Right,
        pressed: true,
        pos: point,
        mods: Modifiers::NONE,
    });
}

/// Run the clock forward by hand until every animation has stopped asking for frames.
fn let_motion_finish(ui: &mut Interface, scale: f64) {
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
        click(&mut ui, start.center(), scale);
        assert_eq!(ui.screen, Screen::Sector);
        let zoom = ui.sector.view.zoom;
        let arcadia = crate::planets::PLANETS
            .iter()
            .find(|planet| planet.name == "Arcadia")
            .expect("the home planet");
        ui.sector.view.snap(arcadia.position, zoom);
        frame(&mut ui, scale);
        let planet = widget(&ui, "sector-map", "Arcadia");
        click(&mut ui, planet.center(), scale);
        assert_eq!(ui.sector.selected, Some(2));
        let close = widget(&ui, "planet-details", "Close");
        click(&mut ui, close.center(), scale);
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
    ui.events.push(Event::Button {
        button: MouseButton::Left,
        pressed: true,
        pos: start.center(),
        mods: Modifiers::NONE,
    });
    ui.rebuild(PhysicalSize::new(900, 640), 1.0);
    ui.events.push(Event::PointerMoved(Vec2::new(850.0, 600.0)));
    ui.events.push(Event::Button {
        button: MouseButton::Left,
        pressed: false,
        pos: Vec2::new(850.0, 600.0),
        mods: Modifiers::NONE,
    });
    ui.rebuild(PhysicalSize::new(900, 640), 1.0);
    assert_eq!(ui.screen, Screen::MainMenu);
}

#[test]
fn map_zoom_ignores_top_bar_eases_and_stationary_drag_settles() {
    let mut ui = setup(1.0);
    let start = widget(&ui, "main-menu", "Start Game");
    click(&mut ui, start.center(), 1.0);
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
    assert_eq!(ui.sector.view.zoom, ui.sector.view.target_zoom);
    ui.events.push(Event::Button {
        button: MouseButton::Left,
        pressed: true,
        pos: point,
        mods: Modifiers::NONE,
    });
    frame(&mut ui, 1.0);
    ui.events
        .push(Event::PointerMoved(point + Vec2::new(30.0, 20.0)));
    frame(&mut ui, 1.0);
    let pan = ui.sector.view.center;
    frame(&mut ui, 1.0);
    assert_eq!(ui.sector.view.center, pan);
    assert!(
        !ui.needs_rebuild() && ui.wake_after().is_none(),
        "Holding the mouse still must not cause a redraw loop"
    );
}

#[test]
fn fleet_orders_glide_and_end_turn_work_through_the_ui_at_both_scales() {
    for scale in [1.0, 1.4] {
        let mut ui = setup(scale);
        let start = widget(&ui, "main-menu", "Start Game");
        click(&mut ui, start.center(), scale);
        let ship = widget(&ui, "sector-map", "Farlight Scout");
        click(&mut ui, ship.center(), scale);
        assert!(ui.sector.fleet_selected);
        let view = &ui.sector.view;
        let camera = Camera::new(map_region(scale), scale, view.center, view.zoom);
        let origin = ui.sector.game.fleets[0].position;
        let destination = origin + Vec2::new(30.0, 0.0);
        let point = camera.to_screen(destination);
        ui.events.push(Event::PointerMoved(point));
        frame(&mut ui, scale);
        assert_eq!(
            ui.sector.game.fleets[0].position, origin,
            "Preview must not move the ship"
        );
        right_click(&mut ui, point);
        // A later cursor event in the same frame must not change the clicked order.
        ui.events
            .push(Event::PointerMoved(point + Vec2::new(80.0, 30.0)));
        frame(&mut ui, scale);
        assert!((ui.sector.game.fleets[0].position - destination).length() < 1e-8);
        assert!((ui.sector.game.fleets[0].remaining - (SCOUT_SPEED - 30.0)).abs() < 1e-8);
        assert!(
            ui.sector.travel.is_some(),
            "A committed order glides on screen"
        );
        assert!(ui.wake_after().is_some(), "The glide keeps frames coming");
        let far = camera.to_screen(destination + Vec2::new(SCOUT_SPEED, 0.0));
        right_click(&mut ui, far);
        frame(&mut ui, scale);
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
        assert!(ui.sector.travel.is_none());
        assert!(ui.wake_after().is_none(), "Nothing moving means no frames");
    }
}

#[test]
fn right_clicking_ui_or_deselected_map_does_not_move_a_fleet() {
    let mut ui = setup(1.0);
    let start = widget(&ui, "main-menu", "Start Game");
    click(&mut ui, start.center(), 1.0);
    let before = ui.sector.game.clone();
    right_click(&mut ui, Vec2::new(600.0, 400.0));
    frame(&mut ui, 1.0);
    assert_eq!(ui.sector.game, before);
    let ship = widget(&ui, "sector-map", "Farlight Scout");
    click(&mut ui, ship.center(), 1.0);
    let end = widget(&ui, "turn-controls", "End Turn");
    for point in [end.center(), Vec2::new(200.0, 45.0)] {
        right_click(&mut ui, point);
        frame(&mut ui, 1.0);
        assert_eq!(ui.sector.game, before);
    }
    ui.events.push(Event::Key {
        key: Key::Escape,
        pressed: true,
        repeat: false,
        mods: Modifiers::NONE,
    });
    frame(&mut ui, 1.0);
    assert!(!ui.sector.fleet_selected);
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
