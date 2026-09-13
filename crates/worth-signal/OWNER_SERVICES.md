# Signal Owner Services

## What This Feature Is

Signal owner services are the shared-borrow entry points for working on one
owner-managed branch without borrowing the whole `SignalRuntime` mutably.
Phase 5/6 install the real owner root, registry, independent branch cells,
managed-reference admission, concrete weak ports, and adversarial evidence
described here. The public bundle delegates to that one canonical owner; it is
not a second graph, head, lifecycle, or authority table.

## Why You Use It

- Keep a stable reference to a live branch while its exact state changes.
- Run unrelated branch work through independently synchronized owner cells.
- Carry exact bases and retention obligations without turning them into branch
  lifecycle authority.
- Receive typed owner, basis, cancellation, capacity, and lifecycle outcomes.

## Stable Entry Points

`worth_signal::facade::branch` exports the owner-issued
`ManagedSignalBranchReference`, exact basis/snapshot/retention vocabulary,
`SignalOwnerServicePorts`, and the three concrete weak ports. Issuance is
`SignalRuntime::owner_component_services(&mut self)`, once per canonical
partition after construction-only configuration is complete. It is the
one-way seal transition and requires `D`, `I`, `E`, `Ctx`, and `T` to satisfy the published
`Send + Sync + 'static` composition bounds. Every port and the bundle is
`Send + Sync + Clone`; none retains the runtime strongly or exposes a public
constructor. Operation control is an additional `test-operation-control`
facade export for deterministic tests only and is absent from normal builds.

The exact public contracts and their canonical owners are recorded below so
service users and parallel implementation lanes cannot improvise weaker inputs
or duplicate authority.

## Installed Shared-Contract Gate

The owner admits at most 64 operations. Admission reserves the packed
lifecycle phase/count before publishing one record into a fixed owner-owned
64-slot table. Each record carries the real `ThreadId` and its atomic hold
posture; there is no TLS semantic ledger, hashed thread identity, global thread
registry, owner-global executor, or allocation on record release. Capacity and
closing denial are pre-effect, and reservation unwind returns both the count
and exact slot.

An admission is thread-affine and cannot be sent or shared with another thread.
Before any metadata, registry, or branch-cell lock, the owner scans its bounded
record table and rejects same-thread/same-owner reentry if any admission on that
thread already holds owner metadata or a branch cell. This includes a fresh
admission, an earlier idle admission, and a different target branch. A hold
borrows its published admission, so its record cannot disappear while the hold
is live. Other-thread contention on one cell continues to serialize normally.
Record-scan work is reported separately as `admission_records_scanned`; it is
not branch-registry work and is never represented as zero-cost bookkeeping.

`Open -> Closing` fences new admissions while already admitted synchronous
calls retain their exact reservations and outcomes. Explicit close first
rejects any live admission on its executing thread, before changing phase, and
otherwise waits without missed wakeups. Root destruction requests close without
waiting. One cleanup claimant drains registry and metadata in actual batches of
at most 64, drops detached heavy state outside their locks, preserves the
retention obligation ledger and bounded retirement receipts, and publishes
`Closed` only after cleanup finishes. `close_batches` counts nonempty cleanup
batches rather than lifecycle phase changes. The last admitted release performs
cleanup when root destruction initiated close; no background worker owns this
transition. Cleanup ownership is an unwind-safe claim: a panic while processing
an actual batch releases the claim and wakes waiting closers, so one of them can
resume the remaining batches and publish `Closed`.

Direct admitted or external retention acquisition on the sealed owner borrows
the operation's existing exact admission and an owner-admitted exact basis; a
serializable basis descriptor is never retention authority. Acquisition neither
creates a second admission nor consults a separate open flag. Foreign or expired
admissions, basis-owner mismatch, and executing-thread owner reentry are rejected
before retention-ledger contact. Consequently, an operation admitted before the
`Open -> Closing` fence may still reserve and convert its lawful outputs while
`Closing`, whereas fresh work cannot obtain an admission during `Closing` or
`Closed`. The raw retention registry remains the lifecycle-agnostic obligation
owner beneath this admitted boundary, but it has no reachable descriptor-only
acquisition entry point.

An exact branch retirement reservation is also the canonical acquisition fence
for that branch. Retention acquisition takes the short metadata guard before the
retention ledger: an acquisition that inserts first is counted by the later
retirement reservation, while a reservation that installs first rejects both
admitted-output and external acquisition before ledger contact. The guard is
dropped before cell work, and unrelated branches remain independently
retainable. Retirement samples admitted/reserved and external counts together
after installing the metadata reservation, so `RetainedAdmittedBasis` and
`RetainedComponentBasis` report their distinct exact obligations.

