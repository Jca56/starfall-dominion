use super::*;
use crate::economy::SECURE_RANGE;
use crate::world::{Fleet, SCOUT_SPEED, SCOUT_VISION};

fn order(destination: Vec2) -> Action {
    Action::MoveFleet {
        fleet_id: 0,
        destination,
    }
}

fn planet(name: &str) -> usize {
    PLANETS
        .iter()
        .position(|planet| planet.name == name)
        .expect("a named planet")
}

/// A full round: the player ends the turn and the Dominion passes.
fn round(game: &mut Game) {
    execute(game, Side::Player, Action::EndTurn).unwrap();
    execute(game, Side::Dominion, Action::EndTurn).unwrap();
}

#[test]
fn multiple_orders_share_a_budget_and_preview_matches_execution() {
    let mut game = Game::default();
    let start = game.fleets[0].position;
    execute(&mut game, Side::Player, order(start + Vec2::new(30.0, 0.0))).unwrap();
    assert_eq!(game.fleets[0].remaining, SCOUT_SPEED - 30.0);
    // Far beyond range along a 3-4-5 direction: the fleet spends everything it has left.
    let next = order(game.fleets[0].position + Vec2::new(3000.0, 4000.0));
    let expected = preview(&game, Side::Player, next).unwrap();
    assert_eq!(execute(&mut game, Side::Player, next).unwrap(), expected);
    assert_eq!(game.fleets[0].remaining, 0.0);
    let left = SCOUT_SPEED - 30.0;
    let reached = start + Vec2::new(30.0 + left * 0.6, left * 0.8);
    assert!((game.fleets[0].position - reached).length() < 1e-8);
    assert_eq!(
        execute(&mut game, Side::Player, next),
        Err(ActionError::NoMovement)
    );
}

#[test]
fn edge_clipping_preserves_direction_and_only_charges_actual_distance() {
    let mut game = Game::default();
    game.fleets[0].position = Vec2::new(WORLD_SIZE.x - 10.0, 500.0);
    execute(
        &mut game,
        Side::Player,
        order(Vec2::new(WORLD_SIZE.x + 90.0, 600.0)),
    )
    .unwrap();
    assert!((game.fleets[0].position - Vec2::new(WORLD_SIZE.x, 510.0)).length() < 1e-8);
    assert!((game.fleets[0].remaining - (SCOUT_SPEED - 10.0_f64.hypot(10.0))).abs() < 1e-8);
    let before = game.clone();
    assert_eq!(
        execute(
            &mut game,
            Side::Player,
            order(Vec2::new(WORLD_SIZE.x + 100.0, 510.0))
        ),
        Err(ActionError::AlreadyThere)
    );
    assert_eq!(game, before);
}

#[test]
fn illegal_actions_do_not_mutate_the_game() {
    let mut game = Game::default();
    let before = game.clone();
    for destination in [Vec2::new(f64::NAN, 1.0), Vec2::new(1.0, f64::INFINITY)] {
        assert_eq!(
            execute(&mut game, Side::Player, order(destination)),
            Err(ActionError::InvalidDestination)
        );
        assert_eq!(game, before);
    }
    assert_eq!(
        execute(&mut game, Side::Dominion, order(Vec2::ZERO)),
        Err(ActionError::NotYourTurn)
    );
    assert_eq!(
        execute(
            &mut game,
            Side::Player,
            Action::MoveFleet {
                fleet_id: 42,
                destination: Vec2::ZERO
            }
        ),
        Err(ActionError::UnknownFleet)
    );
    assert_eq!(
        execute(&mut game, Side::Player, order(before.fleets[0].position)),
        Err(ActionError::AlreadyThere)
    );
    assert_eq!(
        execute(&mut game, Side::Player, Action::SecurePlanet { planet: 99 }),
        Err(ActionError::UnknownPlanet)
    );
    assert_eq!(
        execute(
            &mut game,
            Side::Player,
            Action::SecurePlanet {
                planet: planet("Arcadia")
            }
        ),
        Err(ActionError::AlreadyYours)
    );
    assert_eq!(
        execute(
            &mut game,
            Side::Player,
            Action::SecurePlanet {
                planet: planet("Vesper")
            }
        ),
        Err(ActionError::NoShipInRange)
    );
    assert_eq!(
        execute(
            &mut game,
            Side::Player,
            Action::BuildShip {
                kind: ShipKind::Scout
            }
        ),
        Err(ActionError::CannotAfford)
    );
    assert_eq!(game, before);
    game.fleets[0].owner = Side::Dominion;
    assert_eq!(
        execute(&mut game, Side::Player, order(Vec2::ZERO)),
        Err(ActionError::NotYourFleet)
    );
}

#[test]
fn turn_handoff_refills_without_banking_unused_movement() {
    let mut game = Game::default();
    let destination = game.fleets[0].position + Vec2::new(20.0, 0.0);
    execute(&mut game, Side::Player, order(destination)).unwrap();
    execute(&mut game, Side::Player, Action::EndTurn).unwrap();
    assert_eq!(game.active_side, Side::Dominion);
    assert_eq!(game.fleets[0].remaining, SCOUT_SPEED - 20.0);
    execute(&mut game, Side::Dominion, Action::EndTurn).unwrap();
    assert_eq!(game.turn, 2);
    assert_eq!(game.active_side, Side::Player);
    assert_eq!(game.fleets[0].remaining, SCOUT_SPEED);
    assert_eq!(game.fleets[0].position, destination);
}

