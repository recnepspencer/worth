# Milestone 9.17.1.1: Owner-Port Concurrency And Lifecycle Closure

> **Status:** Closed on 2026-08-29. Every phase is implemented, independently
> reviewed, and verified on the integrated tree. Its corrected Relational,
> retention, and lifecycle surface is frozen in the handoff below and,
> normatively, in `crates/worth-relational/OWNER_COMPONENT_PORT.md` and
> `crates/worth-signal/BRANCH_BASES.md`.
>
> **Historical posture:** Milestone 9.17.1 remains closed and historical. This
> corrective sub-milestone repairs the current owner-component contract exposed
> by independent post-closure review; it does not amend, re-score, or recreate
> the 9.17.1 phase record.
>
> **Successor clarification:** Milestone 9.17.1.2 ports the remaining direct
> Relational basis/lifecycle calls and Signal operations before composition. It
> preserves every meaning frozen here; 9.17.2 consumes only those concrete
> owner bundles through `worth-runtime-world`.

## Goal

Make the owner-component boundary delivered by Milestone 9.17.1 honest enough
for Runtime World composition:

- every Relational owner operation used by composition is independently
  borrowable and cannot serialize unrelated branches through
  `&mut RelationalRuntime`;
- every Relational publication reserves owner-managed, bounded,
  runtime-recoverable settlement state before component movement;
- Signal can retain an exact admitted component basis after that basis ceases
  to be the current branch reference;
- dropping an external Signal retention lease terminates its owner obligation
  instead of leaking private lease identity and branch capacity;
- the concrete Relational publication authority carrier is reachable only
  through the stable public facade while remaining privately mintable;
- publication contention, feature-gated cancellation, structural-sharing scale,
  executable examples, and owner documentation are real maintained evidence;
  and
- defensive publication and inspection surfaces return typed truth rather than
  relying on an unreachable fallback, ambiguous metric, or stale public name.

The corrected ordinary progression is:

```text
exact admitted Relational basis
    -> independently borrowable preparation service
    -> opaque prepared candidate
    -> bounded runtime-owned settlement reservation installed
    -> independently borrowable compare-and-publish service
         no movement -> typed terminal outcome and reservation cleanup
         performed   -> the preinstalled record remains recovery-addressable
    -> independently borrowable settlement service
         settled     -> ordinary commit completion
         deferred    -> bounded runtime-owned recovery remains addressable

exact admitted Signal basis, current or historical
    -> exact-target external retention obligation
    -> explicit release receipt or drop-governed terminal release
```

Neither a lost capability nor an unrelated branch may become a hidden global
coordination or permanent-lifetime failure.

## Roadmap Placement

Milestone 9.17.1.1 consumes the closed 9.17.1 owner-basis, immutable-root,
prepared-candidate, publication, retention, lifecycle, cancellation, Signal
cutover, and Supply Chain certification contracts. It preserves their product
meaning while repairing implementation and evidence that contradict their
explicit concurrency, lifecycle, facade, and documentation laws.

Milestone 9.17.2 may not begin production implementation until this corrective
milestone and Milestone 9.17.1.2 close. Runtime World relies on the ability to:

- hold independently usable Relational owner services while coordinating
  Signal work;
- retain a carried Signal basis even after an owner-local Signal advancement;
- survive loss of a Relational performed capability and continue settlement by
  owner-issued commit identity;
- release every component-retention obligation on all product-publication
  outcomes; and
- name the complete concrete owner authority and outcome surface through the
  audience facade.

This milestone does not introduce composite history, product branches, Query
carriage, persistence, restart recovery, semantic merge, or a physical runtime.
Those ownership boundaries remain unchanged.

## Central Claim

Every 9.17.2-facing Relational preparation, fork, publication, and settlement
operation is independently borrowable, and every corrected Relational or Signal
owner operation is lifecycle-total: unrelated Relational branches remain
expressibly concurrent, an exact admitted Signal basis can be retained
independently of currentness, and losing a public linear capability cannot
create unrecoverable owner state. This milestone does not claim that Signal's
existing mutation engine becomes concurrently borrowable.

The claim is false if any of the following remains possible:

- safe Rust requires exclusive access to the whole Relational runtime to
  prepare, fork, discard, settle, or complete ordinary branch work;
- pausing one branch during preparation or settlement prevents another branch
  from completing an ordinary commit;
- a Signal basis is rejected for external retention solely because its branch
  has advanced to a different current observation;
- dropping a Signal external lease leaves a private registry entry that no
  caller can release;
- dropping `PerformedRelationalCommit` leaves a performed-but-unsettled route
  for which runtime repair returns `RecoveryUnavailable`;
- pending-settlement capacity is discovered only after Relational movement;
- a caller must import a private Relational module to name the concrete
  publication authority carrier;
- the documented scheduled and feature-gated courts compile but execute in no
  automation lane;
- the patch-position contention outcome can regress without a deterministic
  losing-path test;