Retirement planning on the sealed owner consumes the caller's existing
operation admission and exact admitted basis; it never performs a second
admission. The owner validates the basis runtime and definition before registry
contact, observes lineage/merge metadata before the retention ledger, releases
those short guards, and then contacts the canonical branch cell once for its
real handle and complete current observation. Planning creates only the
existing linear `PlannedSignalBranchRetirement`: it installs no retirement
reservation, receipt reservation, registry posture, or retention fence.
Snapshot-aware planning accepts exactly one baseline admitted lease plus each
unique owner-issued snapshot retention identity; duplicate snapshot handles do
not inflate the allowance, and runtime or branch mismatch remains a distinct
typed denial. `reserve_retirement` and the checked target-cell execution still
recheck lineage, retention, complete basis, and sole-holder custody at the
canonical effect boundary; planning does not replace that fence.

The canonical retention owner also exposes private lane-ready reservations:

- advance reserves one admitted output;
- capture reserves two admitted outputs from one snapshot plus a refreshed
  basis;
- restore reserves one admitted output;
- fork reserves one destination admitted output and rebinds its pending
  reservation to the owner-issued child before that child is published.

These reservations consume the existing 4096 admitted-lease capacity before
movement. Pending slots are not reported as issued authority. Conversion is
infallible after movement even if close has begun, while cancellation, denial,
panic, or unused drop returns exactly the remaining slots. The reservation
methods pair the actual owner, borrowed operation admission, and checked
canonical cell before executing that cell's corresponding operation. Their
sealed ready conversions accept no unrelated raw outcome and cannot outlive the
admitted synchronous call. The reservation objects live outside the retention
mutex and require no metadata lock across cell work. They are private kernel
seams, not descriptors, public constructors, or a second graph/head authority
table.

A snapshot reservation binds capacity to both the selected branch and its exact
installed cell incarnation before movement. Foreign-owner validation precedes
that custody check; a sibling or replaced same-id cell is rejected before cell
contact. Snapshot movement installs the matching metadata packet before the
faultable `AfterCanonicalMovement` seam, so an unwind cannot expose a cell head
whose snapshot state is absent from owner metadata.

Owner selection now carries the actual current branch from the sealed
partition; it never infers the minimum id, and the selected pointer names the
same canonical cell stored by the registry. A short metadata reservation
linearizes fork lineage against retirement before cell acquisition. Retirement
also reserves one of 4096 compact receipt slots before effect, fences and checks
live admitted/pending plus external retention before registry removal, and
preconstructs the exact receipt at the movement boundary. A performed receipt
remains recoverable from cell
custody across a caller unwind; no-effect cancellation returns capacity and
reopens only a still-live cell. Retired inert cells are removed and never
relabeled live.

Fork holds source custody only through exact capture and the source mutation-
journal boundary. Custody is released before destination binding or install.
Destination installation and lineage then linearize before the faultable
`ForkDestinationInstallation` and `OutcomeConstruction` seams, so a later
unwind preserves the performed canonical child while returning unissued output
retention. Pre-linearization unwind still releases the reservation; a panic in
source capture quarantines only that source incarnation.

Restore selection now validates a complete `ReadyBranchLifecycleTransfer`
before moving outgoing state or removing the stored target. Its commit is
infallible, preserving raw local restore and unknown-portable import behavior
without a partial fallback lane.

## Core Mental Model

A managed branch reference and an exact basis answer different questions.

- `ManagedSignalBranchReference` identifies one Signal owner and one installed
  branch-cell incarnation. It lets that owner revalidate where to observe or
  readmit. It does not say which generation or snapshot is current.
- `AdmittedSignalBranchBasis` proves one exact owner-admitted state and carries
  its admission retention. It is the expected input for compare-and-move.
- `SignalBranchRetentionLease` keeps one exact target available. It does not
  keep the branch reference current or the runtime alive.

The managed reference contains a concrete `worth-proof` carrier specialized to
a sealed Signal marker. Its owner and cell identities, weak lifecycle binding,
and proof fields are private. It has no public constructor, deserialization,
raw-id conversion, generic authority parameter, or adapter route. Cloning it
copies only the same weak reference contract: no runtime, cell, snapshot, basis,
or lease becomes strongly retained.

Existing owner-root compatibility helpers first compare the reference's sealed
runtime, lifecycle, and weak-allocation affinity with their own identity. This
preserves their inherited `ForeignOwner`-before-admission precedence. Future
weak ports first upgrade their owner binding; upgrade failure is exactly
`OwnerUnavailable` and never fabricates a branch id. After a successful upgrade,
the same affinity checks precede registry contact. A matching owner admits once
and carries that single admission through subordinate helpers; helpers do not
admit a second time. Retirement and replacement are
`BranchLifecycleEnded`/`BranchIncarnationReplaced`. Descriptive branch identity
equality is never consulted as proof.

## Published Bundle Contract

The published issuance shape is:

```rust,ignore
pub fn owner_component_services(
    &mut self,
) -> Result<SignalOwnerServicePorts<D, I, E, Ctx, T>, SignalOwnerServiceIssuanceDenial>
```

