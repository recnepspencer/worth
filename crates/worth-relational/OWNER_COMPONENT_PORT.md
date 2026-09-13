# Relational Owner Component Port

This document freezes the Relational component boundary that Query Milestone
9.17.2 may consume. It is a port over owner-issued artifacts and typed outcomes,
not access to Relational internals and not a composite commit implementation.

This file is the normative component contract for the 9.17.2 Runtime World
composition owner above Runtime Bridge. The complete caller mental model is in
[`BRANCH_LOCAL_MVCC.md`](./BRANCH_LOCAL_MVCC.md), which explains the model to
ordinary Relational callers without assigning product authority to Bridge.
The executable publication flow is
[`examples/branch_local_mvcc.rs`](./examples/branch_local_mvcc.rs). Certification
lanes are documented in [`TESTING_WORLDS.md`](./TESTING_WORLDS.md).

## Ownership rule

Relational alone owns:

- runtime and branch identity;
- the branch-reference catalog, lifecycle, generation, and current root;
- schema admission, transaction validation, and immutable root construction;
- canonical Relational commits, history, patches, snapshots, and replay input;
- branch-local compare-and-publish and publication settlement;
- the pre-effect pending-settlement registry keyed by owner-issued commit
  identity, and the single executor that drains it; and
- component retention and reclamation accounting.

A later composite owner may hold, compare, retain, and coordinate artifacts
issued by this port. It cannot mint them, rewrite their fields, derive identity
from a digest or version, choose a Relational head from a proxy, or mutate a
component branch directly.

An owner-issued `CommitId` is a lookup key into Relational's own registry. It is
never reconstructible settlement authority, and holding one grants no capability
that the owner did not already retain.

## Artifacts available to 9.17.2

| Artifact | Facade route | Authority posture |
| --- | --- | --- |
| `RelationalBranchIdentity` | `facade::branch` | Selects one mutable reference only after owner validation |
| `RelationalBranchBasisDescriptor` | `facade::branch` | Serializable exact target and generation; descriptive, must be readmitted |
| `AdmittedRelationalBranchBasis` | `facade::branch` | Exact owner admission holding its immutable root resident |
| `RelationalBranchObservation` | `facade::mvcc` | Exact read selection; opens owner snapshot reads, not writes |
| `RelationalBranchBasisDenial` | `facade::branch` | Typed readmission and observation refusal |
| `RelationalBranchRetentionLease` | `facade::branch` | External residency obligation; runtime-affine and single-terminal |
| `RelationalBranchRetentionReleaseReceipt` | `facade::branch` | Evidence that one lease reached one terminal path |
| `RelationalBranchRetentionReleaseDenial` | `facade::branch` | Refused release that returns the still-live lease |
| `PreparedRelationalCommitCandidate` | `facade::mvcc` | Validated, branch-bound, single-use proposed successor |
| `DiscardedRelationalCommitCandidate` | `facade::mvcc` | Evidence that a candidate was consumed without movement |
| `RelationalPublicationOutcome` | `facade::mvcc` | Typed result of owner compare-and-publish |
| `PerformedRelationalCommit` | `facade::mvcc` | Non-cloneable witness of in-process component movement |
| `RelationalCommitReceipt` | `facade::history` | The one terminal answer for a settled commit identity |
| `CommitResult` | `facade::mvcc` | Full settlement result, issued once to one claimant |
| `CommitId` | `facade::history` | Owner-issued identity; a registry lookup key only |
| `DeferredPublicationSettlement` | `facade::publication` | Accepted external carrier for one durability-deferred route |
| `DeferredPublicationSettlementError` | `facade::publication` | Typed repair refusal |
| `TransactionCommitError` | `facade::mvcc` | Typed settlement and convenience-commit failure |
| `RelationalBranchPublicationAuthority` | `facade::branch` | Concrete owner-sealed publication authority, with its own marker type |
| `RelationalForkOutcome` and `RelationalForkDenial` | `facade::branch` | Fork evidence and typed refusal; evidence does not mint a basis |
| `ArchivedRelationalBranch` and `RelationalBranchArchiveDenial` | `facade::branch` | Archive result and typed refusal |
| `RelationalBranchDeletionOutcome` | `facade::branch` | Deletion result carrying completed or pending deletion |
| `RelationalBranchDeleteDenial` | `facade::branch` | Typed deletion refusal |
| `RelationalBranchSharingObservation` | `facade::inspection` | Read-only sharing evidence; not a currentness authority |
| `RelationalMvccCostObservation` | `facade::inspection` | Read-only cost evidence; not a currentness authority |