- a rootless publication cell substitutes the candidate's expected root and
  later panics; or
- public guides, names, snippets, and cost fields describe a different contract
  from the facade and runtime.

## Current Boundary And Confirmed Defects

### Relational owner services

`RelationalPublicationPort::compare_and_publish` is independently borrowable,
but the phases around it are not. `prepare_branch_transaction`,
`discard_prepared_candidate`, `observe_fork_source`, `fork_branch`, and
`settle_performed_publication` require `&mut RelationalRuntime`. The ordinary
commit facade therefore passes through exclusive-runtime preparation and
settlement even when selected branches are unrelated. Existing locality
evidence prepares candidates serially before it starts concurrent publication,
so it cannot convict this borrowing-based serialization.

The global patch-position allocator is already a bounded nonblocking mechanical
reservation. Its `PatchPositionReservationContended` outcome consumes the
candidate and correctly releases prepared residue, but no test forces that
outcome and the owner guide does not state that retry begins from a fresh
transaction and fresh preparation.

### Performed publication settlement

The canonical route becomes performed and requires settlement before
`PerformedRelationalCommit` is returned. The runtime registers deferred
settlement only after a caller invokes settlement and a durability step fails.
If the performed witness is dropped before that call, the branch remains on a
route requiring settlement, descendant preparation is denied, and repair by
commit identity reports `RecoveryUnavailable` because no runtime recovery
record exists.

`#[must_use]` is useful caller guidance but is not lifecycle ownership. The
runtime must own recovery before the performed capability escapes.

### Signal exact retention

`retain_signal_component_basis` compares the supplied admitted basis with the
current live observation and returns `StaleBasis` after ordinary branch
advancement. External component residency is not a currentness claim. A basis
that remains exactly admitted and available must be retainable for 9.17.2 even
when the mutable branch reference has moved.

`SignalBranchRetentionLease` contains only runtime, lease, and branch ids. It
does not retain a registry binding and has no `Drop` implementation. Explicit
release is therefore the only terminal path, and ordinary Rust drop loses the
only unforgeable private lease id while leaving capacity and retirement state
live.

### Facade, defensive behavior, evidence, and documentation

The concrete `RelationalBranchPublicationAuthority` alias exists behind the
private `branch` module but is omitted from `facade::branch`. The marker is
privately mintable as intended, but the canonical public carrier cannot be
named by the audience promised in 9.17.1.

The scheduled Relational CI lane executes Scale admission but omits the
documented 4,096-fork and retained-history-ceiling proofs. The documented exact
filters also name nonexistent module paths; the compiled test names are
`root_fork_sharing::...` and `root_cost_scale_axes::...`.
`test-operation-control` gates the production-boundary cancellation courts, but
no repository automation enables it.

The owner documentation overstates deletion waiting, understates candidate
consumption after mechanical deferral, and implies whole-commit concurrency
while only publication is independently borrowable. Signal Markdown snippets
are not compiled. `inspect_branch_sharing` drifts from the frozen
`observe_branch_sharing` owner-port name. `RelationalBranchSharingObservation`
does not define the scope of `branch_metadata_bytes`. Publication currently
fabricates an observed root from the candidate's expected root if a selected
cell is rootless, converting an invalid owner state into a later panic.

## Ownership And Authority Lock

| Responsibility | Owner | Required artifact or service | Cannot authorize |
|---|---|---|---|
| Relational preparation | Relational | `RelationalPreparationPort` bound to one live runtime owner | Publication or settlement by itself |
| Relational branch fork | Relational | `RelationalForkPort` and owner-issued fork-source basis | Cross-runtime or cross-branch substitution |
| Relational publication | Relational | Existing `RelationalPublicationPort` and concrete publication authority | Durability settlement or composite currentness |
| Relational settlement and repair | Relational | `RelationalSettlementPort` plus runtime-owned pending-settlement record | Replaying or republishing a component commit |
| Exact Signal component retention | Signal | Exact-target `SignalBranchRetentionLease` backed by Signal owner state | Signal currentness, mutation, restore, or fork |
| Portable basis/reference vocabulary | Foundational | Existing descriptive descriptors and mismatch axes | Any runtime operation |
| Owner-specialized authority carrier | Proof plus private Relational marker | Public carrier alias, private witness issuance | Caller-selected generic authority |
| Composite publication | Runtime World in 9.17.2 | Not created here | Component truth or owner-local settlement |

All new ports are cloneable bindings to the narrow owner subsystems they use.
They carry runtime affinity and owner lifecycle but no raw runtime reference,
ambient branch, generic authority marker, or composite state. Dropping the
runtime closes admission; existing ports then return typed owner-unavailable
outcomes without opening a second authority lane.

## Adversarial Courtroom

### Production boundary

The decisive Relational court uses only the public `worth_relational::facade`
owner workflow and the production Supply Chain world. The Signal court uses
only `worth_signal::facade`. Tests may use narrow pause/fault hooks to force a
schedule, but the action, authority, state transition, cleanup, and observation
remain production implementations.

