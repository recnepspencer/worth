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

## Retain an initial-empty owner during composite close

Platform composition can consume an unchanged, genuinely sealed initial-empty
`SignalRuntime` with `try_prepare_initial_empty_stop(original_request)`. Busy,
stop, poison or an unsupported graph returns the same runtime in
`SignalInitialEmptyStopRefused`; `into_runtime()` restores the owning value.
Preparation checks the actual private owner state through nonwaiting gates and
stops new admission. It retains genuine sealed definition/service custody.

`PreparedInitialEmptySignalStop::resume()` reopens that same owner before the
composite close commits. `commit(original_request)` closes retention acquisition
and returns `StoppedInitialEmptySignalAllocation`; Busy or stop returns the same
prepared owner for retry. Neither step runs Signal cleanup or destroys the root.
Keep the stopped allocation until explicit, **unbudgeted** `reclaim()`. Generic
Drop is also unbudgeted abandonment. The stopped posture remains Closing, not
Signal cleanup Closed.

This is the narrow initial-empty composition primitive. It does not stop an
arbitrary authored graph, retire Bridge/Native capabilities or close a Home.
See the [governing owner contract](../../plans/database-foundation/milestone-3-home-lifecycle.md#initial-empty-signal-stop-and-composite-close-preparation).

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

## Committed Observation Tiers

`ObservationPolicy::visited()` observes nodes considered during committed
transaction processing. A visit does not imply recomputation or a changed
value. `recomputed()` observes evaluation, and `meaningful_change()` observes
the committed change selected by the node's comparison contract. Use the
semantic commit delta when deciding what changed.

Policy serialization carries schema identity
`worth.signal.observation-policy.v2` and the `Visited` trigger. The decoder
accepts only that schema and spelling: unversioned or v1 policies, the
`Touched` trigger, and unsupported versions are rejected.

Committed observation summaries expose `visited`, and scratch summaries expose
`visited_event_count`; there is no `touched` spelling. These observation names
do not rename native touched records or commit deltas.

## Where It Fits

- web backends and reactive views
- finance and risk pipelines
- ML feature and scoring flows
- document indexing or compiler-style partial recompute

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

`SignalConditionalDecisionEvidence::computed_evaluation()` borrows the actual
`AppliedEffectReport` when Signal computed an output. Its verdict preserves the
original `SuppressionReason` and comparison, available through
`facade::diagnostics`. Dependency reuse, condition suppression and deferral return
`None`. `output_version_before()` and `output_version()` retain the actual
observed output basis. These are descriptions of the installed node's evaluation;
they identify no application producer beyond the node's real installed contract.

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

The attached-Home Relational-only lane records `RecordedSignalInitialEmptyRoot`.
Only the real partition-to-owner transfer can establish its private initial
witness: one untouched original root, no authored nodes, installed definition,
fork, snapshot, retirement, restore or merge history. Allocation irreversibly
invalidates graph-origin eligibility; clone and reconstruction never inherit it.
The recipe carries the actual full schema commitment and versioned Empty
visibility/definition meaning. `SignalBranchBasisPort` exports it from the exact
current basis and readmits it only through a freshly composed qualifying owner,
acquiring genuine basis retention. Decoding the recipe or matching a folded
definition basis grants no admission. General authored graph durability remains
outside this narrow seed contract.

### Authored graph image

`SignalGraph::capture_authored_image` emits the declaration section through
`facade::durability::SignalAuthoredImage`. Capture requires an explicit
Home-scoped branch. It records node-slot generations and allocation order, node
configuration, full conditional definitions and
declared static dependency edges. Evaluation-discovered edges remain separate
from those declarations. Values, aspect and dirty versions, dependency snapshots,
pending causes, traces, telemetry, resources and temporal runtime state are absent.
Local conditional binding counters are also absent; fresh installation remints
them. Authored literal thresholds remain part of their definitions.

`facade::runtime::RuntimeConfig::capture_authored_image` records runtime
registrations in a separate `WSIGCONF` frame. Tier keys use explicitly declared
stable names, and keyed computations use semantic family/key text. Interner
ordinals, memo-only keys and cached results are absent. Readmission resolves
the tier names through the fresh program's vocabulary.

The current-format-only binary frame uses `WSIGAUTH`, protocol version 1, the
aspect storage profile, a payload length and SHA-256 integrity. Integer fields
are little-endian; strings are length-prefixed UTF-8. Unknown versions, mismatched
profiles, invalid fields, changed checksums and incorrect lengths refuse with
typed causes. Capture and byte admission use the original execution request's
owned byte backing. The maximum frame size is 64 MiB.

`SignalAuthoredImage::readmit_bytes` admits descriptive bytes only. It grants no
branch basis, lowering capability or publication permission. Installed condition
and comparator roles carry no graph-instance identity. Their operational bindings
must be reconstructed by owner admission with the installed program and Bridge's
definition section. This graph section alone does not enable authored Home reopen;
the attached-Home seed contract above still applies.

`SignalGraph::readmit_authored_image` consumes the fresh installation graph and
returns a cold graph plus freshly installed conditional contracts. It reconstructs
declared topology once; it does not admit an owner or mint a branch basis.
The owner must still readmit its ordered journals and Bridge section before
serving recovered branches.

### Standalone authored owner readmission

`Box<SignalRuntime>::readmit_authored_owner` consumes the fresh program runtime,
the `SignalDurableMedium`, stable tier vocabulary and recorded identity bounds.
It reads the protected journal cut, validates ordered source references, restores
live branches and snapshots, and installs one cold canonical owner partition.
Every definition body contributes node-incarnation bounds, including unselected
bodies and vacant or retired slots. Home composition must also supply bounds
from its original World opens and intents; standalone owners supply an explicit
empty contribution.

The returned `SignalDurableOwner` retains the runtime. Its weak source port
readmits descriptors against actual branch cells and the current Home incarnation.
Foreign owners, unknown frame versions and stale live handles have typed refusals.
`take_installation_contracts` transfers freshly installed conditional contracts
paired with their durable occurrences for Bridge installation.

Each medium exposes one stable, funded `SignalDurableOwnerAdmissionRegistry`.
Readmission acquires its exclusive claim before journal replay and transfers the
non-cloneable custody into the actual canonical owner allocation. A competing
readmission or retirement reports `OwnerAlreadyReadmitted` with the holding
runtime's identity. Failed readmission releases its claim; retirement permanently
stops new admission. Store attachment consumes the original boxed, unshared
physical runtime, preventing a second adapter from inventing another registry
while that serving share is retained. Attachment refusal retains the same input
for `try_attach`.

The standalone mutation port stages an ordinary advance with the canonical
transaction engine exactly once. Its original task retains the candidate and
result across pending media work without retaining the invoking request. Exact
original definition and branch completions precede canonical movement. A demand
that only recomputes derived values reuses its original definition reference.
Committed, proven-no-effect and indeterminate outcomes remain distinct.

Store-backed courts cover three branches, runtime-published keyed definitions,
conditional source, graceful reopen and quiescent process termination. Values
recompute on first demand after both same-build and changed-build readmission.
This entry does not yet enable authored Home bootstrap, durable fork or mixed
World publication. Dedicated conditional-definition publication and Bridge's
section interpretation remain subsequent work; ordinary advances refuse changes
to conditional definitions.

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
let serial_request = worth_execution::SerialRequest::from_memory(
    worth_execution::SerialMemoryBudget::new(runtime.runtime_policy().serial_memory_bytes),
    worth_execution::CancellationToken::new(),
    None,
);
let execution = worth_execution::ExecutionRequest::serial(&serial_request);
let _next_basis = runtime.advance_signal_branch(execution, &mut state, &basis, |tx| {
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

Checked graph evaluation admits diagnostic structure and callback-result custody
before dispatch. `CheckedEvaluationContext::result_capacity_bytes()` reports the
admitted result bound; exceeding it is a typed callback-contract violation.
`NodeContract::with_max_checked_result_heap_bytes` declares a node's result heap
maximum. Output converters can also declare `maximum_result_heap_bytes()`;
`AspectVersion` supplies its exact zero-heap bound. Rich outputs without a type
bound receive an adaptive grant under finite memory authority. Results are still
measured after evaluation, and their original tickets shrink to the bytes kept
through publication.

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
- [`examples/document_partial_recompute.rs`](./examples/document_partial_recompute.rs) for region-scoped partial recompute

## Walkthroughs

- [Easy task board](./docs/walkthroughs/easy-task-board.md)
- [Compiler targeted rebuild](./docs/walkthroughs/compiler-targeted-rebuild.md)
- [Document partial recompute](./docs/walkthroughs/document-partial-recompute.md)

## Reality check

If you are just getting started, stay in:

- `SignalGraph`
- `SignalRuntime`
- `runtime.observe_signal_branch_basis(...)`
- `runtime.advance_signal_branch(execution, ...)`
- `runtime.diagnostics()`

Or start in `easy` and move out only when you need more room.

For Query-installed operations, start instead at the
[`Conditional Installed Operations`](../../workspaces/worth-query/crates/worth-query/docs/domain-capabilities/conditional-installed-operations.md)
guide. The `easy` facade is for standalone Signal applications, not a shortcut
around Query installation.
