//! Shared rules for UI previews, committed orders, and future AI orders.
use std::fmt;

use lntrn_math::Vec2;

use crate::economy::{Build, Resources, ShipKind};
use crate::planets::PLANETS;
use crate::world::{Game, Side, WORLD_SIZE};

const EPSILON: f64 = 1.0e-8;

pub(crate) struct ActionDefinition {
    pub(crate) name: &'static str,
    pub(crate) description: &'static str,
}

pub(crate) const MOVE_FLEET: ActionDefinition = ActionDefinition {
    name: "Move Fleet",
    description: "Move to the preview endpoint, spending only the distance traveled.",
};
pub(crate) const SECURE_PLANET: ActionDefinition = ActionDefinition {
    name: "Secure Planet",
    description: "Hold a ship within 200 units for the turns shown. Influence decays when no ship stays.",
};
pub(crate) const BUILD_SHIP: ActionDefinition = ActionDefinition {
    name: "Build Ship",
    description: "Spend resources at the shipyard. The ship launches from the home world when done.",
};
pub(crate) const END_TURN: ActionDefinition = ActionDefinition {
    name: "End Turn",
    description: "Finish your orders and let the Dominion take its turn.",
};

#[derive(Clone, Copy, Debug)]
pub(crate) enum Action {
    MoveFleet { fleet_id: u32, destination: Vec2 },
    SecurePlanet { planet: usize },
    BuildShip { kind: ShipKind },
    EndTurn,
}

impl Action {
    pub(crate) fn definition(self) -> &'static ActionDefinition {
        match self {
            Self::MoveFleet { .. } => &MOVE_FLEET,
            Self::SecurePlanet { .. } => &SECURE_PLANET,
            Self::BuildShip { .. } => &BUILD_SHIP,
            Self::EndTurn => &END_TURN,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Preview {
    Move { destination: Vec2, cost: f64 },
    Secure { planet: usize },
    Build { kind: ShipKind, cost: Resources },
    EndTurn,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ActionError {
    NotYourTurn,
    UnknownFleet,
    NotYourFleet,
    NoMovement,
    InvalidDestination,
    AlreadyThere,
    UnknownPlanet,
    AlreadyYours,
    AlreadySecuring,
    NoShipInRange,
    EnemyInRange,
    NoShipyard,
    ShipyardBusy,
    CannotAfford,
    TurnLimit,
}

impl fmt::Display for ActionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NotYourTurn => "It is not your turn.",
            Self::UnknownFleet => "That fleet no longer exists.",
            Self::NotYourFleet => "You do not control that fleet.",
            Self::NoMovement => "No movement remaining this turn.",
            Self::InvalidDestination => "Choose a valid destination.",
            Self::AlreadyThere => "The fleet is already at that destination or map edge.",
            Self::UnknownPlanet => "That planet does not exist.",
            Self::AlreadyYours => "That world is already yours.",
            Self::AlreadySecuring => "A securing effort is already under way there.",
            Self::NoShipInRange => "No ship within 200 units to secure it.",
            Self::EnemyInRange => "An enemy ship within 200 units contests it.",
            Self::NoShipyard => "Only the Farlight home world has a shipyard.",
            Self::ShipyardBusy => "The shipyard is already building a ship.",
            Self::CannotAfford => "Not enough resources.",
            Self::TurnLimit => "The turn counter has reached its limit.",
        })
    }
}

/// Validation is read-only. Both the preview and execution use this exact result.
pub(crate) fn preview(game: &Game, actor: Side, action: Action) -> Result<Preview, ActionError> {
    if game.active_side != actor {
        return Err(ActionError::NotYourTurn);
    }
    match action {
        Action::EndTurn => {
            if actor == Side::Dominion && game.turn == u64::MAX {
                return Err(ActionError::TurnLimit);
            }
            Ok(Preview::EndTurn)
        }
        Action::MoveFleet {
            fleet_id,
            destination,
        } => preview_move(game, actor, fleet_id, destination),
        Action::SecurePlanet { planet } => {
            let target = PLANETS.get(planet).ok_or(ActionError::UnknownPlanet)?;
            let state = &game.planets[planet];
            if state.owner == Some(actor) {
                return Err(ActionError::AlreadyYours);
            }
            if state.securing {
                return Err(ActionError::AlreadySecuring);
            }
            if !game.fleet_in_range(actor, target.position) {
                return Err(ActionError::NoShipInRange);
            }
            if game.enemy_in_range(actor, target.position) {
                return Err(ActionError::EnemyInRange);
            }
            Ok(Preview::Secure { planet })
        }
        Action::BuildShip { kind } => {
            if actor != Side::Player {
                return Err(ActionError::NoShipyard);
            }
            if game.shipyard.is_some() {
                return Err(ActionError::ShipyardBusy);
            }
            let cost = kind.cost();
            if !game.stockpile.covers(cost) {
                return Err(ActionError::CannotAfford);
            }
            Ok(Preview::Build { kind, cost })
        }
    }
}