### Hostile sequence

1. Install the Court Supply Chain world and fork `storm` and `maintenance`
   Relational branches from one exact immutable root.
2. Obtain independent preparation, publication, fork, and settlement services
   from one runtime through shared access only. Compile-time evidence also
   proves each service is `Clone + Send + Sync` without making the mutable
   runtime itself the shared capability.
3. Pause `storm` separately during preparation, branch publication, and
   settlement. During each pause, complete the corresponding ordinary work on
   `maintenance` from another thread.
4. Pause one unrelated prepared candidate while it owns the patch-position
   reservation, invoke the other, and require one performed result and one
   `Deferred(PatchPositionReservationContended)`. Settle the winner before
   observing final residue.
5. Perform another Relational publication, drop its
   `PerformedRelationalCommit` before explicit settlement, verify that a child
   is blocked by the unsettled parent, repair by commit identity through the
   settlement service, then prepare and commit the child successfully.
6. Race immediate settlement through the performed witness against repair by
   commit identity. Both callers must converge on one exact terminal receipt,
   while exactly one durability append and one derived completion execute.
7. Repeat with a durability fault after settlement begins; drop every external
   error capability and recover from the same runtime-owned record.
8. Observe Signal basis `S0`, advance the same Signal branch to `S1`, acquire an
   external lease for still-admitted `S0`, release the ordinary admission for
   `S0`, and prove exact readmission remains available through the external
   obligation.
9. Exercise both Signal terminal paths: explicit release returns its typed
   receipt; dropping a second lease releases the obligation through owner
   accounting. Then retire/reclaim and prove neither lease leaves capacity or
   branch residue.
10. Attempt foreign-runtime, foreign-branch, unavailable-target, double-release,
   owner-loss, and capacity-exhaustion variants at their real entry surfaces.

The rootless selected-cell defense is not a public-world state and therefore is
not mislabeled end-to-end evidence. A focused owner-boundary fault-injection
court separately removes the selected root immediately before comparison and
requires a typed no-movement failure rather than expected-root substitution or
panic.

### Required independent observations

The court must establish:

- the public API can express simultaneous preparation and settlement without
  `unsafe`, a mutex around `RelationalRuntime`, or sequential preconstruction;
- a paused branch adds exactly zero coordination contacts or waits to the
  unrelated branch at every branch-local phase;
- global allocator contention is separately counted, bounded, nonwaiting, and
  never mislabeled as branch coordination;
- the contention loser moves no reference and leaves zero candidate, canonical
  route, head-retirement, next-basis, patch-stream, retention, pending-
  settlement registry, and pending-settlement capacity residue;
- every publication attempt has at most one bounded pending-settlement
  reservation before movement, every no-movement outcome removes it, and every
  performed commit remains addressable through that preinstalled record before
  its witness or moved branch head is observable to another owner operation;
- losing the witness does not lose settlement state, retention, completion, or
  recovery identity;
- immediate settlement, deferred-carrier repair, and commit-identity repair
  share one single-executor gate; concurrent or repeated callers return the
  same exact terminal receipt and never repeat durability or derived effects;
- retaining `S0` depends on exact admission and target availability, not whether
  `S0` is current;
- explicit Signal release and drop release each decrement exactly one
  obligation and free capacity;
- deletion removes the mutable branch reference while snapshots and external
  pins retain only their exact immutable state; and
- public documentation, facade names, compiler contracts, runtime counters,
  and CI commands describe the same behavior.

### Mutation sensitivity

The evidence must turn red if an implementation reintroduces `&mut` on a
9.17.2-facing Relational preparation, fork, publication, or settlement
operation, wraps the runtime in a global lock, checks Signal currentness during
external retention, removes lease drop cleanup, installs settlement recovery
only after movement or after a settlement attempt, leaks a pre-effect
settlement reservation, clears the only recovery state when the performed token
is dropped, restores the root fallback, omits an authority facade export, uses
a wrong exact test filter, or removes a feature-enabled cancellation lane.

## Product Decision Lock

### 1. Independently borrowable Relational owner services

Relational exposes these concrete owner services through its public facade:

```rust
let preparation = runtime.preparation_port();
let publication = runtime.publication_port();
let settlement = runtime.settlement_port();
let forking = runtime.fork_port();

let candidate = preparation.prepare_branch_transaction(transaction)?;
match publication.compare_and_publish(candidate) {
    RelationalPublicationOutcome::Performed(performed) => {
        let committed = settlement.settle_performed_publication(performed)?;
        // use committed
    }
    outcome => {
        // consume the typed no-movement or interruption posture
    }
}
```

The stable audience routes are
`facade::mvcc::{RelationalPreparationPort, RelationalPublicationPort,
RelationalSettlementPort}` and `facade::branch::RelationalForkPort`. Each port
is `Clone + Send + Sync`, carries one runtime affinity and lifecycle gate, and
uses `&self` for its operations. Responsibility is fixed as follows:

| Service | Operations |
| --- | --- |
| `RelationalPreparationPort` | `prepare_branch_transaction`, `discard_prepared_candidate` |
| `RelationalForkPort` | `observe_fork_source`, `fork_branch` |
| `RelationalPublicationPort` | `compare_and_publish` |
| `RelationalSettlementPort` | `settle_performed_publication`, `repair_deferred_publication_settlement`, `repair_pending_publication_settlement` |

The final runtime convenience receiver matrix is:

| Runtime convenience operation | Final receiver | Cut over in |
| --- | --- | --- |
| `begin_branch_transaction` | `&self` | already true in 9.17.1 |
| `prepare_branch_transaction` | `&self` | Phase 2 |
| `discard_prepared_candidate` | `&self` | Phase 2 |
| `observe_fork_source` | `&self` | Phase 2 |
| `fork_branch` | `&self` | Phase 2 |
| `compare_and_publish` / `publication_port` | `&self` | already true in 9.17.1 |
| `settle_performed_publication` | `&self` | Phase 3 |
| `repair_deferred_publication_settlement` | `&self` | Phase 3 |
| `repair_pending_publication_settlement` | `&self` | Phase 3 |
| `commit_branch_transaction` | `&self` | Phase 3 after the full path is independently borrowable |

`RelationalRuntime` convenience methods may remain, but every 9.17.2-facing
Relational operation takes `&self` and delegates to the same service. No public
or internal ordinary path may regain concurrency by wrapping a mutable runtime
in `Arc<Mutex<_>>`. Phase 2 does not claim the full convenience commit receiver
until Phase 3 has removed settlement's exclusive runtime borrow.

Preparation owns validation, immutable-root construction, candidate admission,
and branch-qualified resource reservations. Forking owns exact source
observation and immutable-root sharing. Publication owns exact branch-cell
comparison and movement. Settlement owns durability acknowledgement, derived
completion, and pending recovery. Each service receives only its subsystem
bindings and concrete owner lifecycle.

Branch-local locks may protect the selected branch cell. Global identity and
patch-position allocation remain bounded nonblocking mechanical reservations
with separate counters. No service may hold a global runtime lock while doing
user work, immutable-root construction, branch waiting, durability I/O, or
derived projection work.

### 2. Candidate consumption under mechanical deferral

`compare_and_publish` continues to consume
`PreparedRelationalCommitCandidate` on every outcome, as frozen by 9.17.1.
`PatchPositionReservationContended` is a typed no-movement terminal outcome for
that candidate, not a reusable candidate or automatic rebase authority.

A caller that chooses to retry must begin a fresh transaction and prepare a new
candidate. It may reuse the same still-admitted expected basis; it need not pay
for a new observation solely because the mechanical reservation contended.
Ordinary validation and comparison still return stale if the branch moved. The
owner guide and error DX must state this next action explicitly. The runtime may
not silently retry, silently rebase under a changed basis, return a partially
consumed candidate, or retain losing-candidate residue.

### 3. Settlement is runtime-owned before component movement

Pending-settlement capacity is reserved and one private
`PendingRelationalPublicationSettlement` record is installed before the
publication critical section. Capacity exhaustion therefore returns a typed
no-movement deferral before linearization. Every stale, denied, interrupted,
deferred, or failed-before-movement outcome removes that exact reservation and
releases its capacity before returning.

The record is keyed by the candidate's owner-issued commit identity and owns or
immutably references every precomputable input required to finish settlement:
the canonical publication route, successor basis, prepared completion,
published-snapshot capacity reservation, performed-settlement retention,
interruption posture, and exact owner binding. Successful branch movement
authorizes that already-installed record against the positioned canonical
commit. No observer can see a moved branch head for which owner recovery lookup
has no record. An interruption or panic between movement and return may lose a
caller witness, but it cannot create a recovery-registration gap.

`PerformedRelationalCommit` remains non-cloneable performed evidence and the
preferred immediate-settlement capability. It is not the sole owner of recovery
state. Dropping it may record capability abandonment, but cannot remove the
pending registry entry, release its settlement obligation, mark the route
settled, or make repair unavailable.

`RelationalSettlementPort::settle_performed_publication` consumes the performed
capability and addresses the runtime record.
`repair_pending_publication_settlement` addresses the same record by commit
identity. The settlement port is the runtime-affine repair authority; the
commit identity is only its lookup key and cannot reconstruct settlement
authority by itself. Successful terminal settlement marks the canonical route
settled and removes the registry entry exactly once. Durability failure retains
the record. Repeating repair after successful settlement returns the same exact
commit receipt without repeating an effect. Foreign-runtime, unknown,
owner-unavailable, inconsistent-route, and still-in-progress postures remain
distinct typed failures or deferrals.

