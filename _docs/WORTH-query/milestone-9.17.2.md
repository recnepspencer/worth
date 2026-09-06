# Milestone 9.17.2: Composite Runtime-World History And Coordinated Publication

> **Status:** Implementation and scoped closure complete on 2026-09-05.
> The persistent Astra high gate approved Phases 5–7. Untouched dependency lint
> debt remains explicitly recorded under the Phase 7 closure evidence.
>
> **Product posture:** This milestone establishes the memory-resident Runtime
> World composition authority in the dedicated `worth-runtime-world` owner
> crate. It makes no persistence, restart, physical database, merge, or public
> Query workflow claim.
>
> **Entry gate:** [Milestone 9.17.1.2](./milestone-9.17.1.2.md) closed at
> accepted production revision `95c9aa7455`. Its Relational bundle and
> independently borrowable Signal services are frozen prerequisites for this
> milestone.
>
> **Boundary revalidation:** Re-audited on 2026-09-01 against integrated
> revision `a09f241c61`, including the concrete Relational/Signal owner ports,
> current Runtime Bridge correspondence surface, and Query's live composition
> root.

## Goal And Roadmap Placement

Consume the exact Relational and Signal owner contracts established by
Milestones 9.17.1, 9.17.1.1, and 9.17.1.2 and establish one composition owner
for product branches. A product branch is a mutable reference to one immutable
single-parent composite commit. Each composite commit binds:

- one exact owner-issued Relational basis;
- one exact owner-issued Signal basis;
- the exact installed Bridge correspondence basis under which the two
  components form one interpretable runtime world;
- one ordinary parent when the commit is not a root;
- one owner-issued commit occurrence identity; and
- the exact component evidence and publication-attempt provenance that
  produced it.

Relational and Signal continue to own component truth, branches, publication,
settlement, and retention. The existing `worth-runtime-bridge` crate continues
to own installed semantic correspondence and ordinary Bridge routing. The new
`worth-runtime-world` crate is the cross-domain composition owner above those
participants. Query consumes its facade in Milestone 9.17.3 but cannot mint a
composite commit, move a product reference, or repair an owner guarantee.

This placement resolves the existing legal dependency graph:

```text
worth-relational -----------\
worth-signal ----------------+--> worth-runtime-world --> Query audiences
worth-runtime-bridge --------/
worth-foundational ---------->
worth-proof ----------------->
```

`worth-relational` already depends on `worth-runtime-bridge` for presentation
contracts. Therefore `worth-runtime-bridge` must not acquire a reverse
dependency on Relational, and composite ownership must not be implemented in
`crates/worth-runtime-bridge/src/runtime_world/`.

## Central Claim

No product branch moves unless every operation-named component result is
valid for one exact expected product-head observation and the Runtime World
owner wins one atomic compare-and-publish transition. Unchanged components
remain at their exact carried bases. Readers observe the complete old or
complete new composite basis, never a mixed world.

Owner-local effects are not falsely called cross-owner atomic. If one owner
performs before a sibling owner denies, cancellation arrives, settlement is
deferred, or the product-head comparison loses, the product reference remains
unchanged and the Runtime World owner retains a bounded,
`ProductUnpublishedOwnerEffects` recovery record. That record says exactly
which owner effects occurred, which obligations remain, and which next actions
are legal. It is not a product commit and it is not rollback.

The claim is false if:

- component currentness or ambient `main`/`current`/`latest` defines the world;
- an equal-looking representation substitutes for owner admission;
- component results become product-current before CAS, losing movement is
  hidden, or stale attempts rebase;
- unchanged owners are contacted, unrelated branches share a global lock, or
  repeated bases amplify leases;
- managed state is unbounded or Query/adapters/projections mint authority; or
- this milestone adds persistence, physical runtime, or restart machinery.

## Current Boundary And Required Composition

The completed owners provide concrete Relational preparation/fork/publication/
settlement/basis/lifecycle services; exact admitted bases; typed movement;
pre-movement Relational settlement recovery; and exact, terminal retention.
Descriptors remain diagnostic only.

The accepted predecessor surface is now concrete. Runtime World consumes the
non-generic `RelationalOwnerServicePorts` and the generic
`SignalOwnerServicePorts<D, I, E, Ctx, T>` directly. Relational service issuance
is infallible from `&RelationalRuntime`; Signal service issuance is a fallible,
one-way seal from `&mut SignalRuntime<D, I, E, Ctx, T>` and preserves the
predecessor's explicit `Send + Sync + 'static` bounds. Runtime World is
therefore generic over the exact Signal contract. It may not erase those five
parameters, define a consumer trait, or specialize the composition crate to
Query's current unit-typed Signal runtime.

The public owner/builder/Signal-execution bounds are exactly the predecessor
bounds: `D: Copy + Ord + Debug + Send + Sync + 'static`,
`I: Copy + Ord + Send + Sync + 'static`, `E: Send + Sync + 'static`,
`Ctx: Send + Sync + 'static`, and
`T: Copy + Ord + Send + Sync + 'static`. Runtime World may not strengthen them
with `Clone`, `Default`, serialization, or an application trait merely to ease
storage.

The component contracts are intentionally asymmetric. Signal `fork_exact`
accepts the carried `AdmittedSignalBranchBasis` directly. Relational
`fork_branch` instead consumes an `AdmittedRelationalForkSourceBasis` freshly
issued by `observe_fork_source`. Runtime World must observe through the carried
basis's branch identity, compare every shared exact axis—runtime, branch,
reference observation, and truth version—between the returned fork descriptor
and the admitted source basis, and only then consume the fork token. A raw
branch id selects the owner cell; it never authorizes the fork.

Because Relational already depends on Runtime Bridge, this milestone resolves
the remaining Cargo cycle by composing above Bridge. Neither placement nor
generic carriage changes component authority.

### Query-owned Signal topology constraint

The current Query composition root stores a unit-typed `SignalRuntime` inside
`BridgeOwnedSignalRuntime`, whose ordinary Bridge methods still use
construction-only `graph()`/`graph_mut()`. Sealing it would invalidate those
paths. This milestone instead proves one standalone Signal owner with Bridge
meaning installed against the same graph before sealing.

Milestone 9.17.3 must refactor or replace the Query/Bridge root once before
cutover. A second graph, post-seal legacy access, or erased adapter is forbidden;
this is an integration dependency, not unfinished 9.17.2 authority.

## Ownership And Truth Lock

| Responsibility | Sole owner | Explicit non-owner |
| --- | --- | --- |
| Relational branches, bases, candidates, commits, publication, settlement, and retention | Relational | Runtime World, Bridge, Query |
| Signal branches, bases, definition compatibility, owner-local operations, and retention | Signal | Runtime World, Bridge, Query |
| Installed semantic correspondence and its immutable configuration basis | Runtime Bridge | Relational, Signal, Runtime World, Query |
| Exact product component binding, composite commits, product references, retained partial effects, and coordinated publication | Runtime World composition owner | component owners, Query, Store |
| Product workflow admission and public projections | Query in 9.17.3 | Runtime World, component owners |
| Portable branch, reference, and boundary vocabulary | Foundational | every operational owner |
| Concrete phase and authority carriage | Proof beneath owner-sealed types | generic caller markers |
| Persistence, restart recovery, physical effect fate, and hydration | Store integration | this milestone |

The authoritative objects are component owner state inside Relational and
Signal, immutable composite commits and mutable product reference cells inside
Runtime World, and live retained-partial records for owner effects not yet
represented by a product commit.

History indexes, ancestry accelerators, inspection rows, cost reports,
diagnostics, and Query projections are derived. Destroying them must not change
which product commit a branch selects or which retained partial obligations
remain unsettled.

## Semantic Identities, Keys, And Non-Substitution

The following are distinct sealed meanings even if their representations
match:

- `RuntimeWorldOwnerIdentity` identifies one live composition owner instance;
- `ProductBranchIdentity` identifies one normalized owner-scoped product branch
  name;
- `ProductBranchIncarnation` identifies the live reference cell admitted for
  that name, changes after retirement/recreation, and makes the
  `(owner, branch, incarnation)` tuple non-reusable;
- `ProductBranchReferenceGeneration` changes only when that reference moves;
- `RuntimeWorldBootstrapAttemptIdentity` identifies the one attempt that may
  establish this owner's root commit and first product reference;
- `CompositeCommitIdentity` identifies one immutable commit occurrence and is
  not derived from content;
- `CompositeBasisKey` is the canonical, non-authorizing equality key for the
  exact component/correspondence tuple;
- `CompositePublicationAttemptIdentity` identifies one bounded execution
  attempt; and
- `ProductUnpublishedOwnerEffectsIdentity` identifies one retained recovery
  obligation after at least one owner effect.

Equal component bases may appear in multiple distinct commit occurrences.
Equal `CompositeBasisKey` values permit pin reuse and equality comparison but
do not admit a basis, collapse history, parentage, operation occurrence, or
reference movement. The admitted component bases and admitted Bridge binding
remain the authority. Digests may prove canonical comparison but never mint an
identity or admit an operation.

Every `ProductBranchObservation` compares the complete owner identity, branch
identity, `ProductBranchIncarnation`, reference generation, and selected
composite commit. Comparing only a branch name, commit id, or generation is
forbidden. Identity or generation exhaustion is a typed pre-effect denial.

