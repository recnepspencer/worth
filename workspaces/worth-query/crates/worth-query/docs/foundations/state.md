# State and Readiness Surfaces

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](ordinary-application-front-door.md).

## What This Feature Is

`workspace.state(...)` is the typed posture snapshot for a retained runtime
surface or a public facade family. It lets you ask whether a surface is ready,
pending, unsupported, or otherwise not in a normal ready posture without
guessing from incidental behavior.

## Why You Use It

- you need a typed readiness answer for a live or computed surface
- you need a typed answer for shipped live temporal/async posture, or a typed
  non-ready answer for deferred or unsupported public families
- you want a digest-bound explanation surface rather than an ad hoc boolean

## Stable Entry Points

- `workspace.state(...)`

Stable targets today:

- live view handles
- computed handles
- `WorthQueryRuntimeFacadeFamily` values for support-gated, deferred, or
  unsupported family posture

Related boundaries:

- `workspace.inspections()?.inspect(...)` gives richer retained evidence
- `workspace.public_support_matrix()` and `workspace.admit_public_api_family(...)`
  are the source of truth for stable versus support-gated, deferred, and
  unsupported family posture

## Core Mental Model

State snapshots are not domain data. They are typed posture reports.

Each snapshot binds:

- `kind`
- `basis_digest`
- `result_shape_digest`
- `authority_lane`
- `explanation`
- optional retained async result-state evidence
- optional retained remask posture when policy, tenant, relationship-proof, or
  schema context narrowed temporal/async runtime meaning before public
  projection
- `state_digest`

The point is to make readiness and support posture explicit and inspectable,
especially when shipped live temporal/async meaning and support-gated facade
families must stay distinct.

## How It Executes

For retained handles:

1. The runtime reconstructs retained installation or materialization posture.
2. It returns a `Ready` snapshot when the stable surface has concrete retained
   evidence.

For facade families:

1. The runtime looks at support posture for that family.
2. It returns a `Ready` snapshot for shipped support-gated runtime-backed rows,
   and a typed non-ready snapshot for deferred or unsupported rows.

The same API handles both concrete retained surfaces and support-gated family
questions.

## Small Example

```rust
use worth_query::facade::runtime::WorthQueryRuntimeFacadeFamily;

let temporal_state = workspace
    .state(WorthQueryRuntimeFacadeFamily::Temporal)
    .unwrap();

assert_eq!(temporal_state.kind().as_str(), "ready");
```

This is the smallest honest example because it shows that a separate
support-gated facade-family row can still project shipped runtime-backed
temporal posture through the same state surface without becoming a parallel
runtime root.

## Real Example

```rust
use worth_query::facade::runtime::{
    WorthQueryDerivedViewHandle, WorthQueryLiveView, WorthQueryRuntimeFacadeFamily,
};
use worth_query::facade::runtime::WorthQueryUnrefinedLiveShape;

let mut workspace = runtime.workspace("state-workspace").unwrap();

let view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
    .live_view("tasks.state-table", |q| {
        q.from("Task")
            .select(["identity.id", "title.value"])
            .schema_basis("runtime-task-state")
    })
    .unwrap();

let titles: WorthQueryDerivedViewHandle<WorthQueryUnrefinedLiveShape> = workspace
    .computed(
        "tasks.state-title-list",
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
            .aspect("title.value", "State DX")
    })
    .unwrap();

let live_state = workspace.state(&view).unwrap();
let computed_state = workspace.state(&titles).unwrap();
let temporal_state = workspace.state(WorthQueryRuntimeFacadeFamily::Temporal).unwrap();
let async_state = workspace.state(WorthQueryRuntimeFacadeFamily::AsyncResource).unwrap();
let intent_state = workspace.state(WorthQueryRuntimeFacadeFamily::Intent).unwrap();
```

What is ready:

- stable live and computed surfaces with retained evidence
- shipped support-gated runtime-backed rows such as `Temporal` and
  `AsyncResource`

What is support-gated:

- the separate `Temporal`, `AsyncResource`, and `MixedCauseDelivery`
  facade-family rows, which remain visible extension markers instead of
  standalone runtime roots

What is unsupported:

- unsupported public families such as intent on a runtime that has not admitted
  it

What the snapshot tells you:

- which authority lane the posture belongs to
- which basis and result-shape identity it binds to
- why the state is in that posture
- for async/resource-backed live meaning, which retained result-state posture is
  now true
- whether retained temporal/async meaning is still publicly visible or has been
  remasked or denied by policy, tenant, relationship-proof, or schema drift

## How It Relates To Other Features

- Use [Workspace Overview](workspace-overview.md) for the larger retained
  surface story.
- Use [Live Views](../runtime-surfaces/live-views.md) and [Computed](../runtime-surfaces/computed.md) for the
  handles whose posture you are snapshotting.
- Use [Branches and Previews](branches-and-previews.md) when the real
  question is lane isolation rather than ready versus support-gated posture.

`state(...)` is the concise typed posture surface. `inspect(...)` is the richer
explanation surface.

## Inspection And Debugging

Use state snapshots when you need quick answers to questions like:

- is this handle ready?
- is this family support-gated, deferred, or unsupported?
- which authority lane owns this posture?

Use inspect when you need the underlying retained evidence that explains the
snapshot in detail.

## Anti-Patterns

- Treating `state(...)` like domain data access.
- Reducing state posture to one boolean such as "ready or not".
- Assuming a future family is usable because a method name exists somewhere.
- Ignoring the explanation and authority lane on non-ready states.

## Current Limits

- State snapshots are stable for live handles, computed handles, and support
  families in the runtime-backed facade.
- Runtime-backed live temporal/async subscriptions now project their shipped
  posture through this same state surface instead of a parallel API.
- The separate `Temporal`, `AsyncResource`, and `MixedCauseDelivery`
  facade-family rows still return typed pending state because Query does not
  expose them as sibling public runtime roots.
- Async/resource-backed live subscriptions now retain one Query-owned async
  result-state vocabulary on the same state surface:
  `pending`, `current`, `failed`, `stale`, `cancelled`, `retried`,
  `revalidating`, `superseded`, and `denied`.
- Delivery cause and async result-state stay separate. A time-only wake can
  happen without changing async result-state, and async result-state can change
  without pretending a relational patch existed.
- Remask posture is also separate from async result-state. A live subscription
  can retain an async `current` result-state while the public state snapshot is
  `remasked` or `denied` because policy, tenant, relationship-proof, or schema
  context drift narrowed visibility before projection.

## Related Docs

- [Workspace Overview](workspace-overview.md)
- [Live Views](../runtime-surfaces/live-views.md)
- [Computed](../runtime-surfaces/computed.md)
- [Branches and Previews](branches-and-previews.md)


