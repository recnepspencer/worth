# Computed

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

A computed surface is retained derived runtime state. It declares what upstream
live or computed surfaces it depends on, which aspects it reads, which aspects
it produces, and whether it maintains incremental patches or falls back to
whole-refresh posture.

## Why You Use It

- you need runtime state that should derive automatically from live truth
- you want nested derived surfaces without hand-wiring invalidation
- you need a durable handle that can be materialized, observed, inspected, and
  reused by effects, previews, or later computed surfaces

## Stable Entry Points

- `workspace.computed(...)`
- `workspace.computed_view(...)`
- `workspace.computed_definition(...)`
- `workspace.materialize_result(...)`
- `workspace.inspections()?.inspect(...)`
- `workspace.state(...)`

Preferred ordinary DX is `workspace.computed(...)`. Compatibility surfaces such
as `computed_view(...)` and `computed_definition(...)` still exist for narrower
cases.

For downstream crates that need domain-owned declaration vocabularies before a
workspace is assembled, `WorthQueryComputedBuilder::surface(...)` is also
public. That seam is for declaration reuse, while ordinary runtime code should
still prefer the workspace surface.

## Core Mental Model

Computed state is not authoritative truth. It lives in the derived runtime lane
and must always be rebuildable from the authoritative surfaces it depends on.

What the handle means:

- it points at a retained derived surface
- it has a stable dependency contract
- it carries materialization and patch posture the runtime can inspect later

What the runtime tracks automatically:

- upstream live/computed dependencies
- read and produced aspect contracts
- incremental versus refresh-fallback posture
- materialized rows and pending derived patches

When a computed surface cannot honestly derive from one mutation delta alone,
whole-refresh fallback can rebuild from retained upstream rows instead of
pretending a local incremental update was sufficient.

## How It Executes

1. You declare the computed surface with a name, dependency contract, and
   maintainer.
2. The runtime validates that the upstream surfaces exist and that dependency
   structure is legal.
3. Relevant authoritative writes wake the computed surface through its declared
   upstreams.
4. The maintainer updates derived materialization and pending derived patches.
   Whole-refresh maintainers may rebuild from retained upstream live/computed
   rows when delta-only maintenance would be dishonest. When a whole-refresh
   computed declares more than one upstream live surface, the retained snapshot
   includes rows from every declared upstream live view, not only the view that
   happened to receive the triggering authoritative write. This also holds for
   downstream whole-refresh computeds reached through other computed surfaces:
   the runtime preserves the declared live siblings needed by the downstream
   rebuild contract rather than forcing the maintainer to reconstruct them
   manually.
5. The same whole-refresh contract also applies at declaration time. If a
   maintained computed is declared after its upstream live or computed truth
   already exists, the runtime seeds that computed immediately from retained
   upstream rows instead of waiting for the next write just to make the first
   materialization honest.
6. Whole-refresh maintainers do not receive a fake "write" signal for that
   first seed. They receive retained refresh context that tells them whether
   the rebuild came from a mutation or from declaration-time initialization,
   plus the snapshot token, touched aspect paths, and any runtime-owned refresh
   metadata the basis or write path attached.
7. `workspace.materialize_result(...)` returns current derived rows through the explicit runtime result boundary.
8. `workspace.inspections()?.inspect(...)` explains dependencies, produced aspects, and patch
   posture.

When a caller needs one typed retained row instead of raw `Vec<WorthQueryUnrefinedLiveShape>` row
archaeology, the admitted materialization lane is the stronger floor:

- `workspace.materialize_intent(&derived).execute()` returns one retained
  derived materialization result artifact
- that artifact can decode its single retained row through the explicit
  terminal export helper `terminal_json_decode_single_row::<T>()`
