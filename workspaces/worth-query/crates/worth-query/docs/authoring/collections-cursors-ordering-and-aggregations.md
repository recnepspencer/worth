# Collections, Ordering, Aggregates, And Cursors

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

WORTH Query exposes collection reads and count aggregates as ordinary
declarative capabilities. Product code describes collection meaning, ordering,
projection, and context; Query owns validation, admission, planning, execution,
and receipt construction.

Opaque cursor artifacts also exist in the collection and graph-access
substrate. They are not currently a general continuation argument or paged
result on the ordinary `facade::read` journey. Do not treat an internal
basis-bound cursor contract as a shipped ordinary pagination API.

## Why You Use It

- read a projected collection through `worth_query::facade::read`
- declare stable ascending or descending ordering as part of query meaning
- count the admitted rows of a collection through
  `worth_query::facade::aggregate`
- inspect the result receipt for basis, plan, breadth, and aggregate evidence
- keep cursor and persistence claims within the support actually exposed by
  the selected capability

## Stable Entry Points

Ordinary collection reads use:

- `read::declare(...)`
- collection authoring such as `local_collection(...)`
- `read::OrderingSelector::{ascending, descending}`
- `declaration.using(read::current()).run(workspace)`
- `WorthQueryReadOutcome`

Ordinary count aggregates use:

- `aggregate::declare(...)`
- the same collection authoring vocabulary
- `declaration.using(aggregate::current()).run(workspace)`
- `WorthQueryCountOutcome`
- `WorthQueryCountResult::{count, receipt}`

The collection plan, cursor boundary, requested aggregate family, and planner
functions are foundation or internal topology. They are not application
authoring entry points.

## Core Mental Model

```text
collection meaning + result shape + ordering
-> read::declare(...) or aggregate::declare(...)
-> using(authority-bearing context)
-> run(workspace)
-> typed completion or typed stop
```

Ordering is semantic input to the declaration. Query canonicalizes it and the
receipt reports the admitted execution plan. A count is a separate aggregate
outcome, not a collection row whose payload happens to contain a number.

The result receipt is evidence of what Query executed. Its digests and cursor
evidence are not reusable authority and do not authorize a caller to assemble
the next planning phase.

## How It Executes

1. The declaration closure builds a collection query and typed result shape.
2. `using(...)` attaches the current, policy/tenant, or relationship-proof
   context required by the capability.
3. Query validates ordering and projection, admits the context, builds the
   collection or aggregate plan, and executes it against the selected runtime.
4. A read completion exposes rows and a receipt. A count completion exposes a
   scalar count, the same context evidence, journey counters, and a read
   receipt whose collection result family is the count aggregate.
5. Invalid detail-to-count declarations stop during authoring. Context,
   planning, and runtime failures retain typed stop sources and next actions.

## Small Example

```rust
use worth_query::facade::{aggregate, runtime::WorthQueryWorkspace};

fn task_count(workspace: &mut WorthQueryWorkspace) -> u64 {
    aggregate::declare(|query| {
        query.local_collection(
            "Task",
            task_schema(),
            |tasks| {
                tasks.project(
                    aggregate::AspectFieldSelector::new("identity", "id")
                        .expect("static identity selector"),
                )
            },
            |shape| {
                shape.field(
                    aggregate::AuthoredResultShapeField::new(
                        "identity",
                        "id",
                        "identity.id",
                    )
                    .expect("static result field"),
                )
            },
        )
    })
    .expect("static collection count should declare")
    .using(aggregate::current())
    .run(workspace)
    .into_result()
    .expect("task count should complete")
    .into_result()
    .count()
}
```

`task_schema()` is the same `QuerySchemaView` used by an equivalent ordinary
collection read.

## Real Example

Ordering belongs inside the collection declaration:

```rust
use worth_query::facade::read;

let declaration = read::declare(|query| {
    query.local_collection(
        "Task",
        task_schema(),
        |tasks| {
            tasks
                .project(
                    read::AspectFieldSelector::new("identity", "id")
                        .expect("static identity selector"),
                )
                .project(
                    read::AspectFieldSelector::new("title", "value")
                        .expect("static title selector"),
                )
                .order_by(
                    read::OrderingSelector::ascending("title", "value")
                        .expect("static ordering selector"),
                )
        },
        |shape| {
            shape
                .field(
                    read::AuthoredResultShapeField::new(
                        "identity",
                        "id",
                        "identity.id",
                    )
                    .expect("static identity field"),
                )
                .field(
                    read::AuthoredResultShapeField::new(
                        "title",
                        "value",
                        "title.value",
                    )
                    .expect("static title field"),
                )
        },
    )
})?;

let outcome = declaration
    .using(read::current())
    .run(&mut workspace);

if let Some(completion) = outcome.completed() {
    let receipt = completion.result().receipt();
    assert_eq!(receipt.breadth().execution_records_emitted_count(),
               completion.result().rows().len());
} else if let Some(stop) = outcome.stop() {
    eprintln!("collection read stopped at {:?}: {:?}",
              stop.source(), stop.next_action());
}
```

## How It Relates To Other Features

- [Read Composition](read-composition.md) describes collection predicates,
  projections, and traversal meaning.
- [Graph Read Access Planning](graph-read-access-planning.md) covers bounded
  streaming and graph-frontier cursor sessions for admitted graph shapes.
- [Live Views](../runtime-surfaces/live-views.md) covers managed query-shaped
  collection maintenance.
- [Support Matrix And Admission](../foundations/support-matrix-and-admission.md)
  is the authority for admitted, deferred, and unsupported neighbors.

## Inspection And Debugging

Use the completion's result receipt and journey counters. The receipt exposes
the canonical query, execution plan, basis, result digest, collection result
family, breadth counters, fallback posture, and execution engine without
exposing planner construction authority.

For count aggregates, `execution_aggregate_input_count()` reports the admitted
input breadth and `execution_records_emitted_count()` is one for the scalar
result. These counters are evidence; they are not knobs for choosing a route.

## Anti-Patterns

- importing planner modules or calling planner functions from product code
- parsing a cursor digest into an offset or using it as basis authority
- documenting foundation cursor types as an ordinary continuation API
- counting rows in host code when `facade::aggregate` expresses the operation
- treating a stopped count as zero or a stopped collection read as empty
- exposing raw CDC as though it were a query-shaped collection result

## Current Limits

- Ordinary runtime-backed collection reads and count aggregates are shipped.
- Ordinary ordering is shipped through `OrderingSelector`.
- The ordinary read/count journey does not currently expose a general page
  size, continuation cursor argument, or paged collection result.
- Graph-access streaming has its own admitted cursor session and typed denial
  model; it is not interchangeable with ordinary collection pagination.
- Restart-stable cursor persistence and durable continuation remain Milestone
  11 work.
- Store-backed collection execution and pushdown parity remain Milestone 10
  work.

## Related Docs

- [Declarative Query Experience](../capabilities/declarative-query-experience.md)
- [Read Composition](read-composition.md)
- [Graph Read Access Planning](graph-read-access-planning.md)
- [Support Matrix And Admission](../foundations/support-matrix-and-admission.md)
