# Worth Query Workspace Overview

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](ordinary-application-front-door.md).

## What This Feature Is

`WorthQueryWorkspace` is the internal engine surface for runtime-backed
`worth-query` work. It is not an application API: `worth-ui-query-binding` and
Query's own certification use it, while application code uses
`worth-query-decl` and `worth-query-host`. It is the place where engine code declares
live views, computed state, and effects, performs reads and writes, authors
same-batch graph composition, works against existing authoritative truth, opens
preview or branch sessions, snapshots state, and inspects retained runtime
evidence.

## Why You Use It

- you are working on engine-side code, such as `worth-ui-query-binding`, that
  needs one context for runtime-backed query work
- you want to compose live views, computed state, and effects without reaching
  into lower-runtime plumbing
- you want branch/preview/state/inspection surfaces to line up around one
  stable mental model
- you want one simple public bridge-backed read-runtime bootstrap for hostile
  tests, examples, or downstream runtime bring-up instead of custom minimal
  assembly folklore

## Stable Entry Points

Stable runtime-backed entry points:

- `runtime.workspace(...)`
- `workspace.live_view(...)`
- `workspace.computed(...)`
- `workspace.effect(...)`
- `workspace.preview(WorthQuerySessionLabel, ...)` / `workspace.branch(WorthQuerySessionLabel, ...)`
- `workspace.insert(...)`
- `workspace.update(...)`
- `workspace.delete(...)`
- `workspace.submissions()?.submit(...)`
- `workspace.submissions()?.submit_batch(...)`
- `workspace.write_intent(...)`
- `workspace.write_batch_intent(...)`
- `workspace.compose_graph(...)`
- typed existing-truth binding artifacts consumed by graph composition
- `workspace.probe_existing_intent(...)`
- `workspace.read(...)`
- `workspace.observe(...)`
- `workspace.materialize_result(...)`
- `workspace.state(...)`
- `workspace.inspections()?.inspect(...)`
- `workspace.public_api_contract()`
- `workspace.public_downstream_delivery_contract()`
- `workspace.public_handle_contract()`
- `workspace.public_support_matrix()`
- `workspace.public_mutation_surface_report()`
- `workspace.admit_public_api_family(...)`
- `workspace.downstream_delivery(...)`
- `workspace.domain(DomainMarker)`
- `workspace.observe_operating_world()` or
  `workspace.prepare_mutation_operating_world()`
- `.family(FamilyMarker).bind(&installed_domain, OperationMarker)`

The public support and mutation-surface contracts define the runtime story.

Good to know:

- covered intent families are real now, but they are concrete named families,
  not blanket facade-family support.
- canonical machine identity comes from
  `WorthQueryEvidenceIdentity::compose(...)`, not caller-owned string hashing
- preview and branch entry use `WorthQuerySessionLabel` as the ordinary typed
  identity lane; callers should not mint free-form string labels
- `error.stop_class()` is the machine lane for runtime denials; messages are
  presentation and may change wording without changing the contract
- workflow capability authoring that targets preview inspection or preview
  mutation uses `BridgePreviewSessionIdentity`; session labels name the opened
  preview or branch context, while preview-session identities name the retained
  preview artifact that workflow evidence binds against
- Method presence is not a support claim. Use the support matrix and admission
  gate when you are near deferred or unsupported families.
- Use the mutation surface report when you need explicit preferred versus
  lower-level posture for mutation surfaces.
- direct workspace write, batch, and existing-truth helper seams are sealed;
  command-shaped work enters through explicit intent or submission lanes
- the ordinary public bridge-backed read-runtime bootstrap now lives on the
  real runtime builder path, so hostile/runtime-backed read tests do not need a
  separate custom scaffolding story just to obtain a valid raw read lane

## Core Mental Model

The workspace is not the owner of truth. `worth-relational` still owns truth
semantics, and `worth-signal` still owns reactive evaluation. The workspace is
the public context that lets you declare what you want from those runtimes in a
coherent, inspectable shape.

Think of it this way:

- authoritative truth lives below the workspace
- the workspace declares durable runtime surfaces over that truth
- handles returned by the workspace are retained runtime objects, not raw data
- installed domain handles and operation-family views borrow one runtime; the
  operating world is the only authority-bearing installed-operation root
- `read`, `observe`, `materialize`, `state`, and `inspect` let you ask
  different questions about those retained surfaces