#[test]
fn a_move_charts_the_corridor_it_flies_through() {
    let mut game = Game::default();
    let start = game.fleets[0].position;
    let stop = start + Vec2::new(SCOUT_SPEED, 0.0);
    let beyond = stop + Vec2::new(SCOUT_VISION + 20.0, 0.0);
    let beside = start + Vec2::new(SCOUT_SPEED / 2.0, SCOUT_VISION - 10.0);
    assert!(game.fog.explored_at(start));
    assert!(
        !game.fog.explored_at(beside),
        "Not yet in range of anything"
    );
    assert!(!game.fog.explored_at(beyond));
    let version = game.fog.version();
    execute(&mut game, Side::Player, order(stop)).unwrap();
    assert!(
        game.fog
            .explored_at(stop + Vec2::new(SCOUT_VISION - 20.0, 0.0))
    );
    assert!(
        game.fog.explored_at(beside),
        "The corridor along the way is charted"
    );
    assert!(!game.fog.explored_at(beyond));
    assert_ne!(game.fog.version(), version);
    assert!(game.can_see(Side::Player, stop + Vec2::new(SCOUT_VISION - 20.0, 0.0)));
    assert!(!game.can_see(Side::Player, start - Vec2::new(SCOUT_VISION, 0.0)));
}

#[test]
fn securing_takes_turns_in_range_and_decays_when_the_ship_leaves() {
    let mut game = Game::default();
    let target = planet("L2-b");
    let position = PLANETS[target].position;
    // Park just inside range, then start the effort.
    game.fleets[0].position = position + Vec2::new(SECURE_RANGE - 5.0, 0.0);
    let secure = Action::SecurePlanet { planet: target };
    execute(&mut game, Side::Player, secure).unwrap();
    assert!(game.planets[target].securing);
    assert_eq!(
        execute(&mut game, Side::Player, secure),
        Err(ActionError::AlreadySecuring)
    );
    round(&mut game);
    round(&mut game);
    assert_eq!(game.planets[target].influence, 2);
    assert_eq!(game.planets[target].owner, None);
    // Leave: influence decays, and once it is gone the effort is over.
    game.fleets[0].position = position + Vec2::new(SECURE_RANGE + 50.0, 0.0);
    round(&mut game);
    assert_eq!(game.planets[target].influence, 2 - PLANETS[target].decay);
    assert!(game.planets[target].securing);
    round(&mut game);
    round(&mut game);
    assert_eq!(game.planets[target].influence, 0);
    assert!(!game.planets[target].securing);
    // Come back and hold for the full count: the world is Farlight's.
    game.fleets[0].position = position;
    execute(&mut game, Side::Player, secure).unwrap();
    for _ in 0..PLANETS[target].secure_turns {
        round(&mut game);
    }
    assert_eq!(game.planets[target].owner, Some(Side::Player));
    assert!(!game.planets[target].securing);
    assert_eq!(
        execute(&mut game, Side::Player, secure),
        Err(ActionError::AlreadyYours)
    );
}

#[test]
fn an_enemy_in_range_blocks_securing_and_contests_income() {
    let mut game = Game::default();
    let home = Game::home_planet();
    let pay = PLANETS[home].yield_per_turn;
    assert_eq!(game.income(), pay, "Arcadia pays every turn");
    round(&mut game);
    assert_eq!(game.stockpile, pay);
    let raider = Fleet {
        id: 77,
        name: "Raider".to_string(),
        owner: Side::Dominion,
        position: PLANETS[home].position + Vec2::new(SECURE_RANGE - 10.0, 0.0),
        speed: 100.0,
        remaining: 100.0,
        vision: 150.0,
    };
    game.fleets.push(raider);
    assert!(game.contested(home));
    assert!(game.income().is_empty(), "A contested world pays nothing");
    round(&mut game);
    assert_eq!(game.stockpile, pay);
    // The raider also blocks a securing effort on a nearby free world.
    let target = planet("L2-b");
    game.fleets[0].position = PLANETS[target].position;
    game.fleets[1].position = PLANETS[target].position + Vec2::new(0.0, SECURE_RANGE - 1.0);
    assert_eq!(
        execute(
            &mut game,
            Side::Player,
            Action::SecurePlanet { planet: target }
        ),
        Err(ActionError::EnemyInRange)
    );
}

#[test]
fn the_shipyard_charges_up_front_and_launches_after_two_turns() {
    let mut game = Game::default();
    let cost = ShipKind::Scout.cost();
    game.stockpile = cost.plus(Resources::new(1, 1));
    let build = Action::BuildShip {
        kind: ShipKind::Scout,
    };
    execute(&mut game, Side::Player, build).unwrap();
    assert_eq!(game.stockpile, Resources::new(1, 1));
    assert_eq!(
        execute(&mut game, Side::Player, build),
        Err(ActionError::ShipyardBusy)
    );
    assert_eq!(
        execute(&mut game, Side::Dominion, build),
        Err(ActionError::NotYourTurn)
    );
    round(&mut game);
    assert_eq!(game.fleets.len(), 1);
    assert_eq!(game.shipyard.as_ref().map(|b| b.turns_left), Some(1));
    round(&mut game);
    assert!(game.shipyard.is_none());
    assert_eq!(game.fleets.len(), 2);
    let launched = &game.fleets[1];
    assert_eq!(launched.name, "Scout 2");
    assert_eq!(launched.owner, Side::Player);
    assert_eq!(
        launched.remaining, SCOUT_SPEED,
        "Ready to move on the turn it appears"
    );
    assert!((launched.position - PLANETS[Game::home_planet()].position).length() < SECURE_RANGE);
    assert_eq!(launched.id, 1);
}