Issuance consumes the legacy canonical partition once and leaves
`SignalRuntime` as the sole non-cloneable strong owner root. The composition-
capable issuance method is available only when `D`, `I`, `E`, `Ctx`, and `T`
satisfy `Send + Sync + 'static` in addition to their existing operation bounds
(`D` remains `Copy + Ord + Debug`; `I` and `T` remain `Copy + Ord`). This is an
issuance capability fence, not permission to retain `E`, `Ctx`, or a callback.
Each operation borrows its caller-owned `Ctx` synchronously, and `F` itself has
no added `Send`, `Sync`, or `'static` bound.

Issuance denial is exact and pre-effect:

- `EventSubscriberStateConfigured`, `ObservationRegistrationStateConfigured`,
  and `ManagedQueueStateConfigured { bound_queue_count }` report construction
  state incompatible with independent owner services;
- `LiveBranchCapacityExhausted { maximum_live_branches }` and
  `RetirementReceiptCapacityExhausted { maximum_retained_receipts }` report the
  bounded partition that cannot be transferred.

The caller may remove the reported incompatible construction state or reduce
the relevant live/retained population and retry. A denial leaves the runtime
unsealed; a successful call seals exactly once, and later calls only reissue
weak ports to that same owner.

The bundle accessors are exactly `basis_port()`, `mutation_port()`, and
`lifecycle_port()`. The bundle and every returned port are cloneable weak
bindings; they expose no constructors, close authority, cells, registries, or
generic service trait. With `test-operation-control`, a sealed runtime also issues
`owner_operation_control() -> Result<SignalOwnerOperationControl, SignalOwnerUnavailable>`;
this handle names real progression boundaries but cannot mint authority or
change production semantics.

After sealing, old construction-state entry points intentionally panic before
access: graph/config mutation and validation, checkpoint and telemetry reads,
resource and temporal summaries, detached diagnostics, subscriber/listener
configuration, and graph-backed observer/materializer access. None returns an
empty replacement as canonical truth.
`switch_branch` returns `SignalError::InvalidInput`; branch selection, catalog,
ancestry, and head reads use the owner. Legacy basis, mutation, retention,
retirement, and batch operations delegate to that owner.
`advance_signal_branch` delegates to `advance_exact`; its synchronous callback is
the supported post-seal transaction/view value-read path and cannot retain `Ctx`
or block on the held cell. Reconstruction returns `NonPristineRuntime`.
Surface coverage is `signal_owner_services::legacy_surface::sealed_non_main_selection_and_catalog_are_canonical_through_root_and_observer`,
`signal_owner_services::legacy_surface::detached_construction_state_surfaces_panic_before_access_and_leave_owner_healthy`, and `signal_owner_services::legacy_surface::portable_reconstruction_is_non_pristine_after_owner_sealing`; delegation is exercised by `signal_owner_services::legacy_cutover::legacy_root_calls_cross_issuance_without_a_second_branch_state_lane` and `signal_owner_services::legacy_cutover::sealed_legacy_batch_fences_child_before_parent_and_retains_receipts`.

## Frozen Method Matrix

Every receiver is `&self`. `SignalOwnerCancellationToken` is explicit only for
operations that can cross the pre-movement cancellation cutoff. The canonical
owner column names where the decision remains after the public method exists.

### Basis port