Canonical encodings and digests aid transport and comparison. They do not
replace owner identity, readmission, retention, or currentness.

## Owner services

Relational exposes six independently borrowable owner services through one
`RelationalOwnerServicePorts` bundle. Each is
`Clone + Send + Sync`, takes `&self`, carries one runtime affinity and one
lifecycle gate, and holds no strong reference that could keep a finished runtime
alive. Holding one never excludes another owner operation.

| Service | Facade route | Obtained from | Operations |
| --- | --- | --- | --- |
| `RelationalPreparationPort` | `facade::mvcc` | `preparation_port()` | `prepare_branch_transaction`, `discard_prepared_candidate` |
| `RelationalForkPort` | `facade::branch` | `fork_port()` | `observe_fork_source`, `fork_branch` |
| `RelationalPublicationPort` | `facade::mvcc` | `publication_port()` | `compare_and_publish` |
| `RelationalSettlementPort` | `facade::mvcc` | `settlement_port()` | `settle_performed_publication`, `repair_deferred_publication_settlement`, `repair_pending_publication_settlement`, `retains_pending_settlement`, `runtime_instance_id` |
| `RelationalBranchBasisPort` | `facade::branch` | `basis_port()` | `observe_branch`, `admit_branch_basis`, `readmit_branch_basis`, `readmit_retained_branch_basis`, `retain_component_basis`, `release_component_basis` |
| `RelationalBranchLifecyclePort` | `facade::branch` | `lifecycle_port()` | `archive_branch`, `delete_branch`, `owner_lifecycle_observation` |

`RelationalRuntime::owner_component_services()` returns the bundle. Its six
accessors return concrete ports; there is no wrapping trait or generic service
adapter.

### Complete method and compatibility matrix

This matrix is source-grounded in the current public implementation. Every
receiver is `&self`. The compatibility delegate is the same-named method on
`RelationalRuntime` unless the row says otherwise; both routes reach the same
owner state.

