# Invariants and Read Observations

## What This Feature Is

An installed application invariant validates a proposed graph change at its
configured execution point. It prepares a bounded read scope, then evaluates
that scope using typed fields and relations. Relational owns the candidate,
access checks, interruption and publication decision; Query supplies the
application's typed bindings and rule callbacks.

`WorthQueryInvariantProjectionWork` describes actual Query boundary reads. The
same public type appears on installed invariant contexts, completed snapshot or
operation projections, and declared-query receipts. It grants no access and is
not a resource allowance.

## Why You Use It

Use an invariant when a domain condition must be checked on the complete
proposed change, including changes made by other declared operations. Read
observations help tests compare equivalent reads and check whether unrelated
parts of a graph increase the work of a local rule.

For an ordinary application query, start with `request.query(intent).execute()`.
For a handler's tracked decision inputs, use its `DecisionReader`. Installing a
rule or retaining projection authority does not replace query authorization.

## Stable Entry Points

Import declarations through `worth_query_decl::facade::application_schema` and
installed rule APIs through `worth_query_host::facade::application_invariants`.

| Task | Public surface |
| --- | --- |
| Declare the rule and its access | `ApplicationInvariantDefinition`, typed field/relation declarations and the program's `ApplicationSharedRuleRef` |
| Bind the installed schema once | `setup.invariant(marker, execution_point, resolver_callback)`; `resolver.typed_field(..)` and `resolver.typed_relation(..)` |
| Prepare the read scope | `WorthQueryApplicationInvariantRule::prepare_scope` or its checked counterpart; `planner.proposed()`, `planner.committed()`, `planner.reads()` |
| Evaluate that scope | `evaluate` or `evaluate_checked`; `context.proposed()`, `context.committed()`, `context.reads()` |
| Read through a typed view | `touched_entities`, `field`, `relations_from`, `relations_to`; entity and relation handles retain their native identity and view affinity |
| Inspect a normal query result | `rows.receipt().inspect().read_work().reads()`; the lower host query receipt also exposes `receipt.reads()` |
| Inspect a snapshot projection | `WorthQueryCompletedInvariantProjection::work()` or `into_parts()` |

The host's `primary_graph` facade also exposes
`WorthQueryApplicationInvariantProjectionReader` and its installation-owned
projection authority. Its main methods are `version`, `resolve_entity`,
`resolve_optional_entity`, `field`, `relations_from`, `relations_to`, and
`summarize_exclusive_incoming`. The latter uses the installed aggregate owner,
including its cache and rebuild evidence. This reader observes a retained
published snapshot; it is distinct from an installed rule's proposed-state view.

## Core Mental Model

Preparation discovers and admits the entity and relation scope that evaluation
may use. Field access is governed by the declared bindings and access contract
on admitted entities; preparation does not have to read every field that
evaluation will read. Evaluation cannot expand its prepared entity/relation
scope. Declare every field and relation the rule needs; resolving a binding
does not make undeclared access legal.

`proposed()` sees the candidate being validated. `committed()` sees the committed
side of that validation. Handles carry schema binding, proposal and view
posture: a handle obtained from one view cannot be reinterpreted as a handle for
the other. Obtain the needed entities through the appropriate view.

Each planner owns one observation value shared by all its views. Each evaluation
starts from the immutable preparation observation and adds that evaluation's
reads. Creating another view does not reset counts. A repeated evaluation does
not inherit a previous evaluation's reads.

Snapshot projection authority is retained by the installation/composition owner
while it owns the native bootstrap. Ordinary callers cannot construct it from
runtime identities. `authority.project(..)` reads its graph's main published
branch; admitted-operation projections use their admission's basis. The
completed projection owns its output, snapshot and observations. Keep the
snapshot when further reads must use the same version; release it when finished.
An identity from another projection authority is not a local read grant.

## How It Executes

1. Declare the invariant's operational contract and include the installed rule
   at the correct point in the application's program.
2. Configure its typed implementation with `setup.invariant`. Resolve bindings
   from the supplied schema resolver and retain those bindings in the rule.
3. Preparation uses the planner's views to discover its affected scope. Return
   that scope and propagate access, capacity or interruption errors.