The existing `DeferredPublicationSettlement` carrier and
`repair_deferred_publication_settlement` entry remain accepted exact recovery
surfaces because current Query owner code lawfully wraps and consumes them.
They delegate to the same pending registry and no longer own the only recovery
state. They are not a second registry or a compatibility authority lane.

The pending registry is bounded by
`publication.policy.max_published_snapshot_handles`. Each installed record
reserves one published-snapshot handle before movement, so performed settlement
cannot exceed the same configured resource budget. Owner shutdown closes
admission, resolves remaining retention with typed owner-loss accounting, and
leaves no live external service capable of reopening publication. This
milestone makes no restart-recovery claim.

### 4. Exact Signal retention is independent of currentness

`retain_signal_component_basis` validates:

- Signal runtime affinity and owner lifecycle;
- branch identity and exact admitted-basis provenance;
- descriptor, observation, definition, and immutable target agreement;
- continued target availability; and
- external-retention capacity and identity.

It does not compare the basis to the current branch observation. `StaleBasis`
remains valid for mutation/currentness operations but is removed from external
retention acquisition when staleness is the only mismatch.

The external lease is bound to the exact retained basis target, not merely a
branch id. Holding it keeps that immutable component state available without
claiming that the branch still selects it. Releasing it cannot release a newer
or sibling target.

### 5. Signal lease terminality

`SignalBranchRetentionLease` is a non-cloneable ergonomic guard carrying the
exact retained basis target and a cloneable binding to the narrow Signal
retention owner. The binding keeps cleanup state available after the main
runtime closes without keeping mutation admission open. Explicit
`release_signal_component_basis` consumes and disarms the guard, releases
exactly once, and returns a governed typed receipt identifying the released
target plus remaining exact-target and branch obligation counts. `Drop`
invokes the same owner-internal terminal release and records a dropped-release
counter; it does not fabricate the governed receipt.

Foreign release returns the still-live lease with a typed denial so the proper
owner can release it; dropping that returned lease remains a lawful terminal
release through its original owner binding. Double release is representationally
unavailable through the consuming public token and remains a typed registry
defense. Owner loss, unknown lease, and terminal shutdown are recorded
distinctly. Runtime shutdown closes new acquisition before it marks the narrow
retention owner closed; existing leases can still release and the owner state
dies after its last guard. `mem::forget` remains outside managed lifecycle
guarantees; ordinary Rust drop does not.

### 6. Public authority and observation contracts

`worth_relational::facade::branch` exports both
`RelationalBranchPublicationAuthority` and
`RelationalBranchPublicationAuthorityMarker`. The marker's name is public so
the concrete carrier can be named; its witness constructor and every issuer
remain private to Relational. A valid compile-pass court names the alias through
the facade, and existing forgery courts continue to fail for the intended
reason.

The stable inspection operation is `observe_branch_sharing`, matching
`observe_mvcc_cost`. `inspect_branch_sharing` is removed rather than retained as
a compatibility alias. `RelationalBranchSharingObservation` documents each
metric's truth and byte scope. `branch_metadata_bytes` is explicitly the
shallow inline size of live branch reference-state values and excludes map,
allocator, synchronization, heap, retained-root, and diagnostic storage; it may
not be presented as total resident branch memory.

### 7. Defensive publication truth

A selected publication cell without a root returns a typed
`RelationalPublicationFailureKind::SelectedRootUnavailable` before movement.
It may not substitute `expected_root`, fabricate a comparable observation, or
reach `replace_with` with an impossible root. Construction and recovery still
make rootlessness unreachable in ordinary operation; the typed failure protects
the authority boundary if that invariant regresses.

### 8. Evidence lanes are product configuration

The scheduled CI lane executes all three documented ignored proofs using their
compiled exact names:

```text
scale_invariant_admission::large_runtime_keeps_global_enforcement_and_filters_graph_planning
root_fork_sharing::phase5_standard_fork_copy_slope_is_flat_through_4096_forks
root_cost_scale_axes::selected_publication_cost_is_flat_through_documented_retention_ceiling
```

One ordinary CI lane enables `test-operation-control` and executes at least the
production-boundary
`mvcc_cancellation_publication_boundaries` family in the
`relational_certification` target, using a command equivalent to:

```text
cargo test -p worth-relational --features test-operation-control \
  --test relational_certification mvcc_cancellation_publication_boundaries::
```

The feature supplies observation and pausing only; it cannot change authority,
outcome meaning, or production transition logic. Cheap test listing or an
exact-name preflight must fail the lane when the filter selects zero tests.

Patch-position contention receives a deterministic concurrent court with one
performed result, one typed deferral, exact allocator counters, and zero losing
residue. Timing-only probability is not accepted.

Public Rust snippets in `BRANCH_LOCAL_MVCC.md`, `OWNER_COMPONENT_PORT.md`, and
`BRANCH_BASES.md` either compile as doctests through the real facade or are
replaced by links to executable examples built in CI. Pseudocode is labeled
`text` and makes no compile claim.