fn preview_move(
    game: &Game,
    actor: Side,
    fleet_id: u32,
    destination: Vec2,
) -> Result<Preview, ActionError> {
    let fleet = game
        .fleets
        .iter()
        .find(|f| f.id == fleet_id)
        .ok_or(ActionError::UnknownFleet)?;
    if fleet.owner != actor {
        return Err(ActionError::NotYourFleet);
    }
    if fleet.remaining <= EPSILON {
        return Err(ActionError::NoMovement);
    }
    if !destination.x.is_finite() || !destination.y.is_finite() {
        return Err(ActionError::InvalidDestination);
    }
    let delta = destination - fleet.position;
    let distance = delta.x.hypot(delta.y);
    if !distance.is_finite() {
        return Err(ActionError::InvalidDestination);
    }
    if distance <= EPSILON {
        return Err(ActionError::AlreadyThere);
    }
    let direction = delta / distance;
    let mut travel = distance.min(fleet.remaining);
    // Intersect the movement ray with the map border, preserving direction.
    for (start, direction, limit) in [
        (fleet.position.x, direction.x, WORLD_SIZE.x),
        (fleet.position.y, direction.y, WORLD_SIZE.y),
    ] {
        if direction > 0.0 {
            travel = travel.min((limit - start) / direction);
        }
        if direction < 0.0 {
            travel = travel.min(-start / direction);
        }
    }
    if travel <= EPSILON {
        return Err(ActionError::AlreadyThere);
    }
    Ok(Preview::Move {
        destination: fleet.position + direction * travel,
        cost: travel,
    })
}

/// Commit atomically after validation. Orders have no cancellation or undo path.
pub(crate) fn execute(
    game: &mut Game,
    actor: Side,
    action: Action,
) -> Result<Preview, ActionError> {
    let result = preview(game, actor, action)?;
    match (action, result) {
        (Action::MoveFleet { fleet_id, .. }, Preview::Move { destination, cost }) => {
            let fleet = game
                .fleets
                .iter_mut()
                .find(|f| f.id == fleet_id)
                .expect("validated fleet");
            let (from, vision) = (fleet.position, fleet.vision);
            fleet.position = destination;
            fleet.remaining = (fleet.remaining - cost).max(0.0);
            if fleet.remaining < EPSILON {
                fleet.remaining = 0.0;
            }
            // A ship charts everything it passes. Only the player keeps a chart for now.
            if actor == Side::Player {
                game.fog.reveal_path(from, destination, vision);
            }
        }
        (Action::SecurePlanet { planet }, Preview::Secure { .. }) => {
            game.planets[planet].securing = true;
        }
        (Action::BuildShip { kind }, Preview::Build { cost, .. }) => {
            game.stockpile = game.stockpile.minus(cost);
            game.shipyard = Some(Build {
                kind,
                turns_left: kind.build_turns(),
            });
        }
        (Action::EndTurn, Preview::EndTurn) => {
            if actor == Side::Player {
                game.end_player_turn();
            }
            game.active_side = match actor {
                Side::Player => Side::Dominion,
                Side::Dominion => {
                    game.turn += 1;
                    Side::Player
                }
            };
            for fleet in &mut game.fleets {
                if fleet.owner == game.active_side {
                    fleet.remaining = fleet.speed;
                }
            }
        }
        _ => unreachable!("preview corresponds to its action"),
    }
    // Whatever moved or changed hands, what is in view changed with it.
    game.fog.touch();
    Ok(result)
}

#[cfg(test)]
mod tests;
