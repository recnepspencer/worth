# Conditional Installed Operations

## What This Feature Is

Conditional installed operations let a domain author eligibility, trigger,
comparison, maintenance, and output meaning as part of a portable Query
operation. Query installs that meaning into the runtime’s actual Bridge-owned
Signal graph and preserves Signal’s decision through execution and workflow
progression.

Use this for incremental geometry, derived values, guarded workflow stages,
threshold-sensitive recomputation, time-driven evaluation, and explicit
on-demand work.

## Why You Use It

- Author conditional nodes without exposing Signal node IDs or aspect slots in
  domain packages.
- Preserve exact Relational field, endpoint, structural, and lifecycle change
  meaning through invalidation.
- Keep condition, trigger, comparator, and artifact-reuse semantics in
  canonical operation identity.
- Ensure ineligible and suppressed work performs no compute.
- Ensure reverted-clean work records cost but produces no new Query
  consequence.

## Stable Entry Points

Portable authoring:

- `domain::WorthQueryPortableConditionalNodeDeclaration::declare(...)`
- `domain::WorthQuerySemanticTruthDependency::new(...)`
- `domain::WorthQueryConditionalEvaluationCondition`
- `domain::WorthQueryConditionalTrigger`
- `domain::WorthQueryComparatorRequirement`
- `domain::WorthQueryOutputEquivalenceRequirement`
- `domain::WorthQueryArtifactReuseEquivalence`
- `domain::WorthQueryMaintenancePosture`
- `domain::WorthQueryConditionalNodeOutput`

Runtime installation:

- `primary_graph::WorthQueryConditionalApplicationRuntimeInstallation`
- `primary_graph::WorthQueryTemporalOperationExecution`
- `primary_graph::WorthQueryTemporalReconstructionAccess`
- `primary_graph::WorthQueryConditionalClockHandle`
- `primary_graph::WorthQueryPrimaryGraphApplicationRuntime::reinstall_conditional_runtime(...)`
- `primary_graph::WorthQueryPrimaryGraphApplicationRuntime::reinstall_conditional_runtime_for_installation(...)`

Runtime evidence:

- `primary_graph::WorthQueryConditionalClockObservationOutcome`
- `primary_graph::WorthQueryConditionalClockObservationReceipt`
- `primary_graph::WorthQueryConditionalRuntimeReinstallationReceipt`
- `primary_graph::WorthQueryConditionalRuntimeInspection`

## Core Mental Model

A conditional node crosses four ownership boundaries:

```text
Query portable declaration
  -> Relational authoritative aspect change
  -> Runtime Bridge installed correspondence and lowering
  -> Signal condition decision and evaluation
  -> Query consequence or typed deferral
```

Each owner contributes a different fact:

- Query says what semantic truth the operation depends on and what the node is
  intended to do.
- Relational says exactly what authoritative truth changed.
- Runtime Bridge proves where that semantic dependency lives in this Signal
  graph and lowers portable condition meaning into the installed node.
- Signal decides eligibility, computation, reuse, and whether output changed
  meaningfully.
- Query admits the returned evidence into the bound operation or workflow. It
  does not restamp the decision.

## How It Executes

1. Query validates and canonicalizes the portable node inside the operation.
2. Runtime construction admits exact graph participation, correspondence,
   Signal targets, lowering, and volatile providers.
3. Binding retains the installed conditional set with the operation’s basis
   and graph authority.
4. Execution preflights Query authority, gathers semantic observations through
   Runtime Bridge, and asks Signal for the decision.
5. Query accepts only evidence bound to the exact operation, runtime, graph,
   node, snapshot, attempt, and capability.
6. Changed work continues; unchanged or deferred work cannot mint later
   consequences.

## Small Example

A semantic dependency uses stable aspect meaning:

```rust
let dependency = domain::WorthQuerySemanticTruthDependency::new(
    domain::WorthQueryConditionalGraphReadRole::new("model")?,
    aspect_contract,
    projection_mask,
    aspect_binding,
    domain::WorthQuerySemanticLocality::SourceRecord,
    [domain::AuthoritativeAspectChangeKind::FieldSet],
)?;
```

The dependency retains:

- Foundational aspect key, identity, revision, shape, and absence law
- exact field projection mask
- Relational entity, relation, endpoint, structural, or lifecycle binding
- source-record, partition, or logical-graph locality
- relevant authoritative change kinds
- the installed operation graph-read role

The dependency does not contain a Signal node, partition token, numeric aspect
slot, callback, or runtime identity.

The operation’s graph-read contract must already authorize the dependency’s
semantic projection. Conditional declarations can narrow graph authority; they
cannot create or widen it.

## Real Example

The builder requires every executable semantic dimension:

```rust
let node = domain::WorthQueryPortableConditionalNodeDeclaration::declare(
    "rebuild-face-mesh",
    domain::WorthQueryConditionalNodeRole::Computed,
)
.dependencies([dependency.clone()])
.outputs([domain::WorthQueryConditionalNodeOutput::OperationOutput {
    projection_role: domain::WorthQueryOperationProjectionRole::new("mesh")?,
}])
.required_context([domain::WorthQueryConditionalNodeContext::Snapshot])
.evaluation(
    domain::WorthQueryConditionalEvaluationCondition::aspect_filtered([
        dependency,
    ])?,
    domain::WorthQueryConditionalTrigger::DependencyChange,
)
.comparison(
    domain::WorthQueryComparatorRequirement::FoundationalContractEquivalence,
    domain::WorthQueryOutputEquivalenceRequirement::ExactCanonicalValue,
)
.artifact_policy(
    domain::WorthQueryArtifactReuseEquivalence::DependencyAndOutputEquivalent,
    domain::WorthQueryMaintenancePosture::LazyUntilObserved,
    domain::WorthQueryArtifactPosture::ReusableWhenEquivalent,
)
.output_relationship(
    domain::WorthQueryOutputRelationship::ContributesToOperationOutput,
)
.finish()?;
```

The declaration is canonicalized. Dependency and output order do not create a
new identity. Omitting a semantic dimension is an authoring error; the builder
does not silently choose `Always`, exact comparison, eager evaluation, or an
artifact policy.

## Condition Families

### Aspect-filtered

Use `aspect_filtered(...)` when eligible work depends on a declared semantic
aspect slice changing.

### Delta threshold

Use `delta_threshold(...)` with `WorthQueryDeltaThreshold::new::<Unit>(...)`.
The unit is a marker implementing `WorthQueryQuantityUnit`, and its portable
identity and value family participate in canonical meaning.

```rust
struct Millimeters;

impl domain::WorthQueryQuantityUnit for Millimeters {
    const PORTABLE_IDENTITY: &'static str = "geometry.units.millimeters";
    const VALUE_FAMILY: domain::WorthQueryQuantityValueFamily =
        domain::WorthQueryQuantityValueFamily::Float64;
}
```

Do not compare thresholds in Query or Bridge callbacks. Runtime Bridge lowers
the typed threshold; Signal resolves the decision.

### Temporal

Use `WorthQueryConditionalEvaluationCondition::temporal(...)` with a matching
`WorthQueryConditionalTrigger::Temporal(...)`. The host binds a named clock,
an authoritative intent-reconstruction query, a typed intent projector, and
the ordinary installed application operation before publication. Clock input
is only time evidence. Signal owns wake eligibility; Query still performs a
fresh application-operation admission and compare-and-commit.

The durable source is the domain temporal-intent record, not the Signal wake.
An active intent carries stable identity, revision, due coordinate, operation
input, lifecycle, and idempotency relation. The same application commit that
performs the effect advances the intent revision and lifecycle. Cancellation,
completion, or a successor revision is reconciled before predicate or
operation contact.

Host predicates receive dependency-indexed previous/current observations.
Absence is explicit, and a present value exposes only the declared projection
mask through `scalar()` or `field(...)`; the complete source aspect is not a
host-visible escape hatch.

#### Temporal identity and canonical work

The host does not invent a binding identifier or idempotency hash. Query first
prepares a canonical binding identity from the installed node authority,
clock, source, timeline, reconstruction query and projector, principal source,
and operation invoker. Publication then prepares the runtime-qualified identity
for the exact runtime, installation generation, provider, and branch. Both use
the Foundational canonical-basis and typed-digest contract.

That work occurs once at the cold boundary and is carried by the installed
runtime. `WorthQueryConditionalClockHandle::binding_canonical_work()` reports
the base binding work. `WorthQueryConditionalRuntimeInspection::installation_canonical_work()`
reports the combined binding and runtime-qualification work.

When an eligible wake reaches fresh application admission, Query prepares the
idempotency key and intent identity from the runtime binding plus the current
authoritative intent identity, revision, input, and host idempotency value.
`WorthQueryConditionalExecutionProvenance::canonical_work()` reports that work
in the admission phase. Compare-and-commit consumes the prepared binding;
no later phase of that attempt hashes the same meaning again. A subsequent
lawful fresh admission reports its own derivation in the admission phase, not
in provider execution, projection, live delivery, retry, recovery, or
publication.

### On-demand

Define a marker implementing `WorthQueryOnDemandTriggerFamily`, then use the
same typed family through `WorthQueryConditionalTrigger::on_demand::<Family>()`.
The runtime provider answers whether that exact trigger was requested.

### Domain-specific

Define a marker implementing `WorthQueryDomainConditionFamily` and supply
portable typed parameters:

```rust
struct TopologyReady;

impl domain::WorthQueryDomainConditionFamily for TopologyReady {
    const PORTABLE_IDENTITY: &'static str = "geometry.conditions.topology-ready";
}

let condition = domain::WorthQueryConditionalEvaluationCondition::domain_specific::<
    TopologyReady,
>([
    domain::WorthQueryPortableConditionParameter::u64("minimum-shells", 1)?,
])?;
```

The family marker is portable identity. A string-dispatch callback is not.

## Publish A Primary-Graph Conditional Runtime

Portable declarations must be paired with exact runtime installations:

Application hosts enter only through `worth-query-host`. They bind the
installed conditional operation, typed node, host predicate, named clock,
reconstruction projection, ordinary operation invoker, and fresh admission
source before `publish()` makes the application runtime visible. Runtime
Bridge and Signal types never appear in the host manifest or source.

```rust
let mut publication = graph
    .conditional_application_runtime_installation(
        runtime,
        authority,
        schema,
        conditional_evaluation_budget,
    )?;
let clock = publication.bind_temporal_operation(
    installed_temporal_binding,
    operation_execution,
    reconstruction_access,
)?;
let mut application = publication.publish()?;

let outcome = application.conditional_clock(&clock)?.observe();
```

Publication reconstructs current authoritative intents and reconciles the
derived wake index before returning. The observation port verifies the exact
runtime, binding, clock source, timeline, sequence, and installation affinity.

Runtime construction rejects:

- missing or extra conditional registrations
- a different marker tuple with the same labels
- a foreign graph or Signal node
- dependency contract, mask, binding, locality, or change-kind drift
- unsupported or ambiguous correspondence
- mixed or foreign lower-runtime ownership
- missing condition, trigger, wake, or comparator provider
- provider identity that does not match the portable declaration
- managed clock or wake capacity exhaustion

The registration is volatile. It does not participate in portable package
serialization; its admitted identity is retained by the installed runtime.

## Semantic Correspondence

Runtime Bridge maps one Query semantic dependency to one or more installed
Signal aspect targets. Successful correspondence is either:

- `Exact`
- `DeclaredWidening`

Widening is never implicit. A field-level change cannot silently become a
whole-aspect invalidation. Ambiguous, unsupported, stale, rebind-required, and
failed candidates do not mint an installed correspondence witness.

Many-to-one target allocation is admitted only when precision and ownership
remain explicit. Equal numeric Signal aspects in different graphs or nodes are
unrelated.

## Execute And Observe The Decision

Execution uses the same bound operation journey:

```rust
let world = workspace.observe_operating_world()?;
let bound = world
    .family(GeometryFamily)
    .bind(&installed_domain, RebuildFaceMesh)?;

let executed = bound.execute(input, &mut workspace).unwrap();

for decision in executed.conditional_provenance() {
    inspect(decision.class(), decision.signal_identity());
}
```

The example unwraps the successful execution only for brevity. Preserve every
`TransitionOutcome` category in production code.

Query performs authority preflight before contacting Runtime Bridge or Signal.
The execution attempt identity includes the exact bound capability, snapshot,
and attempt number.

The outcome classes preserve operational meaning:

- eligible and changed: Query may continue to graph work, executor work, and
  declared consequences
- dependency-unchanged, suppressed, or deferred: compute contact count is zero
  and Query returns typed deferral where required
- computed reverted-clean: compute cost is retained, semantic-change count is
  zero, and no new publication or effect is invented
- failure: owner-specific failure counters and evidence survive Query re-entry

Query consequences consume Signal-minted evidence. Query does not call the
condition provider again and does not infer change from output presence.

## Workflow Stages

Conditional nodes may be attached to an operation or to a named installed
workflow stage through `WorthQueryConditionalNodeLocation`.

A stage decision retains the exact workflow run, stage, predecessor frontier,
basis, snapshot, graph authority, lowering, and Signal decision. Unchanged,
deferred, and reverted-clean stages cannot be advanced by manufacturing a
stage receipt.

## Inspection And Debugging

Inspect:

- portable node identity and canonical operation identity
- dependency contract revision, mask, binding, locality, and change kinds
- installed correspondence precision and target count
- lowering identity and graph instance
- Signal decision class and counters
- Query `conditional_provenance()`
- `conditional_compute_contacts`
- `conditional_semantic_changes`
- graph-provider and executor contacts
- installed binding and managed-clock counts
- retained due wakes and reconstructed active intents
- committed, already-committed, failed, and indeterminate re-entry counts
- relevant authoritative-commit count/work-remaining separately from due-wake
  count/work-remaining
- clock-receipt `execution_provenance()` joining intent revision, wake
  ordinals, Signal decision, application-attempt presence, and terminal posture
- clock-handle `binding_canonical_work()`, runtime-inspection
  `installation_canonical_work()`, and provenance `canonical_work()` when
  auditing the cold-binding, runtime-binding, and fresh-admission seams