| Port method | Input | Output | Canonical owner | Compatibility delegate and required evidence |
| --- | --- | --- | --- | --- |
| `prepare_branch_transaction` | `BranchBoundRelationalTransaction` | `Result<PreparedRelationalCommitCandidate, TransactionCommitError>` | preparation owner and candidate registry | runtime `prepare_branch_transaction`; healthy preparation plus validation, owner-loss, capacity, and cancellation/deferral denial |
| `discard_prepared_candidate` | `PreparedRelationalCommitCandidate` | `Result<DiscardedRelationalCommitCandidate, TransactionCommitError>` | preparation owner and candidate registry | runtime `discard_prepared_candidate`; exact by-value consumption, foreign-owner and owner-unavailable denials, and cleanup; reuse is unavailable by move semantics, while an expired candidate remains discardable |
| `observe_fork_source` | `&BranchId` | `Result<(RelationalForkSourceDescriptor, AdmittedRelationalForkSourceBasis), RelationalForkDenial>` | source branch cell | runtime `observe_fork_source`; live non-empty source plus missing, archived, deleting, empty, and owner-loss denials |
| `fork_branch` | `BranchId`, `AdmittedRelationalForkSourceBasis` | `Result<RelationalForkOutcome, RelationalForkDenial>` | source cell plus target reservation | runtime `fork_branch`; shared immutable source plus stale/foreign/duplicate/retired/capacity denials |
| `compare_and_publish` | `PreparedRelationalCommitCandidate` | `RelationalPublicationOutcome` | one branch publication cell | runtime publication convenience delegates through `publication_port`; performed, stale, denied, deferred, failed-before-movement, one-winner, and residue cleanup |
| `settle_performed_publication` | `PerformedRelationalCommit` | `Result<CommitResult, TransactionCommitError>` | pending-settlement registry and executor | runtime `settle_performed_publication`; settled, deferred durability, foreign owner, owner loss, and exactly-once claimant |
| `repair_deferred_publication_settlement` | `&DeferredPublicationSettlement` | `Result<RelationalCommitReceipt, DeferredPublicationSettlementError>` | retained performed route | runtime same name; durable repair, wrong route/owner, already settled, and owner loss |
| `repair_pending_publication_settlement` | `CommitId` | `Result<RelationalCommitReceipt, DeferredPublicationSettlementError>` | pending-settlement registry | runtime same name; lost-capability repair, unknown/foreign identity, already settled, and owner loss |
| `retains_pending_settlement` | `CommitId` | `bool` | pending-settlement registry observation | port-only observation with no direct runtime delegate; true/false around settlement and cleanup; never authority |
| `runtime_instance_id` | none | `u64` | settlement-port diagnostic affinity | no operational delegate; descriptive correlation only |
| `observe_branch` | `&RelationalBranchIdentity` | `Result<(RelationalBranchBasisDescriptor, AdmittedRelationalBranchBasis), RelationalBranchBasisDenial>` | branch reference cell and root retention | runtime `observe_branch`; live exact observation plus foreign/unknown/archived/deleting/owner-loss denial |
| `admit_branch_basis` | `&RelationalBranchIdentity` | `Result<AdmittedRelationalBranchBasis, RelationalBranchBasisDenial>` | branch reference cell and root retention | runtime `admit_branch_basis`; healthy current admission plus lifecycle, owner, and retention denial |
| `readmit_branch_basis` | `&RelationalBranchBasisDescriptor` | `Result<AdmittedRelationalBranchBasis, RelationalBranchBasisDenial>` | branch reference cell and root retention | runtime `readmit_branch_basis`; exact current match plus version, owner, target, generation, lifecycle, and retention denial |
| `readmit_retained_branch_basis` | `&RelationalBranchBasisDescriptor`, `&RelationalBranchRetentionLease` | `Result<AdmittedRelationalBranchBasis, RelationalBranchBasisDenial>` | branch cell plus exact retention registry | runtime same name; retained exact target plus foreign lease, descriptor mismatch, deleted branch, unavailable target, and owner loss |
| `retain_component_basis` | `&AdmittedRelationalBranchBasis` | `Result<RelationalBranchRetentionLease, RelationalBranchBasisDenial>` | exact root-retention registry | runtime same name; current or historical exact retention plus foreign, unavailable, lifecycle, and capacity denial |
| `release_component_basis` | `RelationalBranchRetentionLease` | `Result<RelationalBranchRetentionReleaseReceipt, RelationalBranchRetentionReleaseDenial>` | issuing retention registry | runtime same name; explicit release, foreign owner returning the live lease, owner loss, and drop terminality |
| `archive_branch` | `&RelationalBranchIdentity` | `Result<ArchivedRelationalBranch, RelationalBranchArchiveDenial>` | branch lifecycle cell | runtime same name; performed archive plus owner-unavailable, foreign-runtime, unknown, already-archived, deleting, and generation-overflow denials |
| `delete_branch` | `&RelationalBranchIdentity` | `Result<RelationalBranchDeletionOutcome, RelationalBranchDeleteDenial>` | branch lifecycle cell and retention accounting | runtime same name; deleted or typed pending deletion plus main/unknown/lifecycle/owner-loss denial |
| `owner_lifecycle_observation` | none | `RelationalOwnerLifecycleObservation` | weak owner lifecycle | port-only observation with no direct runtime delegate; exact `Open`, `Closing`, or `Closed` observation after root drop |

