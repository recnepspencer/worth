# Effects

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

An effect is a retained delivery or staging surface that reacts to changes in a
live view or computed surface. Effects are for producing delivery artifacts or
pending write-intent residue from declared reactive inputs. They are not a
general place to mutate truth directly.

## Why You Use It

- you need UI-facing or bridge-facing delivery when runtime meaning changes
- you want conditional delivery tied to explicit trigger and condition contracts
- you need staged pending work that can later be inspected or, when admitted,
  executed through an intent path

## Stable Entry Points

- `workspace.effect(...)`
- `workspace.inspections()?.inspect(...)`

Stable effect declaration covers:

- `when_live(...)`
- `when_computed(...)`
- `condition_expression(...)`
- `deliver(...)`
- `write_intent(...)`
- `meaningful_change_suppression()`

Important boundary:

- effect declaration and inspection are stable
- pending write-intent staging is part of the effect surface
- consuming staged work through `runtime.next_effect_write_intent(...)` is now
  part of the covered intent-admission surface rather than a vague future name

## Core Mental Model

Effects do not own truth and should not become hidden business logic. They sit
after live or computed changes and produce one of a few explicit outcomes:

- delivered
- suppressed
- expression failed
- pending write intent

What the handle means:

- it points at a retained reactive delivery/staging surface
- it remembers the trigger contract, condition contract, and routing posture

What the runtime tracks automatically:

- trigger source kind and source identity
- condition descriptor, inputs, and outputs
- pending delivery or pending write-intent residue
- feedback phase graph and terminal posture

## How It Executes

1. You declare an effect against a live view or computed handle.
2. A relevant write wakes the trigger surface.
3. The effect evaluates its condition and action.
4. The runtime records delivered, suppressed, failed, or pending-intent
   terminal artifacts.
5. `workspace.inspections()?.inspect(...)` explains the effect's retained evidence.
6. If the effect staged pending write intent and the runtime admits that path,
   `runtime.next_effect_write_intent(...)` consumes one pending unit through
   the shared intent-admission path.

Effects are deliberately separate from computed state so derived meaning and
delivery/staging do not blur together.

## Small Example

```rust
use worth_query::facade::runtime::{WorthQueryEffectHandle, WorthQueryLiveView};
use worth_query::facade::runtime::WorthQueryUnrefinedLiveShape;

let mut workspace = runtime.workspace("ui").unwrap();

let live: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
    .live_view("tasks.table", |q| {
        q.from("Task")
            .select(["identity.id", "title.value"])
            .schema_basis("tasks-table")
    })
    .unwrap();

let badges: WorthQueryEffectHandle<WorthQueryUnrefinedLiveShape> = workspace
    .effect("ui.title-badges", |e| {
        e.when_live(&live, ["title.value"])
            .condition_expression("expr.title.badge", ["title.value"], ["ui.badge"])
            .deliver("ui.badges")
    })
    .unwrap();
```

This is the smallest honest example because it shows the full effect contract:
trigger, optional expression semantics, and delivery target.

## Real Example

```rust
use worth_query::facade::runtime::{
    WorthQueryDerivedViewHandle, WorthQueryEffectHandle, WorthQueryInspection, WorthQueryLiveView,
};
use worth_query::facade::runtime::WorthQueryUnrefinedLiveShape;

let mut workspace = runtime.workspace("workflow").unwrap();

let live: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
    .live_view("tasks.table", |q| {
        q.from("Task")
            .select(["identity.id", "title.value"])
            .schema_basis("tasks-table")
    })
    .unwrap();

let titles: WorthQueryDerivedViewHandle<WorthQueryUnrefinedLiveShape> = workspace
    .computed(
        "computed.titles.effect",
        |c| {
            c.depends_on_live(&live)
                .reads(["title.value"])
                .produces(["title.summary"])
        },
        TitleListMaintainer,
    )
    .unwrap();

let effect: WorthQueryEffectHandle<WorthQueryUnrefinedLiveShape> = workspace
    .effect("ui.summary-badges", |e| {
        e.when_computed(&titles, ["title.summary"])
            .condition_expression(
                "expr.summary.badge",
                ["title.summary"],
                ["ui.badge"],
            )
            .deliver("ui.summary")
            .meaningful_change_suppression()
    })
    .unwrap();

workspace
    .insert("Task", |task| {
        task.aspect("identity.id", "task-1")
            .aspect("title.value", "Computed effect task")
    })
    .unwrap();

let inspection = workspace.inspections()?.inspect(&effect).unwrap();

match inspection {
    WorthQueryInspection::Effect(effect) => {
        assert_eq!(effect.trigger_source(), "computed.titles.effect");
        assert_eq!(effect.pending_delivery_count(), 1);
    }
    other => panic!("expected effect inspection, got {other:?}"),
}
```

What is authoritative:

- the underlying truth and the live surface over it

What is derived:

- computed output
- effect delivery or staged pending-intent residue

What gets retained:

- trigger digest
- condition digest
- declaration digest
- pending delivery digest
- feedback graph and terminal family

What gets inspected:

- trigger source kind
- condition contract
- delivery/suppression/failure/pending-intent counts
- latest feedback phase posture

## How It Relates To Other Features

- Use [Live Views](../runtime-surfaces/live-views.md) when the trigger should be current truth.
- Use [Computed](../runtime-surfaces/computed.md) when the trigger should be derived runtime
  state.
- Use the workspace overview when you need the broader branch/preview/state
  story.

Effects should usually sit after live or computed surfaces. If what you really
need is another piece of derived state, that is usually a computed surface, not
an effect.

## Inspection And Debugging

`workspace.inspections()?.inspect(&effect)` tells you:

- effect name
- trigger source and trigger source kind
- condition descriptor, inputs, and outputs
- pending delivery counts by terminal family
- latest delivery family
- feedback graph and termination posture

This is how you tell the difference between "nothing happened", "suppressed",
"expression failed", and "pending work was staged".

## Anti-Patterns

- Using effects to propagate ordinary derived state instead of using computed.
- Treating `write_intent(...)` as if it immediately mutated truth.
- Hiding important condition semantics outside the declared expression
  contract.
- Ignoring suppression and failure posture and assuming every wake delivers.

## Current Limits

- Effect declaration, delivery, suppression, failure accounting, and inspection
  are stable in the runtime-backed synchronous facade.
- Pending write-intent staging is available, but authoritative execution still
  depends on admitted intent support through the covered effect-intent family.
- Store-backed effect execution and durable replay are **deferred**—see
  [authority-scoped effect execution](authority-scoped-effect-execution.md), not
  implied by effect authoring alone.
- Runtime-backed mixed truth/time/async delivery meaning now projects through
  the shipped effect delivery, inspection, remask, and downstream-delivery
  surfaces rather than waiting on a separate effect-only facade family.

## How It Relates To Authority-Scoped Execution

This doc stops at declaration, staging, delivery, and `workspace.inspections()?.inspect` for
effects. Lowering, eligibility, admission, and execute receipts live in
[authority-scoped effect execution](authority-scoped-effect-execution.md) with
`effect_lifecycle_support_matrix()` honesty (`StoreBackedExecutionDeferred`,
`DurableReplayDeferred`, advisory-only rows). Do not document a “full effect
pipeline” here without checking that matrix.

## Related Docs

- [Authority-scoped effect execution](authority-scoped-effect-execution.md)
- [Workspace Overview](../foundations/workspace-overview.md)
- [Live Views](../runtime-surfaces/live-views.md)
- [Computed](../runtime-surfaces/computed.md)
- [Intent Admission](intent-admission.md)


