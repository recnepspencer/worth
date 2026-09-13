# Milestone 9.17.1.2: Final Owner Services And Signal Independent Progress

> **Status:** Closed on 2026-09-01. The accepted production revision is
> `95c9aa7455`.
>
> **Product posture:** This is a corrective predecessor to Milestone 9.17.2.
> It does not reopen or reinterpret the closed 9.17.1 and 9.17.1.1 meanings.
> It completes the Relational service bundle and replaces whole-`SignalRuntime`
> exclusive access with owner-issued, independently borrowable services.
>
> **Why this milestone exists:** Relational observation, retention, and branch
> lifecycle were direct runtime methods, while Signal mutation required
> `&mut SignalRuntime`. Runtime World could not consume either gap honestly.

## Goal And Roadmap Placement

Complete the concrete Relational owner bundle and establish the Signal-owned
service boundary through which a composition owner can use exact component
branches without:

- owning or exclusively borrowing an entire `SignalRuntime`;
- serializing unrelated Signal branches behind one runtime-wide mutex;
- copying the Signal graph into a second authority structure;
- reconstructing an admitted basis from ids or descriptors; or
- keeping a closed Signal owner alive through a cloneable service handle.

Closed 9.17.1/.1.1 basis, MVCC, publication, settlement, and retention meanings
remain authoritative. This milestone ports existing Relational operations and
establishes Signal service progress.

## Central Claim

Every composition-facing Signal operation executes through an owner-issued
service against one exact admitted branch basis. Operations on unrelated
Signal branches can make synchronous progress independently. Operations that
target the same branch are serialized at that branch's owner cell and compare
the complete expected basis before movement. Owner closure, stale basis,
cancellation, panic, capacity denial, and caller-capability loss have explicit
terminal or recoverable posture; none is represented as an ambiguous tuple or
an indefinitely held global lock.

The claim is false if:

- a consumer must place `SignalRuntime` behind `Arc<Mutex<_>>`;
- one branch's blocked operation prevents an unrelated admitted branch from
  completing;
- a service clone strongly owns the runtime lifecycle;
- the service maintains a shadow branch graph or second head table;
- public service methods accept raw branch ids in place of admitted bases;
- same-branch races both report performed movement;
- a panic poisons unrelated branch progress or loses canonical branch truth;
- cancellation is called no-effect after owner movement; or
- existing `SignalRuntime` methods remain an ungoverned second mutation lane.

## Governing Ownership Decision

| Truth or responsibility | Sole owner after this milestone | Explicit non-owner |
| --- | --- | --- |
| Relational basis and lifecycle operations | Existing Relational owner engines behind concrete ports | Runtime World adapters |
| Signal branch graph, head, generation, and stored state | `worth-signal` branch owner state | Runtime World, Query, Bridge |
| Exact basis admission and retention | Signal basis service and existing retention registry | Runtime World registries |
| Same-branch serialization | Signal branch execution cell | Caller mutexes or Query locks |
| Cross-branch independence | Signal owner-service topology | A global runtime lock or worker queue |
| Owner lifecycle and close transition | Signal owner root | Cloneable ports and leases |
| Component operation outcome | Existing Signal owner outcome families | Composite publication outcome |
| Product currentness | Milestone 9.17.2 Runtime World owner | Signal services |

## Relational Service-Bundle Completion

`RelationalRuntime::owner_component_services()` returns one concrete weak
`RelationalOwnerServicePorts` bundle aggregating the four frozen ports plus new
basis and lifecycle ports without wrapping them. Its exact accessors are
`preparation_port()`, `fork_port()`, `publication_port()`, `settlement_port()`,
`basis_port()`, and `lifecycle_port()`.

The basis port exposes existing observe, admit/readmit, retain, and release
operations through `&self`. The lifecycle port exposes existing archive,
delete, typed pending deletion, and exact
`RelationalOwnerLifecycleObservation::{Open, Closing, Closed}` through `&self`.
Weak-owner failure is exactly `OwnerUnavailable` in the existing basis,
archive, and delete denial enums; cold root reclamation stays owner maintenance.

The bundle and ports are concrete `facade::branch`/`facade::mvcc` exports with
weak owner connectivity. They cannot keep the owner alive, mint authority,
accept raw descriptors as authority, or introduce a global runtime lock. The
existing direct runtime methods delegate to the same owner state.

Runtime World may only carry, call, and correlate concrete owner-facade types.

## Owner Root And Port Lifecycle

`SignalRuntime` remains the one non-cloneable owner root. After construction-
only configuration, `owner_component_services()` is the one-way seal: it moves
canonical branch state behind the owner cells and issues one concrete, weak
`SignalOwnerServicePorts`. Its three accessors return cloneable `Send + Sync`
ports; Runtime World receives only those ports and never selects a pre-seal
lane. Lifecycle inspection is descriptive and carries no authority.

Issuance requires the captured types to satisfy explicit `Send + Sync +
'static` bounds. It denies, pre-effect, configured event subscribers,
observation registrations or managed queues, and live-branch or retirement-
receipt capacity exhaustion. Local-only configurations keep ordinary
compatibility methods but cannot enter the 9.17.2 bundle; no erased wrapper may
counterfeit the bounds.

The bundle and ports hold only weak connectivity plus diagnostic identity.
Every concrete port denial has `OwnerUnavailable(SignalOwnerUnavailable)`; no
generic wrapper or substitute represents a closing or gone owner. Ports and
observations do not keep the owner alive. Strong retention leases keep only
their existing exact residency obligation and terminal contract.

Closing the owner:

1. prevents admission of new operations;
2. allows already-linearized owner movements to finish and publish their exact
   outcomes;
3. terminally settles registered retention and operation obligations;
4. leaves no waiter that requires a dropped caller capability; and
5. makes every later weak-port call return the same typed unavailable posture.

There is no background service whose lifetime or queue drain defines truth.

## Signal Branch Basis Port

`SignalBranchBasisPort` is the sole composition-facing read/admission surface.
It supports, with `&self`:

- observation from an owner-issued managed branch reference;
- exact readmission from that managed reference or a live exact retention
  artifact where the existing owner contract permits it;
- full comparison of an admitted basis with the current canonical branch;
- acquisition of an exact retention lease;
- explicit release through the existing lease lifecycle; and
- bounded structural counters and lifecycle inspection.

The port never accepts a digest, descriptor, branch name, id, or generation as
authority. Descriptor-only readmission remains a compatibility method on the
owner root and is not composition-facing. Port readmission consults the owner
from managed authority and returns an admitted basis or typed denial.

A held observation or retention operation may briefly synchronize with the
target branch or retention registry, but it cannot retain an exclusion guard
across a caller callback, a Runtime World lock, or another owner call.

## Signal Branch Mutation Port

`SignalBranchMutationPort` exposes the canonical Signal operations with
`&self`:

- `fork_exact` from one admitted source basis and validated requested identity;
- `advance_exact` from one admitted expected basis and one caller-owned runtime
  context;
- exact snapshot capture where the current snapshot contract permits it; and
- exact restoration through the existing owner-issued restoration request and
  outcome progression.

Portable snapshot reconstruction is construction-time compatibility work on
the owner root; it is not a `SignalBranchMutationPort` operation.

Every operation progresses from owner admission through exact-basis preflight,
target-cell admission, branch-local execution, an owner outcome, and a refreshed
basis only after movement.

With `test-operation-control`, `SignalOwnerOperationBoundary` freezes the real
seams as `OwnerLifecycleAdmission`, `BranchRegistryLookup`,
`BranchRegistryReservation`, `ExactBasisPreflight`, `TargetCellAdmission`,
`BeforeCanonicalMovement`, `AfterCanonicalMovement`, `ForkSourceCapture`,
`ForkDestinationInstallation`, `OutcomeConstruction`, and `OwnerCloseBatch`.
The names are absent from default builds and production bundles expose no test
control accessor.

The service must reuse the canonical Signal transaction, fork, snapshot, and
restoration engines. It must not reproduce their rules in a coordinator or
maintain a second graph to make sharing convenient.

### Same-branch serialization

One branch execution cell owns the mutable state needed to execute against
that branch. The cell compares the complete admitted expected basis while it
holds that branch's movement exclusion. Two operations racing from the same
basis cannot both move the branch. One may perform; every loser observes the
new canonical basis and returns the existing typed stale/no-movement posture.

The exclusion scope ends before an owner outcome is handed to external code.
No guard or borrowed mutable runtime escapes in a public artifact.

### Cross-branch independence

Unrelated branch cells are independently synchronizable. Shared immutable
schema, compiled plan, configuration, and diagnostic resources may be
reference-counted. Shared mutable registries may use short metadata critical
sections for lookup, identity reservation, insertion, and removal, but they
may not cover transaction execution or a caller mutation callback.

Fork needs a short source-basis validation and destination identity/insertion
sequence. The new destination receives owner-managed branch state without
holding source execution exclusion across later destination work. If exact
source capture requires a source guard, it is held only for the canonical
capture and released before any external call.

An internal single worker, actor mailbox, global executor lane, or mutex around
all branch cells does not satisfy this contract even if the public port is
cloneable.

### Runtime context

The caller continues to supply the operation's mutable runtime context where
the existing transaction contract requires it. The service does not retain,
clone, globally register, or reuse that context after the synchronous call.
Context belonging to one request therefore cannot become a hidden owner-wide
lock or cross-request ambient selection source.

## Signal Branch Lifecycle Port

`SignalBranchLifecyclePort` exposes canonical retirement and owner-lifecycle
inspection with `&self`. Retirement consumes the exact linear authority
required by the existing contract and synchronizes only with the target branch
plus short registry updates.

Retirement is denied while an exact basis remains protected by a live owner
retention obligation. Product-branch retirement in 9.17.2 therefore releases
its Runtime World pin first and asks Signal to retire only when explicit
component custody permits it. Dropping a product branch is never permission to
delete a Signal branch that is still shared by another product commit, branch,
observation, attempt, or historical pin.

Batch retirement, if exposed through this port, is bounded and processes
targets in canonical identity order. It may report per-target terminal
outcomes; it may not hold all branch execution cells while performing work or
turn batch size into an unbounded pause for unrelated branches.

## Internal State Partition

The canonical Signal owner state is partitioned by responsibility:

- immutable runtime definition resources shared by every branch;
- one branch registry responsible only for canonical membership and identity;
- one independently synchronizable execution cell per live branch;
- the existing retention registry and lease terminal accounting;
- owner lifecycle state; and
- bounded diagnostics/counters.

Each live branch identity/lifecycle incarnation has exactly one canonical
execution cell. Moving state out of the monolithic runtime value must
remove the old direct field authority or make it delegate to the same cell; it
must not mirror branch state between two mutable homes.

Lock ordering is explicit and mechanically reviewed:

1. owner lifecycle admission;
2. short branch-registry lookup or reservation;
3. one target branch execution cell;
4. short retention or diagnostic accounting update.