| Exact method | Inputs | Output | Cancellation | Canonical owner | Named case / executable lane |
| --- | --- | --- | --- | --- | --- |
| `issue_managed_branch_reference` | `&AdmittedSignalBranchBasis` | `Result<ManagedSignalBranchReference, ManagedSignalBranchReferenceAdmissionDenial>` | none; bounded admission only | owner lifecycle + registry cell incarnation | public healthy: `signal_owner_services::facade_smoke::public_facade_issues_weak_ports_over_the_canonical_owner`; no standalone issuance-denial case; kernel: `branch::owner_services::tests::managed_reference::owner_issued_reference_reenters_one_live_cell_without_retaining_exact_state` |
| `observe_current` | `&ManagedSignalBranchReference` | `Result<AdmittedSignalBranchBasis, SignalBranchBasisObservationDenial>` | none; read-only target-cell contact | checked target cell + admission retention registry | public: `signal_owner_services::signal_world::baseline::cargo_routing_baseline_is_real_and_publicly_observable`, `signal_owner_services::signal_world::lifecycle::retired_child_denies_new_work_while_the_owner_and_unrelated_branch_remain_healthy`, `signal_owner_services::independent_oracle::model_sequences::public_trace_seeded::seeded_public_trace_matches_an_independent_oracle_and_covers_terminal_outcomes`; feature race: `signal_owner_services::adversarial::operation_control::observation_race::same_branch_observe_retire_returns_a_complete_pre_or_post_state`; kernel: `branch::owner_services::tests::managed_reference::canonical_movement_stales_exact_basis_without_staling_managed_reference`, `branch::owner_services::basis_port::tests::descriptor_denials::foreign_managed_authority_denies_before_receiving_owner_registry_contact` |
| `readmit_exact` | `&ManagedSignalBranchReference`, `&SignalBranchBasisDescriptor` | `Result<AdmittedSignalBranchBasis, SignalBranchBasisReadmissionDenial>` | none; read-only compare | checked target cell + admission retention registry | public healthy/stale/foreign: `signal_owner_services::independent_oracle::model_sequences::public_trace_seeded::seeded_public_trace_matches_an_independent_oracle_and_covers_terminal_outcomes`, `signal_owner_services::signal_world::baseline::cargo_routing_mutation_changes_effects_and_stales_the_consumed_basis`, `signal_owner_services::signal_world::facade::concrete_facade_ports_compile_and_cover_the_owner_method_shapes`; kernel: `branch::owner_services::basis_port::tests::method_matrix::basis_port_observation_and_readmission_method_matrix_uses_one_real_cell`, `branch::owner_services::tests::managed_reference::transaction_panic_quarantines_managed_readmission_without_unknown_branch` |
| `compare_current_exact` | `&AdmittedSignalBranchBasis` | `Result<(), SignalBranchBasisReadmissionDenial>` | none; read-only compare | concrete basis authority + its target cell | public healthy only: `signal_owner_services::signal_world::facade::concrete_facade_ports_compile_and_cover_the_owner_method_shapes`, `signal_owner_services::signal_world::baseline::cargo_routing_baseline_is_real_and_publicly_observable`; no public denial case; kernel: `branch::owner_services::basis_port::tests::method_matrix::basis_port_observation_and_readmission_method_matrix_uses_one_real_cell` |
| `readmit_retained_exact` | `&SignalBranchBasisDescriptor`, `&SignalBranchRetentionLease` | `Result<AdmittedSignalBranchBasis, SignalBranchRetainedReadmissionDenial>` | none; read-only retained-target admission | issuing retention ledger + exact target admission | public healthy only: `signal_owner_services::independent_oracle::model_sequences::public_trace_seeded::seeded_public_trace_matches_an_independent_oracle_and_covers_terminal_outcomes`; no public denial case; kernel: `branch::owner_services::basis_port::tests::method_matrix::retained_readmission_preserves_historical_target_and_release_custody` |
| `retain_exact` | `&AdmittedSignalBranchBasis` | `Result<SignalBranchRetentionLease, SignalBranchRetentionAcquisitionDenial>` | pre-effect, no token | retention registry | public healthy: `signal_owner_services::signal_world::facade::concrete_facade_ports_compile_and_cover_the_owner_method_shapes`, `signal_owner_services::independent_oracle::model_sequences::public_trace_seeded::seeded_public_trace_matches_an_independent_oracle_and_covers_terminal_outcomes`; ignored capacity: `signal_owner_services::adversarial::capacity_cleanup::retention_capacity_denies_then_all_releases_restore_one_lease`; kernel: `branch::owner_services::tests::retention_lifecycle::prior_admission_retains_acquisition_rights_during_closing_and_closed_denies_fresh_work`, `branch::owner_services::basis_port::tests::retention_capacity::retention_capacity_denies_every_artifact_path_without_leak_then_reopens` |
| `release_exact` | `SignalBranchRetentionLease` | `SignalBranchRetentionReleaseOutcome` | terminal and non-cancellable | weak port admission, then issuing retention ledger | public: `signal_owner_services::signal_world::facade::concrete_facade_ports_compile_and_cover_the_owner_method_shapes`, `signal_owner_services::independent_oracle::model_sequences::public_trace_seeded::seeded_public_trace_matches_an_independent_oracle_and_covers_terminal_outcomes`; kernel: `branch::owner_services::basis_port::tests::method_matrix::foreign_retention_custody_returns_the_live_lease_to_its_issuer`, `branch::owner_services::tests::retention_lifecycle::direct_lease_terminality_linearizes_before_or_during_owner_close` |
| `owner_lifecycle_observation` | none | `SignalOwnerLifecycleObservation` | not applicable | owner lifecycle | public: `signal_owner_services::facade_smoke::public_facade_issues_weak_ports_over_the_canonical_owner`, `signal_owner_services::signal_world::lifecycle::retired_child_denies_new_work_while_the_owner_and_unrelated_branch_remain_healthy`; feature boundary: `signal_owner_services::adversarial::operation_control::close::close_fences_new_work_but_releases_an_already_admitted_operation`; kernel: `branch::owner_services::lifecycle_port::tests::lifecycle_observation::weak_port_observes_open_closing_closed_and_owner_loss` |
| `owner_service_cost_snapshot` | none | `Result<SignalOwnerServiceCostSnapshot, SignalOwnerUnavailable>` | not applicable | owner counters | public: `signal_owner_services::adversarial::cost::one_public_advance_reports_one_local_structural_delta`, `signal_owner_services::adversarial::cost::closed_basis_port_reports_owner_unavailable`; kernel: `branch::owner_services::basis_port::tests::method_matrix::lifecycle_and_cost_inspection_account_for_their_own_weak_upgrades`, `branch::owner_services::lifecycle_port::tests::lifecycle_observation::lifecycle_and_cost_inspection_account_for_their_weak_upgrades_only` |