## Composite Commit Contract

`CompositeRuntimeWorldCommit` is immutable and contains exactly:

- its owner-issued `CompositeCommitIdentity`;
- `Root` or one `OrdinaryParent` identity;
- the exact `CompositeRuntimeWorldBasis`;
- explicit Relational and Signal change posture;
- exact owner-issued performed outcomes and successor bases for every changed
  component, without inventing a component occurrence id the owner does not
  expose;
- the exact admitted Bridge Runtime World correspondence binding and its
  descriptive equality key;
- exact occurrence provenance: the root bootstrap attempt or one composite
  publication attempt;
- caller correlation accepted as descriptive, non-authorizing boundary
  provenance.

The root-admitted correspondence basis is inherited by every descendant.
Publication compares its installed generation and denies drift pre-effect as
`CompositeCorrespondenceRebindRequired`; changing correspondence requires a
new Runtime World owner and bootstrap.
The canonical `PerformedCompositePublication` envelope binds the immutable
commit to the product-reference movement that selected it. The commit does not
contain that later movement and can therefore be materialized before the
product-head compare-and-publish. Query 9.17.3 derives receipts, history, live
identity, aftermath, and outbox eligibility from the commit plus this one
performed-publication artifact; it may not create an operation-to-commit
authority table beside them.

This milestone admits one ordinary parent only. Parentage is placed beneath a
stable `history/parentage/` axis. Later multi-parent work may add an ordered
parent-set sibling or versioned commit family there, but no multi-parent API,
placeholder, optional vector, or merge behavior is created now.

## Bridge Runtime World Correspondence Contract

Runtime Bridge adds one narrow `RuntimeWorldCorrespondencePort`. Its admission
method accepts a reference to a real `BridgeInstalledSemanticCorrespondence`
and returns `AdmittedRuntimeWorldCorrespondenceBasis` or a typed
`RuntimeWorldCorrespondenceAdmissionDenial`. The port validates owner/runtime
affinity, source installation identity and generation, graph participation,
Signal graph instance, and the fact that the supplied artifact is an installed
witness. It does not accept a detached `BridgeCorrespondenceBasis` as
authority. Runtime Bridge has no managed close/liveness contract in the frozen
surface, so this port must not invent one.

The frozen facade shape is:

```rust,no_run
impl RuntimeBridge {
    pub fn runtime_world_correspondence_port(&self) -> RuntimeWorldCorrespondencePort;
}

impl RuntimeWorldCorrespondencePort {
    pub fn admit_installed(
        &self,
        installed: &BridgeInstalledSemanticCorrespondence,
    ) -> Result<AdmittedRuntimeWorldCorrespondenceBasis,
                RuntimeWorldCorrespondenceAdmissionDenial>;

    pub fn revalidate(
        &self,
        admitted: &AdmittedRuntimeWorldCorrespondenceBasis,
    ) -> Result<(), RuntimeWorldCorrespondenceAdmissionDenial>;
}
```

The denial variants distinguish `ForeignBridgeRuntime`, `ForeignSignalGraph`,
`SourceInstallationIdentityMismatch`, `SourceInstallationGenerationMismatch`,
`SourceAuthorityBindingMismatch`, `GraphParticipationMismatch`, and
`CorrespondenceConfigurationMismatch`. Admission and revalidation are
read-only; they allocate no Signal target, mutate no registry, and perform no
delivery.

The admitted result is cloneable inspection-and-composition authority bound to
the Bridge owner and installed generation. It exposes a canonical descriptive
key for composite equality, but its constructor and authority fields remain
private. Revalidation through the same port distinguishes foreign Bridge,
foreign Signal graph, source-generation drift, and correspondence mismatch.
Runtime World stores the admitted result and may compare or revalidate it; it
never owns mappings, deliveries, conditional evaluation, or Signal targets.

Multiple installed correspondences may share one exact admitted Bridge basis.
That equality does not make their dependency or target declarations
interchangeable. This milestone binds the world-level interpretation basis,
not an unordered inventory of every installed mapping.

## Root Bootstrap

`RuntimeWorldOwner<D, I, E, Ctx, T>` construction creates an empty composition
owner. It does
not inspect ambient component heads, infer a default branch, or manufacture a
root. Exactly one `bootstrap_root` operation may establish the owner graph. It
requires:

- one exact owner-issued admitted Relational basis;
- one exact owner-issued admitted Signal basis;
- one `AdmittedRuntimeWorldCorrespondenceBasis` issued by the Bridge Runtime
  World correspondence port from a real installed semantic-correspondence
  witness;
- one validated initial `ProductBranchCreationIntent`; and
- installed history, branch, pin, attempt, and metadata budgets.

Bootstrap first validates owner/runtime/definition correspondence, reserves the
root commit occurrence and first branch cell, and acquires the two exact owner
retention obligations through the unique pin registry. It then installs the
root commit, first product reference, and canonical creation record in one
Runtime World critical section. It performs no component mutation.

The terminal topology is `PerformedRuntimeWorldBootstrap` or
`NoEffectRuntimeWorldBootstrap`. No-effect guarantees that any acquired lease
was released and no commit or product reference became visible. Performed is
linear and carries the admitted root basis, root commit, first branch
observation, exact pin accounting, and bootstrap occurrence. A second
bootstrap attempt, foreign basis, incompatible correspondence, identity or
capacity exhaustion, cancellation, or owner loss is a typed pre-effect denial.

Every later commit and product branch in this milestone descends from this one
root. Multiple roots, root import, orphan adoption, and root replacement are
not hidden bootstrap modes; later import or merge work must introduce their
own admitted semantics.

## Product Branch And Observation Contract

A product branch is one independently synchronized reference cell. Its public
observation is an admitted, managed object rather than a tuple:

```text
ProductBranchIdentity
    -> ProductBranchObservation
    -> AdmittedCompositeRuntimeWorldBasis
```

Admission pins the selected composite commit and its exact component bases for
the observation lifetime. Clones share one internal observation obligation;
they do not acquire a new owner lease or re-read a head. Serialization, if a
later boundary adds it, must discard operational admission.

Readers resolve the branch cell and commit under one publication-compatible
visibility boundary. A reader can observe the old cell or the new cell. It
cannot observe a moved reference whose commit is absent from history or whose
component pins are not yet installed.

Product branch creation starts from an admitted retained composite basis and
requires one owner-specialized explicit posture per component:

```text
RelationalBranchCreationPlan
    ReuseExact
    ForkExact { target: BranchId }

SignalBranchCreationPlan
    ReuseExact
    ForkExact { target: ValidatedSignalBranchName }
```

Omission is invalid. Two `ReuseExact` postures may select the existing commit.
Creation otherwise uses publication's pre-effect reservation and
Relational-then-Signal order. Each fork creates a new composition occurrence
under the source commit even when content is equal; a performed fork followed
by sibling denial terminates as `ProductUnpublishedOwnerEffects`.

Branch creation does not also publish or advance component state. Callers that
need a changed new branch first create it through the two-by-two reuse/fork
matrix, observe its product head, and submit a separate publication. This keeps
creation implementable through the frozen owner ports, prevents a hidden
post-fork transaction factory, and gives every owner movement one explicit
attempt and terminal artifact.

For Relational `ForkExact`, Runtime World calls `observe_fork_source` with the
source basis's carried branch id, compares all shared exact axes of the returned
source descriptor to the admitted source basis, then calls `fork_branch`. For
Signal `ForkExact`, it calls `fork_exact` with the admitted source basis.
Neither path re-resolves `main`, `current`, or `latest`.

Product branch retirement removes only the product reference and releases its
composition obligations. It never assumes a component branch is exclusive and
never deletes a component reference as a side effect. Owner-created component
branches carry explicit custody records and produce typed owner-retirement work
through owner lifecycle services; denied retirement remains bounded managed
work rather than a hidden leak.

## Component Change And Execution Order

Every publication plan contains exactly one specialized posture for each
component:

```text
RelationalComponentPlan
    RetainExact
    PublishPrepared

SignalComponentPlan
    RetainExact
    AdvanceExact
```

Plans remain owner-specialized. Relational plans carry owner-issued candidates.
Signal advance posture is admitted during preparation, but its mutation closure
and caller-owned runtime context enter only as generic arguments to the
synchronous Signal execution call. The attempt never retains, clones, erases,
boxes, or registers either value. Fork-then-change is expressed as branch
creation followed by publication, not as a third component-plan variant.

For a combined change, canonical effect order is:

1. prepare every Runtime World compatibility, correspondence, budget,
   retention, history-slot, and attempt-record requirement that can be checked
   before owner effects;
2. prepare the Relational candidate without component movement;
3. recheck the exact product-head observation;
4. perform and settle Relational publication;
5. recheck the exact product-head observation;
6. perform the Signal owner operation;
7. recheck and compare-and-publish the product reference; and
8. issue the performed composite artifact.

Relational moves first because its owner installs recoverable settlement before
movement. Signal moves last because its canonical operation returns its final
successor basis synchronously and has no separate external settlement phase.
This order minimizes the least-recoverable residual without pretending either
owner can roll the other back.