4. Evaluation reads the prepared scope and returns `Pass`, a typed `Violation`,
   or an execution error. A blocking invariant refuses publication on violation
   or failure. Typed refusal data follows its declared binding.
5. Inspect `planner.reads()` or `context.reads()` at the callback boundary. A
   caller-owned diagnostic sink can retain those values for a test. It must
   not select the verdict, issue queries reentrantly or change graph state.

Checked callbacks receive the existing `CustomInvariantLeaseBudget`. Cooperate
with its checkpoints and retention claims; read observations neither replace
that budget nor establish a new budget. Cancellation and the original request
continue to control native reads.

## Small Example

The planar reference application's evaluation wrapper records the actual
context after evaluation returns. Its ordinary violation return is still a
completed verdict; an execution error is recorded with `returned_verdict: false`.
Preparation uses the same wrapper pattern and marks whether it returned a scope.
The observer is optional application configuration, with the same thread and
unwind-safety bounds as its installed rule.

<!-- compiled: planar_invariant/evaluation.rs -->
```rust
    pub(super) fn evaluate_with_budget(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, Schema>,
        scope: &<Self as WorthQueryApplicationInvariantRule<Schema>>::Scope,
        budget: Option<&dyn CustomInvariantLeaseBudget>,
    ) -> Result<
        WorthQueryApplicationInvariantVerdict<()>,
        WorthQueryApplicationInvariantExecutionError,
    > {
        let outcome = self.evaluate_inner(context, scope, budget);
        self.observe(PlanarInvariantReadEvent::Evaluated {
            reads: context.reads(),
            returned_verdict: outcome.is_ok(),
        });
        outcome
    }
```

