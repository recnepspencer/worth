# Remaining owner boundaries

## Changed cold region DATA (required funding proof)

Three Native changed-tail recapture cases and two Host recapture cases fail
Region(0):SourceUnavailable. Full logs: funding-finish-cold-recapture-corrected.log
and funding-finish-host-bounded-reopen-kill-ack.log. The unchanged selected
root carry and the honest discarded-branch growth court pass.

Search: `rg -n "capture_complete_for_request|lease_resident_full|fn capture_checkpoint"
crates/worth-relational/src/storage/residency`; complete owner reads of
checkpoint_capture/complete.rs, cell/resident_leases.rs, region_load.rs,
resident_history.rs, history_fold.rs and region_capture/complete.rs.

`capture_complete_for_request` requires `lease_resident_full`; that lease denies
any checkpoint-backed cell even if its Full DATA is cached. The prepared
one-file DATA-emitter extraction in proposed-selected-region-capture.patch
remains unapplied. It was independently approved only for same-cell DATA and
output funding, not for the input-materialization path.

The ordinary Full loaders build owned images/maps/vectors, consult the request
and bound input/output lengths, but return ResidentRegion with _memory:None.
HistoryFolder::charge_output counts serialized length against a read limit;
it does not reserve retained request memory or charge all reconstruction work.
Consequently demand_full followed by the new emitter is not a complete
request-funded repair. It would introduce unaccounted maintenance allocations.
Ordinary capture also wraps output as external bytes and cannot substitute.

Requested focused storage/residency owner handoff: either fund actual cold
DATA materialization before Full projection, or emit borrowed prefix-preserving
DATA with request-funded dependency directories and bounded consolidation.
Keep actual held-root and selected-base authority, original Full-only admission,
wire grammar and canonical-body exclusion. Any exact history-directory funding
constructor requires its data-owner handoff too. This prerequisite is beyond
the currently exact grants; none of those owner files has been edited. The
precise file set/design must be reviewed before an implementation.

## Initializer-refusal ordering (Query installation/integration)

Exact failing court:
initializer_refusal_reopens_at_missing_world_origin_without_replaying_initializer.
The unchanged test first verifies its actual initializer panic and closes the
partial Home. Reopen then gets CheckpointRecoveryRejected: recovered runtime
omitted Query index definition application-principal.IdentityBinding. Expected
public refusal is ExistingWorldRoot/OwnerUnavailable, and callback/lease
assertions remain unchanged. No test weakening or reclassification occurred.

Nearest analogues: root/index_installation_tests.rs; actual owner
root/index_installation.rs::IndexInstallation::RequireRecovered matches only
recovered definitions. Search `rg -n "IndexInstallationPosture::RequireRecovered|
IndexInstallationPosture::Register"` under Query primary_graph finds the
existing Home preparation call in application_installation/home_preparation/
existing.rs and fresh registration in home_preparation/native.rs. Those are
Lifecycle-owned/protected, not this funding grant. Host World selection occurs
later in opening/existing.rs::attach_world. No order change or fresh index
default has been applied. Integration/Lifecycle must settle preservation and
refusal order for this partial Home; this report does not claim which product
contract should be revised.

## Final observed closure

The final public Result diagnostic in funding-close-owner-store-actual-result-
corrected.log confirms Native(Capture(...Region(0):SourceUnavailable)) before
anchor registration. The5s gate timeout is secondary, not a performance claim.
Exact original Store court bytes are restored; no assertion or timeout changed.
The separate registration gap is fixed through the approved shared Store Home
worker design; fresh Host9/0/0 supports its local approval. Full funding remains
REJECT. No storage/residency/history-directory implementation is authorized yet.

No disk cleanup is proposed: other-worktree artifacts were not inventoried or
touched. At the conservative floor observation C had25,611,960,320 bytes free
(23.85GiB). Free space later fluctuated above that, but no further build/test
work was started after the stop. Reusable artifacts remain in existing targets.