`readmit_exact` is not descriptor-only: the managed reference admits the exact
owner and checked cell before comparing the descriptor. The compatibility
`SignalRuntime::readmit_signal_branch_basis(descriptor)` remains owner-root-only.
`readmit_retained_exact` instead uses the lease's inherited owner and exact
historical target, with no managed reference or currentness comparison.
`compare_current_exact` validates receiving-owner, definition, and branch
affinity before comparing complete exact state against that cell. The public
denial matrix must separate same-Rust-type foreign authority from state drift.

Managed-reference admission is mapped without flattening: matching-owner loss
is top-level `OwnerUnavailable`, while foreign affinity, ended/replaced
incarnation, visible retirement, and invariant failure use the additive
`ManagedReferenceDenied` variant. After admission returns its checked cell, a
retirement race keeps the cell's `RetirementInProgress` or `RetiredBranch`
denial. Public methods must continue from that cell, never a second raw-id lookup.

### Typed installed-cell posture

Once a registry lookup returns an installed target cell, its posture is never
flattened into `UnknownBranch` or a diagnostic string. `advance_exact`,
`fork_exact`, `capture_exact`, `restore_exact`, and retirement planning or
execution preserve operation-specific `RetirementInProgress { branch_id }`,
`RetiredBranch { branch_id }`, `QuarantinedBranch { branch_id }`, or
`OwnerCellMisuse { branch_id }`; managed exact readmission preserves those and
owner-invariant postures in `SignalBranchBasisReadmissionDenial`.

`UnknownBranch` remains reserved for actual registry absence; owner invariants
and expired retirement custody retain distinct typed postures. A quarantined
cell is terminal contained-panic state: it cannot be readmitted, moved, or
retired, retains registry membership and one live-branch capacity slot until
owner destruction, and has no purge path. Unrelated cells remain available.
`OwnerCellMisuse` remains typed, while executing-thread reentry is additive
`OwnerReentry` without a fabricated branch id. Evidence is
`branch::owner_services::tests::cell_posture_outcomes::every_operation_preserves_reachable_cell_posture_without_unknown_fallback`
and `branch::owner_services::tests::managed_reference::transaction_panic_quarantines_managed_readmission_without_unknown_branch`.

Two inherited public root methods remain explicit compatibility surfaces rather
than disappearing into this port contract:

- `validate_signal_basis_compatibility(&AdmittedSignalBranchBasis,
  &AdmittedSignalBranchBasis) -> Result<(),
  SignalBranchBasisCompatibilityDenial>` compares two admitted artifacts. It
  does not compare either artifact with current owner state, so it is distinct
  from `compare_current_exact`. Current healthy and denial evidence is
  `branch_basis_contract::snapshot_and_restore_each_move_the_exact_reference`.
- `signal_component_retention_terminal_counts() ->
  SignalBranchRetentionTerminalCounts` is root inspection of recorded release,
  drop, and owner-loss totals. Current evidence is
  `branch_retention_lifecycle::explicit_release_returns_governed_exact_target_evidence`,
  `dropping_an_obligation_is_the_same_terminal_release`, and
  `an_obligation_outlives_its_owner_and_records_owner_loss`.

Both receivers remain `&self`. Phase 4/5 compatibility delegation must preserve
their behavior and denial meanings, but neither becomes a fourth port or a
second currentness/retention authority lane.

`release_exact` first upgrades the weak port. If that fails, it returns
`SignalBranchRetentionReleaseOutcome::Denied` with the still-live lease and
`SignalBranchRetentionReleaseDenial::OwnerUnavailable`. Calling
`SignalBranchRetentionLease::release` directly (or dropping it) remains the
lease's distinct terminal route and may report terminal owner loss.

### Mutation port

