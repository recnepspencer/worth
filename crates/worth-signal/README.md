# worth-signal

`worth-signal` is a deterministic incremental runtime for derived work.

Your app owns the real state.
`worth-signal` owns:

- dependency tracking
- invalidation
- recompute
- rollback
- diagnostics
- replay and history
- installed condition decisions and semantic-change classification

This crate is not just trying to rerun less work.
It is trying to keep updates, transactional truth, explanation, and history in
one system.

There are two normal entry paths:

```rust
use worth_signal::easy::*;
use worth_signal::facade::*;
```

Use `easy` for the shortest path.
Use `facade` when you want the broader runtime surface from the start.

## What Makes It Different

The important line is this:

- not "reactive graph plus some debug helpers"
- not "incremental cache plus a separate audit layer"
- not "rerun less work and figure out the rest later"

Worth Signal keeps change propagation, transactions, diagnostics, and history
in the same runtime.

That means:

- updates should land as one unit
- rollback should leave the runtime in a sane state
- diagnostics should explain why work happened
- replay and history should keep the trail

## Fast Mental Model

Most days, the shape is:

- build a `SignalGraph`
- build a `SignalRuntime`
- mark changes in a transaction
- read the derived node you care about
- use diagnostics when something smells off

If you start in `easy`, that is still the same system.
You are not signing up for a toy path you need to throw away later.

