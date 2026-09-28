# Declarative Query Experience

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

The declarative Query experience is the ordinary product API for reads,
aggregates, live resources, history, comparisons, previews, mutations,
workflows, and inspection. You declare domain meaning in a capability
namespace, attach the required context, and ask Query to run or open it. Query owns
canonicalization, authority admission, planning, lower-runtime selection,
execution, lifecycle, receipts, and typed outcomes.

## Why You Use It

- product code expresses the job without advancing internal phases
- basis, policy, tenant, relationship, and lifecycle authority stay attached
  to typed context rather than raw identifiers
- one-shot and managed work share the same declaration grammar
- typed stops preserve where and why work did not complete
- downstream runtimes consume Query-owned outcomes and projection authority
  instead of reconstructing authority from receipts or digests

## Stable Entry Points

Choose the namespace that names the job:

- `worth_query::facade::read` for one-shot reads and projection facts
- `worth_query::facade::aggregate` for count aggregates over collection reads
- `worth_query::facade::live` for managed live resources
- `worth_query::facade::history` for retained historical reads
- `worth_query::facade::comparison` for diff and correspondence journeys
- `worth_query::facade::preview` for read-only and promotion-eligible previews
- `worth_query::facade::mutation` for declared mutations
- `worth_query::facade::workflow` for branch merge and writeback workflows
- `worth_query::facade::inspection` for outcome-attached inspection
- `worth_query::facade::domain` for domain-contributed workflows

Use `worth_query::facade::runtime` for workspace and backend-owned runtime
types. Use `worth_query::facade::consumer_kit` for test runtimes, support pins,
and downstream adoption proof. `facade::certification` is for explicit
certification fixtures, not product code.

## Core Mental Model

Every ordinary journey has the same shape:

```text
declare domain meaning
-> refine capability-specific intent
-> attach typed context with using(...)
-> run(...) or open(...)
-> handle the capability-owned outcome
```

The declaration is the consumer's authority boundary. Internal canonical,
admitted, planned, lowered, and executed artifacts are Query-owned. A receipt
describes completed work; it is not an input that can mint authority for a new
journey.

Context is declarative. `read::current()` selects the ordinary current truth
world. Policy, tenant, relationship, historical, comparison, preview, and
inspection contexts refine that choice without exposing phase transitions.
Managed live handles own activation, observation, update continuity, and
closeout.

Projection consumption returns `WorthQueryProjectionOutcome`; completed and
advisory authority moves through `into_admitted()`, while violations, deferral,
and unavailability remain typed.

## How It Executes

1. The capability namespace admits the declaration and preserves its typed
   meaning.
2. `using(...)` binds the required authority context.
3. Query canonicalizes and admits the request, plans it, and selects the
   supported backend path.
4. `run(...)` returns a typed completion or stop. `open(...)` returns a managed
   handle or stop.
5. Completion exposes results, receipts, counters, projection consumption, or
   inspection declarations without exposing the internal phase chain.

## Small Example

```rust
use worth_query::facade::{read, runtime::WorthQueryWorkspace};

fn load_tasks(workspace: &mut WorthQueryWorkspace) -> usize {
    let declaration = read::declare(|read| {
        read.local_collection(
            "Task",
            task_schema(),
            |query| {
                query.project(
                    read::AspectFieldSelector::new("identity", "id")
                        .expect("static identity selector"),
                )
            },
            |shape| {
                shape.field(read::AuthoredResultShapeField::new(
                    "identity",
                    "id",
                    "identity.id",
                ).expect("static identity result field"))
            },
        )
    })
    .expect("static task declaration should admit");

    declaration
        .using(read::current())
        .run(workspace)
        .completed()
        .map_or(0, |completion| completion.result().rows().len())
}

fn task_schema() -> read::QuerySchemaView {
    read::QuerySchemaView::new(
        "task-query",
        [read::SchemaFieldView::new(
            read::AspectName::new("identity").expect("static aspect"),
            read::FieldName::new("id").expect("static field"),
            read::ScalarAspectType::String,
        )],
        [],
    )
}
```

## Real Example

After a completed read, projection consumption extracts typed facts through
the authority sealed into that completion. Inspection starts from the same
completion and uses an explicit inspection basis.

```rust
use worth_query::facade::{foundation::basis_lifecycle, inspection, read};

let completion = declaration
    .using(read::current())
    .run(&mut workspace)
    .into_result()
    .expect("task read should complete");

let projection = completion.consume_projection(
    read::project_facts().entity_identities(),
);
let (_authority, warnings) = projection
    .into_admitted()
    .expect("identity projection should admit");
assert!(warnings.is_none());

let basis = basis_lifecycle()
    .historical_snapshot("task-inspection", true)
    .inspect()
    .expect("inspection basis should admit");
let inspection_outcome = inspection::declare(&completion)
    .with_rich_inspection()
    .using(inspection::inspection_basis(basis))
    .run(&workspace);
if let Some(inspected) = inspection_outcome.settled() {
    assert!(inspected.materialization().is_some());
} else if let Some(stop) = inspection_outcome.stop() {
    eprintln!("inspection stopped at {:?}", stop.source());
} else if let Some(unavailable) = inspection_outcome.unavailable() {
    eprintln!("inspection unavailable at {:?}", unavailable.source());
}
```

## How It Relates To Other Features

- Author query shape with [Read Composition](../authoring/read-composition.md).
- Check runtime support with [Support Matrix And Admission](../foundations/support-matrix-and-admission.md).
- Use [Projection Consumption](projection-consumption.md) when another runtime
  needs sealed Query facts.
- Use [Basis Capability Lifecycle](basis-capability-lifecycle.md) for advanced
  basis semantics outside ordinary context selection.
- Use [Graph Read Access Planning](../authoring/graph-read-access-planning.md)
  when a graph-shaped read needs explicit access-plan accountability.

## Inspection And Debugging

Start from the typed outcome. Read completions expose journey counters and the
runtime receipt. Stops expose their source and preserve any context receipt
created before the stop. Managed live handles expose activation and closeout
work. Inspection declarations add richer evidence without reopening source
authority.

Use the Consumer Kit when the question is whether downstream source still
contains local canonicalization, planning, execution, backend selection, or
subscription lifecycle assembly.

## Anti-Patterns

- importing canonicalization, planning, or execution phases into product code
- choosing a backend or serial/parallel route locally
- constructing authority from ids, digests, receipts, or result rows
- wrapping the internal phase chain in a domain-named coordinator
- opening, maintaining, and closing a live subscription through separate local
  helpers
- treating a typed stop as an empty result
- using certification helpers in production code

## Current Limits

- the in-memory backend is a test and certification surface, not a production
  persistence claim
- store-backed and durable continuation neighbors remain support-gated where
  the support matrix says so
- advanced graph access plans and lower-runtime capability registration remain
  explicit accountability surfaces; ordinary ergonomics do not erase their
  costs or denial posture
- capability namespaces share a grammar, but each keeps its own meaningful
  context and outcome types

## Related Docs

- [Read Composition](../authoring/read-composition.md)
- [Collections, Ordering, Aggregates, And Cursors](../authoring/collections-cursors-ordering-and-aggregations.md)
- [Live Views](../runtime-surfaces/live-views.md)
- [Historical Diff And Basis](historical-diff-and-basis.md)
- [Projection Consumption](projection-consumption.md)
- [Inspection](inspection.md)
- [Writes And Intent Boundaries](../execution/writes-and-intents.md)
- [Consumer Kit](../foundations/consumer-kit.md)
