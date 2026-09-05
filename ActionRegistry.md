# Action Registry

The shared action rules live in `src/actions.rs`. UI previews and committed
orders use the same validation. Future AI orders can use the same API with a
different acting side; the rules do not depend on a window, cursor, or renderer.
This is an initial catalog, not a completed AI planner or action history.

## Move Fleet

- **Input:** fleet ID and a destination in world units.
- **Available when:** it is the acting side's turn, the fleet belongs to that
  side, and movement remains.
- **Preview:** a reachable endpoint and the exact distance cost. Targets beyond
  remaining range are capped in their original direction. The map border also
  caps that ray; movement does not slide along the border.
- **Commit:** update position and subtract actual distance traveled. The preview
  and commit call the same calculation. Multiple orders share one turn budget.
- **Controls:** left-click selects; right-click commits. Escape deselects but
  cannot undo an order. Input over interface panels cannot issue a move.
- **Warnings:** no movement, invalid destination, unknown fleet, wrong owner,
  wrong turn, or an endpoint identical to the fleet's current position.
- **Current resolution:** instantaneous, with no cancellation or interruption.
  Animation and fog reveal are future work; uninterrupted orders are provisional.

## End Turn

- **Input:** no parameters.
- **Available when:** it is the acting side's turn. Spending all movement is not
  required; holding position is valid.
- **Commit:** hand control to the other side and refresh that side's fleets to
  their movement-speed budgets. Unspent movement does not accumulate.
- **Counter:** starts at 1; advances after the Dominion finishes its turn.
- **Current enemy behavior:** the Dominion immediately passes using End Turn.
  No combat or strategic AI is implied by this placeholder.

## Extending the catalog

Add a typed `Action` variant, its definition, read-only validation/preview,
execution, and rule tests. Keep shared legality separate from AI evaluation and
UI presentation. Reminders and future AI candidate generation should consult
these rules rather than maintain competing definitions of legal actions.