No operation may hold a later item while attempting to acquire an earlier one.
No operation may hold two existing branch execution cells simultaneously.
Fork captures the source, releases it, then installs the destination through a
registry reservation. Restoration replaces only the target cell under the
same exact-basis comparison.

## Cancellation, Panic, And Capability Loss

Cancellation is checked before branch-local movement starts. At that point it
returns an existing or new typed no-movement denial. Once canonical Signal
movement linearizes, the owner-issued performed outcome wins; a late
cancellation flag may be descriptive but cannot erase movement.

Signal transaction panic behavior remains governed by the existing transaction
rollback contract. This milestone additionally proves that panic containment
is branch-local: an unwinding caller cannot leave a registry reservation,
branch-cell lock, or lifecycle obligation that blocks unrelated branches.
Mutex poisoning, if the chosen synchronization primitive exposes it, must be
converted into an owner-defined terminal or recoverable posture. Blind
`unwrap()` and repository-wide poisoning are forbidden.

The synchronous call owns no detached continuation. If a caller drops the
returned performed artifact, canonical branch truth remains observable through
the basis port. If an operation requires a preinstalled owner recovery record,
that record is bounded, owner-owned, and discoverable without the original
caller capability.

## Capacity And Cost Contract

All owner-managed registries install explicit bounds before this milestone is
closed:

- maximum live Signal branches;
- maximum in-flight branch reservations;
- retention entries and leases under the 9.17.1.1 policy;
- bounded diagnostic events; and
- any recovery records introduced by the service refactor.

Operational capacity denial occurs before canonical branch movement. Fork
reserves its destination identity and registry capacity before source capture
can create an externally relevant obligation. Diagnostic exhaustion is
different: it increments an exact dropped/aggregated-event counter and exposes
a typed diagnostic omission without denying or changing an otherwise lawful
owner operation. No service adds an unbounded request queue.

Ordinary work is target-local; fork adds source capture and bounded insertion,
retirement adds retention accounting, and close uses bounded batches. Storage
changes must preserve non-forking performance, not merely cheap fork counters.
Freeze workload setup (including preallocation), measured boundaries, warmups,
repetitions, and budgets before refactoring. Compare complete affected
families on pre-milestone and final release trees using identical harnesses,
toolchains, hardware, effective configuration, and ordering. Record actual
settings; isolate allocation probes from timing. Preserve workload assertions
and absolute bounds alongside relative regression budgets. Stale goldens,
regenerated baselines, partial captures, and selected successes cannot establish
preservation; no aggregate may hide a failing case.

Measure elapsed distributions, allocation calls/bytes, structural work, and scoped
peak live bytes separately from end-live bytes. Peak means observed high-water
mark, not end-live or cumulative allocations; missing instrumentation means
unavailable, never zero. Byte claims require exact owner-owned accounting;
scoped allocation is not total owner memory.

The public diagnostic `SignalOwnerServiceCostSnapshot` exposes at least
`owner_upgrade_attempts`, `branch_registry_lookups`,
`branch_registry_reservations`, `branch_registry_entries_scanned`,
`target_cell_contacts`, `target_cell_waits`, `canonical_movements`,
`retention_registry_contacts`, `fork_source_captures`,
`fork_destination_installations`, `forked_mutable_graph_nodes_copied`,
`diagnostic_events_recorded`, `diagnostic_events_dropped`, and `close_batches`.
Counters are updated at the work sites they describe rather than reconstructed
by inspection. The certification requires these exact deltas:

- one single-branch operation contacts exactly one target cell and zero
  unrelated cells;
- one same-basis race records exactly one canonical movement;
- an unrelated operation causes zero contact or wait deltas on the parked
  branch;
- an exact fork performs one source capture and one destination installation
  while `forked_mutable_graph_nodes_copied` remains zero; and
- ordinary observe, advance, fork, restore, and retire perform no scan whose
  breadth grows with total live branches, including exactly zero
  `branch_registry_entries_scanned`.

## Public Facade And Compatibility

The stable composition-facing exports live under `worth_signal::facade::branch`.
They include the three concrete ports, owner-unavailable posture, and only the
existing branch basis, request, outcome, denial, retention, and lifecycle types
needed to call them.

Existing `SignalRuntime` convenience methods may remain for compatibility, but
they must delegate to the same canonical owner services and branch cells. They
must not retain separate direct mutation authority. A compile-time facade test
proves Runtime World can import the required public types without reaching
through `logic`, `data`, or private branch-state modules.

No `dyn SignalBranchService`, consumer-defined adapter trait, erased
`Any` payload, string operation kind, or raw `(branch_id, generation)` tuple is
part of the stable boundary.

## Destination Topology

The correction lands in the two component owners, never Query, Bridge, or
Runtime World:

```text
crates/worth-relational/
  src/branch/owner_services/
    mod.rs                              [assembly only]
    basis_port.rs                       [observe/readmit/retain/release]
    lifecycle_port.rs                   [archive/delete/owner status]
    service_ports.rs                    [concrete bundle issuance]
  src/facade/branch.rs                  [extend curated exports]
  tests/relational_certification/
    owner_service_completion.rs         [extend existing integration target]
  OWNER_COMPONENT_PORT.md               [revise frozen audience guide]

crates/worth-signal/
  Cargo.toml                           [add test-operation-control feature]
  src/
    branch/
      owner_services/
        mod.rs                         [module assembly only]
        owner.rs                       [SignalRuntime port issuance]
        lifecycle_observation.rs       [public owner lifecycle observation]
        lifecycle_state.rs             [owner open/closing/closed state]
        branch_registry.rs             [canonical membership and reservations]
        branch_execution_cell.rs       [per-branch synchronization boundary]
        basis_port.rs                  [observation/readmission/retention port]
        mutation_port.rs               [fork/advance/capture/restore port]
        lifecycle_port.rs              [retirement and owner status port]
        unavailable.rs                 [typed lost-owner posture]
        counters.rs                    [bounded structural accounting]
        operation_control.rs           [feature-gated real progression seams]
      ...                              [existing canonical branch vocabulary]
    facade/
      branch.rs                        [curated owner-service facade]
  tests/
    signal_owner_services.rs           [one intentional integration target]
    signal_owner_services/
      world.rs                         [court assembly only]
      world/
        definition.rs                  [semantic cargo-routing inputs]
        compiler.rs                    [public-facade production compilation]
        observation.rs                 [neutral public observations]
      oracle.rs                        [pure oracle assembly only]
      oracle/
        state.rs                       [test-local semantic state]
        transition.rs                  [independent expected transitions]
        comparison.rs                  [neutral-state comparison]
      cases/
        provenance.rs
        facade.rs
        fork_and_sharing.rs
        legacy_port_cutover.rs
        runtime_context_locality.rs
        lifecycle.rs
        capacity.rs
        model_sequences.rs
        cost.rs
        operation_control.rs             [feature-gated assembly]
        operation_control/
          independent_progress.rs
          same_branch_races.rs
          cancellation.rs
          panic_containment.rs
          close_race.rs
          capacity_cleanup.rs
      ui/                               [grouped compile pass/fail fixtures]
  BRANCH_BASES.md                       [revise existing owner guide]
  OWNER_SERVICES.md                     [new concurrency/lifecycle guide]
  examples/
    independent_branch_services.rs      [executable owner workflow]

scripts/ci/
  run_signal_owner_service_selection.sh [create: fail zero-test named lanes]
```

Split files at semantic boundaries to meet the line cap; never collapse this
tree into `services.rs`, `helpers.rs`, `common.rs`, `util.rs`, or catch-all modules.

## Dependency Enforcement

This milestone adds no dependency from either owner to:

- `worth-runtime-world`;
- `worth-runtime-bridge` for composition behavior;
- any `worth-query-*` crate;
- Store, replay, correction, or cert crates; or
- any `worthy-*` crate.

Boundary checks also reject consumer-defined owner traits and descriptor-based
authority reconstruction.

## Adversarial Courtroom

Park Signal A after cell admission, before movement; finish unrelated B before
releasing A. Race admissible, effectful requests from one basis: exactly one
moves. Public observations expose shadow state, global locking, leaked capacity,
owner clones, and cross-branch poison. Exercise Relational through its bundle.

## Phase Progression

Contract evolution alternates serial gates and bounded parallel waves. Gates
alone edit files shared by lanes, top-level manifests/crate roots, facades, or
shared phase/outcome types. Named lane exceptions below are additive and exclusive.
Each lane owns its subordinate assembly/test registration and ships
behavior or independent evidence, not placeholders or merge-only work. Contract
defects return to one serial gate, never adapters, aliases, duplicate traits,
or compatibility lanes.

Freeze exact compiler-visible signatures, types, ownership, denials, visibility,
and module ownership before dependent source work. Lanes may implement during prerequisite verification but cannot integrate until it passes.
Contract-preserving fixes invalidate only affected proofs; a contract break stops integration for one serial synchronization/rebase and invalidates dependent approvals.
Stubs, private-cell tests, and worker summaries never certify public services.

### Phase 1: Serial owner-contract gate

- freeze exact Relational bundle and Signal basis/mutation/lifecycle signatures
  in owner-guide matrices, plus owner/cell contracts, denials, counters, and
  operation-control seams;
- create permanent shared vocabulary, feature/facade assembly, current fences,
  and exclusive shared-file ownership; and
- assign subordinate roots, future source fences, and focused commands below.

### Phase 2: Parallel owner foundations

- **Relational lane:** own all `worth-relational/branch/owner_services` files
  plus causal `OwnerUnavailable` additions to the three existing denial enums;
  add only their `pub use` line, exact facade entries, and
  `#[path = "relational_certification/owner_service_completion.rs"] mod owner_service_completion;`; run
  `cargo test -p worth-relational --features test-operation-control
  --test relational_certification owner_service_completion`;
- **Signal kernel lane:** own Signal `owner_services/mod.rs`, lifecycle state,
  registry, execution cell, unavailable posture, counters, and their unit tests;
  run `cargo test -p worth-signal --lib branch::owner_services::`;
- each lane first proves its filter lists at least one lane-owned test;
- converge serially only if either lane exposes a defect in a frozen contract.

### Phase 3: Serial Signal-kernel gate

- integrate the kernel behind the sole non-cloneable `SignalRuntime` root;
- prove independent cell progress, real fork sharing, and ordinary-cost preservation before service migration;
- freeze cell operations and port inputs/outcomes, then pre-register the basis, mutation, and lifecycle port slots in `owner_services/mod.rs`; and
- leave facade aggregation and old-method delegation under one gate owner.

### Phase 4: Parallel Signal services

- **Basis lane:** own basis observation, readmission, retention, release, and
  their capacity behavior;
- **Mutation lane:** own fork, advance, capture, restore, refreshed bases, and
  same-branch one-winner behavior; and
- **Lifecycle lane:** own retirement, owner close, lease terminals,
  cancellation, panic containment, and caller-capability loss.

All lanes consume the frozen cell API without facade or assembly edits. Capacity
denials remain pre-effect and old runtime methods remain untouched until convergence.

