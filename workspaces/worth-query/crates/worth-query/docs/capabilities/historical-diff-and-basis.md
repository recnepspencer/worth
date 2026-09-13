# Historical Basis, Diff, And Comparison Queries

## What This Feature Is

Historical basis and diff queries let you run the same validated query against
different admitted versions of truth, then compare the query-shaped results.
They preserve Query meaning across current, branch, preview-derived, and
retained-history worlds instead of exposing raw storage deltas.

## Why You Use It

- Run one canonical query against current or retained historical truth.
- Make branch or preview comparisons without changing the query expression.
- Reuse retained history only when the runtime can prove that reuse is valid.
- Receive typed denials for unsupported replay, reconstruction, or basis
  substitution.
- Produce added, removed, modified, and unchanged query-result rows rather than
  low-level record churn.

## Stable Entry Points

Declare the truth world through the foundation facade:

```rust
use worth_query::facade::foundation::basis_lifecycle;
```

Use the policy facade to bind that declaration to Query-owned runtime or
historical evidence:

- `admit_query_basis_context(...)`
- `execute_query_basis_context(...)`
- `execute_and_build_query_basis_result_bundle(...)`
- `bind_diff_query_context(...)`
- `shape_query_diff_change_set(...)`
- `build_query_diff_result_bundle(...)`

Historical path selection is exposed through the foundation facade:

- `HistoricalEvaluationRequest`
- `admit_historical_evaluation_path(...)`
- `resolve_historical_materialization_path(...)`
- `HistoricalEvaluationAdmission`
- `HistoricalMaterializationPathMetadata`

Store-backed replay and reconstruction remain support-gated. A visible type or
request family does not imply that the active runtime profile admits it.

## Core Mental Model

The query stays the same; the admitted basis changes.

For a product branch, Runtime World owns the selected composite commit and its
bounded single-parent ancestry. Query may expose current or exact selected
truth only after World admission. An exact selection remains repeatable while
the branch advances; asking for current truth is a distinct fresh operation.
`application.branches().history(branch, maximum)` returns an owner-issued,
bounded ancestry page. `continue_ancestry(maximum)` follows only the page's
protected `next_parent_commit()`, and `select(&entry)` admits that exact
historical product through World before Query can execute a read.

Branch names, commit IDs, component IDs, digests, page cursors, and inspection
records are descriptive. They cannot mint a product observation or substitute
one branch's component basis into another. The live history page and admitted
observations protect exact World resources. A `RuntimeWorldRecoveryCursor` is
only a descriptive position in the separate recovery catalog and carries no
retention or recovery authority.

Query owns three linked decisions:

1. which truth world the caller declared
2. whether the supplied runtime or history evidence authorizes that world for
   this query
3. whether two admitted query contexts can form a meaningful comparison

The resulting `ScopedQueryBasisContext` keeps the operation-scoped basis proof
and the admitted query context together. Its digest getters are useful for
inspection, but cannot be used to construct another context.

A diff is defined over query results. It is not an API for comparing arbitrary
snapshots or storage blobs.

## How It Executes

```text
validated query + basis declaration + Query-owned binding evidence
  -> scoped query-context admission
  -> execution against the admitted world
  -> result with basis metadata

two compatible scoped query contexts
  -> diff-context admission
  -> query-shaped change set
  -> diff result bundle
```

Historical work adds an admitted materialization path before context binding.
The path records whether retained state, replay-tail reuse, or reconstruction
is legal and what work it predicts.

## Small Example

```rust
use worth_query::facade::{
    foundation::basis_lifecycle,
    policy::{
        admit_query_basis_context, execute_query_basis_context,
        QueryContextBindingSource,
    },
};

let context = admit_query_basis_context(
    basis_lifecycle().current_head(),
    QueryContextBindingSource::RuntimeCurrent(&preflight_bundle),
)?;

let result = execute_query_basis_context(&context)?;
assert_eq!(result.query_digest(), context.query_digest());
```

The caller declares current truth and supplies the preflight evidence Query
already produced. Query performs lifecycle admission and context binding as one
transition; the caller never assembles a scoped context.

## Real Example

