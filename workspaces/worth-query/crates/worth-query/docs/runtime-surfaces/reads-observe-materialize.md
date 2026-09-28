# Reads, Observation, and Materialization

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

These are the primary data-consumption surfaces for retained runtime handles.
They answer different questions:

- `read(...)` gives the current snapshot rows of a live view
- `observe(...)` drains incremental patch batches for a live view
- `materialize(...)` gives the current rows of a computed surface

They are not interchangeable. Each one reflects a different retained runtime
surface and cost shape.

These generic retained surfaces do not create application graph-obligation
authority. Installed application queries use the canonical graph-read plan and
session progression before their result reaches a retained or published
consumer surface. Do not add local graph legality checks around either path.

## Why You Use It

- you need the current rows for a live surface
- you need query-shaped incremental updates rather than broad mutation noise
- you need the current derived rows for a computed surface
- you need to consume an already declared retained surface without rebuilding
  its query or invalidation logic

## Stable Entry Points

- `workspace.read(...)`
- `workspace.observe(...)`
- `workspace.materialize_result(...)`

Related explanation surfaces:

- `workspace.inspections()?.inspect(...)`
- `workspace.state(...)`

Narrower helper:

- `workspace.observe_computed(...)` exists, but the stabilized closeout names
  `read`, `observe`, and `materialize` as the primary stable consumption set.
- `workspace.materialize_intent(&computed).execute()` is the stronger admitted
  floor when the caller needs retained proof, typed single-row decode, or a
  result artifact instead of naked rows.

## Core Mental Model

These APIs do not create new runtime surfaces. They consume surfaces you already
declared.

Think in terms of retained handles:

- a live view handle can be read as a snapshot or observed as patch batches
- a computed handle can be materialized as derived rows

The runtime keeps the handle identity, patch sequencing, and materialization
state. These methods only let you consume what is already retained.

Good to know:

- if you only need rows or patch batches, these APIs are the right surface
- if you need the explicit admitted proof chain behind covered read and
  materialization families, use
  [Intent Admission](../execution/intent-admission.md)
- if you need typed identity, membership, provenance, continuity, or
  view-shape-qualified facts from retained read/materialization work, use
  [Projection Consumption](../capabilities/projection-consumption.md) as the
  ordinary typed lane instead of reopening retained-artifact helper seams
- if you need one typed retained computed row through serde export, use the
  admitted materialization result and `terminal_json_decode_single_row::<T>()`
  instead of rebuilding local
  `serde_json` decode helpers over `workspace.materialize_result(...)`
- if you need several typed retained computed rows as one coherent next-step
  artifact, use `workspace.materialize_derived_artifact_bundle(...)` instead of
  local loops over repeated derived materialization calls
  - if that coherent next step has an exact named artifact contract, bind the
  bundle through `bind_retained_artifact(...)` instead of treating a naked
  bundle as caller-owned artifact identity
  - if you already know the next step is one exact named retained artifact, use
  `materialize_derived_artifact_binding(...)` instead of a caller-owned
  bundle-then-bind sequence
- if a post-mutation step already holds one retained batch-write receipt
  and needs the matching receipt inspection plus one exact retained derived
  artifact, use `materialize_batch_write_artifact_binding(...)` instead of
  reopening those as separate caller-owned inspection and materialization
  chores
- if you need typed identity, membership, provenance, or continuity facts from
  a read result, write receipt, or query-context execution, use
  [Projection Consumption](../capabilities/projection-consumption.md) instead of rebuilding
  those facts in caller code
- retained derived-artifact bindings, retained live-artifact bindings, scalar
  decode helpers, and bundle/binding helpers still exist as narrower expert
  utilities for exact named-artifact chores, but they are no longer the
  ordinary typed-fact path for retained runtime-backed product work

## How It Executes

1. Declare a live view or computed surface first.
2. Route authoritative writes through `workspace.insert(...)`,
   `workspace.update(...)`, or `workspace.delete(...)`.
3. Consume current live rows with `read(...)`.
4. Drain live patch batches with `observe(...)`.
5. Consume current derived rows with `materialize(...)`.

`read(...)` and `materialize(...)` are snapshot reads. `observe(...)` is a
drain of retained incremental evidence. It only gives you what has accumulated
for that live surface since the last drain.

## Small Example

