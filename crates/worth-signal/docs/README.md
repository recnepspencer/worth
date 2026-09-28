# Worth Signal Docs

These docs are organized by subject, not by internal module shape.

They cover both main entry paths:

```rust
use worth_signal::easy::*;
use worth_signal::facade::*;
```

Use `easy` when you want the shortest path.
Use `facade` when you want the broader runtime surface from the start.

## Why This Is Different

Worth Signal is not just about rerunning less work.

It handles the simple path well, and it keeps working when the problem gets
harder.

The point is not to start small and then throw the system away later.
The point is to start with something clean and keep the same runtime story as
you grow.

That means:

- updates should land as one unit
- observation should happen at the same committed boundary
- rollback should leave the runtime in a sane state
- diagnostics should explain why work happened
- history should show what happened over time
- replay, snapshots, branches, and restore should be part of the same runtime story

The differentiator is not one feature in that list.
The differentiator is that those things belong to the same runtime instead of
being stitched together from separate tools.

That is the line:

- not "reactive graph plus some debug helpers"
- not "incremental cache plus a separate audit layer"
- not "rerun less work and figure out the rest later"

Worth Signal keeps change propagation, transactional truth, explanation, and
history in one system.

That is what `easy` is for.
It is the shortest path in.
It sits on top of the same runtime model.
It is the simple path into the full system.

If you want a small, clean setup, that path should feel good.
If you want the broader runtime surface, it is there.
It is one system either way.

## Same System, Different Size

Here is the shape we want:

- simple case: one input changes, one computed result updates
- bigger case: one input changes, a transaction updates several results, diagnostics explain why, and history keeps the time story

Side by side, that looks like this:

| Simple | Bigger |
| --- | --- |
| todo count updates when one item changes | checkout summary, shipping, and tax update together |
| one file preview updates after one edit | one source file change reruns diagnostics, symbol indexing, and the right build target |
| use `easy` and keep moving | use `facade` when you want the broader runtime surface |

The important part is that this is not a handoff between two different tools.
You can start small and stay on the same runtime as the system grows.

## One Continuous Story

Here is the flagship shape in one pass:

- a source file changes
- a transaction updates the build session
- only the right downstream targets rerun
- diagnostics explain why the bundle moved
- history and replay keep the trail

That is the point of the runtime.
It does not stop at "something recomputed."
It keeps the update coherent, explainable, and inspectable.

That now includes runtime-backed observation.
You can watch derived values on the `easy` path, or register runtime observers on
the broader surface, and both sit on the same commit-bounded observation model:

- one committed transaction delivers one boundary per matching observer
- rollback suppresses normal delivery
- diagnostics retain the latest observation boundary beside the latest flow

If you want to see that end to end, read:

- [guides/running-the-runtime.md](./guides/running-the-runtime.md)
- [guides/observation-and-effects.md](./guides/observation-and-effects.md)
- [guides/debugging-and-diagnostics.md](./guides/debugging-and-diagnostics.md)
- [guides/snapshots-branches-and-history.md](./guides/snapshots-branches-and-history.md)

And look at:

- [`examples/compiler_targeted_rebuild.rs`](../examples/compiler_targeted_rebuild.rs)

## Start Here

- [GETTING_STARTED.md](./GETTING_STARTED.md)
- [API_OVERVIEW.md](./API_OVERVIEW.md)
- [walkthroughs/compiler-targeted-rebuild.md](./walkthroughs/compiler-targeted-rebuild.md)

Start with `worth_signal::easy` if you want the shortest guided path.
Start with `worth_signal::facade::*` if you want the broader runtime surface.

## Core Concepts

- [core-concepts/README.md](./core-concepts/README.md)
- [core-concepts/graph-and-nodes.md](./core-concepts/graph-and-nodes.md)
- [core-concepts/aspects-and-dependencies.md](./core-concepts/aspects-and-dependencies.md)
- [core-concepts/runtime-and-transactions.md](./core-concepts/runtime-and-transactions.md)
- [core-concepts/diagnostics-and-history.md](./core-concepts/diagnostics-and-history.md)

## Guides

- [guides/defining-computation.md](./guides/defining-computation.md)
- [guides/observation-and-effects.md](./guides/observation-and-effects.md)
- [guides/running-the-runtime.md](./guides/running-the-runtime.md)
- [guides/debugging-and-diagnostics.md](./guides/debugging-and-diagnostics.md)
- [guides/runtime-policy.md](./guides/runtime-policy.md)
- [guides/transactions.md](./guides/transactions.md)
- [guides/snapshots-branches-and-history.md](./guides/snapshots-branches-and-history.md)
- [guides/parallel-execution.md](./guides/parallel-execution.md)

The runtime policy guide also documents the Milestone 10 objective/activation
handoff: `SignalRuntimePolicyRequest` → admitted → resolved → installed. The
installed policy is the planner authority; diagnostics tiers remain descriptive
presets and do not independently choose execution strategy.

## Walkthroughs

- [walkthroughs/easy-task-board.md](./walkthroughs/easy-task-board.md)
- [walkthroughs/compiler-targeted-rebuild.md](./walkthroughs/compiler-targeted-rebuild.md)
- [walkthroughs/document-partial-recompute.md](./walkthroughs/document-partial-recompute.md)

## Reference

- [reference/conditions-and-comparators.md](./reference/conditions-and-comparators.md)
- [reference/artifact-access.md](./reference/artifact-access.md)
- [reference/lifecycle-and-gc.md](./reference/lifecycle-and-gc.md)
- [reference/certification-and-harness.md](./reference/certification-and-harness.md)

If you are new here, read the start section and the core concepts first.

If you are already building something real, jump from `GETTING_STARTED` to
`running-the-runtime` and then pull in the subject guides you need.
