//! The gameplay loop through the interface: securing, building, and salvage.
use super::tests::{
    camera, click, frame, let_motion_finish, planet, right_click, setup, start_game, tab, widget,
};
use super::*;
use crate::actions::{self, Action};
use crate::economy::Resources;
use crate::sector::Selection;
use crate::world::{SCOUT_SPEED, Side};

#[test]
fn the_home_world_shipyard_builds_a_scout_when_you_can_afford_it() {
    let mut ui = setup(1.0);
    start_game(&mut ui, 1.0);
    let zoom = ui.sector.view.zoom;
    ui.sector.view.snap(planet("Arcadia").position, zoom);
    frame(&mut ui, 1.0);
    let arcadia = widget(&ui, "sector-map", "Arcadia");
    click(&mut ui, arcadia.center(), 1.0);
    let shipyard = tab(&ui, "planet-details", "Shipyard", 1);
    click(&mut ui, shipyard.center(), 1.0);
    let build = widget(&ui, "planet-details", "Build Scout");
    click(&mut ui, build.center(), 1.0);
    assert_eq!(ui.sector.message.as_deref(), Some("Not enough resources."));
    assert!(ui.sector.game.shipyard.is_none());
    ui.sector.game.stockpile = Resources::new(10, 5);
    frame(&mut ui, 1.0);
    let build = widget(&ui, "planet-details", "Build Scout");
    click(&mut ui, build.center(), 1.0);
    assert!(ui.sector.game.shipyard.is_some());
    assert!(ui.sector.game.stockpile.is_empty());
    right_click(&mut ui, Vec2::new(1000.0, 300.0), 1.0);
    assert_eq!(ui.sector.selected, None, "A right click closes the panel");
    let end = widget(&ui, "turn-controls", "End Turn");
    click(&mut ui, end.center(), 1.0);
    click(&mut ui, end.center(), 1.0);
    assert_eq!(
        ui.sector.game.fleets.len(),
        2,
        "Two turns later the scout launches"
    );
    assert_eq!(ui.sector.game.fleets[1].name, "Scout 2");
    let pay = planet("Arcadia").yield_per_turn;
    assert_eq!(
        ui.sector.game.stockpile,
        pay.plus(pay),
        "Arcadia paid out twice meanwhile"
    );
}

#[test]
fn securing_a_planet_from_the_panel_takes_turns_in_range() {
    let mut ui = setup(1.0);
    start_game(&mut ui, 1.0);
    let ship = widget(&ui, "sector-map", "Farlight Scout");
    click(&mut ui, ship.center(), 1.0);
    let origin = ui.sector.game.fleets[0].position;
    let target = planet("L2-b");
    let toward = target.position - origin;
    // Short of a full move so the target point stays inside the small test window.
    let stop = origin + toward / toward.length() * (SCOUT_SPEED - 20.0);
    let point = camera(&ui, 1.0).to_screen(stop);
    click(&mut ui, point, 1.0);
    let_motion_finish(&mut ui, 1.0);
    assert!(
        ui.sector.game.fog.explored_at(target.position),
        "Now in sight"
    );
    right_click(&mut ui, point, 1.0);
    assert!(ui.sector.selected_fleet.is_none());
    let zoom = ui.sector.view.zoom;
    ui.sector.view.snap(target.position, zoom);
    frame(&mut ui, 1.0);
    let marker = widget(&ui, "sector-map", "L2-b");
    click(&mut ui, marker.center(), 1.0);
    let secure = widget(&ui, "planet-details", "Secure Planet");
    click(&mut ui, secure.center(), 1.0);
    assert!(ui.sector.game.planets[3].securing);
    right_click(&mut ui, Vec2::new(1000.0, 300.0), 1.0);
    let end = widget(&ui, "turn-controls", "End Turn");
    for _ in 0..target.secure_turns {
        click(&mut ui, end.center(), 1.0);
    }
    assert_eq!(ui.sector.game.planets[3].owner, Some(Side::Player));
}

#[test]
fn a_wreck_hides_until_charted_then_salvages_from_its_panel() {
    let mut ui = setup(1.0);
    start_game(&mut ui, 1.0);
    let start = ui.sector.game.fleets[0].position;
    let home = crate::layout::region_index(start).expect("the scout starts on the map");
    let wreck = ui
        .sector
        .game
        .pois
        .iter()
        .find(|poi| poi.sector == home)
        .expect("a starter wreck")
        .clone();
    assert!(
        !ui.sector.chart.explored_at(wreck.position),
        "Just out of sight"
    );
    let marker = format!("poi-{}", wreck.id);
    let marker_id = WidgetId::ROOT.with("sector-map").with(&marker);
    assert!(
        !ui.state.rects.contains_key(&marker_id),
        "Hidden wrecks are not on the map"
    );
    // Fly a full move toward it by the rules; the corridor charts it.
    let toward = wreck.position - start;
    let direction = toward / toward.length();
    let order = Action::MoveFleet {
        fleet_id: 0,
        destination: start + direction * SCOUT_SPEED,
    };
    actions::execute(&mut ui.sector.game, Side::Player, order).unwrap();
    frame(&mut ui, 1.0);
    assert!(ui.sector.chart.explored_at(wreck.position), "Now charted");
    let zoom = ui.sector.view.zoom;
    ui.sector.view.snap(wreck.position, zoom);
    frame(&mut ui, 1.0);
    let rect = widget(&ui, "sector-map", &marker);
    click(&mut ui, rect.center(), 1.0);
    assert_eq!(ui.sector.selected, Some(Selection::Poi(wreck.id)));
    let salvage = widget(&ui, "planet-details", "Salvage");
    click(&mut ui, salvage.center(), 1.0);
    assert_eq!(
        ui.sector.message.as_deref(),
        Some("No ship within 100 units to salvage it."),
        "Still more than a hundred units short"
    );
    // Next turn, close the gap by the rules and haul it home.
    let end = widget(&ui, "turn-controls", "End Turn");
    click(&mut ui, end.center(), 1.0);
    let ship = ui.sector.game.fleets[0].position;
    let gap = wreck.position - ship;
    let order = Action::MoveFleet {
        fleet_id: 0,
        destination: ship + gap / gap.length() * (gap.length() - 50.0),
    };
    actions::execute(&mut ui.sector.game, Side::Player, order).unwrap();
    frame(&mut ui, 1.0);
    let before = ui.sector.game.stockpile;
    let salvage = widget(&ui, "planet-details", "Salvage");
    click(&mut ui, salvage.center(), 1.0);
    assert!(ui.sector.game.poi(wreck.id).is_none(), "The wreck is gone");
    assert_eq!(ui.sector.selected, None, "Its panel closed with it");
    assert!(ui.sector.game.stockpile.alloys >= before.alloys + 6);
    assert!(
        ui.sector
            .message
            .as_deref()
            .is_some_and(|m| m.starts_with("Salvaged "))
    );
}