The scope has already been prepared. `evaluate_inner` reads coordinates through
`context.proposed()` and typed field bindings. The `reads()` call observes those
reads; it performs no additional graph read. The full rule lives in
[topology_entry's planar invariant](../../../worth-query-certification/fixtures/consumer_entry/topology_entry/src/planar_invariant.rs).

## Real Example

Run the public reference application from its consumer workspace:

```text
cargo run -p worth-query-topology-entry --example invariant_reads
```

The example installs genuine topology declarations and a blocking planar rule,
clears seed-time observations, then edits the body named `anchor-a`. It verifies
that exactly that command's one directly touched body identifies the forward
read interval. Its ordinary `PlanarRead` query starts at the same immutable body
key and follows the same two successor edges.

The invariant also discovers predecessors and evaluates other affected triples.
Those reads are outside this matching interval, so comparing the query with the
invariant's whole cumulative observation would compare different operations.

<!-- compiled: invariant_reads.rs -->
```rust
fn compare_forward_reads(
    events: &[PlanarInvariantReadEvent],
    query: WorthQueryInvariantProjectionWork,
) -> (
    WorthQueryInvariantProjectionWork,
    WorthQueryInvariantProjectionWork,
) {
    let touched: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            PlanarInvariantReadEvent::Touched { entity } => Some(*entity),
            _ => None,
        })
        .collect();
    assert_eq!(touched.len(), 1, "this command edits one identified body");
    let forward: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            PlanarInvariantReadEvent::ForwardHops {
                root,
                before,
                after,
            } if *root == touched[0] => Some((*before, *after)),
            _ => None,
        })
        .collect();
    assert_eq!(forward.len(), 1, "compare the same directly touched root");
    let (before, after) = forward[0];
    assert!(query.adjacency_lists_read() > 0);
    assert_eq!(
        after.adjacency_lists_read() - before.adjacency_lists_read(),
        query.adjacency_lists_read()
    );
    assert_eq!(
        after.adjacency_edges_inspected() - before.adjacency_edges_inspected(),
        query.adjacency_edges_inspected()
    );
    let prepared: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            PlanarInvariantReadEvent::Prepared {
                reads,
                succeeded: true,
            } => Some(*reads),
            _ => None,
        })
        .collect();
    let evaluated: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            PlanarInvariantReadEvent::Evaluated {
                reads,
                returned_verdict: true,
            } => Some(*reads),
            _ => None,
        })
        .collect();
    assert_eq!(prepared.len(), 1);
    assert_eq!(evaluated.len(), 1);
    (prepared[0], evaluated[0])
}
```

The example separately compares complete preparation, evaluation and query
observations with zero, one and six unrelated rings. It makes no fixed numerical
read-cost promise. It also submits an invalid coordinate edit, requires the
actual `PositivePlanarTurn` domain violation, checks that the committed coordinate
is unchanged, and inspects the reads retained for that refused evaluation.
See the [complete runnable example](../../../worth-query-certification/fixtures/consumer_entry/topology_entry/examples/invariant_reads.rs).

## How It Relates To Other Features

A handler's `DecisionReader` retains the facts its decision depends on, so native
currentness checks can guard publication. The completed operation projection
reports the same observation type through `work()`. Managed computation reuse
retains its existing charged work contract, but adds no historical reads to the
new run. A fallback keeps reads already performed before restarting.

A declared-query receipt reports that query's reads independently of its
authorization work. One-shot results, continuation pages and live refreshes use
the same count names. Their projection record/field capacity and work evidence
remain separate from actual field acquisitions. A root predicate that compares
fields is a field read; it is not an index lookup merely because it selects rows.

## Inspection And Debugging

| Observation | Meaning |
| --- | --- |
| `equality_lookups()` | Actual equality lookup operations |
| `index_candidates_examined()` | Candidates/entries examined by the actual equality or ordered-index read |
| `adjacency_lists_read()` | Acquired directional adjacency lists |
| `adjacency_edges_inspected()` | Relation records inspected, including declared kinds later filtered from a result |
| `endpoint_records_read()` | Explicit entity/endpoint metadata observations, including acquired metadata subsequently rejected; internal record fetches solely to obtain a field do not add an endpoint observation |
| `field_reads()` | Acquired field observations, including lawful absence and values whose decoding then fails |
| `aggregate_lookups()`, `aggregate_cache_hits()`, `aggregate_rebuild_input_rows()` | Actual aggregate lookup, reuse and reconstruction observations |
| `output_lineage_source_selections()`, `output_lineage_role_lookups()` | Output-lineage observation operations |

Unavailable kinds stay zero. Counts saturate at `usize::MAX`; they do not admit
work. `provider_work_units()` is a diagnostic combination, not the projection's
private charge or a query's total work evidence. Compare the relevant count
kinds for equivalent operations; equal result lengths do not prove equal reads.

For installed rules, inspect `WorthQueryInvariantAccessDenial::kind()` and
`subject()`. Important distinctions include `OutsideDeclaredAccess`,
`OutsidePreparedScope`, `ForeignBinding`, `ForeignView`, `WrongEntityKind`,
`EntityUnavailable`, missing/invalid fields and native work refusal. Propagate
these failures rather than treating them as an absent value or a passing rule.

For retained projections, inspect the outer projection result as well as each
read. Resolution distinguishes absence from ambiguity; optional resolution only
turns unknown identity into `None`. Traversal retains its typed denial.
Projection completion can reject output after native interruption, decode
failure, work or retention refusal. A field's `None` alone does not prove the
whole projection succeeded. A denied projection's `projection_work()` may
retain observations made before that stop.

## Anti-Patterns

- Replacing the platform values with an application counter derived from loop
  iterations, returned rows or an admission formula.
- Asserting a fixed read count in production, branching on it, or using it to
  fund a read or authorize a commit.
- Comparing full observations for paths that intentionally perform different
  preparation, authorization or projection work.
- Carrying a proposed-view handle into a committed view, or widening the scope
  during evaluation after preparation admitted it.
- Treating a projection's retained snapshot or a receipt as permission for a
  later query; fresh requests still require their normal authorization.

## Current Limits

The installed read view exposes declared fields and one-hop relations. It does
not currently expose transitive closure, orbit/component identity or derived
relations as proposed-state reads. These counts describe Query boundary
operations, not storage-engine I/O, allocation, elapsed time or unique entities.

The runnable observer example uses serial requests and one explicit edit per
observation window. Concurrent applications must correlate their own diagnostic
windows; an event stream does not create invocation or publication authority.

## Related Docs

- [Ordinary application entry](ordinary-application-front-door.md)
- [Typed invariant refusal](../../../../../../docs/build-an-application.md#typed-refusal-from-installed-invariants)
- [Program adoption](programs-and-adoption.md)
- [Execution resources and managed runs](../domain-capabilities/execution-resource-admission-and-managed-runs.md)