## Destination Topology And Migration Ledger

```text
crates/worth-relational/
    src/
        facade.rs                                 [existing, retain deferred-carrier export]
        facade/
            branch.rs                              [existing, revise authority/fork exports]
            mvcc.rs                                [existing, revise owner-service exports]
        branch/
            fork.rs                                [existing, retain mechanics]
            fork_port.rs                           [create: independent owner service]
        mvcc/
            transaction/
                commit.rs                          [existing, &self facade]
                preparation_port.rs                [create: preparation service]
            publication/
                port.rs                            [existing, publication service]
                port_cutover.rs                    [existing, typed root failure and preinstalled settlement]
                outcome.rs                         [existing, corrected outcomes]
        publication/
            authority/
                settlement_port.rs                 [create: ordinary settlement service]
                pending_settlement.rs              [create: settlement/repair orchestration]
                deferred_settlement.rs             [existing, delegate exact carrier recovery]
        runtime/
            state/
                runtime_state/
                    publication_lifecycle.rs        [existing, pending registry owner]
                    publication_settlement_registry.rs [create]
                    publication_recovery.rs         [remove after migration]
        inspection/
            mvcc/
                sharing.rs                         [existing, document artifact]
                sharing_inspection.rs              [existing, rename operation]
        tests/                                     [existing src-internal focused tests]
            transactions/core/
                publication_branch_locality.rs      [existing, expand across phases]
                publication_settlement.rs           [existing, lost-capability recovery]
                patch_position_contention.rs        [create]
    examples/
        branch_local_mvcc.rs                        [existing, keep executable]
    OWNER_COMPONENT_PORT.md                         [existing, correct]
    BRANCH_LOCAL_MVCC.md                            [existing, correct]
    TESTING_WORLDS.md                               [existing, correct commands]

crates/worth-signal/
    src/
        branch/
            retention/
                mod.rs                              [create from retention.rs facade]
                lease.rs                            [create: external/admission guards]
                registry.rs                         [create: exact-target obligations]
                outcome.rs                          [create: acquisition/release types]
            retention.rs                            [remove after split]
        logic/transaction/runtime/state/branching/
            retention.rs                            [existing, exact non-current admission]
    tests/
        branch_basis_contract.rs                    [existing, correct currentness expectation]
        branch_retention_lifecycle.rs               [create]
    examples/
        branch_bases.rs                             [create: executable owner workflow]
    BRANCH_BASES.md                                 [existing, correct]

.github/workflows/ci.yml                            [existing, feature and scheduled lanes]
_docs/WORTH-query/
    milestone-9.17.md                               [existing, insert corrective prerequisite]
    milestone-9.17.1.1.md                           [this specification]
    milestone-9.17.2.md                             [existing, consume corrected handoff]
    WORTH_query_roadmap.md                          [existing, insert sequence]
```

The port files own public service binding and orchestration, not the deep
transaction, publication, durability, or branch mechanics they call. The
pending-settlement registry owns its private reserved/performed state machine,
runtime lifecycle, and capacity; settlement orchestration owns terminal
effects. The existing deferred carrier and convenience entry remain one exact
addressing surface over that registry, not a second state owner. Signal lease,
registry, and outcome files separate public guard lifecycle, owner state, and
typed result meaning.

No `owner_ports`, `helpers`, `shared`, compatibility, or milestone-named source
bucket is permitted. Runtime World 9.17.2 enters only through the stable
facades and adds no file to either owner topology.

## Ordered Phase Plan

### Phase 1: Close Independent Public-Contract Defects

Export the concrete Relational publication carrier, rename
`observe_branch_sharing`, define the exact byte-metric scope, add the typed
selected-root failure, and install the corresponding compile-pass/fail and
focused defensive contracts. The final owner-service receiver matrix is frozen
by this specification, but Phase 1 does not publish placeholder preparation,
fork, or settlement services before their real mechanics exist.

### Phase 2: Make Relational Preparation And Forking Independently Borrowable

Introduce preparation and fork services over narrow cloneable owner bindings.
Move validation, immutable-root construction, candidate registration, discard,
fork-source observation, and fork execution off exclusive runtime borrowing.
Convert their runtime convenience entry points to `&self`; ordinary
`commit_branch_transaction` remains `&mut self` until settlement is corrected
in Phase 3. Focused compiler and runtime courts prove branch A can pause during
preparation or fork while branch B progresses.

### Phase 3: Make Settlement Runtime-Owned And Recoverable

Reserve settlement capacity and install the pending record before effect,
introduce the settlement service, and route immediate, deferred-carrier, and
commit-identity repair through the one pending registry. Remove the old post-
failure-only registry, convert settlement runtime conveniences and the full
`commit_branch_transaction` facade to `&self`, and complete the owner-service
compiler matrix. Prove immediate settlement, every no-movement reservation
cleanup, lost-capability repair, interruption immediately after movement,
concurrent immediate-versus-repair single execution, durability-fault repair,
owner loss, capacity exhaustion, child blocking before settlement, and child
progress afterward.