Every step branches on its owner outcome. Relational no-movement ends as
`NoEffectCompositePublication`; Relational performed-but-unsettled ends as
`ProductUnpublishedOwnerEffects` without calling Signal; a stale product head
after settled Relational work also stops before Signal; and Signal denial or a
lost final product CAS after any owner movement ends as
`ProductUnpublishedOwnerEffects`. Only fully settled required Relational work
and performed required Signal work may reach the final product CAS.

Single-component operations execute only their changing owner. An unchanged
component incurs no execution call, latest lookup, compatibility rediscovery,
or new owner lease when its exact basis is already pinned by Runtime World.

No Runtime World lock, product-reference lock, history lock, or pin-registry
lock may be held while calling a component owner. Owner calls occur only from
named execution phases after Runtime World preparation locks have been
released.

### Cancellation handoff

Runtime World owns `RuntimeWorldCancellationSource` and its cloneable
`RuntimeWorldCancellationToken`. The source records one shared Runtime World
flag and one private `SignalOwnerCancellationSource`; `cancel()` changes both.
World phases read only the Runtime World flag, while a Signal fork, advance, or
retirement receives the embedded `SignalOwnerCancellationToken`. Callers cannot
supply a raw Signal token in place of the Runtime World token.

Relational has no matching publication cancellation token. Runtime World checks
cancellation and deadline immediately before entering Relational; once the
owner call begins, its performed/no-movement and settlement outcomes decide
truth. No timer thread is introduced: the installed clock is sampled only at
named phase boundaries, and concurrent cancellation during Signal's synchronous
call is observed by Signal's own cutoff.

## Pre-Effect Reservation And Attempt Authority

Before the first owner effect, Runtime World installs one bounded
`CompositePublicationAttempt` containing:

- exact expected product-head observation;
- admitted predecessor composite basis;
- explicit per-component plan;
- reserved composite commit identity and history slot;
- reserved product-unpublished recovery capacity;
- all predecessor and prospective retention obligations obtainable before
  effects;
- cancellation/deadline policy and the last safe no-effect point;
- canonical owner execution order; and
- structural counters initialized before execution.

Capacity for the attempt record, potential retained-partial record, commit
metadata, and required Runtime World pins is reserved before owner effects. An
owner may still return an owner-local bounded-capacity denial according to its
own contract, but Runtime World never discovers that its own bookkeeping or
history capacity is exhausted after an owner has moved.

The attempt is linear. Every terminal path consumes it into exactly one of:

- `NoEffectCompositePublication`;
- `ProductUnpublishedOwnerEffects`;
- `PerformedCompositePublication`; or
- a branch-creation/retirement terminal governed by the same resource rules.

Dropping a pre-effect attempt performs deterministic local cleanup. Dropping an
attempt after any owner effect cannot erase the attempt; ownership first moves
into the preinstalled retained-partial registry, and Drop only abandons the
caller capability while owner recovery remains addressable.

## Product-Unpublished Owner Effects

`ProductUnpublishedOwnerEffects` means at least one component owner performed
work and no product reference moved. It is never called a prepared candidate,
rollback, conflict-only result, or failed commit.

It records:

- the exact expected and last observed product heads;
- a typed progress row for each component: untouched, prepared, performed,
  settlement pending, or settled;
- every owner-issued performed outcome and successor basis;
- every still-live component and composite retention obligation;
- the cause: sibling denial, owner settlement pending, cancellation after
  effect, stale product head, owner loss, or product publication loss;
- legal next actions: resume owner-local settlement, expose the exact record
  for a new Query-admitted operation, abandon after owner-safe cleanup, or
  inspect only; and
- deadline, age, count, and byte-accounting posture.

Resume consumes a fresh recovery capability issued from the retained record
and may perform only owner settlement or owner-safe cleanup. It cannot call an
unperformed sibling owner or move a product reference. Continuing or adopting
owner-local movement is always a new Query-admitted operation in 9.17.3 with a
fresh expected product head and current authority; that new operation names
the retained record as exact evidence but does not derive authority from it.

The registry is bounded by installed count and Runtime World metadata-byte
budgets. Exhaustion rejects before owner effects. Age expiry makes the record
eligible for governed cleanup; it never causes a background thread to discard
owner settlement, release a still-required component basis, or move a product
reference. The owner lifecycle can enumerate every record and terminate or
report it during close.

## Publication And Outcome Topology

The compiler-visible progression is:

```text
ProductBranchObservation + CompositePublicationIntent
    -> ResolvedExpectedProductHead
    -> AdmittedCompositeRuntimeWorldBasis
    -> LoweredOwnerComponentPlan
    -> ReservedCompositePublicationAttempt
    -> OwnerExecutionSettlement
    -> CompositePublicationReady
    -> RuntimeWorldPublicationOutcome
         Performed(PerformedCompositePublication)
         NoEffect(NoEffectCompositePublication)
         ProductUnpublished(ProductUnpublishedOwnerEffects)
```

`NoEffectCompositePublication` guarantees that no component owner moved and no
product reference moved. It distinguishes stale expected head, rejection,
cancellation, deadline, owner denial, and internal pre-effect failure.

`ProductUnpublishedOwnerEffects` guarantees that the product reference did not
move and that at least one named owner effect occurred. Its recovery record is
the only ordinary continuation authority.

`PerformedCompositePublication` proves that the exact reserved commit was
installed and the expected product reference moved once. It carries the
canonical commit, exact old observation snapshot and reference movement, component
results, late-cancellation posture, retention transfer, and cost counters. It
is linear private authority consumed by Query 9.17.3. A cloneable inspection
projection may describe it but cannot authorize a Query committed terminal.

Cancellation is no-effect only through the last safe point before the first
owner effect. Between owner calls or before product CAS it produces
`ProductUnpublishedOwnerEffects`. After product CAS, movement wins and the
performed artifact carries late-cancellation evidence.

There is no indeterminate in-memory product-reference movement: the Runtime
World owner either observes its local reference cell move or not move. Owner
settlement may remain pending or unavailable, which is represented by the
retained partial rather than by inventing an indeterminate product head.

## Atomic Product Publication

Each product branch has an independently borrowable reference cell. The final
critical section:

1. compares the complete expected `ProductBranchObservation`;
2. validates the reserved history slot and already-installed component pins;
3. materializes the immutable commit into that reserved slot;
4. emits the canonical reference-movement record; and
5. changes the branch cell to the new commit and next generation.

These steps are one reader-visible critical section for that branch. Every
fallible Runtime World allocation or capacity check happens earlier. Failure
or panic before reference movement leaves the old head selected and the
attempt recoverable. Once the cell moves, commit and movement evidence are
already available and the performed artifact can be reconstructed by the live
owner if its caller capability is lost.

The branch cell is the composition-currentness linearization point. History
insertion alone, a reserved commit identity, complete owner settlement, or a
diagnostic event does not make a product world current.

## Retention And Reclamation

Runtime World owns a unique component-basis pin registry. Its map key is the
complete canonical descriptor axes for one exact owner basis, never a hash
alone. The key is descriptive and cannot acquire or recover anything. The
first claimant must carry the corresponding owner-admitted basis; retention is
performed only through that basis and the concrete owner basis port. One
registry entry holds at most one external owner retention lease for that exact
basis and accounts for Runtime World dependents by semantic class:

- product branch heads;
- retained composite history;
- admitted observations;
- active publication attempts;
- product-unpublished owner effects; and
- explicit historical inspection or correction obligations.

Repeated composite commits that reuse one Signal or Relational basis increment
Runtime World dependency counts; they do not acquire one owner lease per
commit. The last dependent releases the owner lease exactly once. A denied
foreign release preserves and rebinds the still-live lease according to the
component owner's contract.

Concurrent first use is single-flight per exact basis: one claimant installs a
bounded acquiring reservation, releases the registry lock, and alone calls the
owner. Contenders join that reservation. Success installs one lease; denial
removes the reservation and wakes contenders with the same typed result.

Composite commit records contain exact descriptors and owner-issued performed
evidence, not operational owner leases or Runtime World-invented component
occurrence ids. The history retention graph owns the pins needed to interpret
retained records.

The in-memory history budget has exact commit-count and Runtime World
metadata-byte limits. Reachable ancestry is not silently truncated because
9.18 consumes the complete retained single-parent tree. Reclamation may remove
only commits unreachable from every product reference and explicit obligation,
in bounded maintenance batches. If the budget is full and no eligible node can
be reclaimed, new branch creation or publication is denied before owner
effects as `CompositeHistoryCapacityExhausted`.

Component-state byte totals remain owner-scoped. Runtime World reports:

- exact Runtime World metadata bytes;
- unique component-basis pin counts;
- owner lease acquisition and release counts;
- Relational byte observations only under Relational's declared metric scope;
  and
- Signal capacity and obligation counts without relabeling them as exact
  Signal resident bytes.

No counter claims a cross-owner total that the owners do not expose.

## Lifecycle And Construction

`RuntimeWorldOwner<D, I, E, Ctx, T>` is a non-cloneable managed owner carrying
the exact Signal type contract. It constructs cloneable weak observation,
publication, recovery, and lifecycle ports. Ports share the live owner state
but cannot keep a closed owner alive. Calls after close return typed
`RuntimeWorldOwnerUnavailable`. Basis, commit, observation, and history types
remain non-generic because the admitted Signal basis already owner-seals the
exact branch target without exposing those type parameters; only owners,
builders, publication ports, and Signal-changing
execution surfaces carry the five Signal parameters.