### Phase 5: Serial facade and behavior freeze

- assemble `SignalOwnerServicePorts`, issue weak ports, and convert existing
  runtime conveniences into delegates to the same owner engines;
- create `tests/signal_owner_services.rs` with a real public-facade smoke so Phase 6 lanes append only their owned modules;
- resolve any contract revision once at its canonical owner and rerun every
  dependent focused suite; and
- freeze the exact concrete Relational and Signal facades consumed by 9.17.2.

### Phase 6: Parallel certification and documentation

- **Relational closure lane:** own the focused bundle proof and owner guide;
- **Signal world lane:** own public-facade compilation, neutral observations,
  baseline cases, lifecycle cases, and facade cases;
- **Independent oracle lane:** own pure state, transitions, comparisons, and
  model sequences without production semantic helpers; and
- **Adversarial lane:** own operation-control schedules, compiler fixtures,
  boundary source rules, exact-selection script, capacity/cost cases, executable
  example, and Signal owner guide.

### Phase 7: Serial closure gate

Frozen contracts gate source; shared prerequisites gate integration; the complete court gates acceptance/commit.
Correction review covers only `last-known-good..candidate`, its owned files, touched shared seams, and affected proofs.
Reconstruct the complete surface only at Phase 5 convergence, a contract-breaking synchronization, and here: assemble test roots; run behavioral, concurrency,
compiler, feature, scale, format, lint, line-cap, boundary, and context checks; structurally review the final diff; then freeze the 9.17.2 handoff.
Review-only or merge-only lanes do not count as implementation.

## Test Evidence Architecture

Before implementation, freeze owner-guide matrices from this spec and
inherited APIs, never implemented methods. Map each required method, receiver,
input, outcome, and canonical owner to named public-facade cases and executable
lanes. Cases prove healthy results, reachable typed denials, exact observable
effects, and cleanup. `Ok`, counters, or agreement between delegates alone cannot
prove behavior; no-op and always-deny implementations must fail. Unimplemented
methods remain mandatory at closure.

Integration evidence follows the Relational Supply Chain and Query Bank courts:

- the production side is causally compiled through
  `worth_signal::facade::branch` and executes the real `SignalRuntime` owner;
- the expected side is a separately authored, pure semantic oracle;
- observations cross public owner ports and are converted into neutral
  test-local values before comparison;
- hostile twins differ from a healthy case in one relevant fact;
- setup, compilation, owner execution, observation, and oracle disagreement
  have distinct failure postures so a setup failure cannot make a denial test
  green for the wrong reason; and
- every case receives a fresh owner and ends by releasing observations and
  leases, closing the owner, and proving every registry is empty or in its
  documented terminal posture.

Production-path comparisons are not oracles. The oracle uses only test-local
semantic identifiers, ordinary maps/sets, and independent transition rules;
never the Signal evaluator, production graph traversal, basis comparator,
private branch state, or transition logic.

## Canonical Signal Court World

`CargoRoutingSignalCourt` is the canonical production-valid world. It declares
through the public Signal facade a small cargo-routing graph whose inputs
include storm severity, berth availability, and manifest clearance, and whose
derived outputs include voyage dispatchability, medical-cargo release, and
inspection requirement. The semantic names make a wrong result diagnosable;
arbitrary numbered nodes are insufficient.

The compiler installs that declaration in a real `SignalRuntime`, admits an
exact starting basis, and forks two branches from that same basis. The storm
branch changes weather and route posture. The maintenance branch changes berth
posture. Public observations expose semantic output values, branch identity,
lifecycle incarnation, generation, and the structural counters relevant to the
case. They do not expose private cells or turn diagnostics into authority.

`CargoRoutingSignalOracle` independently predicts outputs and generation/
lifecycle transitions from semantic inputs. Compiler and oracle share no
builder, transition, comparison helper, or production representation. Prove
the healthy baseline before each hostile change.

## Required Owner-Service Scenario Families

The Signal target and existing Relational certification target own these
families:

1. **Relational port matrix:** call observe, readmit, retain, release, archive,
   delete, pending deletion, and owner-status through the new ports. For each
   operation, prove the healthy result, exact foreign/stale/terminal denial,
   canonical runtime visibility, weak-owner loss, and equivalence with its
   compatibility delegate. Park one branch-local lifecycle operation and prove
   an unrelated branch basis operation completes.
2. **Signal facade provenance and baseline:** prove the declaration, admitted
   basis, public observation, and oracle agree. Deny foreign or equal-looking
   owner, branch, definition, lifecycle, generation, and restored-snapshot
   substitutions before movement with the exact typed reason, not generic error
   presence. Same-Rust-type foreign-owner substitutions require runtime proof.
3. **Independent-progress matrix:** park advance, fork capture, snapshot capture,
   restoration, retirement, and close-drain on A at every reachable branch-work
   seam after short metadata guards release. Before releasing A, each lawful
   basis/mutation/lifecycle operation on B must produce its expected result and
   observable effect, with zero A contact/wait/movement. Rejection is not progress.
   During close, already-admitted B completes; new admission denies. B cannot
   release A. Widening metadata exclusion across branch work must fail.
4. **Same-branch race matrix:** from the same admitted posture run
   advance/advance, advance/restore, advance/retire, restore/restore,
   restore/retire, snapshot/advance, and observe/retire. Mutating pairs produce
   exactly one movement when both requests are independently admissible and
   effectful; prove that precondition with uncontended twins. Retention-blocked
   retirement is a separate typed-denial case. Losers preserve exact denial
   reasons; observers receive complete pre- or post-state, never mixed state.
   Force both legal winner orderings; each ends with a healthy follow-up.