### Phase 4: Correct Signal Exact Retention And Lease Terminality

Split Signal retention responsibilities, bind external obligations to exact
targets, remove currentness from external retention admission, and make ordinary
drop terminal. Prove historical admitted pinning, exact readmission, explicit
release, drop release, foreign denial, owner loss, capacity recovery, deletion,
and reclamation.

### Phase 5: Close Adversarial Concurrency And Evidence Lanes

Extend Supply Chain locality across preparation, publication, and settlement;
add deterministic patch-position contention; enable the production-boundary
cancellation feature in CI; and run all three maximum scheduled proofs under
their real exact names. Evidence must use independent state, counter, and
residue observations.

### Phase 6: Documentation, Executable DX, And Successor Freeze

Correct the three owner guides, metric rustdoc, deletion semantics, deferral
next action, lifecycle recovery, and concurrency claims. Compile public examples
through the real facades. Run facade/residue inventories, remove obsolete names
and files, update the 9.17 umbrella and roadmap, and freeze the exact corrected
handoff consumed by 9.17.2.

## Documentation Deliverables

- `crates/worth-relational/BRANCH_LOCAL_MVCC.md`, for Relational callers, must
  show the independently borrowable preparation/publication/settlement path,
  ordinary `&self` convenience path, concurrency boundary, patch-position
  deferral, and current memory-resident limits.
- `crates/worth-relational/OWNER_COMPONENT_PORT.md`, for Runtime World 9.17.2,
  must define candidate consumption, fresh-attempt and still-admitted-basis
  retry posture, preinstalled pending settlement, lost-capability repair,
  deletion versus immutable retention, public authority names, and exact
  terminal outcomes.
- `crates/worth-signal/BRANCH_BASES.md`, for Runtime World 9.17.2, must show
  retention of a non-current exact basis, explicit and drop release, owner-loss
  posture, and the distinction between residency and currentness.
- `crates/worth-relational/TESTING_WORLDS.md`, for maintainers, must name valid
  commands and the CI lane that actually executes each scheduled or feature-
  gated proof.
- public Rust examples must compile in CI against only the documented facades.
  Documentation made false by this milestone is corrected in place rather than
  preserved as a competing guide.

## Performance And Resource Contract

- Preparation, fork, publication, and settlement coordination scales with the
  selected branch and declared transaction footprint, not total branches or
  unrelated writers.
- An unrelated paused branch contributes exactly zero branch coordination
  contacts and waits to another branch in every owner phase.
- Global identity and patch-position reservations remain O(1), bounded,
  nonblocking, and separately counted.
- Pending settlement admission, lookup, state transition, and removal are O(1)
  indexed by commit identity and bounded by the configured maximum. Registry
  synchronization is a separately counted constant-bounded owner contact and
  no registry lock may cross branch waiting, durability I/O, derived work, or a
  test pause. No performed publication may exist outside that bound.
- Lost-capability repair performs settlement work once; it does not scan branch
  history or reconstruct a candidate.
- Signal external retention acquisition and release are O(1) indexed owner
  operations over one exact target. Retaining an old basis performs zero Signal
  evaluation, graph copy, branch advancement, or latest lookup.
- Drop cleanup performs the same constant-bounded registry release as explicit
  release and records its terminal accounting without constructing a governed
  receipt.
- Scheduled scale claims retain their declared workloads, axes, and counter
  scopes; ordinary CI does not replace them with smaller substitutes.

## QA Considerations

Architecture review must confirm there is no mutable-runtime compatibility lane
or global lock hidden beneath the new services. Authority review must confirm
the public marker remains unforgeable and performed settlement cannot be
reconstructed from descriptive identity alone. Lifecycle review must cover
pre-effect reservation cleanup, movement-before-return interruption, capability
drop, explicit release, owner shutdown, bounded capacity, partial effect, and
repair. Concurrency review must force deterministic pauses and contention
rather than infer independence from sequential setup. Test review must keep the
rootless invariant defense at its honest focused boundary, verify that
scheduled and feature-gated commands really select the named tests, and ensure
losing-path residue observations are independent. DX review must compile the
public examples and reconcile every guide with the facade.

## Must Preserve

- every historical 9.17.1 phase status and accepted semantic contract not
  explicitly corrected here;
- Relational ownership of authoritative graph truth, commits, branch
  references, publication, durability posture, and settlement;
- Signal ownership of exact component bases, derived state, advancement,
  retention, and lifecycle;
- immutable commit versus mutable branch-reference separation;
- exact branch/runtime/definition affinity and owner-sealed authority;
- one-winner same-reference comparison and atomic old-or-new visibility;
- prepared candidate opacity and single consumption;
- canonical commit and patch-stream truth with derived projections remaining
  non-authoritative;