Request-local `RuntimeBridge::fork_managed_request_lane` never clones, resets,
or forks product history; Query composition explicitly carries one Runtime
World port beside any request-local Bridge execution lane.

Construction is compiler-total. The application composition root must supply:

- the base Bridge correspondence-basis port;
- one complete `RelationalOwnerServicePorts` bundle;
- one complete `SignalOwnerServicePorts<D, I, E, Ctx, T>` bundle, already
  issued successfully by the Signal owner root;
- installed history, attempt, retained-partial, observation, branch, and pin
  budgets; and
- an explicit clock only for attempt deadlines and cleanup eligibility.

Adding or omitting one required subsystem breaks every construction site. The
clock cannot affect product meaning, identity, parentage, or authority.
The service bundles are weak: the application composition root, not Runtime
World, retains the Relational and Signal owner roots for their intended
lifetime. Closing Runtime World never closes a component owner.

Close stops new admission, waits only for declared in-flight critical sections,
settles or exposes every retained owner obligation, releases product/reference
pins, and returns a terminal report. It does not keep owners alive through a
strong cycle and does not manufacture success when a component owner is lost.

## Owner-Facing DX Contract

The integration path must be expressible through the real facades without
private imports. The fallible Signal seal and Bridge admission occur before
Runtime World construction. `installed_correspondence` below was installed
against the same Signal graph before that graph entered `signal` and before the
one-way service seal:

```rust,no_run
use worth_runtime_world::facade::{
    CompositePublicationIntent, ProductBranchCreationIntent,
    RuntimeWorldBootstrapIntent, RuntimeWorldCancellationSource, RuntimeWorldOwner,
    RuntimeWorldPublicationOutcome, RuntimeWorldBootstrapOutcome,
};

let bridge_correspondence_port = bridge.runtime_world_correspondence_port();
let signal_services = signal.owner_component_services()?;
let bridge_correspondence = bridge_correspondence_port
    .admit_installed(&installed_correspondence)?;

let world = RuntimeWorldOwner::builder()
    .with_bridge_correspondence(bridge_correspondence_port)
    .with_relational_services(relational.owner_component_services())
    .with_signal_services(signal_services)
    .with_budgets(runtime_world_budgets)
    .with_clock(runtime_world_clock)
    .build()?;

let bootstrapped = world.lifecycle_port().bootstrap_root(
    RuntimeWorldBootstrapIntent::new(
        ProductBranchCreationIntent::named("main")?,
        initial_relational_basis,
        initial_signal_basis,
        bridge_correspondence,
    ),
)?;
let initial = match bootstrapped {
    RuntimeWorldBootstrapOutcome::Performed(performed) => performed,
    RuntimeWorldBootstrapOutcome::NoEffect(denial) => return inspect_bootstrap_denial(denial),
};
let product_branch = initial.product_branch().branch_identity();

let expected = world
    .observation_port()
    .observe_product_branch(product_branch)?;

let cancellation = RuntimeWorldCancellationSource::new();
let prepared = match world.publication_port().prepare_without_signal(
    expected,
    CompositePublicationIntent::without_signal(relational_change),
    &cancellation.token(),
    None,
) {
    Ok(prepared) => prepared,
    Err(no_effect) => return inspect_no_effect(no_effect),
};

match world.publication_port().execute_without_signal(
    prepared,
    &cancellation.token(),
) {
    RuntimeWorldPublicationOutcome::Performed(performed) => {
        query_handoff.accept(performed)?;
    }
    RuntimeWorldPublicationOutcome::NoEffect(no_effect) => {
        inspect_no_effect(no_effect);
    }
    RuntimeWorldPublicationOutcome::ProductUnpublished(retained) => {
        recovery_queue.retain(retained.recovery_handle());
    }
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

`CompositePublicationIntent::without_signal` and
`CompositePublicationIntent::with_signal` produce distinct prepared typestates.
The former is compiler-visible shorthand for an explicit Signal
`RetainExact`; it is not an omitted component plan.
The latter is consumed only by `execute_with_signal`, whose generic method
arguments are `&mut Ctx` and the exact
`FnOnce(&mut SignalTransaction<'_, D, I, E, Ctx, T>) -> Result<(), SignalError>`
accepted by `SignalBranchMutationPort::advance_exact`. The borrow and closure
end with that synchronous call. No-signal execution has no phantom closure type
to infer; Signal-changing execution cannot omit its context or apply function.
Builder grouping may refine, but callers never supply a raw owner runtime,
generic authority marker, component-id string, erased callback, or private
history handle.

## Destination Dependency And Module Topology

The current slice creates this owned topology; omitted `mod.rs` files are
assembly only:

```text
crates/worth-runtime-world/
    Cargo.toml
    README.md
    COMPOSITE_HISTORY.md
    COORDINATED_PUBLICATION.md
    RETENTION_AND_RECOVERY.md
    src/
        lib.rs
        facade.rs
        basis/{composite,admission,equivalence}.rs
        identity/{owner,branch,commit,bootstrap,attempt}.rs
        history/
            {commit,catalog,retention,reclamation}.rs
            parentage/ordinary_parent.rs
        branch/{bootstrap,reference_cell,observation,creation,retirement}.rs
        publication/
            {intent,cancellation,signal_execution,reservation}.rs
            {owner_execution,product_comparison,performed,no_effect}.rs
            component_plan/
                {relational,signal}.rs
        recovery/{product_unpublished,progress,continuation,cleanup}.rs
        retention/{unique_component_pin,dependency_counts,obligation_transfer}.rs
        lifecycle/{owner,ports,close}.rs
        inspection/{history,retention,recovery,cost}.rs
    examples/runtime_world_publication.rs
    tests/
        runtime_world_certification.rs
        runtime_world_certification/
            world.rs + world/{definition,compiler,relational,signal,bridge,observation}.rs
            oracle.rs + oracle/{state,transition,comparison}.rs
            cases/{provenance_and_bootstrap,branch_lifecycle,component_plans}.rs
            cases/{publication_outcomes,recovery,retention_history,substitution}.rs
            cases/{model_sequences,cost,facade,operation_control}.rs
            cases/operation_control/{concurrency,publication_boundaries,recovery_boundaries}.rs
            ui/

crates/worth-runtime-bridge/src/
    correspondence/runtime_world_admission.rs
    facade/runtime_world.rs
```

The stable facade is `worth_runtime_world::facade`. Internal modules are
private or `pub(crate)`. The facade aggregates but implements no publication,
history, retention, or recovery behavior.

Forbidden destinations include:

- `worth-runtime-bridge/src/runtime_world/`;
- Query-owned product references or history;
- a generic `branch_manager`, `runtime_world_manager`, `helpers`, `common`, or
  `shared` bucket;
- Relational-owned Signal correspondence;
- Bridge access to private owner storage;
- a global mutex around any component runtime or the whole composition owner;
- replay, persistence, codec, Store adapter, physical-runtime,
  `correction/`, or multi-parent modules.

A listed file that would exceed 400 lines splits by its named semantic
responsibilities; it may not become a catch-all or require an exemption.

## Dependency Enforcement

Mechanical dependency checks must enforce:

- `worth-runtime-world` may depend on public facades of Relational, Signal,
  Runtime Bridge, Foundational, and Proof;
- `worth-runtime-world` may not depend on any Query package, Store package,
  replay facade, certification crate, UI crate, or application crate;
- `worth-runtime-bridge`, `worth-relational`, and `worth-signal` may not depend
  on `worth-runtime-world`;
- `worth-runtime-bridge` remains forbidden from depending on
  `worth-relational`;
- Query 9.17.3 consumes `worth-runtime-world` only through its facade; and
- no ordinary package reaches a certification-only replay surface.

Facade compile evidence must deny raw basis or commit construction,
cross-owner/cross-head pairing, phase skipping, duplicate performed-witness
use, retained-partial promotion, and generic authority-marker substitution.

## Ordered Phase Plan

Contract evolution uses serial freeze gates followed by parallel waves of two
to four lanes. Only a gate edits manifests, assembly, facades, shared
phase/outcome types, or test roots. Lanes own disjoint paths and ship behavior
or independent evidence. A contract-breaking discovery triggers one serial
correction and consumer rebase; no lane adds an adapter, alias, duplicate trait,
or provisional surface.

Every incremental-review prompt begins by limiting review to the exact last
approved-to-candidate diff and directly invalidated seams. Reviewers read that
patch once, reuse prior conclusions and evidence, and revisit only named hunks
or line ranges; they do not reopen the whole diff or reread the milestone,
crate, kernel, manifests, or dirty surface. Any expansion is announced before
inspection. Holistic review occurs only at an explicit phase or closure gate.

### Phase 1: Serial composition-contract gate

Create the package, dependency fences, assembly, and Bridge admission. Freeze
generic owner/bundle inputs, identities, budgets, correspondence denials,
composite basis/key/incarnation meanings, prepared Signal typestates,
phase/outcome vocabulary, attempt progress, and service seams. The types carry
real invariants. Record each next lane's exclusive paths and focused commands.

### Phase 2: Parallel foundations

- **Bridge:** correspondence admission and focused facade proof.
- **Basis/history:** basis, identity, commits, parentage, and bounded catalog.
- **Retention:** unique pins, counts, transfer, and reclamation.
- **Reference:** product cells, observations, and old-or-new reader semantics.

They consume Phase 1 contracts and cannot edit assembly or facades.

### Phase 3: Serial bootstrap and publication gate

Integrate owner construction, capacities, exact bootstrap, pins, history, and
references. Prove immutable history differs from mutable currentness, then
freeze plan, reservation, execution, CAS, recovery, and close interfaces.

### Phase 4: Parallel product mechanics

- **Branch:** bootstrap completion, reuse/fork creation, retirement, custody,
  and capacity.
- **Preparation:** lowering, specialized plans, checks, reservations, pins, and
  linear attempt installation.
- **Owner/recovery:** canonical execution, settlement, product-unpublished
  progress, continuation, cleanup, and caller-loss transfer.
- **Publication/lifecycle:** final comparison, commit install, reference move,
  terminal issuance, close, and inspection.

Branch implements only the two-by-two reuse/fork matrix; change is a later
publication. No lane holds a product lock across an owner call. Predictable
World failures remain pre-effect; continuation toward product movement after a
partial remains a fresh Query-admitted operation.

#### Phase 4 closure record (2026-09-04)

Phase 4 is implemented on branch `codex/9-17-2-phase4-contract-sync`. The
certified code revision is `598bf7f88a`; this record is the docs-only commit
that follows it. Nothing is merged to master and nothing is pushed.

**Commits since the Phase 3 head `f71c88cc8a`** (in order): `b2145be130`,
`4f858fc81e`, `7c8d0f6c91` (Codex forked-branch creation), `bec8baee1b` (spec
sync to the 2026-09-02 revision), `6ac9d37f84` (operation reservation held
across forked-branch recovery install), `b93fbd6a1b` (two-by-two creation and
typestate publication contract freeze), lane B `3aa3df85ab` + `a5dff614c7`
(lowering, compatibility, reservation), lane A `a4806059d9` + `440fb82419`
(branch creation custody and retirement), lane C `0f4b28a74e` + `e68d8a887a`
+ `1f6daf4481` (execution, recovery, evidence), lane D `88b244496b` +
`c9d71bc219` (product CAS and close report), `33f8e6fc94` (lane integration),
`e255d01ba7` (integration findings INT-001..010), `d0312854f3` (custody drain
on every recovery release), `598bf7f88a` (closure review findings CLS-001..006).

**Gate on `598bf7f88a`** (every cargo command through the shared
`CARGO_TARGET_DIR`):

| Gate | Result |
| --- | --- |
| `cargo test -p worth-runtime-world --lib` | 154 passed |
| `cargo test -p worth-runtime-world --lib --features test-operation-control` | 161 passed |
| `cargo clippy -p worth-runtime-world --lib --tests --features test-operation-control` (crate lints `deny(clippy::all)`) | 0 errors |
| `cargo test -p worth-runtime-world --test runtime_world_certification` | 13 passed |
| `cargo fmt --all -- --check`, `git diff --check` | clean |
| `scripts/ci/check_workspace_rust_line_caps.sh` (dirty and whole workspace) | PASS, no allowlist edits |
| `scripts/quality/scrutinize_rust_functions.py` (whole crate) | 47 candidates, all pre-existing, 0 net new |
| `tools/boundary-check`, `tools/agent-context check` | green |
| runtime-world compiler warning locations | 69 against the 78 at `6ac9d37f84`; all pre-existing dead-code warnings |

**Reviewer verdicts.** Lane B APPROVE after one correction round; lane A
APPROVE after one; lane C APPROVE after two; lane D APPROVE after one. The
integration delta (`33f8e6fc94`, `e255d01ba7`) was reviewed by the
orchestrator, producing `d0312854f3`. Holistic closure review over
`f71c88cc8a..d0312854f3`: REQUEST CHANGES on two Medium findings (the retained
record's live obligation count had two authorities in two units, so a
pre-movement CAS loser was reported with zero composite obligations while it
held its recovery slot; two in-range lines failed the crate's deny-level clippy
gate), fixed in `598bf7f88a` together with the Low findings. Re-review over
`f71c88cc8a..598bf7f88a`: APPROVE.

**Deferred with reason (not Phase 4 defects):**

- `SPEC-P4-017` facade cutover (`RuntimeWorldOwner`/`builder()`, public
  `CompositePublicationIntent`, facade-driveable ports) is Phase 5 implementation
  work; Phase 4 froze the crate-internal seams it cuts over.
- `SPEC-P4-018` crate documentation (`COMPOSITE_HISTORY.md` branch-creation
  section, basis-identity naming) is a Phase 6 documentation item.
**Pre-Phase-5 corrections (completed in the working change, 2026-09-04).** The earlier deferrals
`CLS-004` and `INT-BLOCK-1` are implementation prerequisites and have now been
corrected in the working change. Retention failures preserve actual
owner-unavailable evidence; bookkeeping and destination-admission failures
cannot derive owner closure. Retirement takes a `ProductBranchObservation`,
which proves an installed occurrence, and checks owner/name/incarnation against
the live registry. This removes the historical retired-name set and prevents an
old request from retiring a recreated name.

The same correction enforces active-observation capacity before creation
effects, reports live observations at close, preserves cancellation at both
World and Signal cutoffs, and records actual per-component execution contacts.
Retained exact component bases remain usable by a mixed plan after another
owner-local attempt advances a component; the real product CAS selects the
winner. Crate documentation now describes these implemented boundaries.

Ordinary publication now carries immediate owner-managed custody through
reservation, settlement, readiness, and final product movement. Relational
identity is recorded before settlement consumes its performed capability, and
settled progress is recorded before entering Signal. Production tests cover
two-owner settlement Drop, actual Signal apply unwind, invalid ready evidence,
ready Drop/close/cleanup, and materialization-boundary unwind. The shared custody
owner also has focused real-owner proofs for abandonment accounting, exclusive
resource-lease unwind, concurrent inspection, close, and exact identity repair.
Explicit retained delivery acquires its caller view atomically with catalog
accounting, preventing concurrent cleanup from removing the terminal before
return. Pin-pair transfers now keep their guarded claims through
validation, and publication/retained history installation issues protection in
the same catalog transition. Post-CAS caller loss now recovers the canonical
performed envelope beside its bounded history entry, including complete owner
results, exact movement, late cancellation, and final cost counters. Its
allocation and conservative shared-name retention charge are reserved before
effects. The cell records committed facts before releasing its write lock or
dropping old protection. Normal delivery and recovery share one linear claim;
caller Drop releases that claim, while consumption prevents another delivery.
The exact old snapshot is descriptive evidence and does not permanently hold
an active-observation permit. Real committed-boundary unwind, concurrent claim,
exact-budget, and reclamation tests cover this terminal. History admission now
allocates pending slots in the eventual catalog and reachability maps;
promotion fills them in place. The final branch write lock compares the exact
expected observation before history promotion and bound-pin transfer. A real
competing-winner test proves that the final losing comparison performs neither.
Pending entries remain invisible to lookup, protection, parent admission, and
reclamation, and their capacity releases once on Drop.

Branch creation now carries the same custody through both actual owner forks,
settled finalization, observation, and destination installation. Each returned
fork is recorded before custody bookkeeping. Registry admission binds an exact
destination witness before effects; insertion stamps its commit under the
source and registry guards. The cell stays borrowed until actual insertion,
and retirement cannot erase the witness. Caller abandonment preserves the
original pin classes, history, and any assembled observation. Explicit cleanup
returns the exact destination's component-retirement work. Obsolete independent
retention constructors and their catalog installation lane are removed.

Five real-owner tests cover first-fork unwind, settled two-fork Drop, assembled
cell/observation unwind, post-insertion unwind, and retirement/recreation before
caller Drop. The assembled abandoned creation reports all seven held
obligations; actual insertion never produces a false unpublished record.
Inspection remains non-authorizing. Independent review found no remaining
creation correctness, test, or topology gap.

The working-change gate is 195 default owner tests, 202 with operation control,
and 13 certification tests, plus clippy, formatting, diff, dirty line caps,
boundary-check, and agent-context. Composition/function advisories were reviewed
as coherent transactions and focused real-owner evidence, with no hard scoped
violation. These corrections are ready for Phase 5; the broader Phase 6
certification and documentation work remains in Phase 6.

**Accepted residual risk (design contracts, not defects):**

- Close releases every non-root product reference and drains custody into
  `outstanding_owner_retirement_work`; the caller must dispatch that work,
  close does not retire component branches.
- `next_actions_for_progress` derives `CloseOwner` only from `OwnerLost`;
  `ProductPublicationLost` carries no owner-unavailable evidence, so a loser
  that later finds its owner gone must be re-observed, not inferred.

**Residual-risk closure (2026-09-04, `e940dfd4c8`, `ed824c8f9a`,
`b8f8137cc1`, `98ce88d85e`).** These commits are after the certified revision
`598bf7f88a`; the closure review above does not cover them. They were
reviewed separately, below. The two residual items that were code rather than
design are closed on the same branch, facade untouched:

- `CLS-007` and the transient re-index window. `ProductBranchRegistry` no
  longer keeps a basis-to-commit copy of the head. Exact reuse installs the
  exact commit the source observation names, after the same current-head
  admission the fork path applies, so a displaced source is `StaleSourceHead`
  before any charge and there is no window in which a derived index lags the
  cell. `record_published_head`, `commit_for_basis`, and the index helpers are
  deleted; `publish()` no longer reports to the registry. Proof:
  `exact_reuse_from_a_displaced_source_head_denies_as_stale_before_any_charge`
  (mutation: dropping the head check fails it by name). This removes a
  mechanism integration finding INT-001 introduced; INT-001's behavioural
  proof, `exact_reuse_after_a_publication_selects_the_published_commit`,
  stands unchanged.
- Observation issuance after the fork's product movement has its proof.
  `observation_issuance_adds_no_unique_pin_beyond_the_withheld_route` takes
  the same both-owner fork twice in one world, once with the observation
  authority withheld by the rehearsal seam and once issued, and pins each
  route at exactly the pin pair the destination basis costs; the issuance
  step itself adds no unique pin. (`fork_observation_issuance_adds_no_unique_pin_beyond_the_published_head`
  pins the same total over a plain successful fork.) No route mutation can
  make the observation pin a basis the world does not already hold, so the
  proof is checked for exactness rather than against a failing mutant. The
  issuance stays reserved by construction rather than by a token.
- The publication figure above names `ProductBranchObservation +
  CompositePublicationIntent` as its entry, which is what
  `prepare_publication` takes.

The review of `e940dfd4c8..ed824c8f9a` found two holes those commits left
open, and `b8f8137cc1` closes them in code rather than recording them:

- The check-then-act window between a creation's last source-head recheck
  and its registry installation. Exact reuse checked the head before its
  branch reservation and installed afterwards; a fork rechecked before its
  first owner fork and installed after both. A publication landing in either
  window installed a child from a head that was no longer current. Both
  routes now install through `install_from_source`, which inserts inside
  `ProductBranchReferenceCell::while_current`, holding the source cell's
  read guard across the insertion under the registry lock (the lock order
  `root_snapshot` already uses). A source displaced in the window is refused
  at the install itself: reuse answers `StaleSourceHead` with nothing
  installed and every pin released; a settled fork is retained as
  `StaleProductHead` naming the head that won, its component branches kept in
  custody, its protection taken back from the cell the registry refused.
  Proofs, reached through a bounded rehearsal seam
  (`lifecycle::owner::rehearsal`, shared with the creation boundary):
  `while_current_holds_the_guard_across_its_section_and_refuses_a_displaced_head`,
  `exact_reuse_whose_source_moves_before_install_is_refused_under_the_guard`,
  `a_fork_whose_source_moves_before_install_retains_the_winner_instead_of_installing`.
  Mutations: inserting without the guard fails both window proofs; dropping
  the guard before the section fails the cell proof.
- A source branch that holds no occurrence was `StaleSourceHead` on the
  reuse route and `RetiredBranch` from observation. Every creation route now
  names it `RetiredBranch` through one `admit_source_head` helper; a displaced
  or recreated head stays `StaleSourceHead`. Proof:
  `creation_from_a_retired_source_is_named_retired_on_every_route`
  (mutation: naming a missing cell stale fails it by name).
- Exact reuse held no operation reservation, so `close()` could drain the
  registry underneath an installing reuse. It now holds one across the
  installation; the reuse window proof asserts the active count while the
  creation is held (mutation: dropping the reservation fails it by name).

What `e940dfd4c8` changed beyond the two residuals, and still stands:

- The rule "deny a reuse whose basis is installed on more than one branch" is
  gone with the index. An observation names one exact occurrence, so the
  ambiguity that rule guarded against cannot arise from an observation.
- `insert_installed` no longer compares the installed cell's basis and commit
  owner against the registry owner. `ProductBranchReferenceSnapshot::owner_issued`
  refuses a commit or basis whose owner differs from the snapshot owner, and
  the reservation already compares the snapshot owner, so the invariant holds
  by construction; there is no registry-level test for it because no
  foreign-owner snapshot can be built to exercise one.

**Reviews of the post-certification commits.** `e940dfd4c8..ed824c8f9a`
(review RR-001..006, APPROVE): the review confirmed the index removal and
the withheld-route pin proof, and named the check-then-act window (RR-001),
the retired-source naming (RR-002), the reuse operation reservation
(RR-004), and one doc citing a superseded proof name (RR-003, corrected
above). `ed824c8f9a..b8f8137cc1` (review SG-001..004, APPROVE, no High or
Medium): the reviewer traced the lock order for every path that takes the
registry mutex and a cell lock, the same-cell, recreated-name, and
retired-between-admission-and-install cases, and every refused path's
custody, pins, reservations, and operation ledger, and confirmed the window
closed on both routes. Its four Low findings are closed in `98ce88d85e`: the
unguarded named install `ProductBranchRegistryReservation::install` is
deleted; the install-time retired arm is proven on both routes
(`exact_reuse_whose_source_retires_before_install_is_named_retired_under_the_guard`,
`a_fork_whose_source_retires_before_install_is_retained_without_a_winner`,
mutations fail by name); the boundary gate doc names every way it closes;
the reuse proofs measure obligations rather than unique slots.

Gate on `98ce88d85e` (shared `CARGO_TARGET_DIR`): lib 162, feature-lib 169,
certification 13, clippy deny gate 0 errors, fmt, diff-check, line caps (dirty
and workspace), scrutiny 47 candidates (all pre-existing), boundary-check and
agent-context green, lib test warning locations 68 (69 before; the deleted
unguarded install was one of them). The lane worktrees and branches
`claude/9-17-2-lane-{a,b,c,d}` are removed; every commit they held was
already on `codex/9-17-2-phase4-contract-sync`.

### Phase 5: Serial progression and facade freeze

Phase 5 public-boundary implementation decision: non-generic observation,
branch, recovery, and lifecycle ports hold weak references to private World
service traits. Only the private World engine implements those traits; callers
cannot register implementations. This dispatch hides the engine's generic
storage, never replaces or erases either concrete component-owner bundle, and
never stores a Signal context or mutation callback. Publication ports retain the
five exact Signal parameters. The public non-Clone owner is the sole strong root;
owner loss closes admission even while an already admitted call holds a temporary
engine reference. Consuming a performed publication permanently closes its
exclusive delivery lane; the returned receipt is descriptive canonical evidence,
not authority for another product terminal.


Phase 5 inspection and custody decisions: every installed history occurrence
owns a mandatory pair of exact component dependencies, forked from already held
claims before installation without acquiring another component-owner lease.
Bounded ancestry observations protect their starting occurrence and retain the
catalog lifetime; they return borrowed commit references. Explicit history
reclamation has named candidates and a work bound, with no invented history-age
policy. Exact pin maintenance examines only caller-named keys. Recovery pages
charge every examined slot, including vacancies, against the work bound, and recovery
age starts at the owner-clock admission sample carried through caller loss.
Cleanup takes an explicit minimum age. Inspection counters are catalog or
registry observations with documented accounting scope; cumulative concurrent
work is never attributed to a single attempt. Per-attempt history reservation
cost is recorded at successful reservation.

Phase 5 final handoff decisions: canonical Relational result/receipt and Signal
advance/fork outcomes are borrowed through the result facade; settlement routes
remain private. Bootstrap accepts an optional explicit cancellation token on its
intent. Its final installation checks cancellation and owner/close state under
bootstrap admission, then publishes the complete root state under the same guard.
Inspection is unavailable until that state is complete. Close holds bootstrap
through operation admission, publishes Closing, then drains component custody
outside lifecycle locks.

Assemble the typed progression; audit cancellation/capacity edges; prove three
outcomes, one-winner CAS, unrelated progress, bounded recovery, and no mixed
world; freeze the facade and 9.17.3 artifacts.

### Phase 6: Parallel certification and documentation

- **Production world:** real fixtures, neutral observations, and sequential cases.
- **Oracle:** pure state, transitions, comparisons, and model sequences.
- **Adversarial:** controlled schedules, substitutions, compiler cases, races.
- **Operability:** cost/scale, facade, example, docs, dependency/residue proof.

Phase 6 implementation evidence (2026-09-05): one real facade court with the
non-unit Signal bundle, the full reuse/fork matrix, hostile publication/recovery
and retention cases, six deterministic operation-control scenarios, independent
seeded product transitions, and three named ignored profiles are implemented.
The gate approved the model/cost slice after requiring fresh live-head observation,
all known basis dependency classes, varied seeded operation order/lifetimes and
live-pin U populations. Ordinary certification has 33 tests plus three ignored
profiles; the operation-control feature adds six cases. The grouped compiler
family executes sixteen fixtures. The facade-only executable example and public
owner docs complete Phase 6. The persistent gate approved the documentation and
example slice, followed by the Phase 7 closure below.

The Signal single-target routing choice and reproducible untouched batch-read
follow-up are recorded in `crates/worth-runtime-world/RETENTION_AND_RECOVERY.md`.
No Query cutover or batch-read repair is claimed. Scheduled profile configuration,
measurement posture and population co-variation are documented there as well.

### Phase 7: Serial closure gate

Run the one integration target's default/feature lanes plus owner, compiler,
scale, docs, format, lint, line-cap, boundary, and context gates. Review/merge
is not an implementation lane; no contract fork crosses into 9.17.3.

Phase 7 closure evidence (2026-09-05, final gate approved): the ordinary package
passes 209 owner tests, 33 certification cases (including the sixteen compiler
fixtures), and the executable example; the doc lane contains no doctests. The
feature owner lane passes 217 tests, deterministic operation-control passes six,
and the scheduled lane passes all three profiles. Focused predecessor checks
pass seven Bridge Runtime World cases, 207 Signal owner-service cases and four
Relational lineage resolution cases. Formatting, dirty line caps, dependency
boundaries and generated context validation pass.

The exact planned Clippy command was run and failed on existing Signal dependency
dead-code diagnostics. All warnings in scoped files were corrected. Strict World
checks with `--no-deps` pass for default and all features; dependency diagnostics
remain in untouched files (62 Signal and seven Relational warning groups). Under
`AGENTS.md`, untouched pre-existing failures are repository debt and do not expand
or block scoped completion. This is not a claim that the broader strict command
passes. The Bridge selector was corrected from `runtime_world_correspondence`
(which executed zero cases) to `runtime_world` (seven executed cases).

## Performance And Resource Contract

Name these scale axes in implementation and evidence:

- `B`: live product branches;
- `H`: retained composite commits;
- `U`: unique exact component bases pinned by Runtime World;
- `A`: active publication attempts;
- `P`: retained product-unpublished records;
- `O`: active admitted observations; and
- `W`: concurrent writers.

Owner-local hash indexing uses expected/amortized constant-time bounds, as in
the exact-pin registry; this is not a worst-case collision guarantee. History
reservation carries stable entry/reachability slots through publication. Index
growth and slot allocation precede component effects; final installation writes
those slots without index insertion. Logical-call counters and direct-slot-write
counters describe different work and cannot establish hash collision bounds.
Recovery likewise uses pre-effect reusable slots and an identity hash index;
Active/Retained/Busy transitions update one existing slot. Bounded pages use an
opaque owner-affine position cursor and charge vacancies against the examined
bound. Slot order is observational, not identity order or a frozen snapshot;
concurrent reuse before the cursor requires a fresh scan to observe that new row.

Required bounds are:

- product-head observation and comparison are O(1) in `B` and `H`;
- basis admission is O(1) in the fixed two-component cardinality;
- branch creation is O(1) plus only operation-named owner fork work;
- ordinary publication is O(changed components) plus O(1) product publication;
- unchanged component execution and new owner-lease acquisition are exactly
  zero when the unique basis pin already exists;
- one branch operation acquires no other product branch's reference cell;
- no ordinary operation scans `B`, `H`, `U`, `A`, `P`, or `O`;
- history traversal is explicit O(requested span) work with a caller-visible
  bound, never a cheap-looking getter;
- reclamation is explicit maintenance bounded by the requested batch;
- retained-partial cleanup cannot export unbounded work to a background queue;
  and
- total retained Runtime World metadata, attempts, observations, branches,
  commits, and recovery records stay within installed limits.

Counters expose at least product cells touched, history slots reserved and
installed, owner contacts by component, expected-head rechecks, unique pin
hits/acquisitions/releases, no-effect outcomes, owner-effect progress,
retained-partial creation/resume/cleanup, CAS attempts/wins/losses, observations,
and reclamation breadth.

The deterministic court must use controlled schedules rather than timing to
prove unrelated progress and same-head races. The scheduled scale profile must
measure structural slopes across `B`, `H`, `U`, `A`, and `W` with named runtime
configuration, cold/warm posture, repetitions, variance, and percentiles.

## Test Evidence Architecture

The certification target follows the established Supply Chain and Bank World
standard: a causally compiled real world, an independently authored oracle,
direct evidence from every external owner crossed by the claim, and hostile
twins that differ in one relevant fact.

Its decisive schedule parks an owner call while an unrelated branch publishes,
then overlaps Relational-only and Signal-only attempts on one expected head
before product CAS. Exactly one is `Performed`; direct owner evidence requires
the loser to be `ProductUnpublished`. The pure oracle owns semantic state, not
outcome classification.

The production side uses only the real public facades of Runtime World,
Relational, Signal, and Runtime Bridge. The expected side uses a pure
`CompositeWorldOracle` over test-local semantic identifiers, maps, sets, and
transition rules. Production observations are converted at the boundary into
neutral values before comparison. The oracle must not call or copy Runtime
World history, publication, digest, comparison, pinning, retention,
reclamation, or recovery algorithms.

World declaration/compiler failures, component-owner failures, Runtime World
denials, observation failures, and oracle disagreements have distinct test
errors. Every hostile family proves a healthy twin at the same boundary. A
denial after invalid setup, an absent owner call, or production-comparator reuse
is not evidence.

## Canonical Composite Court World

`CompositeSupplyChainCourt` is a compact but semantically credible port,
voyage, and manifest world. Its compiler:

1. installs component state in a real Relational owner through its public
   facade;
2. constructs and configures the real Signal graph, including the cargo-routing
   derivation, without sealing owner services;
3. binds Runtime Bridge to that same construction-phase graph, installs and
   retains a real `BridgeInstalledSemanticCorrespondence`, and performs no
   detached-graph substitution;
4. moves that graph into the real Signal runtime, issues
   `SignalOwnerServicePorts<D, I, E, Ctx, T>`, and admits the exact initial
   Signal basis through its public facade;
5. admits the installed Bridge witness through
   `RuntimeWorldCorrespondencePort`; and
6. creates a real Runtime World owner and performs explicit root bootstrap from
   those exact admitted bases.

The court must not copy the private test kit of another crate or replace an
owner with a mock. Its scope is the composition contract, so the component
world is intentionally smaller than the Relational Supply Chain certification
world while still crossing the real owner boundaries.

The `CompositeWorldOracle` models independently:

- explicit bootstrap and one root occurrence;
- immutable commit occurrences with exactly one ordinary parent;
- product branch identity, `ProductBranchIncarnation`, head, and generation;
- exact Relational, Signal, and correspondence bindings;
- the two-by-two owner-specialized `ReuseExact`/`ForkExact` creation matrix and
  the independent `RetainExact`/owner-change publication matrix;
- unique component-pin dependency classes for heads, history, observations,
  active attempts, and retained partials; and
- bounded attempt, history, reclamation, and recovery state.

Runtime World public history, reference, observation, retention, recovery, and
cost projections provide product evidence. Relational and Signal public owner
observations and counters independently establish component contact, movement,
and settlement; a composite result cannot establish any of them.

## Required Runtime World Scenario Families

The one certification target must contain these named families:

1. **Provenance/bootstrap:** healthy root with zero mutations; duplicate,
   foreign, incompatible, cancelled, and capacity denials are exact no-effect
   and release temporary obligations.
2. **Branch lifecycle:** full two-by-two `ReuseExact`/`ForkExact` matrix;
   omission, equal-content forks, sibling denial custody, retirement, ABA, and
   separate create-then-publish.
3. **Component plans:** full `RetainExact`/owner-change matrix; candidate
   affinity, exact Signal context/apply signature, typestate separation, zero
   sibling contact, canonical order, and no Signal after unsettled Relational.
4. **Outcomes:** cancellation, stale/preflight/owner denial, sibling denial,
   late head change, CAS loss, Signal panic, post-movement unwind, late cancel,
   capability loss, and owner loss classified from direct evidence.
5. **Concurrency:** unrelated progress at every park; same-head
   Relational-only versus Signal-only and combined-versus-single races yield one
   performed winner, product-unpublished losers, and old-or-new readers.
6. **Recovery:** owner/attempt affinity; forged/foreign rejection; settlement-
   or-cleanup-only resume; caller loss, cleanup races, close, age eligibility,
   and complete obligation enumeration. Adoption stays in 9.17.3.
7. **Retention/history:** one lease for reused Signal basis; every dependency
   class; clone/final-drop behavior; bounded unreachable-only reclamation;
   ancestry protection and history-full pre-effect denial.
8. **Substitution:** equal-looking foreign owner/branch/definition/Bridge/World
   artifacts and correspondence drift; raw ids/descriptors authorize nothing.
9. **Model sequences:** seeded bootstrap, branch, publish, observe, retain,
   cancel, stale, reclaim, resume, and cleanup sequences; print seed and minimal
   failing prefix.
10. **Cost:** vary `B`, `H`, `U`, `A`, `P`, `O`, and `W`; prove zero unrelated
    contacts, pin-hit acquisition, foreign-cell touch, or invented Signal bytes,
    plus bounded reclamation and required slopes.
11. **Facade/dependency/docs:** facade-only and non-unit compile-pass;
    compile-fail construction, detached Bridge admission, cross-affinity,
    typestate exchange, duplicate consumption, authority promotion, and private
    access; run fences and the executable example.

## Outcome Classification Table

These boundary cases are mandatory because they distinguish component truth
from product truth:

| Last independently observed fact | Required outcome | Forbidden interpretation |
| --- | --- | --- |
| cancellation, preflight/stale denial, or owner denial before any owner movement | `NoEffect` | an attempt or partial record presented as movement |
| an owner moved, required sibling work is incomplete, and the product head stayed put | `ProductUnpublished` | rollback, performed product state, or implicit sibling continuation |
| Relational settled, then the product head changed before Signal | `ProductUnpublished` | calling Signal against a now-stale product intent |
| Signal moved, then final product CAS lost | `ProductUnpublished` | silently adopting owner-local movement |
| product CAS installed the reserved commit and movement envelope carrying exact changed-owner outcomes and successor bases | `Performed` | late cancellation or an inspection projection replacing linear authority |
| caller capability disappeared after an owner effect | preinstalled retained partial | losing custody, duplicating work, or finishing the sibling/product step |

Each row needs a healthy twin, direct component-owner counters, Runtime World
attempt/recovery observations, and the oracle's expected semantic state.

## Lane And Harness Contract

All runtime evidence remains under `runtime_world_certification.rs`; directory
modules do not become additional Cargo integration targets. UI fixtures execute
as a grouped compiler family.

- The ordinary CI lane owns bootstrap denials, sequential no-effect/owner-loss
  outcomes, branch lifecycle, component plans, recovery authority,
  retention/history, substitution, facade evidence, and the Court model.
- The CI operation-control lane runs with the test-only
  `test-operation-control` feature and owns deterministic owner parks,
  cancellation/fault boundaries, same-head races, reader atomicity, and their
  healthy twins.
- The scheduled ignored lane owns longer model sequences, larger histories and
  contention, and named Court/Standard/Scale profiles with configuration,
  cold/warm posture, repetitions, variance, p50/p95/p99, and structural slopes.

The operation-control feature may only park or inject faults at named points in
the real phase progression. It may not mint authority, bypass an owner, replace
publication logic, or change production semantics. Synchronization uses
barriers or channels, bounded waits, and drop-safe release guards—not sleeps.
It is disabled by default and forwards only the corresponding Relational and
Signal test-operation-control capabilities required by the court. CI must fail
if a command intended to select a required feature-gated family executes zero
cases.

The manifest spelling is exact:

```toml
[features]
test-operation-control = [
    "worth-relational/test-operation-control",
    "worth-signal/test-operation-control",
]
```

The planned commands are:

```text
cargo test -p worth-runtime-bridge runtime_world
cargo test -p worth-runtime-world --test runtime_world_certification
cargo test -p worth-runtime-world --features test-operation-control --test runtime_world_certification operation_control::
cargo test -p worth-runtime-world --features test-operation-control --test runtime_world_certification -- --ignored
cargo test -p worth-runtime-world --doc
cargo test -p worth-runtime-world --example runtime_world_publication
cargo clippy -p worth-runtime-world --all-targets --all-features -- -D warnings
```

The ignored certification command belongs to the scheduled lane. The example
is declared with `test = true` and `harness = false`, so the ordinary package
test lane also executes its real `main`. This milestone does not add
Docker, TCP, or process tests: Runtime World is intentionally memory-resident,
and those boundaries would be theatre here. Query's real composition root is
9.17.3; durable restart belongs to Store successors.

## Sensitivity And Teardown Contract

Targeted hostile twins or mutation probes must turn red when final CAS/head
comparison or owner order is removed; locks cross owner calls; leases amplify;
latest/descriptors substitute for admission; owner effects become product
truth; recovery calls a sibling/CAS; reachable ancestry is pruned; publication
evidence is incomplete; or a retired incarnation is reused.

Every family uses fresh owners. Success and failure paths release observations,
leases, parks, and test faults; close Runtime World and both component owners;
and prove there are no unexplained attempts, pins, custody records, waiters, or
partial records. A drop-safe harness must unblock parked workers before
propagating a panic so an assertion failure cannot hang the suite or poison the
next case.

## Documentation Deliverables

- `README.md`: construction, bootstrap, ownership, publication, outcomes, and
  memory-resident limits.
- `COMPOSITE_HISTORY.md`: occurrence/equivalence, parentage, references,
  observations, creation, and retained history.
- `COORDINATED_PUBLICATION.md`: phases, owner order, cancellation, three
  outcomes, and recovery.
- `RETENTION_AND_RECOVERY.md`: pins, budgets, partials, reclamation, close, and
  owner loss.
- `crates/worth-runtime-world/examples/runtime_world_publication.rs` as an
  executable facade contract covering bootstrap, ordinary, stale, and
  retained-partial handling.
- revise Bridge `API_OVERVIEW.md` and `REFERENCE_MAP.md` to point product
  history to Runtime World.

The example must compile and execute against the real public facade in an
ordinary package test command. Documentation must not claim durability,
restart recovery, multi-parent history, or Query public completion.

## Must Ship

- the acyclic `worth-runtime-world` owner, exact installed-witness Bridge
  admission, and explicit one-root bootstrap with no ambient head;
- non-substitutable identities, immutable single-parent commits, synchronized
  product cells, managed observations, and unique exact-basis pins;
- the explicit reuse/fork branch lifecycle, owner-specialized publication,
  canonical owner order, and pre-effect attempt/recovery reservation;
- bounded history/recovery/reclamation and typed `NoEffect`,
  `ProductUnpublished`, and `Performed` outcomes around one product CAS; and
- the stable facade, honest counters, executable docs, dependency/compiler
  enforcement, and adversarial court.

## Must Preserve

- all 9.17.1/.1.1/.1.2 owner-basis, service, settlement, retention, lifecycle,
  independent-progress, cost-scope, and no-global-lock guarantees;
- distinct Relational, Signal, Bridge, Runtime World, Query, Store, and
  Foundational authority, with concrete owner-specialized Proof carriers;
- each owner's canonical performed artifact and Query's existing outbox payload
  inside the Relational result; and
- ordinary/history/recovery/maintenance/diagnostic lane separation with replay
  remaining certification-only.

## Explicit Non-Goals

- Query public branch workflow, public facade cutover, or complete Query
  carriage;
- persistence, PostgreSQL, Store adapters, checkpoints, restart recovery,
  physical residency, replication, or distributed publication;
- semantic undo/redo, compensation product semantics, merge, rebase,
  multi-parent history, tags, best-common-ancestor selection, or offline sync;
- automatic adoption of owner-local movement after a losing product race;
- cross-owner rollback; and
- total component-state byte accounting not exposed by component owners.

## Acceptance And Handoff

Milestone 9.17.2 closes only when:

- the new package and dependency checks prove one acyclic composition owner;
- exactly one explicit bootstrap establishes the initial root and product
  reference, while every duplicate, foreign, incompatible, cancelled, or
  capacity-denied bootstrap is exact no-effect;
- the real owner facade proves exact correspondence and rejects every hostile
  substitution before effects;
- one non-unit Signal contract compiles through the generic owner while the
  builder rejects omitted bundles and the two prepared execution typestates
  cannot be exchanged;
- product branch observations remain valid under concurrent publication and
  pin exact component bases;
- immutable root/single-parent commits and mutable product references remain
  distinct;
- branch creation supports exact reuse and owner-issued fork without ambient
  selection, including Relational's fresh fork-source-token comparison;
- independent branches progress without a global composition or Signal lock;
- same-head mixed-plan races overlap owner effects, produce one winner, and
  classify every loser from observed owner movement;
- partial owner effects survive caller-capability loss and converge through one
  bounded recovery record without duplicate owner work;
- repeated exact-basis reuse does not amplify owner leases by commit count;
- history and every managed registry enforce their installed bounds before new
  owner effects;
- compiler evidence denies minting, phase skipping, cross-head pairing,
  duplicate performed use, and retained-partial promotion;
- cost observations use honest scopes and exact structural counters;
- executable documentation, focused tests, the intentional integration target,
  scheduled scale evidence, formatting, lint, dirty line caps, boundary check,
  and generated context validation pass;
- the causally compiled composite court, independent oracle, outcome table,
  deterministic operation-control lane, seeded model family, sensitivity
  cases, and teardown assertions satisfy their contracts; and
- review finds no legacy or competing composition authority.

Closure does not require the 9.17.3 Query cutover, but review must confirm that
9.17.2 added no second Signal graph, no `BridgeOwnedSignalRuntime` sealing
shortcut, and no erased bridge between the current Query topology and the new
owner.

Milestone 9.17.3 receives only:

- `PerformedRuntimeWorldBootstrap` and its initial admitted branch observation
  at application construction;
- `RuntimeWorldObservationPort` and exact product branch observations;
- `AdmittedCompositeRuntimeWorldBasis`;
- owner-facing branch creation and publication intents;
- `PerformedCompositePublication` as the sole committed-terminal authority;
- `NoEffectCompositePublication` and
  `ProductUnpublishedOwnerEffects` with their typed next actions;
- immutable history and bounded inspection projections; and
- exact component/publication correlation needed to gate the existing outbox.

Query may carry, consume, and project these artifacts. It may not construct
them, rebuild them from component ids, store a competing operation-to-commit
authority map, move product heads, settle component owners, reinterpret a
retained partial as committed, or repair a missing 9.17.2 guarantee with facade
logic.