- `observe` can now surface time-only live delivery directly, so a freshness
  shift, window crossing, or deadline does not need a fabricated relational
  patch to become visible
- `state` and `inspect` retain that last live delivery cause, which keeps
  runtime snapshots and explanations aligned after patch batches have been
  drained
- `downstream_delivery(...)` projects that same retained last delivery into one
  typed downstream envelope with explicit basis negotiation and durable-resume
  debt instead of forcing another runtime to interpret drained delivery batches
- `state` and ordinary `inspect` now also share one compact runtime posture
  projection for live temporal/async handles, so product code can read
  `current`, `time_only`, `mixed_cause`, `stale`, `cancelled`, `retried`,
  `revalidating`, `superseded`, or `denied` posture from the same scalar
  surfaces instead of reverse-engineering rich retained delivery artifacts
- `state` and `inspect` also retain one Query-owned async result-state
  vocabulary for async/resource-backed live subscriptions, so product code does
  not need to invent its own `loading` / `cancelled` / `retrying` taxonomy
- `compose_read` lets you execute one bounded graph-shaped read without
  installing a retained live view first
- graph composition lets you execute one symbolic same-batch authoring program
  without flattening graph lifecycle meaning into caller-owned command batches
- existing-truth work uses typed binding artifacts, graph-composition lanes,
  and probe intents so target identity and verification evidence remain
  explicit without caller-owned workspace helper seams
- projection consumption lets you turn read results, write receipts, or
  query-context execution artifacts into typed facts when rows or payload bags
  are not a strong enough contract
- `compose_read` and `live_view` now share the same lower declarative request
  substrate, so traversal, predicate, ordering, and hidden query-only
  projection are not maintained in two separate stories

If you are building ordinary runtime-backed product features, the workspace is
the place you should start from.

If you are building a downstream runtime or domain crate, read
[Downstream Runtime Integration](downstream-runtime-integration.md) before you
invent local mutation, basis, or inspection patterns above Query.

If you are bringing up a bridge-backed read runtime for examples, tests, or
serious downstream integration, start from the public bridge-backed bootstrap
support instead of rebuilding an ad hoc runtime out of one-off adapter
fixtures. Phase 9.5 closed that raw read bootstrap gap on the ordinary builder
lane.

## How It Executes

The typical workspace lifecycle looks like this:

1. Open a named workspace from a configured runtime.
2. Declare live views, computed surfaces, and effects.
3. Write authoritative changes through `workspace.insert(...)`,
   `workspace.update(...)`, `workspace.delete(...)`, or explicit
   `workspace.submissions()` / write-intent lanes.
   Aspect-level reset stays on the same path through builder calls such as
   `task.clear("description.value")`.
4. Use `workspace.compose_graph(...)` when one logical authoring step needs
   symbolic same-batch handles or graph lifecycle evidence instead of plain
   ordered writes. Installed application invariants run through provider-session
   progression, not a workspace callback.
5. Use typed existing-truth binding artifacts inside graph composition, or
   `workspace.probe_existing_intent(...)`, when the target is already
   authoritative and target binding or backend verification is part of the
   contract.
6. Read current rows, drain patches, or materialize derived rows from retained
   handles.
7. Use an installed application query when a typed graph-shaped read needs one
   canonical access plan, managed session, and terminal receipt.
8. Inspect handles or snapshot state when you need explanations or readiness.
9. Open preview or branch sessions when you need isolated experimentation.

The workspace keeps the durable handles and their retained evidence aligned with
the same runtime. A handle from one runtime is not portable into another
runtime's inspection or state APIs.

## Small Example

```rust
use worth_query::facade::runtime::WorthQueryLiveView;
use worth_query::facade::runtime::WorthQueryUnrefinedLiveShape;

let mut workspace = runtime.workspace("editor").unwrap();

let tasks: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
    .live_view("tasks.table", |q| {
        q.from("Task")
            .select(["identity.id", "title.value"])
            .order_by("title.value")
            .schema_basis("editor-task-table")
    })
    .unwrap();

workspace
    .insert("Task", |task| {
        task.aspect("identity.id", "task-1")
            .aspect("title.value", "Buy milk")
    })
    .unwrap();

let rows = workspace.read(&tasks);
let patches = workspace.observe(&tasks);
```

This is the smallest honest example because it shows the full loop:
declaration, authoritative write, snapshot read, and incremental observation.

If the live surface needs bounded graph reach, declare that on the builder:

```rust
let topology = workspace
    .live_view("topology.half-edge-detail", |q| {
        q.from("WorthTopologyEntity")
            .detail()
            .select(["identity.id", "half_edge.kind"])
            .allow_traversal_relation("HalfEdgeNext", 2)
            .schema_basis("topology-half-edge-detail")
    })
    .unwrap();
```

That traversal declaration is part of the live-query request itself, not just a
schema annotation for later tooling.

## Real Example

```rust
use worth_query::facade::runtime::{
    WorthQueryDerivedViewHandle, WorthQueryEffectHandle, WorthQueryInspection,
    WorthQueryLiveView,
};
use worth_query::facade::runtime::WorthQueryUnrefinedLiveShape;

let mut workspace = runtime.workspace("workflow.editor").unwrap();

let canvas: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
    .live_view("workflow.editor.canvas", |q| {
        q.from("WorkflowNode")
            .select(["identity.id", "layout.frame", "validation.state"])
            .order_by("identity.id")
            .schema_basis("workflow-editor-canvas")
    })
    .unwrap();

let readiness: WorthQueryDerivedViewHandle<WorthQueryUnrefinedLiveShape> = workspace
    .computed(
        "workflow.editor.readiness",
        |c| {
            c.depends_on_live(&canvas)
                .reads(["validation.state", "layout.frame"])
                .produces(["runtime.readiness"])
        },
        WorkflowReadinessMaintainer,
    )
    .unwrap();

let badges: WorthQueryEffectHandle<WorthQueryUnrefinedLiveShape> = workspace
    .effect("workflow.editor.badges", |e| {
        e.when_computed(&readiness, ["runtime.readiness"])
            .condition_expression(
                "expr.workflow.badges",
                ["runtime.readiness"],
                ["ui.badges"],
            )
            .deliver("ui.badges")
            .meaningful_change_suppression()
    })
    .unwrap();

workspace
    .insert("WorkflowNode", |node| {
        node.aspect("identity.id", "node-1")
            .aspect("layout.frame", "0,0,320,80")
            .aspect("validation.state", "ready")
    })
    .unwrap();

let canvas_patches = workspace.observe(&canvas);
let readiness_rows = workspace.materialize_result(&readiness)?;
let badge_explanation = workspace.inspections()?.inspect(&badges).unwrap();

match badge_explanation {
    WorthQueryInspection::Effect(effect) => {
        assert_eq!(effect.trigger_source(), "workflow.editor.readiness");
    }
    other => panic!("expected effect inspection, got {other:?}"),
}
```

What is authoritative:

- `workspace.insert(...)`, `workspace.update(...)`, `workspace.delete(...)`,
  and explicit submission/intent lanes are the preferred authoritative
  mutation paths
- direct workspace write and batch helpers are sealed from consumers

What is derived:

- `readiness` is derived runtime state
- `badges` is effect delivery state over derived output

What gets retained:

- live subscription installation for `canvas`
- materialized derived rows and patch posture for `readiness`
- delivery and phase evidence for `badges`

What gets inspected:

- one unified `workspace.inspections()?.inspect(...)` call can explain each retained surface
- mutation receipts preserve declared aspect operations, so inspection can show
  whether an authored aspect was a `set` or a `clear`

## Minimal CRUD Walkthrough

If you want the smallest end-to-end mental model, think in three layers:

- authoritative truth such as `Car` and `Person`
- live views over that truth
- computed surfaces that turn those runtime surfaces into UI-ready rows

```rust
let mut workspace = runtime.workspace("garage").unwrap();

let cars = workspace
    .live_view("garage.cars", |q| {
        q.from("Car")
            .select(["identity.id", "make.value", "model.value"])
            .order_by("make.value")
            .schema_basis("garage-cars")
    })
    .unwrap();

let people = workspace
    .live_view("garage.people", |q| {
        q.from("Person")
            .select(["identity.id", "name.value", "car_id.value"])
            .order_by("name.value")
            .schema_basis("garage-people")
    })
    .unwrap();

workspace
    .insert("Car", |car| {
        car.aspect("identity.id", "car-1")
            .aspect("make.value", "Honda")
            .aspect("model.value", "Civic")
    })
    .unwrap();

workspace
    .insert("Person", |person| {
        person
            .aspect("identity.id", "person-1")
            .aspect("name.value", "Ava")
            .aspect("car_id.value", "car-1")
    })
    .unwrap();

let car_rows = workspace.read(&cars);
let person_rows = workspace.read(&people);
```