- obligation-bound retention, archive/delete distinction, and separate
  maintenance reclamation;
- Foundational descriptive vocabulary without operational authority;
- Proof-owned concrete carriers specialized by private owner markers;
- Query-agnostic owner crates, cert-only replay, tier direction, structural
  sharing, and memory-resident application state; and
- the exact 9.17.2 rule that Runtime World composes owner artifacts without
  restamping component authority.

## Explicit Non-Goals

- Runtime World composite commits or product branch references;
- Query plan, session, receipt, live, history, or facade carriage;
- persistence, restart recovery, distributed settlement, or Store integration;
- semantic merge, rebase, multi-parent history, offline synchronization, or
  correction policy;
- automatic retry, rebase, or candidate reuse after publication deferral;
- a new Signal concurrency engine; and
- repository-wide cleanup of unrelated warnings or documentation.

## Acceptance Evidence

Closure requires all of the following on the final scoped change:

- focused Relational owner-service, settlement, selected-root, contention, and
  locality tests;
- focused Signal historical-retention, release, drop, capacity, owner-loss,
  deletion, and reclamation tests;
- the full Relational library and certification targets;
- the feature-enabled production-boundary cancellation certification command;
- all three ignored scheduled proofs under their compiled exact names;
- public facade compile-pass and authority-forgery compile-fail courts;
- executable Relational and Signal examples through public facades;
- documentation/residue searches proving obsolete names, false commands,
  mutable-runtime owner signatures, and the removed post-failure-only registry
  path are gone while accepted deferred-carrier consumers still compile;
- focused Clippy, formatting, dirty Rust line-cap enforcement, boundary-check,
  and generated agent-context validation; and
- code review against this specification, 9.17.1 inherited laws, and the QA
  guide with no known material scoped defect remaining.

Test counts are observations, not acceptance authority. A green suite that did
not exercise the feature, ignored proof, contention schedule, capability-drop
path, or non-current exact retention does not close the corresponding claim.

## Exact Relational And Retention Handoff Toward Milestone 9.17.2

The Runtime World composition owner may depend only on public owner facades to:

- obtain independently borrowable Relational preparation, fork, publication,
  and settlement services through
  `facade::mvcc::{RelationalPreparationPort, RelationalPublicationPort,
  RelationalSettlementPort}` and `facade::branch::RelationalForkPort`, each
  taken by shared borrow;
- after 9.17.1.2, obtain observation, readmission, retention, release, and
  lifecycle through its added Relational ports and aggregate bundle;
- carry exact owner-issued Relational and Signal bases through
  `facade::branch::{AdmittedRelationalBranchBasis,
  AdmittedRelationalForkSourceBasis}` and
  `worth_signal::facade::branch::AdmittedSignalBranchBasis`;
- retain and release a current or historical exact component basis for one
  named composite obligation, through
  `facade::branch::RelationalBranchRetentionLease` and
  `worth_signal::facade::branch::SignalBranchRetentionLease`, whose ordinary
  drop is terminal;
- prepare and consume one opaque Relational candidate without borrowing the
  entire runtime mutably, through
  `facade::mvcc::{PreparedRelationalCommitCandidate,
  DiscardedRelationalCommitCandidate}`;
- receive and settle a performed Relational publication, or recover it by owner
  commit identity through the runtime-affine settlement service after
  capability loss, through `facade::mvcc::PerformedRelationalCommit` and
  `RelationalSettlementPort`;
- receive typed stale, denied, interrupted, deferred, failed, owner-loss, and
  terminal release outcomes, through
  `facade::mvcc::{RelationalPublicationOutcome, RelationalPublicationDenial,
  RelationalPublicationDeferred, RelationalPublicationFailureKind,
  StaleRelationalBranchObservation}` and
  `facade::branch::RelationalBranchRetentionTerminalOutcome`; and
- observe exact cost and sharing artifacts through the frozen facade names
  `observe_mvcc_cost` and `observe_branch_sharing`, whose observations and
  declared byte scope are
  `facade::inspection::{RelationalMvccCostObservation,
  RelationalBranchSharingObservation, RelationalSharingByteMetricScope}`.

These routes name the surface; they do not restate its contract. The normative
owner contract, including exact preconditions, outcome meanings, and lifecycle
obligations, is frozen in `crates/worth-relational/OWNER_COMPONENT_PORT.md` and
`crates/worth-signal/BRANCH_BASES.md`. Where this section and those guides could
be read differently, the guides govern.

Runtime World may not inspect pending-settlement or retention registries,
construct an owner authority, retain only a branch id, treat currentness as
residency, retry a consumed candidate, settle by editing history, wrap an owner
runtime in a global mutex, or create a compatibility representation. The next
9.17.1.2 completes the Relational bundle and Signal operation boundary without
changing these contracts. Milestone 9.17.2 does not repair either owner from
the composition layer.