5. **Legacy/port cutover:** interleave each remaining public
   `SignalRuntime` convenience operation with the corresponding port operation
   against one branch. Movement through either surface immediately stales the
   other's prior basis and is visible through both observation routes. Race one
   legacy call with one port call and require one winner. This family must fail
   if either surface addresses separate mutable state or bypasses the cell.
6. **Fork and sharing:** fork the exact source basis into a distinct owner-issued
   branch sharing immutable storage. Populate each carried state family;
   empty-container sharing proves nothing. Capture/installation each count once;
   copied mutable graph nodes stay zero. Compare allocation identities and
   measure the whole public fork at every declared graph scale, including
   reservation, capture, and installation; container-only probes cannot substitute.
   Allocation has no node-count slope. Mutate each branch and assert both its
   expected changed oracle values and its sibling's unchanged values. First-write
   copying obeys the declared changed-state granule, never deferred whole-graph
   cloning. An eager-copy twin violates the same allocation bound.
7. **Cancellation cutoffs:** at every cancellable pre-movement boundary for
   advance, fork, restore, and retire, cancel and prove no canonical effect,
   released reservations, and an immediate healthy twin. At every
   post-linearization boundary, cancellation is descriptive and the exact
   owner-issued performed result wins.
8. **Panic containment:** inject a panic during transaction execution, fork
   capture, restoration, and outcome construction after movement where each is
   reachable. Prove the documented target posture, complete reservation/guard
   cleanup, and unrelated progress. No runtime-wide poison, missing canonical
   truth, leaked reservation, or stranded waiter is permitted.
9. **Lifecycle and capability loss:** drop the Relational and Signal owners
   while every port and lease kind remains. Prove weak ports cannot prolong the
   owner, strong leases retain only their documented obligation, drop is
   terminal, and later calls return one stable unavailable posture. Race Signal
   close against every admitted operation family; the result is exact no-effect
   or a complete owner outcome, never lost work or an indefinite waiter.
10. **Operational-capacity cleanup matrix:** independently exhaust branch,
    reservation, retention, operation, and recovery bounds. Cross every
    reserved resource with denial, cancellation, injected panic, owner close,
    and caller-capability loss where reachable. Each cell proves pre-effect
    denial or the documented performed posture, exact capacity restoration,
    zero unexplained registry entries, and an immediate healthy operation.
    Diagnostic exhaustion instead proves typed omission and exact drop counts
    while a lawful owner operation still performs.
11. **Facade and compiler:** compile-pass every public port method through the
    concrete owner facades with required `&self`, `Send + Sync`, and generic
    bounds. Compile-fail local-only issuance, raw identifiers, forged bases,
    basis-port mutation, retirement without linear authority, consumed-outcome
    reuse, generic marker substitution, private-cell access, default-build
    test-control names/accessors, and public port/unavailable construction.
    Each negative targets the real API, type, or construction, with a valid
    counterpart changing only the disputed condition. Test-local lookalike
    functions, missing imports, and unrelated compiler failures prove nothing.
12. **Seeded model sequences:** apply deterministic sequences of observe,
    readmit, retain/release, fork, advance, snapshot/restore, retire, close, and
    capability loss to the real owner and independent oracle. The CI profile
    must exercise every operation, every public outcome class, every lifecycle
    transition, and every ordered adjacent pair of state-changing operations at
    least once; the scheduled profile adds longer traces and concurrency.
    Reject missing reachable coverage; justify unreachable combinations from
    owner contracts rather than silently skipping them. Print the seed and
    minimal failing prefix; accounting stays test-local.
13. **Structural cost:** vary live branches, graph size, unrelated operations,
    same-branch contenders, and retained bases independently. Assert the exact
    counter deltas and zero-copy/zero-unrelated-contact rules from the capacity
    and cost contract. Court/Standard/Scale use 2/64/4,096 live branches and
    64/4,096/65,536 graph nodes, varying one axis at a time. Freeze repetitions,
    contention/retention ceilings, and allocation bounds before implementation;
    no ordinary target-work slope may depend on unrelated live branches.
14. **Runtime-context locality:** execute with distinct non-`Clone` request
    contexts carrying drop sentinels. After success, denial, cancellation, and
    panic, prove the context is immediately caller-accessible and eventually
    drops, no owner registry or outcome retains it, and a second branch uses a
    different context without cross-request state. Compiler evidence denies
    service-bundle issuance when owner-captured configuration lacks the required
    `Send + Sync + 'static` bounds.

## Lane And Harness Contract

Use one Signal integration target, `signal_owner_services.rs`, with subordinate
modules, and one grouped compiler-fixture family; no target-per-case proliferation.

- **Focused:** both public-method matrices, provenance, facade, baseline,
  sharing, legacy cutover, lifecycle, capacity, and bounded model coverage.
- **CI operation-control:** deterministic parks/races, cancellation, panic,
  capacity cleanup, close, and healthy twins under `test-operation-control`.
- **Scheduled ignored:** longer models, contention, Court/Standard/Scale, and
  affected performance profiles; fixed configurations/repetitions, p50/p95/p99,
  and structural counts.

The configuration matrix is the Cartesian product of `profile-compact`,
`profile-standard`, and `profile-extended` with `parallel` disabled and
enabled. The ordinary package regression and operation-control target run in
all six configurations; default-feature `profile-extended` also runs to catch
manifest drift. Every supported configuration preserves the same authority,
lifecycle, outcome, and cleanup topology. The scheduled Scale profile runs at
least under default features and `profile-extended,parallel`.

