# Runtime And Transactions

The graph describes the dependency shape.

The runtime is what actually moves change through that shape.

## What The Runtime Does

`SignalRuntime` is the main operating surface.

It lets you:

- mark source changes
- run computed work
- read current results
- keep diagnostics and history

## Why Transactions Matter

Use a transaction when a change should land as one unit.

Real examples:

- a user edits a document and preview, diagnostics, and publish checks need to stay in sync
- one file changes and the right rebuild targets need to update together
- a pricing update affects tax, shipping, and checkout summary together
- a failed update should roll back cleanly instead of leaving a half-applied state

## Main Surfaces

- `SignalRuntime`
- `runtime.advance_signal_branch(execution, ...)`
- `tx.mark_changed(...)`
- `tx.target(node).run(...)`
- `tx.target(node).read(...)`

## Practical Rule

If partial application would be a bug, use a transaction.