| Exact method | Inputs and operation-specific generic | Output | Cancellation | Canonical owner | Named case / executable lane |
| --- | --- | --- | --- | --- | --- |
| `fork_exact` | `ValidatedSignalBranchName`, `&AdmittedSignalBranchBasis`, `&SignalOwnerCancellationToken` | `Result<SignalBranchForkOutcome, SignalBranchForkOperationDenial>` | before source capture/installation; performed installation wins | source cell + bounded registry reservation | public healthy: `signal_owner_services::facade_smoke::public_facade_issues_weak_ports_over_the_canonical_owner`, `signal_owner_services::signal_world::facade::concrete_facade_ports_compile_and_cover_the_owner_method_shapes`; feature cancellation: `signal_owner_services::adversarial::operation_control::cancellation::pre_movement_cancellation_denies_every_cancellable_public_operation`; ignored capacity: `signal_owner_services::adversarial::capacity_cleanup::live_branch_capacity_denies_then_retirement_restores_one_slot`; kernel: `branch::owner_services::tests::fork_contracts::exact_fork_shares_graph_roots_and_isolates_touched_node_state`, `branch::owner_services::mutation_port::tests::baseline::fork_exact_returns_the_installed_owner_handle_without_reconstruction` |
| `advance_exact<F>` | `&AdmittedSignalBranchBasis`, `&mut Ctx`, `&SignalOwnerCancellationToken`, `F: FnOnce(&mut SignalTransaction<'_, D, I, E, Ctx, T>) -> Result<(), SignalError>` | `Result<SignalBranchAdvanceOutcome, SignalBranchAdvanceDenial>` | before canonical movement; performed movement wins | one target cell + transaction engine | public: `signal_owner_services::signal_world::baseline::cargo_routing_mutation_changes_effects_and_stales_the_consumed_basis`, `signal_owner_services::signal_world::lifecycle::same_branch_stale_and_retired_postures_are_exact_and_recoverable`, `signal_owner_services::independent_oracle::model_sequences::public_trace_seeded::seeded_public_trace_matches_an_independent_oracle_and_covers_terminal_outcomes`; feature cancellation: `signal_owner_services::adversarial::operation_control::cancellation::pre_movement_cancellation_denies_every_cancellable_public_operation`; kernel: `branch::owner_services::mutation_port::tests::denials::stale_matrix_cleans_every_reservation_and_allows_healthy_follow_up`, `branch::owner_services::mutation_port::tests::denials::advance_cancellation_requested_after_cutoff_cannot_erase_performed_truth`, `branch::owner_services::tests::cancellation::cancellation_while_waiting_for_same_cell_denies_without_movement` |
| `capture_exact` | `&AdmittedSignalBranchBasis`, `&SignalOwnerCancellationToken` | `Result<SignalBranchSnapshotCaptureOutcome, SignalBranchSnapshotCaptureDenial>` | before capture movement; performed capture wins | target cell + snapshot registry | public: `signal_owner_services::signal_world::facade::concrete_facade_ports_compile_and_cover_the_owner_method_shapes`, `signal_owner_services::independent_oracle::model_sequences::public_trace_seeded::seeded_public_trace_matches_an_independent_oracle_and_covers_terminal_outcomes`; feature cancellation: `signal_owner_services::adversarial::operation_control::cancellation_progress::pre_movement_snapshot_cancellation_returns_output_custody`, `signal_owner_services::adversarial::operation_control::cancellation_progress::post_movement_snapshot_cancellation_keeps_the_performed_capture`; kernel: `branch::owner_services::tests::exact_cell_contracts::exact_snapshot_and_restore_contracts_move_one_cell_and_install_metadata_between_locks`, `branch::owner_services::mutation_port::tests::denials::pre_movement_cancellation_matrix_is_no_effect_and_releases_capacity` |
| `restore_exact` | `&AdmittedSignalBranchBasis`, `&AdmittedSignalBranchSnapshot`, `&SignalOwnerCancellationToken` | `Result<AdmittedSignalBranchBasis, SignalBranchRestoreDenial>` | before restore movement; performed movement wins | target cell + snapshot registry | public: `signal_owner_services::signal_world::facade::concrete_facade_ports_compile_and_cover_the_owner_method_shapes`, `signal_owner_services::independent_oracle::model_sequences::public_trace_seeded::seeded_public_trace_matches_an_independent_oracle_and_covers_terminal_outcomes`; feature cancellation: `signal_owner_services::adversarial::operation_control::cancellation_progress::pre_movement_restore_cancellation_returns_output_custody`, `signal_owner_services::adversarial::operation_control::cancellation_progress::post_movement_restore_cancellation_keeps_the_performed_restore`; kernel: `branch::owner_services::tests::exact_cell_contracts::exact_snapshot_and_restore_contracts_move_one_cell_and_install_metadata_between_locks`, `branch::owner_services::tests::cancellation::restore::restore_cancellation_at_cutoff_denies_but_after_movement_is_performed_wins` |

`F` executes synchronously while the caller retains `&mut Ctx`; the owner does
not clone, register, return, or keep either value. Portable snapshot
reconstruction remains an owner-root construction compatibility operation and
is not a mutation-port method.

Snapshot reservations carry a checked, runtime-global identity as well as
storage capacity. Forking or restoring an older snapshot cannot reset that
identity allocator. Cancellation, denial, and unwinding return reservation
capacity without reusing its identity; exhaustion is
`SnapshotIdentityExhausted` before movement. Active-branch capture or
reconstruction stores an immutable snapshot and updates metadata, never a
second live mutable branch. These rules preserve distinct exact retention
targets across sibling captures and the handoff into owner cells.

`ValidatedSignalBranchName` names the frozen `fork_exact` input and is exported
through the curated branch facade; accepting an unvalidated `String` is not
compatible with this row.

### Lifecycle port

