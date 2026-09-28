# Installed Operation Re-Execution And Replay

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

Installed operation re-execution runs a normalized workflow intent through a
fresh Query-owned run. Certification replay does the same execution under a
sealed replay capability, then compares the new trace with an earlier trace.
Use ordinary re-execution for product work. Use certification replay only when
you must prove that an installed operation still has the same meaning.

## Why You Use It

- Retry a complete installed workflow without copying stage bookkeeping.
- Prove results, publications, effects, invariants, conditional decisions, and
  lineage remain semantically equivalent.
- Localize drift to the exact stage and semantic category.
- Replay against the exact retained historical snapshot when Query can prove
  that snapshot belongs to the original trace.

## Stable Entry Points

Ordinary host/runtime code uses:

- `domain::WorthQueryBoundDomainOperation::reexecute(...)`
- `domain::WorthQueryNormalizedWorkflowIntent`
- `domain::WorthQueryWorkflowIntentStage`
- `domain::WorthQueryWorkflowReexecutionStop`

Certification code imports `worth_query_replay::facade` and uses:

- `issue_query_certification_replay_capability()`
- `replay_installed_workflow(...)`
- `admit_installed_historical_replay_basis(...)`
- `replay_installed_workflow_historical(...)`
- `WorthQueryCertificationReplayResult`
- `WorthQueryReplayComparison` and `WorthQueryReplayDivergence`

An installed replay contract also requires a matching
`WorthQueryDomainReplaySemanticComparator`. Runtime construction rejects a
replayable definition whose workflow executor does not provide that comparator.

## Core Mental Model

Re-execution is a new execution, not reuse of an old trace. It receives a new
run identity and new receipts even when its semantic meaning is identical.

Certification replay adds two gates:

1. Query's mandatory comparator checks the exact typed workflow semantics.
2. The installed domain comparator may impose stricter domain meaning, but it
   cannot override drift Query already detected.

Replay authority stays in the cert-only `worth-query-replay` package. The
ordinary host facade can execute and publish a workflow but cannot issue a
certification replay capability. A successful certification result contains
inspection data, not the ordinary completed-trace phase object.

## How It Executes

```text
bind the installed operation under one admitted basis
  -> normalize the complete stage intent
  -> execute a fresh workflow run
  -> retain its trace, effects, conditions, publication, and lineage
  -> certification only: compare against the original semantic trace
  -> return Equivalent or one localized divergence
```

The comparator checks operation identity, stage set and predecessor topology,
typed inputs and outputs, result state, warnings outside declared noise,
effects and mutation targets, invariant-to-effect coverage, conditional
observations and decisions, trace-bound lineage, and derived publication.

## Small Example

Ordinary re-execution uses the same public bound-operation surface as the first
run:

```rust
use worth_query::facade::domain;

let intent = domain::WorthQueryNormalizedWorkflowIntent::new(vec![
    domain::WorthQueryWorkflowIntentStage::new(
        "start",
        domain::WorthQueryWorkflowIntentValue::NotRequired,
    ),
    domain::WorthQueryWorkflowIntentStage::new(
        "publish",
        domain::WorthQueryWorkflowIntentValue::Text("ready".into()),
    ),
])?;

let trace = bound.reexecute(intent, &mut workspace).unwrap();
```

`reexecute` consumes a freshly bound operation. Rebind before each independent
execution; do not try to reset or clone a workflow run.

## Real Example

Certification replay keeps the original trace, fresh bound operation, intent,
and workspace explicit:

```rust
use worth_query_replay::facade as replay;

let result = replay::replay_installed_workflow(
    replay::issue_query_certification_replay_capability(),
    &original_trace,
    replay_bound,
    intent,
    &mut workspace,
)
.unwrap();

assert!(matches!(
    result.comparison(),
    &replay::WorthQueryReplayComparison::Equivalent
));
assert_ne!(
    result.original_trace_identity(),
    result.replay_trace_identity(),
);
assert_eq!(
    result.original_execution_counters(),
    result.replay_execution_counters(),
);
```

Equivalent semantics do not mean identical execution identity. The returned
Foundational attachment describes the original/replay relationship without
becoming replay authority. `replay_semantics()` is an inspection projection;
it cannot be published, consumed, or settled as an ordinary workflow trace.

## How It Relates To Other Features

- [Conditional Installed Operations](./conditional-installed-operations.md)
  supplies the realized decision path that replay compares.
- [Installed Operation Lineage And Promotion](./installed-operation-lineage-and-promotion.md)
  supplies the exact lineage meaning included in the semantic trace.
- [Projection Consumption](../capabilities/projection-consumption.md) remains
  the ordinary path for consuming either execution's published facts.

## Inspection And Debugging

Inspect:

- `comparison()` and its localized `WorthQueryReplayDivergence`
- `original_semantics()` and `replay_semantics()`
- `original_execution_counters()` and `replay_execution_counters()`
- replay counters such as `original_stage_index_entries`,
  `intent_stage_checks`, and `semantic_stage_comparisons`
- `foundational_attachment()` for descriptive cross-boundary evidence

Every stopped workflow exposes `executed_effects()`. A failed or deferred
re-execution therefore does not hide mutations that happened before the stop.

## Anti-Patterns

- Importing replay through `worth-query-host` or an ordinary entry crate.
- Treating equal trace digests or equal outputs as semantic equivalence.
- Reusing the original run identity or stage receipts.
- Letting a domain comparator mask Query-detected drift.
- Passing an unrelated historical correspondence or current snapshot label.

## Current Limits

- Certification replay is cold, explicit, and cert-only.
- Installed historical replay currently admits retained snapshots. Delta replay
  and full reconstruction fail closed because the in-memory owner has no such
  execution substrate.
- Diagnostic warning differences are ignorable only when the installed replay
  contract explicitly declares that noise.
- Replay does not return an ordinary completed workflow trace. Product code
  that needs publication executes and publishes through a separately bound
  ordinary operation, then compares its typed result with certification
  inspection evidence where required.

## Related Docs

- [Runtime-Installed Domains And Operations](./runtime-installed-domains.md)
- [Conditional Installed Operations](./conditional-installed-operations.md)
- [Installed Operation Lineage And Promotion](./installed-operation-lineage-and-promotion.md)
- [Historical Diff And Basis](../capabilities/historical-diff-and-basis.md)