`observe_branch_with_control` is the operation-control counterpart used by the
deterministic concurrency court. It has the same basis outcome and owner path as
`observe_branch`; it is not a seventh production operation or a composition
authority.

The runtime also exposes transaction admission and the same owner authorities
through compatibility methods. Those methods do not form a competing state
lane: movement through either route is immediately visible to the ports.

`run_branch_root_reclamation_pass` remains an exclusive owner-maintenance
operation. It is not a seventh port method, and its
`RelationalBranchRootReclamationOutcome` remains re-exported through
`facade::branch` only for owner inspection.

Reclamation is separate maintenance, not part of the handoff. Runtime World does not
invoke `run_branch_root_reclamation_pass`; the owner runs it on its own explicit
cold path, and it never occurs inside observation, transaction admission, or
ordinary publication.

### Focused owner-service proof

The grouped `relational_certification::owner_service_completion` court reaches
this matrix only through the concrete public facades. Its semantic cutover cases
compile the Court Supply Chain world, lower named storm and berth-maintenance
deltas through the existing production compiler, and compare neutral public
observations with the independently authored Supply Chain oracle. A performed
port publication must therefore change domain truth, immediately stale the
predecessor accepted by the direct runtime route, and be visible through both
observation routes. The reverse case commits through the compatibility route
from a port-issued basis and requires the port to observe the exact oracle state.
These assertions deliberately fail for a success-shaped no-op or split mutable
state even if both routes return success.

The same-owner cutover case passes direct descriptors and port-issued leases
across the opposite route, installs fork targets through both surfaces, and
deletes each target through the other surface. The candidate-discard case uses
a real update-only candidate, proves it invented no record reservations, leaves
neutral branch truth unmoved, and then deletes the branch immediately. That
terminal delete is cleanup evidence: a discarded candidate cannot leave an
active-operation obligation hidden in the owner. Separate denial, weak-owner,
`Open`/`Closing`/`Closed`, and parked-branch cases preserve exact typed failures
and prove unrelated basis progress without private-state observation.

The reviewed selector is
`owner_service_completion::`; CI lists and runs that same roster through
`scripts/ci/run_relational_named_test_selection.sh` so a renamed, ignored, or
zero-selected focused proof is a gate failure rather than a silent pass.

## Observation and readmission

The composite owner starts from an explicit `RelationalBranchIdentity` and
asks Relational to `observe_branch`. If a descriptor crosses a storage,
process, message, or trust boundary, it returns through
`readmit_branch_basis`. When an external lease already preserves the exact
target, `readmit_retained_branch_basis` checks both the descriptor and lease
against the same owner.

`RelationalBranchBasisDenial` separates, among others,
`UnsupportedDescriptorVersion`, `ForeignRuntime`, `UnknownBranch`,
`ArchivedBranch`, `DeletingBranch`, `StaleReferenceGeneration`,
`WrongImmutableTarget`, `MixedAxis`, `UnavailableRetainedTarget`,
`RetentionCapacityExhausted`, and `OwnerFailure`. Runtime World must preserve these
classes; flattening them into a generic "head changed" result destroys retry and
safety semantics.

## Retention contract

Before a composite artifact promises that a component basis remains usable,
it acquires `retain_component_basis`. Ownership of the returned
`RelationalBranchRetentionLease` is part of the composite artifact's lifecycle.
Exactly one of these three terminal actions occurs, and there is no fourth:

- `release_component_basis` consumes the lease and returns
  `RelationalBranchRetentionReleaseReceipt`, carrying the exact `descriptor()`
  and a `RelationalBranchRetentionTerminalOutcome` of `Released` or
  `OwnerUnavailable`;