| Exact method | Inputs | Output | Cancellation | Canonical owner | Named case / executable lane |
| --- | --- | --- | --- | --- | --- |
| `plan_retirement_exact` | `AdmittedSignalBranchBasis`, `SignalBranchRetirementReason` | `TransitionOutcome<PlannedSignalBranchRetirement, SignalBranchRetirementDenial>` | pre-effect planning; no token | metadata then retention, followed by one target-cell contact | public: `signal_owner_services::signal_world::lifecycle::retired_child_denies_new_work_while_the_owner_and_unrelated_branch_remain_healthy`, `signal_owner_services::independent_oracle::model_sequences::public_trace_seeded::seeded_public_trace_matches_an_independent_oracle_and_covers_terminal_outcomes`; kernel: `branch::owner_services::tests::retirement_planning::owner_exact_retirement_plan_preserves_pre_effect_state_and_executes_real_handle`, `branch::owner_services::tests::retirement_planning::owner_retirement_planning_distinguishes_current_canonical_and_live_child`, `branch::owner_services::lifecycle_port::tests::planning_denials::planning_preserves_component_admitted_and_shared_holder_denials` |
| `plan_retirement_releasing_snapshots_exact` | `AdmittedSignalBranchBasis`, `&[&AdmittedSignalBranchSnapshot]`, `SignalBranchRetirementReason` | `TransitionOutcome<PlannedSignalBranchRetirement, SignalBranchRetirementDenial>` | pre-effect planning; no token | metadata then retention, followed by one target-cell contact | no direct public lifecycle-port case; compatibility coverage: `signal_owner_services::legacy_cutover::sealed_legacy_retirement_preserves_unknown_before_basis_mismatch`; kernel: `branch::owner_services::tests::retirement_planning::snapshots::owner_snapshot_release_plan_counts_unique_owner_issued_custody_exactly`, `branch::owner_services::tests::retirement_planning::snapshots::owner_snapshot_release_plan_denies_foreign_runtime_before_registry_contact`, `branch::owner_services::tests::retirement_planning::snapshots::owner_snapshot_release_plan_denies_real_wrong_branch_custody` |
| `retire_exact` | `PlannedSignalBranchRetirement`, `&SignalOwnerCancellationToken` | `TransitionOutcome<SignalBranchRetirementReceipt, SignalBranchRetirementDenial>` | before movement; performed retirement wins | target cell then short registry removal | public: `signal_owner_services::signal_world::lifecycle::retired_child_denies_new_work_while_the_owner_and_unrelated_branch_remain_healthy`, `signal_owner_services::independent_oracle::model_sequences::public_trace_seeded::seeded_public_trace_matches_an_independent_oracle_and_covers_terminal_outcomes`; feature cancellation: `signal_owner_services::adversarial::operation_control::cancellation_progress::post_movement_retirement_cancellation_keeps_the_performed_receipt`; kernel: `branch::owner_services::lifecycle_port::tests::retirement::retire_exact_performs_one_real_cell_movement_and_preserves_exact_receipt`, `branch::owner_services::tests::exact_retirement_contracts::exact_retirement_contract_consumes_a_linear_plan_before_registry_removal` |
| `owner_lifecycle_observation` | none | `SignalOwnerLifecycleObservation` | not applicable | owner lifecycle | public: `signal_owner_services::facade_smoke::public_facade_issues_weak_ports_over_the_canonical_owner`, `signal_owner_services::signal_world::lifecycle::retired_child_denies_new_work_while_the_owner_and_unrelated_branch_remain_healthy`; feature boundary: `signal_owner_services::adversarial::operation_control::close::close_fences_new_work_but_releases_an_already_admitted_operation`; kernel: `branch::owner_services::lifecycle_port::tests::lifecycle_observation::weak_port_observes_open_closing_closed_and_owner_loss` |
| `owner_service_cost_snapshot` | none | `Result<SignalOwnerServiceCostSnapshot, SignalOwnerUnavailable>` | not applicable | owner counters | public: `signal_owner_services::adversarial::cost::lifecycle_operations_report_exact_structural_deltas`, `signal_owner_services::adversarial::cost::closed_lifecycle_port_reports_owner_unavailable`; kernel: `branch::owner_services::basis_port::tests::method_matrix::lifecycle_and_cost_inspection_account_for_their_own_weak_upgrades`, `branch::owner_services::lifecycle_port::tests::lifecycle_observation::lifecycle_and_cost_inspection_account_for_their_weak_upgrades_only` |

Batch retirement remains a bounded owner-root compatibility family in this
handoff. It is intentionally not added to the composition port: 9.17.2 needs
individual component custody, and the specification permits a batch only “if
exposed.” Existing batch methods and outcomes remain available on the root and
are not deleted or reinterpreted.

## How It Executes

1. Admit the owner lifecycle.
2. Perform one short registry lookup or reservation.
3. Enter the checked target cell returned by reference admission, or at most one
   existing target cell selected by exact basis authority.
4. Compare the complete expected basis before movement.
5. Apply the canonical owner operation.
6. Release the cell before constructing the external outcome.
7. Update short retention, registry, or diagnostic accounting as required.

The lock order is lifecycle, registry metadata, one target cell, then short
retention/diagnostic accounting. No call holds two existing branch cells,
metadata/global owner locks, a Runtime World lock, or a runtime-wide mutex.
`advance_exact` intentionally executes its synchronous callback while holding
the one target-cell exclusion that protects mutation; no registry or other-owner
lock crosses that callback.

Callbacks must not synchronously wait on work that needs their held cell or
nest blocking calls across owners. Cross-owner wait-cycle detection is not
provided. The Phase 4 reentry fence is owner-scoped, not a global executor or
cross-owner lock registry.