- `inspect_conditional_runtime()` before and after lifecycle transitions
- `reinstall_conditional_runtime(product_branch)` receipts with separate reconstructed
  binding/intent counts and structural query work: examined candidates,
  projected records, projected fields, and total work units
- `conditional_runtime_lifecycle_probe()` retained outside the application
  runtime when abandonment/`Drop` release needs exact-zero proof; its
  `live_inventory()` reads weak liveness for the concrete Query, Bridge, and
  Signal resource owners and is never written by a Drop callback
- `workspace.rebuild_conditional_execution_index()`
- `bridge.rebuild_correspondence_allocation_index()`

Rebuild reports must show exact parity. Index identity is diagnostic evidence,
not operational authority.

## How It Relates To Other Features

- [Runtime-Installed Domains And Operations](./runtime-installed-domains.md)
  owns the package, provider, binding, execution, and publication journey.
- [Aspects And Authority Lanes](../modeling/aspects-and-authority-lanes.md)
  defines the stable truth meaning carried by dependencies.
- Runtime Bridge owns installed semantic correspondence and condition
  lowering; Signal owns the runtime decision.
- Query effect conditions are a separate downstream-effect contract. They do
  not substitute for node evaluation conditions.

## Anti-Patterns

- Authoring a raw Signal `Aspect`, node ID, or mask in a domain package.
- Treating a mapping stable name as semantic identity.
- Implementing condition selection with string matching.
- Re-evaluating eligibility in Query or in a domain executor.
- Inferring “changed” because compute returned an output.
- Widening field or endpoint changes without a declared precision posture.
- Creating a second Signal graph for conditional Query work.
- Registering a provider without a matching portable declaration.
- Treating conditional nodes as permission to widen graph reads or effects.
- Defining a host-local hash grammar for temporal binding or idempotency
  identity, or regenerating canonical identity during commit.

## Current Limits

- The production primary-graph host path installs every conditional binding
  before runtime publication; post-publication provider or clock replacement
  is not an ordinary mutation.
- One application runtime owns one Bridge-owned Signal runtime. Compatible
  reinstallation rebuilds that volatile owner from current authoritative
  Relational truth and the retained exact binding inventory. A successor
  installation requires fresh typed rebinding and otherwise returns
  `RebindRequired` without mutating the incumbent runtime.
- Application commit publication synchronously refreshes the derived
  temporal-intent index. Ordinary clock observation then routes a bounded
  route-local journal for each exact reconstructed source record, plus a
  separate whole-graph route only for a dependency explicitly declared
  `WholeLogicalGraph`. Unrelated commits consume neither exact-route retention
  nor scan work. Ordinary observation does not execute the reconstruction
  query; cold reconstruction remains separately bounded and reported by the
  installed temporal projection.
- `SourcePartition` dependencies are rejected at installation until the
  primary-graph host path has a typed installed source-partition role binding;
  they are never silently widened to whole-graph observation.
- Missing records and aspects are carried as authoritative snapshot absence.
  Field clears remain field-precise within a present aspect and are not treated
  as absence of the entire dependency. Consumers must match the explicit
  present/absent posture; the former present-only snapshot accessors were
  removed instead of preserving a panic-prone compatibility path.
- `close_conditional_runtime()` revokes clock handles and releases the
  installed provider, binding, managed-clock, wake, operation-attempt, lease,
  reconstructed-intent, scheduler-task, and scheduler-queue inventory.
- Provider replacement is a fresh runtime publication with a fresh typed
  provider identity. The predecessor clock/provider affinity is foreign to the
  replacement runtime; there is no mutable in-place provider swap.
- Installed-operation certification replay compares the exact realized
  conditional observations, Signal evidence, and decision path. Ordinary
  shared-owner delivery retains the current admitted Signal decision in each
  lease-bound invalidation delta. A suppressed, dependency-unchanged,
  reverted-clean, or deferred decision cannot be promoted into a computed
  patch. Query-shaped patch payloads remain a separate later capability.

## Related Docs

- [Ordinary Application Front Door](../foundations/ordinary-application-front-door.md)
- [Runtime-Installed Domains And Operations](./runtime-installed-domains.md)
- [Aspects And Authority Lanes](../modeling/aspects-and-authority-lanes.md)
- [Downstream Runtime Integration](../foundations/downstream-runtime-integration.md)
- [Support Matrix And Admission](../foundations/support-matrix-and-admission.md)
- [Projection Consumption](../capabilities/projection-consumption.md)
- [Installed Operation Re-Execution And Replay](./installed-operation-reexecution-and-replay.md)
- [Bound Projection Lifecycle, Sharing, And Consumer Invalidation](./bound-projection-sharing-and-invalidation.md)