- release against a runtime that does not own the lease returns
  `RelationalBranchRetentionReleaseDenial`, which carries the typed `denial()`
  and the still-live lease. `into_lease()` recovers it, and it remains
  releasable by its true owner; or
- dropping the lease performs the same owner-internal release and records
  dropped-release accounting, but produces no receipt.

There is no operation that transfers a lease into a performed successor
artifact. A composite owner that needs residency past publication acquires a new
lease from the successor basis.

Dropped-release accounting is owner-internal instrumentation, not a public
counter. A lease preserves residency, not freshness: the branch may advance
while an older exact target remains retained, so a composite owner must compare
or readmit before treating any basis as current.

## Preparation, candidate consumption, and publication

The component publication progression is:

```text
admitted predecessor basis
  -> begin_branch_transaction
  -> preparation_port.prepare_branch_transaction
  -> PreparedRelationalCommitCandidate
  -> publication_port.compare_and_publish
  -> Performed | Stale | Denied | Interrupted | Deferred | Failed
  -> settlement_port.settle_performed_publication, only for Performed
```

Preparation performs validation and immutable-root construction without moving
the branch. It also reserves the candidate's published-snapshot handle, so that
bounded capacity is refused here rather than at movement.

`compare_and_publish` takes the candidate by value and consumes it on every
outcome, including every no-movement outcome. There is no partially consumed
candidate, no residue, and no reusable candidate after any result. A candidate
the composition owner no longer intends to publish is consumed through
`discard_prepared_candidate`, which returns `DiscardedRelationalCommitCandidate`.

`compare_and_publish` is the Relational linearization point, so the composite
owner must not infer movement from candidate creation, a reserved commit ID,
patch bytes, or diagnostics.

## Retry posture after no movement

Every no-movement outcome leaves the branch reference where it was. A retry is a
fresh transaction and a fresh candidate, because the consumed candidate cannot
be resubmitted. The still-admitted `AdmittedRelationalBranchBasis` may be reused
directly: `begin_branch_transaction` takes the basis by reference and
revalidates it, so holding a basis across a failed attempt is lawful and no new
observation is owed merely because the attempt did not move the branch.

`RelationalPublicationDeferred::PatchPositionReservationContended` in particular
is a loss on the runtime-wide patch-position allocator, taken before any branch
movement and shared across branches. The losing attempt performed no movement
and did not make this branch stale, so a same-basis retry is the correct
response.

Ordinary validation still answers when the branch really did move: admission
returns `RelationalBranchTransactionAdmissionDenial::StaleBasis`, and comparison
returns `RelationalPublicationOutcome::Stale` carrying complete `expected()` and
`observed()` descriptors. The owner never silently retries, never rebases under
a changed basis, and never returns a reusable candidate.

## Preinstalled bounded pending settlement

Before the publication critical section, the owner installs one pending
settlement record in its own registry, keyed by the candidate's owner-issued
commit identity. Installation happens ahead of movement so that no observer can
ever see a moved branch head whose owner recovery lookup has no record.

Every stale, denied, interrupted, deferred, or failed-before-movement path
releases that exact reservation before returning. Successful movement authorizes
the already-installed record against the exact positioned canonical commit; it
does not create the record.

Between movement and authorization the identity is addressable as work in
progress. Repair by identity answers
`DeferredPublicationSettlementError::SettlementInProgress`, never a missing
record.

The registry is bounded by the same published-snapshot handle budget the
candidate already reserved at preparation. Exhaustion is reported there as
`RelationalPublicationDeferred::PublishedSnapshotCapacityExhausted`, carrying
`maximum_handles`, so a pending record can exist only for a candidate that
already holds one of the configured handles. Prepared candidates are separately
bounded by `CandidateCapacityExhausted`, carrying `maximum_candidates`. Reusing
a commit identity that already owns a pending record is refused before any
effect as `RelationalPublicationFailureKind::PendingSettlementIdentityConflict`.