The operation-control feature may expose named park and fault points already
inside the real owner progression. It may not change production semantics,
mint authority, replace the owner engine, or synchronize with wall-clock
sleeps. It is disabled by default. Parks use barriers or channels, bounded
waits, and a drop-safe release guard so a failed assertion cannot hang the
suite. With no park or fault armed, the feature-enabled binary must produce the
same public outcome, observation, counter, and cleanup posture as the ordinary
binary. CI must fail if a command intended to select a required family executes
zero cases or if a declared required test becomes ignored, renamed, or absent.

Both selectors require reviewed executable rosters derived from the scenario
matrix, not discovered tests. List and run identical targets, default-feature
posture, features, filters, and ignored posture; support `--no-default-features`.
Parameterized families also assert completion of every required semantic case;
one green test containing an empty or shortened loop is insufficient.

The planned commands are:

```text
bash scripts/ci/run_relational_named_test_selection.sh --test relational_certification --selection owner_service_completion::
cargo test -p worth-signal --test signal_owner_services
cargo test -p worth-relational --all-features --all-targets
cargo test -p worth-signal --all-targets
cargo test -p worth-signal --doc
cargo run -p worth-signal --example independent_branch_services
cargo fmt --all --check
bash scripts/ci/check_owner_service_clippy.sh
bash scripts/ci/check_workspace_rust_line_caps.sh dirty
cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .
cargo run --manifest-path tools/agent-context/Cargo.toml -- check
```

For each of the six feature configurations, CI additionally runs:

```text
cargo test -p worth-signal --no-default-features --features <configuration> --all-targets
bash scripts/ci/run_signal_owner_service_selection.sh --test signal_owner_services --no-default-features --features <configuration>,test-operation-control --selection operation_control::
```

The default-feature and `profile-extended,parallel` operation-control selectors
also execute their declared ignored Scale roster with `--ignored`. Package
commands are preservation gates, not substitutes for the focused court.
Supply repeated `--expect-name` arguments from the reviewed roster to every
selector invocation above. Global test counts are not acceptance criteria.

## Wrong-Reason-Green And Sensitivity Contract

The suite must become red when a targeted negative twin or equivalent mutation
introduces any of these defects: a runtime-wide lock at any real progression
boundary, a second branch graph, a legacy method bypassing the cell, a missing
public port operation, success-shaped no-ops, raw-id authority, missing
expected-basis comparison, two movements from one basis, fork-by-copy,
cancellation overriding performed truth, leaked capacity on any terminal path,
diagnostic pressure denying a lawful mutation, owner clones hidden in ports,
unarmed operation control changing behavior, or panic poisoning unrelated branches.

Before closure, demonstrate sensitivity to success-shaped no-ops, global locking,
omitted exact-basis comparison, fork copying, and cancellation erasing performed
truth. Targeted mutations or hostile twins must reach the disputed production
boundary and fail on its required behavior, not setup or compilation. Restore
production behavior and rerun affected cases; no mutation framework or ledger.

## Structural Completion Review

Passing tests is necessary and insufficient. A reviewer independent of the
implementation inspects the complete milestone diff, including earlier phases,
and reconciles required method matrices with facade-to-owner paths, real effects,
and executed cases. Worker approvals cannot substitute. Review must establish:

- every pre-milestone public Signal branch observation, readmission, retention,
  fork, advance, snapshot, restoration, retirement, and owner-status entry point
  either delegates to the named owner service/cell operation or is explicitly
  outside this milestone's branch contract;
- every mutable branch graph, head, generation, stored-state, lifecycle, and
  reservation field has exactly one canonical owner after the refactor; old
  mutable homes are removed rather than synchronized with the new cells;
- each live branch identity/lifecycle incarnation maps to exactly one execution
  cell, and registry values cannot reconstruct or mirror branch truth;
- ports contain weak owner connectivity and diagnostic identity only; no port,
  closure, test hook, observation, or counter object strongly owns the runtime;
- facade/module/visibility and Cargo dependency review reveal no consumer trait,
  public constructor, raw authority route, global worker/queue, Runtime World
  dependency, or private-module reach-through;
- the operation-control diff adds only named parks/faults and unarmed
  accounting, with no alternate evaluator, mutation engine, authority
  constructor, or feature-dependent product semantics; and
- every added or changed production responsibility resides in the destination
  topology, every touched Rust file satisfies the line cap, and no catch-all
  module absorbed the refactor.

Final review receives the spec, integrated diff, fixtures, commands/timings, docs, and constraints.
Correction review receives only its last-known-good revision, owned delta, frozen-contract assumptions, shared-seam changes, and affected commands;
fix findings at their owner and rerun only affected evidence unless contracts changed.

## Documentation Deliverables

`crates/worth-relational/OWNER_COMPONENT_PORT.md` documents the concrete bundle,
preserves the four frozen publication-service contracts, and maps all six
ports' methods/outcomes to compatibility delegates.

`crates/worth-signal/BRANCH_BASES.md` preserves 9.17.1.1 meanings, shows basis
observation/retention through `SignalBranchBasisPort`, and explains why Runtime
World carries but cannot construct authority from descriptors or ids.

`crates/worth-signal/OWNER_SERVICES.md` owns the complete method/input/outcome/
receiver matrix and 9.17.2 handoff. Explain weak-port/root lifecycle, cell lock
ordering, same-branch serialization and unrelated progress, cancellation/panic/
close, operational capacity versus diagnostic omission, exact cost counters,
and synchronous runtime-context borrowing without retention.