```rust
use worth_query::facade::runtime::WorthQueryLiveView;
use worth_query::facade::runtime::WorthQueryUnrefinedLiveShape;

let mut workspace = runtime.workspace("tasks").unwrap();

let table: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
    .live_view("tasks.table", |q| {
        q.from("Task")
            .select(["identity.id", "title.value"])
            .schema_basis("tasks-table")
    })
    .unwrap();

workspace
    .insert("Task", |task| {
        task.aspect("identity.id", "task-1")
            .aspect("title.value", "Buy milk")
    })
    .unwrap();

let rows = workspace.read(&table);
let patches = workspace.observe(&table);
```

This is the smallest honest example because it shows the two different live
consumption paths on the same handle.

## Real Example

```rust
use worth_query::facade::runtime::{WorthQueryDerivedViewHandle, WorthQueryLiveView};
use worth_query::facade::runtime::WorthQueryUnrefinedLiveShape;

let mut workspace = runtime.workspace("builder").unwrap();

let view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
    .live_view("tasks.builder-table", |q| {
        q.from("Task")
            .select(["identity.id", "title.value"])
            .order_by("title.value")
            .schema_basis("runtime-task-builder")
            .as_surface("tasks.builder-table")
    })
    .unwrap();

let titles: WorthQueryDerivedViewHandle<WorthQueryUnrefinedLiveShape> = workspace
    .computed(
        "tasks.builder-title-list",
        |c| {
            c.depends_on_live(&view)
                .reads(["title.value"])
                .produces(["runtime.title_list"])
        },
        TitleListMaintainer,
    )
    .unwrap();

workspace
    .insert("Task", |task| {
        task.aspect("identity.id", "task-1")
            .aspect("title.value", "Builder DX")
    })
    .unwrap();

let live_rows = workspace.read(&view);
let live_patches = workspace.observe(&view);
let computed_rows = workspace.materialize_result(&titles)?;
```

What is authoritative:

- the `Task` truth and the live surface over it

What is derived:

- the computed title list

What gets retained:

- live patch batches for `view`
- materialized derived rows for `titles`

What gets consumed:

- `read(&view)` returns current live rows
- `observe(&view)` drains incremental live delivery
- `materialize(&titles)` returns current derived rows

## How It Relates To Other Features

- Use [Live Views](live-views.md) to declare the retained live surface.
- Use [Computed](computed.md) to declare the retained derived surface.
- Use [Intent Admission](../execution/intent-admission.md) when you need the
  covered `read_family_intent(...)`, `read_live_intent(...)`, or
  `materialize_intent(...)` review/admit/execute path instead of the snapshot
  convenience surfaces.
- Use [Projection Consumption](../capabilities/projection-consumption.md) when plain rows are
  not the real contract and the caller needs typed consumed facts with a
  receipt.
- Use [State and Readiness Surfaces](../foundations/state.md) when you need typed posture
  rather than data rows.

These APIs are consumption boundaries. They should come after declaration, not
instead of it.

## Inspection And Debugging

If the data does not look right:

- inspect the live view to verify subscription and installation posture
- inspect the computed handle to verify dependencies, produced aspects, and
  patch posture
- snapshot state when you need a typed explanation of ready versus pending

## Anti-Patterns

- Treating `observe(...)` like a snapshot read.
- Treating `read(...)` like an event stream.
- Calling `materialize(...)` on a live view mental model.
- Assuming a drain returns historical truth forever instead of retained
  incremental evidence since the last drain.
- Reconstructing a typed single retained computed row from raw `materialize(...)`
  rows when
  `materialize_intent(...).execute().terminal_json_decode_single_row::<T>()` is
  the admitted runtime-owned terminal export floor.
- Rebuilding a multi-surface retained artifact in caller code from repeated
  `workspace.materialize_result(...)` or repeated one-off materialization entry when
  `materialize_derived_artifact_bundle(...)` is the runtime-owned floor for
  that job.
- Treating a retained derived-materialization bundle as if it were already a
  named artifact contract instead of binding the exact target set through
  `bind_retained_artifact(...)`.
- Treating retained-artifact helper seams such as scalar extraction, row-pair
  decode, or live-artifact bundle binding as the default typed product lane
  when projection consumption owns retained and live typed facts.

## Current Limits

- These APIs are stable for runtime-backed synchronous live and derived
  surfaces.
- They reflect retained runtime state, not store-backed replay or durable
  continuation.
- Runtime-backed temporal delivery causes and async/resource-backed retained
  posture now flow through these ordinary retained surfaces rather than through
  a parallel API.
- Store-backed replay and durable continuation for those lanes remain later
  work.

## Related Docs

- [Live Views](live-views.md)
- [Computed](computed.md)
- [Projection Consumption](../capabilities/projection-consumption.md)
- [State and Readiness Surfaces](../foundations/state.md)


