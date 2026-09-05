use super::*;
use crate::world::SCOUT_SPEED;

fn order(destination: Vec2) -> Action {
    Action::MoveFleet {
        fleet_id: 0,
        destination,
    }
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