The executable example must create or obtain two admitted branches, issue the
public ports, advance both branches without whole-runtime exclusive access,
retain and release an exact basis, and show typed owner unavailability after
closure. Documentation examples compile in the normal documentation lane.

## Must Ship

- complete weak Relational and Signal bundles through the curated facades;
- sole non-cloneable Signal root, canonical registry/cells, and compatibility
  delegation;
- same-branch compare-and-move, unrelated progress, lifecycle-total terminal
  behavior, bounded resources, and measured cost preservation; and
- every scenario family, compiler fence, configuration/scale lane,
  exact-selection gate, executable example, and specified owner guide.

## Must Preserve

- every closed 9.17.1 admitted-basis, branch-local MVCC, snapshot, restoration,
  and retirement meaning;
- every closed 9.17.1.1 exact-retention and terminal-lease guarantee;
- current Signal transaction rollback and owner-issued outcome semantics;
- pure domain placement and one-way dependency direction;
- existing public convenience behavior except where an old ambiguity must
  become a typed denial; and
- the rule that Signal never decides product currentness.

## Non-Goals

- composite commits, product branches, or coordinated publication;
- Query carriage, public Query facade work, or outbox dispatch;
- persistence, restart, replay, correction, or merge;
- asynchronous Signal workers, actor systems, or distributed scheduling;
- a new generic component-owner framework;
- approximate total-memory accounting; and
- deletion of compatibility methods merely to make the new ports appear sole.

## Acceptance Gate

Milestone 9.17.1.2 closes only when:

- both complete public method matrices execute through concrete ports using
  `&self`, with issuance/receiver bounds compiler-checked and no whole-runtime
  borrowing or global mutex;
- every independent-progress matrix cell completes deterministically, and every
  same-branch race satisfies its exact movement/observation outcome;
- mixed legacy/port sequences and races prove both public routes use the same
  owner cells;
- the structural completion review proves one canonical branch graph, one cell
  per live lifecycle incarnation, weak-only ports, and no mirrored authority;
- cancellation, panic, owner close, capability loss, and every operational
  capacity terminal cross-product are lifecycle-total with exact cleanup;
- diagnostic exhaustion preserves lawful owner behavior and exposes typed
  omission rather than masquerading as operational denial;
- model-sequence semantic coverage reaches every operation, public outcome,
  lifecycle transition, and ordered state-changing adjacency required above;
- counters and independent measurements prove sharing, bounded first writes,
  zero unrelated work, and ordinary-cost preservation;
- request runtime contexts are synchronously borrowed and never retained;
- documentation and the executable example match the public facade;
- every focused, exact-selection, six-configuration package and
  operation-control, default-feature, documentation, example, scheduled Scale,
  formatting, lint, dirty-line-cap, boundary, and generated-context command has
  actually run and passed on the final integrated tree, except independently
  established out-of-scope baseline debt under the rule below;
- all specified world, oracle, harness, sensitivity, and teardown contracts
  hold; and
- final implementation-diff review finds no material defect or unresolved
  finding against the authority, lifecycle, topology, or evidence contracts.

Missing, skipped, timed-out, zero-selected, or unrun required evidence leaves
the milestone open. Prior green revisions, worker approvals, and phase summaries
cannot close it; corrections invalidate affected results. Closure names the
final integrated revision, commands/configurations, terminal results, unresolved
debt, and residual risk. Baseline failures require unchanged reproduction and
independent out-of-scope judgment; they never count as passes. Neither test
counts nor reviewer confidence guarantees completeness.

### Closure Record

Revision `95c9aa7455` passed the final integrated acceptance gate. Relational's
exact 14-test owner-service roster passed with its full all-features/all-targets
suite. Signal's exact 40-test operation-control roster passed in default,
compact, standard, and extended profiles, both serial and parallel where
available; the default and extended-parallel scheduled Scale cases also passed.
The restored six-profile serial/parallel proof asserted real parallel dispatch
and equal operational digests. Lifecycle, race, independent-oracle, terminal,
documentation, example, formatting, scoped Clippy, dirty line-cap, boundary,
generated-context, Query, Runtime Bridge, and public-contract workspace gates
all passed on the final tree.

Candidate review also repaired pre-existing but in-scope-adjacent Runtime Bridge
Clippy findings, one stale compile-fail diagnostic, and the Bank producer
allowlist. The public `workspaces/worth-contracts` inventory is tracked; private
CAD paths are absent from live automation and tooling. An empty `mirrored_docs`
set is intentional: `road1.toml` is the sole machine authority. There is no
unresolved in-scope debt or known residual defect. Subsequent CI preserves this
closure; it is not deferred evidence required to obtain it.

## Exact Handoff To Milestone 9.17.2

Milestone 9.17.2 receives only:

- `RelationalOwnerServicePorts`, aggregating the four frozen services plus the
  new exact basis and lifecycle ports;
- `SignalBranchBasisPort` for exact observe/readmit/retain operations;
- `SignalBranchMutationPort` for exact fork, advance, capture, and restore;
- `SignalBranchLifecyclePort` for explicit retirement and owner status;
- the unchanged owner-issued admitted bases, requests, outcomes, denials,
  snapshots, retention leases, and lifecycle artifacts from 9.17.1/.1.1; and
- documented structural counters and installed capacity bounds.

Runtime World may order concrete owner calls, retain outcomes, correlate them
with one attempt, and release exact leases. It may not clone component state,
define an adapter trait, hold its lock across an owner call, promote a
descriptor, or repair either owner contract in composition code.