When work must name an exact branch rather than whatever is current later,
observe an `AdmittedSignalBranchBasis`. Governed fork, restore, advance, and
retention operations consume that owner-issued basis. A serialized
`SignalBranchBasisDescriptor` is intentionally non-authoritative until the
owning runtime readmits it. See [Signal Branch Bases](./BRANCH_BASES.md).
Its [Owner Component Port](./BRANCH_BASES.md#owner-component-port) is the only
branch/basis surface available to later composite-owner integration; legacy
head tuples and graph internals are not production facade contracts.

## Change And Commit Semantics

`mark_changed` records source recompute intent. It says which producer-local
aspect may need to be recomputed; it does **not** assert that the producer has
already emitted a semantic change. The returned `ChangeBatchAdmission` proves
that Signal admitted that root work. Older `ChangeBatchCommit` and
`SemanticBatchCommit` names are deprecated aliases for the same admission and
must not be interpreted as output-commit evidence.

After evaluation, Signal compares the candidate output with the producer's
last committed output. Only a meaningful committed difference creates a
producer delta and downstream dependency causes. Configure the two comparison
roles separately:

- `.output_equivalence(...)` is producer-side: did this evaluation publish a
  meaningfully different output?
- `.dependency_comparator(...)` is consumer-side: does the committed upstream
  difference matter to this consumer?

Nodes that can change more than one aspect should declare
`.produces_aspects(...)`. Use
`NodeEvaluationResult::with_changed_aspect_region(aspect, region)` when a
partition or detail applies to one particular output aspect. The legacy
`.with_changed_region(...)` form is conservative when several aspects change:
its scope is treated as a union, never as exact cross-aspect locality.

Aspects are producer-local. If source aspect `A` makes a middle node publish
aspect `B`, a leaf that depends on the middle node receives a cause for `B`.
Signal does not copy `A` through the graph. Conditions are evaluated only after
the leaf's immediate dependency causes have settled, so an
`aspect_filter(B)` sees `B`, not the original root aspect. The executable
counterpart is
`aspect_filter_uses_the_immediate_producers_translated_aspect` in the node
condition test suite.

## Locality And Execution Evidence

Exact producer changes use a graph-owned reverse-subscription index keyed by
the immediate producer and its local aspect, with partition/detail membership
kept in separate buckets. The index is derived from authoritative dependency
topology: it narrows candidates, but every returned candidate still passes the
live edge, snapshot, revision, contract, and cause checks before Signal admits
work. A downstream hop begins only after that downstream node performs its own
output commit; Signal does not pre-mark or walk a transitive subscriber
closure.

Invalidation work progresses through distinct source, dependency-commit, or
structural origins and then through resolved, lowered, ready, and executed
forms. Ready work is process-local and is rebuilt after restore. Callers that
need measured execution evidence can bracket real work with
`begin_invalidation_execution_observation` and
`finish_invalidation_execution_observation`, or use
`observe_invalidation_execution`. The resulting
`SignalInvalidationExecutionReceipt` contains realized counters from performed
execution; planning estimates and diagnostic summaries cannot substitute for
it.

## Where It Fits

- web backends and reactive views
- finance and risk pipelines
- ML feature and scoring flows
- geometry or compiler-style partial recompute

## Installed Conditional Nodes

Signal also provides the execution authority used by Query-installed
conditional operations. In that path:

- Query authors portable semantic dependencies, condition families, triggers,
  comparison posture, and output relationships
- Runtime Bridge installs those declarations into the actual Signal graph,
  node, partition, and aspect slots
- Signal owns dependency versions, eligibility, suppression, compute, artifact
  reuse, and the decision about whether output changed meaningfully
- Query carries Signal-minted decision evidence into operation or workflow
  progression without restamping it

The core types are `InstalledSignalConditionalContract`,
`InstalledSignalConditionDecision`, `SignalConditionalDecisionEvidence`, and
`SignalConditionalExecutionRequest`.

Signal aspects are runtime-local slots. They are not Foundational semantic
aspect identities and must not be persisted or authored in portable Query
packages.

When Signal is hosted by Query, use the one graph owned through
`worth-runtime-bridge::BridgeOwnedSignalRuntime`. Do not create a second graph
for the same installed operations.

That sealed service boundary owns exact branch-basis admission, conditional
evaluation, component mutation, and lifecycle cleanup. Runtime Bridge supplies
installed semantic correspondence and returns Signal's exact performed
evidence; Query and Runtime World carry it without reopening graph access or
restamping component identity. A product branch may reuse the same Signal basis
while its Relational sibling advances independently.

## One Continuous Story

The flagship story looks like this:

- a source file changes
- a transaction lands the update
- only the right downstream targets rerun
- diagnostics explain why the bundle moved
- replay keeps the trail

That full version lives here:

- [Compiler targeted rebuild walkthrough](./docs/walkthroughs/compiler-targeted-rebuild.md)

## Small example

```rust
use worth_signal::facade::*;

const PRICE: Aspect = Aspect::new(0);
const TOTAL: Aspect = Aspect::new(1);

#[derive(Default)]
struct CheckoutState {
    price_version: u64,
    total_version: u64,
}

let mut graph = SignalGraph::new();
let price = graph
    .node()
    .produces_aspects(AspectMask::from_aspect(PRICE))
    .build();
let total = graph
    .node()
    .produces_aspects(AspectMask::from_aspect(TOTAL))
    .on_demand()
    .build();

graph.set_dependencies(total, [DependencyEdge::new(price, PRICE)])?;

let mut runtime = SignalRuntime::build_for::<CheckoutState>(graph);

let mut state = CheckoutState {
    price_version: 2,
    total_version: 5,
};

let evaluate = |view: &mut EvaluationContext<'_, CheckoutState>| {
    let result = if view.node() == price {
        view.finish(NodeEvaluationResult::from_version(
            AspectVersion::from_updates([(PRICE, view.domain().price_version)]),
        ))
    } else {
        let _upstream = view.read_aspect_version(price, PRICE)?;
        view.finish(NodeEvaluationResult::from_version(
            AspectVersion::from_updates([(TOTAL, view.domain().total_version)]),
        ))
    };
    Ok::<_, SignalError>(result)
};

let basis = runtime
    .observe_signal_branch_basis(runtime.current_branch())
    .expect("current branch should admit an owner basis");
let _next_basis = runtime.advance_signal_branch(&mut state, &basis, |tx| {
    tx.mark_changed(price, PRICE)?;
    tx.target(total).read(&evaluate)?;
    Ok(())
})
.expect("admitted branch advance should succeed")
.into_basis();

let version = runtime.target(total).read(&state, &evaluate)?;
assert_eq!(version.get(TOTAL), 5);
# Ok::<(), SignalError>(())
```

## Start here

- [Docs index](./docs/README.md)
- [Getting started](./docs/GETTING_STARTED.md)
- [API overview](./docs/API_OVERVIEW.md)
- [Compiler targeted rebuild walkthrough](./docs/walkthroughs/compiler-targeted-rebuild.md)
- [Running the runtime](./docs/guides/running-the-runtime.md)
- [Debugging and diagnostics](./docs/guides/debugging-and-diagnostics.md)
- [Exact branch bases and owner component port](./BRANCH_BASES.md)

## Examples

- [`examples/easy_task_board.rs`](./examples/easy_task_board.rs) for the short path
- [`examples/compiler_targeted_rebuild.rs`](./examples/compiler_targeted_rebuild.rs) for targeted rebuilds, diagnostics, and replay
- [`examples/geometry_partial_recompute.rs`](./examples/geometry_partial_recompute.rs) for region-aware invalidation

## Walkthroughs

- [Easy task board](./docs/walkthroughs/easy-task-board.md)
- [Compiler targeted rebuild](./docs/walkthroughs/compiler-targeted-rebuild.md)
- [Geometry partial recompute](./docs/walkthroughs/geometry-partial-recompute.md)

## Reality check

If you are just getting started, stay in:

- `SignalGraph`
- `SignalRuntime`
- `runtime.observe_signal_branch_basis(...)`
- `runtime.advance_signal_branch(...)`
- `runtime.diagnostics()`

Or start in `easy` and move out only when you need more room.

For Query-installed operations, start instead at the
[`Conditional Installed Operations`](../../workspaces/worth-query/crates/worth-query/docs/domain-capabilities/conditional-installed-operations.md)
guide. The `easy` facade is for standalone Signal applications, not a shortcut
around Query installation.