- when one downstream step needs a coherent retained artifact across multiple
  computed surfaces, `workspace.materialize_derived_artifact_bundle(...)`
  retains that multi-surface materialization as one Query-owned bundle instead
  of forcing caller-owned loops over repeated materialization entry
  - when that next step also needs an exact named artifact contract over that
  bundle, bind it through `bind_retained_artifact(...)` so the runtime owns the
  target-set check and artifact digest instead of caller code pretending the
  naked bundle was already the final artifact
  - when the caller already knows it wants one exact named retained artifact,
  use `materialize_derived_artifact_binding(...)` instead of spelling
  `materialize_derived_artifact_bundle(...).bind_retained_artifact(...)`
  manually
- when the next step needs typed identity, membership, provenance, continuity,
  or view-shape-qualified facts from that retained artifact, use the retained
  artifact binding's source-specific projection-consumption operation and
  inspect the admitted authority; this is an advanced substrate source, while
  ordinary read completions use
  `completion.consume_projection(read::project_facts()...)`
- when one mutation step already has a retained batch-write receipt and needs
  the matching inspection plus one exact retained derived artifact as the next
  authoritative package, use `materialize_batch_write_artifact_binding(...)`
  instead of stitching `workspace.inspections()?.inspect(...)` and
  `materialize_derived_artifact_binding(...)` together in caller code
- whole-refresh maintainers can also decode one retained computed upstream row
  through `WorthQueryRetainedUpstreamInputs::decode_single_computed_row(...)`
  instead of teaching local `serde_json` helper folklore
- scalar extraction helpers, row-pair decode helpers, and scalar-alignment
  helpers remain available as narrower expert seams when the artifact contract
  itself is the product surface, but they are no longer the ordinary typed fact
  lane after the retained/live projection-consumption closure

Nested computed surfaces execute in dependency order rather than in accidental
call order.

## Small Example

```rust
use worth_query::facade::runtime::{WorthQueryDerivedViewHandle, WorthQueryLiveView};
use worth_query::facade::runtime::WorthQueryUnrefinedLiveShape;

let mut workspace = runtime.workspace("tasks").unwrap();

let live: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
    .live_view("tasks.table", |q| {
        q.from("Task")
            .select(["identity.id", "title.value"])
            .schema_basis("tasks-table")
    })
    .unwrap();

let titles: WorthQueryDerivedViewHandle<WorthQueryUnrefinedLiveShape> = workspace
    .computed(
        "tasks.titles",
        |c| {
            c.depends_on_live(&live)
                .reads(["title.value"])
                .produces(["title.summary"])
        },
        TitleListMaintainer,
    )
    .unwrap();

let rows = workspace.materialize_result(&titles)?;
```

This is the smallest honest example because it shows the core pieces:
upstream dependency, read contract, produced contract, and derived
materialization.

## Real Example

```rust
use worth_query::facade::runtime::{WorthQueryDerivedViewHandle, WorthQueryInspection, WorthQueryLiveView};
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
        "computed.titles",
        |c| {
            c.depends_on_live(&live)
                .reads(["title.value"])
                .produces(["title.summary"])
        },
        TitleListMaintainer,
    )
    .unwrap();

let summary: WorthQueryDerivedViewHandle<WorthQueryUnrefinedLiveShape> = workspace
    .computed(
        "computed.summary",
        |c| {
            c.depends_on_computed(&titles)
                .reads(["title.summary"])
                .produces(["validation.state"])
        },
        SummaryMaintainer,
    )
    .unwrap();

workspace
    .insert("Task", |task| {
        task.aspect("identity.id", "task-1")
            .aspect("title.value", "Nested title")
    })
    .unwrap();

let rows = workspace.materialize_result(&summary)?;
let inspection = workspace.inspections()?.inspect(&summary).unwrap();

match inspection {
    WorthQueryInspection::DerivedView(derived) => {
        assert_eq!(derived.upstream_derived_views(), &["computed.titles".to_string()]);
        assert_eq!(derived.produced_aspects(), &["validation.state".to_string()]);
    }
    other => panic!("expected derived inspection, got {other:?}"),
}
```

What is authoritative:

- `Task` truth and the live view over it

What is derived:

- `computed.titles`
- `computed.summary`

What gets retained:

- dependency digests
- produced-aspect digests
- materialized rows
- pending incremental or refresh-fallback patch posture
- retained refresh context for whole-refresh rebuilds, including declaration-
  initialization posture when the runtime seeds from already-retained truth
- retained derived materialization artifacts that can decode one typed row
  through the runtime-owned `terminal_json_decode_single_row::<T>()` terminal
  export seam
- retained derived materialization bundles that preserve one snapshot token and
  one bundle digest across multiple typed computed rows when the next step
  needs a coherent retained artifact instead of three separate local calls
- retained derived artifact bindings that turn an exact multi-surface bundle
  into one named retained artifact with an explicit target-set digest
- retained scalar fact sets that bind one named derived artifact row to a
  stable set of dotted scalar fields when the next step needs historical or
  proof-bearing scalar evidence without reopening raw rows
- retained typed pair/triple decode helpers that let one named derived artifact
  yield a small typed pack without caller-owned repeated single-row decoding
- retained scalar alignment artifacts that prove two named retained rows stayed
  correspondence-aligned for one declared set of scalar field pairs

What gets inspected:

- upstream live/computed membership
- dependency and produced aspect contracts
- materialization and pending patch evidence

## How It Relates To Other Features

- Use [Live Views](live-views.md) as the normal authoritative upstream.
- Use [Effects](../execution/effects.md) when a change in computed output should deliver
  something or stage pending work.
- Use workspace state snapshots when you need readiness/posture rather than the
  derived rows themselves.

Computed surfaces are the normal place to keep runtime-derived readiness,
validation, rollups, or UI-friendly derived state. They are not the place to
hide authoritative writes.

## Inspection And Debugging

`workspace.inspections()?.inspect(&computed)` tells you:

- authority lane
- upstream live and computed dependencies
- dependency aspects and produced aspects
- whether delivery is incremental or whole-refresh fallback
- materialized row count
- pending patch counts and digests

This is the main way to verify whether a computed surface is wired correctly.

## Anti-Patterns

- Using computed state to smuggle authoritative mutations.
- Declaring hidden dependencies outside the `reads(...)` and
  `depends_on_live(...)` / `depends_on_computed(...)` contract.
- Assuming nested computeds are just callback chains instead of ordered retained
  runtime surfaces.
- Ignoring refresh-fallback posture and pretending it is incremental.
- Rebuilding from host-side shadow state instead of using retained upstream
  runtime rows when refresh fallback is declared.
- Reconstructing sibling upstream truth yourself because a write touched only
  one live view. The runtime already hands whole-refresh maintainers the
  retained rows for every declared upstream live surface.
- Decoding one retained computed row through local `Vec<WorthQueryUnrefinedLiveShape>` helper folklore
  when the runtime already owns typed decode on retained upstream inputs or on
  the derived materialization result artifact.
- Looping over several computed handles in caller code and pretending the
  resulting pack is a local product. If the next step needs a retained artifact
  over multiple computed rows, use the runtime-owned
  `materialize_derived_artifact_bundle(...)` seam.
- Treating declaration-time seeding as a fake mutation. If a computed depends
  on basis-specific or write-specific metadata, that metadata belongs in the
  runtime-owned refresh context, not in caller-side workaround logic.

## Current Limits

- Computed surfaces are stable for runtime-backed synchronous derived state.
- They must remain rebuildable from authority; they are not persistent truth.
- Runtime-backed temporal and async/resource posture now projects through the
  same retained computed state/inspection world rather than a parallel facade.
- Separate sibling facade-family roots for `Temporal`, `AsyncResource`, and
  `MixedCauseDelivery` still stay support-gated instead of becoming standalone
  computed-entry APIs.

## Related Docs

- [Workspace Overview](../foundations/workspace-overview.md)
- [Live Views](live-views.md)
- [Effects](../execution/effects.md)


