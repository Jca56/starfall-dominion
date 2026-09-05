//! Shared rules for UI previews, committed orders, and future AI orders.
use std::fmt;

use lntrn_math::Vec2;

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
pub(crate) const END_TURN: ActionDefinition = ActionDefinition {
    name: "End Turn",
    description: "Finish your orders and let the Dominion take its turn.",
};

#[derive(Clone, Copy, Debug)]
pub(crate) enum Action {
    MoveFleet { fleet_id: u32, destination: Vec2 },
    EndTurn,
}

impl Action {
    pub(crate) fn definition(self) -> &'static ActionDefinition {
        match self {
            Self::MoveFleet { .. } => &MOVE_FLEET,
            Self::EndTurn => &END_TURN,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Preview {
    Move { destination: Vec2, cost: f64 },
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
        } => {
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
    }
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
            fleet.position = destination;
            fleet.remaining = (fleet.remaining - cost).max(0.0);
            if fleet.remaining < EPSILON {
                fleet.remaining = 0.0;
            }
        }
        (Action::EndTurn, Preview::EndTurn) => {
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
    Ok(result)
}

#[cfg(test)]
mod tests;