`PerformedRelationalCommit` is a non-cloneable witness and the preferred
immediate settlement path, but it does not own recovery. Dropping it records
capability abandonment and nothing else: it cannot remove the record, release
the retention obligation, mark the route settled, or make repair unavailable.

## Lost-capability repair

Two repair routes address the same runtime-owned record, and both run through
the one executor:

- `repair_deferred_publication_settlement` accepts the exact
  `DeferredPublicationSettlement` carrier a durability deferral produced. The
  carrier is a view of the record, never a second registry or a compatibility
  authority lane. It is checked against the record's retained route before any
  effect, and a mismatch is refused as `PerformedRouteMismatch`.
- `repair_pending_publication_settlement` addresses the same record by `CommitId`
  when the capability holder was lost entirely.

Settlement artifacts are runtime-affine. Before claiming the executor gate,
`settle_performed_publication` compares the performed witness's retained record
with the settlement service's runtime, and
`repair_deferred_publication_settlement` performs the same check against its
carrier. A mismatch is typed as `RelationalPublicationDenial::ForeignRuntime`
inside `TransactionCommitError::PublicationDenied` for the performed-witness
entry, or `DeferredPublicationSettlementError::ForeignRuntime` for deferred
repair. In both errors, `expected_runtime_instance_id` is the runtime of the
service that received the call, and `actual_runtime_instance_id` is the runtime
carried by the witness or deferred route.

`settle_performed_publication` consumes its witness argument even when a foreign
service rejects it, but rejection does not claim or remove the source runtime's
pending record. The source owner can still complete the obligation through
`repair_pending_publication_settlement(commit_id)`. A foreign deferred-repair
attempt borrows its carrier, so the same carrier remains usable with its source
owner.

After affinity validation, `settle_performed_publication` and both repair routes
enter one per-commit executor gate before claiming any work. Concurrent or
repeated callers therefore converge on one terminal receipt and never repeat a
durable append or a derived completion. A repeated call after settlement
returns the same `RelationalCommitReceipt`, including when the record itself has
already been released.

Both repair routes return `RelationalCommitReceipt`. Only
`settle_performed_publication` returns the full `CommitResult`, which is issued
once to one claimant along with responsibility for releasing its published
snapshot.

`retains_pending_settlement` reports whether this runtime still holds an
unsettled record for an identity. It is a posture question, not a capability.

When the owning runtime closes settlement admission, it drains every pending
record exactly once. A later repair then answers
`DeferredPublicationSettlementError::OwnerUnavailable` rather than a receipt.

## Cancellation contract

Before Relational linearization, cancellation or timeout returns a typed
`Interrupted` outcome and preserves the predecessor reference. During or after
the critical section, movement wins: the owner returns `Performed` with
`late_interruption` evidence, and settlement remains mandatory.

Interruption reported after movement is not a no-movement path. The pending
record survives it, so recovery by commit identity stays available even if the
witness is then lost.

This distinction is the handoff needed by coordinated publication. It prevents
a composite owner from claiming "nothing happened" after one component has
already moved. Milestone 9.17.2 must use its own typed prepare/compatibility/
publication protocol to ensure no half-current product world; it may not
simulate component rollback through raw Relational mutation.

## Fork posture

Fork is a separate owner transition. `observe_fork_source` issues a linear,
fork-only `AdmittedRelationalForkSourceBasis` alongside its
`RelationalForkSourceDescriptor`. `fork_branch` consumes that basis, creates one
new reference cell, and shares the exact immutable source root by identity.

