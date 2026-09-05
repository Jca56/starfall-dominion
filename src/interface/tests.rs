use super::*;
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

#[test]
fn start_planet_details_and_return_work_at_both_display_scales() {
    for scale in [1.0, 1.4] {
        let mut ui = setup(scale);
        let start = widget(&ui, "main-menu", "Start Game");
        assert!(start.height() >= 65.0 * scale);
        assert!(start.max.y < 800.0 * scale);
        click(&mut ui, start.center(), scale);
        assert_eq!(ui.screen, Screen::Sector);
        let planet = widget(&ui, "sector-map", "Farlight");
        click(&mut ui, planet.center(), scale);
        assert_eq!(ui.sector.selected, Some(0));
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
fn map_zoom_ignores_top_bar_and_stationary_drag_settles() {
    let mut ui = setup(1.0);
    let start = widget(&ui, "main-menu", "Start Game");
    click(&mut ui, start.center(), 1.0);
    ui.events.push(Event::PointerMoved(Vec2::new(200.0, 45.0)));
    ui.events.push(Event::Wheel {
        delta: WheelDelta::Lines(Vec2::new(0.0, 1.0)),
        pos: Vec2::new(200.0, 45.0),
        mods: Modifiers::NONE,
    });
    frame(&mut ui, 1.0);
    assert_eq!(ui.sector.zoom, 1.0);
    let point = Vec2::new(100.0, 140.0);
    ui.events.push(Event::PointerMoved(point));
    ui.events.push(Event::Wheel {
        delta: WheelDelta::Lines(Vec2::new(0.0, 1.0)),
        pos: point,
        mods: Modifiers::NONE,
    });
    frame(&mut ui, 1.0);
    assert!(ui.sector.zoom > 1.0);
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
    let pan = ui.sector.pan;
    frame(&mut ui, 1.0);
    assert_eq!(ui.sector.pan, pan);
    assert!(
        !ui.needs_rebuild(),
        "Holding the mouse still must not cause a redraw loop"
    );
}

#[test]
fn fleet_orders_and_end_turn_work_through_the_ui_at_both_scales() {
    use crate::camera::Camera;
    for scale in [1.0, 1.4] {
        let mut ui = setup(scale);
        let start = widget(&ui, "main-menu", "Start Game");
        click(&mut ui, start.center(), scale);
        let ship = widget(&ui, "sector-map", "Farlight Scout");
        click(&mut ui, ship.center(), scale);
        assert!(ui.sector.fleet_selected);
        let map = Rect::from_xywh(0.0, 95.0 * scale, 1280.0 * scale, 595.0 * scale);
        let camera = Camera::new(map, scale, ui.sector.pan, ui.sector.zoom);
        let origin = ui.sector.game.fleets[0].position;
        let destination = origin + Vec2::new(30.0, 0.0);
        let point = camera.to_screen(destination);
        ui.events.push(Event::PointerMoved(point));
        frame(&mut ui, scale);
        assert_eq!(
            ui.sector.game.fleets[0].position, origin,
            "Preview must not move the ship"
        );
        ui.events.push(Event::Button {
            button: MouseButton::Right,
            pressed: true,
            pos: point,
            mods: Modifiers::NONE,
        });
        // A later cursor event in the same frame must not change the clicked order.
        ui.events
            .push(Event::PointerMoved(point + Vec2::new(80.0, 30.0)));
        frame(&mut ui, scale);
        assert!((ui.sector.game.fleets[0].position - destination).length() < 1e-8);
        assert!((ui.sector.game.fleets[0].remaining - 70.0).abs() < 1e-8);
        let far = camera.to_screen(destination + Vec2::new(300.0, 0.0));
        ui.events.push(Event::Button {
            button: MouseButton::Right,
            pressed: true,
            pos: far,
            mods: Modifiers::NONE,
        });
        frame(&mut ui, scale);
        assert_eq!(ui.sector.game.fleets[0].remaining, 0.0);
        assert!(
            (ui.sector.game.fleets[0].position - (origin + Vec2::new(100.0, 0.0))).length() < 1e-8
        );
        let end = widget(&ui, "turn-controls", "End Turn");
        assert!(end.max.x <= 1280.0 * scale && end.max.y <= 800.0 * scale);
        click(&mut ui, end.center(), scale);
        assert_eq!(ui.sector.game.turn, 2);
        assert_eq!(ui.sector.game.fleets[0].remaining, 100.0);
    }
}

#[test]
fn right_clicking_ui_or_deselected_map_does_not_move_a_fleet() {
    let mut ui = setup(1.0);
    let start = widget(&ui, "main-menu", "Start Game");
    click(&mut ui, start.center(), 1.0);
    let before = ui.sector.game.clone();
    ui.events.push(Event::Button {
        button: MouseButton::Right,
        pressed: true,
        pos: Vec2::new(600.0, 400.0),
        mods: Modifiers::NONE,
    });
    frame(&mut ui, 1.0);
    assert_eq!(ui.sector.game, before);
    let ship = widget(&ui, "sector-map", "Farlight Scout");
    click(&mut ui, ship.center(), 1.0);
    let end = widget(&ui, "turn-controls", "End Turn");
    for point in [end.center(), Vec2::new(200.0, 45.0)] {
        ui.events.push(Event::Button {
            button: MouseButton::Right,
            pressed: true,
            pos: point,
            mods: Modifiers::NONE,
        });
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