That is already enough to replace a lot of ordinary app wiring. The same
declared surfaces are now ready for reads, observation, materialization, and
inspection without hand-built cache or invalidation glue.

## How It Relates To Other Features

- Use [Live Views](../runtime-surfaces/live-views.md) when you need query-shaped current truth.
- Use [Computed](../runtime-surfaces/computed.md) when runtime state should derive from live or
  other computed surfaces.
- Use [Effects](../execution/effects.md) when a surface should deliver or stage something
  because another surface changed.
- Use [Graph Composition Authoring](../authoring/graph-composition-authoring.md) when one
  logical write must carry symbolic same-batch handles or lifecycle evidence
  instead of plain ordered writes.
- Use [Existing Truth](../capabilities/existing-truth.md) when a mutation or probe starts
  from already authoritative truth instead of creating new truth from scratch.
- Use [Projection Consumption](../capabilities/projection-consumption.md) when a read result,
  write receipt, or query-context execution must become typed consumed facts
  instead of staying a raw payload or receipt artifact.
- Use [Intent Admission](../execution/intent-admission.md) when you need the proof-bearing
  review/admit/execute path for covered runtime intent work.
- Use preview or branch sessions when the work should remain isolated from
  current truth.

`workspace.read(...)` and `workspace.materialize_result(...)` are snapshot-style reads.
`workspace.observe(...)` is the incremental patch path. `workspace.state(...)`
and `workspace.inspections()?.inspect(...)` are explanation surfaces, not substitutes for
domain data access.

## Inspection And Debugging

The workspace gives you two main explanation paths:

- `workspace.state(...)` for a typed readiness/supported/pending snapshot
- `workspace.inspections()?.inspect(...)` for retained evidence about a handle or receipt
- `workspace.downstream_delivery(...)` for the latest retained live delivery
  when another runtime or server boundary needs one transport-safe Query-owned
  contract instead of raw delivery batches

For live temporal/async handles, both of those surfaces now expose the same
compact runtime posture before you drop into the richer retained inspection
artifact:

- overall runtime posture kind
- delivery-cause posture
- async posture when retained async result-state is present
- basis-drift posture when retained async meaning stayed typed across basis or
  generation drift instead of collapsing into a generic denied note
- retained support evidence digest so the scalar surface does not silently drop
  the installed support context

Use the public support and handle contracts when you need to understand whether
an ordinary runtime lane is shipped, whether a separate facade-family root is
still intentionally deferred, or whether a surface is unsupported before
exposing it in another runtime.

## Anti-Patterns

- Treating the workspace as the owner of truth semantics.
- Reaching for lower-runtime APIs in ordinary product code when the workspace
  already owns the public surface.
- Assuming method presence means feature support without checking the support
  matrix or admission gate.
- Passing handles across runtimes and expecting inspection or state APIs to work.

## Current Limits

- The workspace surface is safe for runtime-backed, synchronous composition.
- Runtime-backed temporal basis, async/resource execution, mixed-cause
  delivery, remask posture, and downstream delivery projection now ship
  through the same `workspace` / handle / state / inspection world.
- What remains intentionally deferred is a separate sibling facade-family root
  for `Temporal`, `AsyncResource`, or `MixedCauseDelivery`; those rows stay
  visible as extension and support markers rather than becoming parallel entry
  points beside live handles.
- Store-backed execution and durable artifact reload remain deferred.
- covered intent families are documented in
  [Intent Admission](../execution/intent-admission.md), but broader intent
  vocabulary is still not blanket stable facade-family support.

## Related Docs

- [Runtime-Installed Domains And Operations](../domain-capabilities/runtime-installed-domains.md)
- [Conditional Installed Operations](../domain-capabilities/conditional-installed-operations.md)
- [Live Views](../runtime-surfaces/live-views.md)
- [Computed](../runtime-surfaces/computed.md)
- [Effects](../execution/effects.md)
- [Graph Composition Authoring](../authoring/graph-composition-authoring.md)
- [Downstream Runtime Integration](downstream-runtime-integration.md)
- [Async Resources And Result State](../capabilities/async-resources-and-result-state.md)
- [Existing Truth](../capabilities/existing-truth.md)
- [Projection Consumption](../capabilities/projection-consumption.md)
- [Intent Admission](../execution/intent-admission.md)