`RelationalForkOutcome` is evidence: it reports source, target, and provenance
observations, and does not mint a basis. `RelationalForkDenial` is the complete
set of sixteen: `SourceBranchMissing`, `SourceArchived`, `SourceDeleting`,
`EmptySource`, `DuplicateTarget`, `RetiredTarget`, `ForeignRuntime`,
`StaleSource`, `MissingArtifact`, `InvalidTarget`, `Cell`,
`RetentionCapacityExhausted`, `RetentionOwnerUnavailable`,
`RetentionIdentityExhausted`, `RetentionInvariantViolation`, and
`OwnerUnavailable`. `Cell` carries a `RelationalBranchCellDenial` from the target
reference cell itself and is neither a source refusal nor a bounded-retention
class. A composite owner may request or record a fork but cannot manufacture the
target cell.

## Deletion and immutable retention

`RelationalBranchLifecyclePort::delete_branch` takes `&self`; the direct
`RelationalRuntime` compatibility method retains its existing mutable receiver
and reaches the same owner state. Deletion is forbidden for the main branch,
which is refused as `RelationalBranchDeleteDenial::MainBranch`.

Deletion is gated on branch-scoped active operations, and those are exactly
three: open branch-bound transactions, prepared candidates, and performed but
unsettled settlements. While any remain, the owner returns
`RelationalBranchDeletionOutcome::WaitingForActiveOperations`, whose
`RelationalBranchDeletionPending` reports `identity()` and
`active_operation_count()`. That outcome also marks the branch `Deleting` and
reserves its retired name, so new transaction admission is denied as `Deleting`
and publication is denied as `RelationalPublicationDenial::Deleting`.

Admitted observations, pinned snapshots, and external component-retention leases
are not active operations and do not delay deletion. A composition owner that releases every
lease before requesting deletion pays for a barrier the owner never required.

When no active operation remains, deletion removes the mutable branch reference
and retires that branch's head root, reporting
`DeletedRelationalBranch::retired_root_identity()`. It retires only the removed
head; shared ancestors survive while another branch or obligation retains them.

Immutable retention is orthogonal to reference deletion. A lease, an admitted
basis, or a snapshot taken before deletion continues to name its exact immutable
target after the reference is gone, and that state stays resident for the
lifetime of the obligation. Residency is not existence: once the branch is gone,
`readmit_retained_branch_basis` still checks descriptor and lease against the
same owner, and the retained target is not readmissible as a live branch basis.
Runtime World may not treat retained residency as evidence that the branch still
exists, and it must still reach every retained lease's terminal action.

Archive is the separate, non-destructive posture: `archive_branch` advances
lifecycle metadata and denies new ordinary work without retiring the root.

Composite cleanup must retain the pending state and retry through the owner. It
cannot delete owner catalog entries itself.

## Exact terminal outcomes

These are the complete sets the Runtime World composition owner must match on.

`RelationalPublicationOutcome`: `Performed`, `Stale`, `Denied`, `Interrupted`,
`Deferred`, `Failed`.

`RelationalPublicationDenial`: `OwnerUnavailable`, `ForeignRuntime`,
`OwnerMismatch`, `BranchUnavailable`, `Archived`, `Deleting`.

`RelationalPublicationDeferred`: `PatchPositionReservationContended`,
`RetentionBackpressure`, `CandidateLifetimeExpired`,
`CandidateCapacityExhausted`, `PublishedSnapshotCapacityExhausted`.

`RelationalPublicationFailureKind`: `SnapshotIdentityExhausted`,
`CandidateIdentityExhausted`, `PreparedRootBudgetExhausted`,
`PreparedRootMismatch`, `PreparedBasisDescriptor`, `NextBasisAdmission`,
`SelectedRootUnavailable`, `BranchObservation`, `PatchPositionCapacityExhausted`,
`RetentionIdentityExhausted`, `RetentionOwner`,
`PendingSettlementIdentityConflict`.

`DeferredPublicationSettlementError`: `RecoveryUnavailable`, `ForeignRuntime`,
`PerformedRouteMissing`, `PerformedRouteMismatch`, `SettlementInProgress`,
`OwnerUnavailable`, `DurableAppend`.