## Small Example

The Phase 3 public vocabulary can be carried and cloned, but not constructed:

```rust
use worth_signal::facade::branch::ManagedSignalBranchReference;

fn carry(reference: &ManagedSignalBranchReference) -> ManagedSignalBranchReference {
    reference.clone()
}
```

Owner issuance and service calls are shown in the runnable
[`independent_branch_services.rs`](./examples/independent_branch_services.rs)
example. The small snippet above remains useful when a component only needs to
carry a managed reference.

## Real Example

The executable example builds a real `SignalRuntime`, obtains two owner-issued
branches, issues all required weak ports, advances both branches concurrently
without a whole-runtime mutable borrow, retains and explicitly releases one
exact basis, then drops the strong root and proves that the weak port reports
`SignalOwnerUnavailable`. It is intentionally facade-only and is the developer
workflow for the handoff into 9.17.2.

### Deterministic operation control

Tests compiled with `test-operation-control` may obtain
`runtime.owner_operation_control()` after owner-service issuance. The control
handle only names real owner boundaries: `arm_pause_once(boundary)` returns a
drop-safe pause whose `wait_until_reached(Duration)` is bounded and whose
`release()` resumes the owner; `inject_panic_once(boundary)` faults the next
operation at that boundary. It exposes no constructor, evaluator, authority,
or alternate engine. Schedules use channels/barriers, never sleeps. Unarmed
control must leave outcomes, observations, counters, and cleanup unchanged.

The boundary vocabulary is fixed to `OwnerLifecycleAdmission`,
`BranchRegistryLookup`, `BranchRegistryReservation`, `ExactBasisPreflight`,
`TargetCellAdmission`, `BeforeCanonicalMovement`, `AfterCanonicalMovement`,
`ForkSourceCapture`, `ForkDestinationInstallation`, `OutcomeConstruction`, and
`OwnerCloseBatch`. A schedule may park one boundary at a time; it must never
replace the canonical operation or make a private cell observable.

Adversarial cases park metadata work only long enough to prove that unrelated
branch cells continue, then release the guard and assert exact winner,
cancellation, panic, close, capacity, and structural-cost behavior. A parked
operation retains its own synchronous context and reservation; a cancellation
requested after canonical movement cannot erase its performed owner outcome.
Contained cell panics quarantine only the affected cell, while a sibling must
remain usable; a post-movement outcome-construction unwind preserves performed
truth and releases output custody. During `Closing`, already admitted work completes and new weak
calls deny; after cleanup, every weak port reports the same unavailable posture.

## How It Relates To Other Features

- Exact bases remain compare-and-move authority; managed references never
  replace them.
- External leases preserve exact residency, including historical state; they
  never preserve branch currentness or owner liveness.
- `SignalBranchHandle`, `SignalBranchIdentity`, descriptors, snapshot ids, and
  digests remain descriptive compatibility/inspection values.
- Relational owner ports are separate concrete owner services. Runtime World
  will coordinate both owners without restamping either authority.

## Inspection And Debugging

`SignalOwnerServiceCostSnapshot` reports owner upgrades, registry lookup and
reservation work, target-cell contacts/waits, movements, retention contacts,
fork capture/installation and copy work, diagnostic recording/omission, and
close batches. Counters are descriptive; they cannot authorize an operation.

Operational capacity denial is pre-effect. Diagnostic capacity exhaustion
records an omission/drop count and does not deny an otherwise lawful owner
operation. Closing rejects new admissions and gives every later weak call the
same typed unavailable posture. Direct lease termination linearizes under the
issuing retention ledger: a release that consumes its capability before close
may return `Released`, while a call beginning after completed close cannot
report `Live` merely because a temporary weak upgrade succeeded. Ledger entries
are not cleared at close. Explicit close rejects calls from that owner's
admitted execution thread before starting close; otherwise it waits for
admitted work and real cleanup. Root destruction requests close without waiting,
so destruction from inside an admitted callback cannot wait on itself.

## Anti-Patterns

- Do not construct authority from a `SignalBranchHandle`, branch identity,
  descriptor, generation, snapshot id, or digest.
- Do not use a managed reference as an exact basis or retention lease.
- Do not use descriptor-only readmission from composition code.
- Do not wrap `SignalRuntime` in a global mutex or define an adapter trait over
  these concrete ports.
- Do not keep `Ctx`, callbacks, cell guards, or owner admissions after the
  synchronous call returns.

## Current Limits

- Signal services decide only Signal component truth. Product currentness and
  composite publication belong to Milestone 9.17.2.
- Persistence, restart, replay, correction, and merge are outside this contract.

## Related Docs

- [`BRANCH_BASES.md`](./BRANCH_BASES.md)
- [`../worth-relational/OWNER_COMPONENT_PORT.md`](../worth-relational/OWNER_COMPONENT_PORT.md)
- [`../../_docs/WORTH-query/milestone-9.17.1.2.md`](../../_docs/WORTH-query/milestone-9.17.1.2.md)