```rust
use worth_query::facade::{
    foundation::{
        admit_historical_evaluation_path, basis_lifecycle,
        resolve_historical_materialization_path, HistoricalEvaluationRequest,
        HistoricalPathReuseDescriptor,
    },
    policy::{
        admit_query_basis_context, bind_diff_query_context,
        shape_query_diff_change_set, QueryContextBindingSource,
        QueryDiffChangeFamily,
    },
};

let current = admit_query_basis_context(
    basis_lifecycle().current_head(),
    QueryContextBindingSource::RuntimeCurrent(&current_preflight),
)?;

let request = HistoricalEvaluationRequest::retained_snapshot(
    "workflow-main@snapshot-42",
    0,
    0,
    HistoricalPathReuseDescriptor::retained_reuse(),
);
let admitted_path = admit_historical_evaluation_path(&validated, &request)?;
let materialization = resolve_historical_materialization_path(&admitted_path)?;

let historical = admit_query_basis_context(
    basis_lifecycle().historical_snapshot(
        "workflow-main@snapshot-42",
        true,
    ),
    QueryContextBindingSource::Historical {
        query_preflight: &historical_preflight,
        admission: &admitted_path,
        metadata: &materialization,
    },
)?;

let comparison = bind_diff_query_context(&current, &historical)?;
let changes = shape_query_diff_change_set(&comparison)?;

assert!(changes.rows().iter().all(|row| {
    matches!(
        row.change_family(),
        QueryDiffChangeFamily::Added
            | QueryDiffChangeFamily::Removed
            | QueryDiffChangeFamily::Modified
            | QueryDiffChangeFamily::Unchanged
    )
}));
```

The historical path is admitted before the context exists. The comparison then
uses two scoped contexts that still refer to the same canonical query meaning.
The output describes changes to that query's result, not every underlying
storage mutation between the two worlds.

## How It Relates To Other Features

- [Basis Capability Lifecycle](./basis-capability-lifecycle.md) owns the sealed
  world and operation proof used by query-context admission.
- [Schema Validation](../modeling/schema-validation.md) establishes the
  canonical query meaning that must remain equal across a comparison.
- [Lineage And Correspondence](./lineage-and-correspondence.md) explains entity
  continuity when identity changes across history.
- [Projection Consumption](./projection-consumption.md) transfers facts from a
  historical result without reopening its basis authority.
- [Scopes, Templates, Saved Queries, And View Shapes](../authoring/scopes-templates-saved-queries-and-view-shapes.md)
  describes reusable query authoring before basis admission.

## Inspection And Debugging

Inspect the typed artifacts rather than reconstructing posture from labels:

- `ScopedQueryBasisContext` reports family, authority family, cost, budget,
  drift, and historical admission posture.
- `SnapshotResolutionReport` explains snapshot resolution.
- `HistoricalEvaluationAdmission` explains whether the requested path was
  admitted.
- `HistoricalMaterializationPathMetadata` reports retained-state, replay, or
  reconstruction posture and predicted work.
- `QueryDiffChangeSetArtifact` reports comparison family and row-level change
  classification.

If admission stops early, check basis substitution, support posture, replay or
reconstruction limits, and whether both sides preserve the same query digest.

## Anti-Patterns

- Treating a basis as a timestamp or free-form snapshot label.
- Building a query context from independently assembled identifiers or
  reporting digests.
- Comparing contexts produced from different canonical queries.
- Asking historical APIs for raw storage deltas.
- Rebuilding retained artifact packs with consumer-owned loops when a typed
  materialization or projection-consumption surface already owns the result.
- Assuming durable reload or store-backed reconstruction because retained
  runtime history is supported.

## Current Limits

- Runtime-backed current, branch, preview-derived, and retained-history paths
  are the strongest supported surfaces.
- Store-backed retained-history parity exists only for admitted rows in the
  support matrix.
- Store-backed replay, reconstruction, and durable reload remain deferred
  where the active profile does not admit them.
- Cross-basis substitution and cross-query comparison fail closed.
- Policy, tenant, relationship-proof, and schema-context changes can remask or
  deny a retained result before public projection.

## Related Docs

- [Basis Capability Lifecycle](./basis-capability-lifecycle.md)
- [Schema Validation](../modeling/schema-validation.md)
- [Lineage And Correspondence](./lineage-and-correspondence.md)
- [Projection Consumption](./projection-consumption.md)
- [Support Matrix And Admission](../foundations/support-matrix-and-admission.md)