`TransactionCommitError`: `Conflict`, `Publication`, `Preparation`,
`Interrupted`, `PublicationDenied`, `PublicationDeferred`, `PublicationFailed`,
and `PerformedButDurabilityDeferred`. Only the last carries performed work; it
supplies the `DeferredPublicationSettlement` carrier for repair.

`RelationalBranchRetentionTerminalOutcome`: `Released`, `OwnerUnavailable`.

`RelationalBranchDeletionOutcome`: `Deleted`, `WaitingForActiveOperations`.

`RelationalBranchDeleteDenial`: `OwnerUnavailable`, `ForeignRuntime`, `UnknownBranch`, `MainBranch`,
`RetentionBackpressure`, `RetentionIdentityExhausted`,
`RetiredIdentityCapacityExhausted`, `OwnerFailure`.

One current gap is stated here rather than implied: preparing a descendant of a
commit that still requires explicit owner settlement is refused as an untyped
`TransactionCommitError::Publication` at `PublicationStage::Visibility` with a
detail string. There is no dedicated typed kind in this milestone, and Runtime World
must not match a variant that does not exist.

## Allowed 9.17.2 integration

Milestone 9.17.2 may:

- obtain the six independently borrowable owner services from one runtime
  through shared access;
- carry exact Relational and Signal component descriptors in one Runtime World-owned
  correspondence;
- readmit and retain each component basis through its owner;
- prepare component work without claiming currentness changed;
- compare component predecessor observations with the correspondence;
- consume typed component publication and settlement outcomes;
- recover a performed component movement by owner commit identity after
  capability loss;
- publish a Runtime World-owned product reference only through its specified
  coordinated protocol; and
- transfer or release every component retention obligation explicitly.

## Forbidden composition behavior

Milestone 9.17.2 may not:

- construct or deserialize an admitted Relational basis;
- select a branch through `None`, a raw name, a version, a commit ID, a digest,
  a snapshot, or an ambient `"main"` fallback;
- accept a generic `AuthorityMarker` in place of the concrete owner-sealed
  Relational authority types;
- retain only a branch id, or treat currentness as residency or residency as
  currentness;
- resubmit a consumed candidate, or treat prepared, stale, denied, deferred, or
  interrupted work as performed;
- infer that a no-movement outcome invalidated a still-admitted basis;
- inspect, enumerate, mirror, or repair the owner's pending-settlement or
  retention registries, or build a second recovery index beside them;
- settle a component movement by editing history, or relabel an unsettled
  movement as rollback because a sibling component failed;
- call private root, branch-cell, history-catalog, or raw publication mutation;
- create a second Relational currentness table inside Runtime World, Bridge, or Query;
- construct a compatibility representation of an owner artifact, or expose a
  combined component authority as though Relational issued it;
- wrap an owner runtime in a global lock or otherwise reintroduce whole-runtime
  exclusivity across the six shared services;
- consume any `facade::replay` surface, including `CanonicalCommitEnvelope`,
  `RelationalReplayRequest`, and `RelationalReplayOutcome`. Replay and
  reconstruction are certification-only and are granted to no composition artifact;
  the lawful terminal answers for a settled movement are `RelationalCommitReceipt`
  and `CommitResult`; or
- promise physical persistence or restart recovery for the in-memory owner.

## Memory and durability boundary

Relational branch cells, roots, owner admission, the pending-settlement
registry, and retention accounting are memory-resident. Canonical history and
durability settlement remain Relational-owned, but restart durability for this
branch-owner model is deferred to Worth Store integration. The pending-settlement
registry makes no restart-recovery claim: it recovers a lost in-process
capability, not a lost process. The composite owner must preserve this posture
rather than inventing a serialized owner token or hidden persistence abstraction.

Signal's corresponding port is documented in
[`../worth-signal/BRANCH_BASES.md`](../worth-signal/BRANCH_BASES.md#owner-component-port).
