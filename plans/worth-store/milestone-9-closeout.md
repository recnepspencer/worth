# Milestone 9 Closeout: Deterministic Bulk Ingest And Bulk Transform Paths

Status: Completed on 2026-04-16

Parent spec: [milestone-9.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-9.md)

Roadmap: [worth_store_roadmap.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/worth_store_roadmap.md)

## Summary

Milestone 9 is closed.

`worth-store` now supports deterministic bulk ingest and bulk transform
programs that freeze their basis up front, lower through ordinary canonical
commit history, persist non-authoritative resume artifacts explicitly, recover
interrupted work through WAL-safe publication rules, and emit machine-checkable
certification evidence instead of relying on narrative parity claims.

The closure claim is:

- bulk programs execute through canonical commits rather than a second durable
  write model
- resumable checkpoints and chunk witnesses remain explicit support artifacts,
  never truth authority
- interruption, restart, and WAL recovery converge to the same canonical truth
  and history as the logically serial control lane
- deterministic chunk planning and bounded execution claims are carried into the
  machine-checkable certification surface

## What Shipped

- proof-bearing planning and basis freezing for ingest and transform programs in
  [crates/worth-store/src/bulk](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/bulk)
- persisted bulk support artifacts for manifests, transform bases, partitions,
  deterministic plans, chunk witnesses, checkpoints, and per-program witness
  indexes
- canonical and durable bulk execution surfaces in
  [crates/worth-store/src/facade.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/facade.rs)
- restart-path recovery, witness/checkpoint reconstruction, and recovered-resume
  admission backed by the existing recovery and bulk subsystems
- machine-checkable Milestone 9 certification output in
  [crates/worth-store/src/evidence/milestone_9.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/evidence/milestone_9.rs)
- named milestone certification coverage in
  [crates/worth-store/src/tests/milestone_9_certification.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/tests/milestone_9_certification.rs)

## Acceptance Mapping

Milestone 9 is considered closed against
[milestone-9.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-9.md)
and
[test-requirements.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/test-requirements.md)
because the required named suite and the supporting hostile evidence now map
directly to code and tests.

### `Bulk Ingest And Transform Resume Parity Test`

Covered by:

- [crates/worth-store/src/tests/milestone_9_certification.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/tests/milestone_9_certification.rs)

What is proven:

- interrupted ingest reaches the same final truth, canonical history, and
  restore-visible state as the logically serial control lane through
  `tests::milestone_9_certification::milestone_9_certification_bundle_proves_ingest_resume_control_and_restore_parity`
- interrupted transform reaches the same final truth, canonical history, and
  restore-visible state as the logically serial control lane through
  `tests::milestone_9_certification::milestone_9_certification_bundle_proves_transform_resume_control_and_restore_parity`
- WAL-recovered ingest reaches the same canonical outcome as the clean control
  lane through
  `tests::milestone_9_certification::milestone_9_certification_bundle_proves_wal_recovered_ingest_control_and_restore_parity`
- WAL-recovered transform reaches the same canonical outcome as the clean
  control lane through
  `tests::milestone_9_certification::milestone_9_certification_bundle_proves_wal_recovered_transform_control_and_restore_parity`
- deterministic chunk planning is proven by `chunk_plan_digest` equality between
  equivalent planning lanes inside the certification suite rather than by
  narrative assertion

### Supporting hostile recovery and integrity evidence

Covered by:

- [crates/worth-store/src/tests/wal_recovery.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/tests/wal_recovery.rs)
- [crates/worth-store/src/tests/bulk.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/tests/bulk.rs)

What is proven:

- missing checkpoints can be rebuilt from canonical chunk witnesses and
  published truth without duplicating chunk execution
- repeated crash/restart loops converge after hosted-result recovery and after
  published-truth recovery with partial or complete support artifacts
- transform basis drift and persisted transform-artifact drift fail typed rather
  than silently rebasing
- resume lookup remains bounded through explicit witness indexes and checkpoint
  families, with reopen-time corruption rejection for witness-count drift,
  ordinal regression, missing checkpoints, and checkpoint-sequence corruption
- fast-path/fallback visibility and bounded-memory evidence remain observable
  through `StoreCounterSnapshot` fields and the bulk cost/counter assertions in
  `tests::bulk`

### Supporting mechanical enforcement evidence

Covered by:

- [crates/worth-store/tests/phase_boundaries_compile_fail.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/tests/phase_boundaries_compile_fail.rs)
- `tests/ui/milestone_9_reference_exposes_no_commit_authority.rs`
- `tests/ui/partial_chunk_metadata_rejected.rs`

What is proven:

- milestone-facing references do not expose canonical commit authority across
  the bulk and physical-chunk boundary
- partial or raw chunk metadata cannot be smuggled across proof-bearing
  boundaries as though it were an admitted chunk surface
- the public milestone surface keeps compile-time boundary enforcement in the
  closeout lane instead of relying on documentation alone

## Acceptance Evidence

The closeout bundle now emits:

- `truth_digest`
- `history_digest`
- `restore_digest`
- `counter_snapshot`

The current bundle also carries:

- `chunk_plan_digest`

That additional digest is used to prove deterministic chunk planning across
equivalent admitted planning lanes instead of only asserting that a plan
exists.

## Certification Result

The Milestone 9 named suite now covers four explicit certification lanes:

- ingest resumed lane
- transform resumed lane
- WAL-recovered ingest lane
- WAL-recovered transform lane

## Verification

The final verification run used:

- `cargo test -p worth-store milestone_9_certification`
- `cargo test -p worth-store`

Both passed. The full crate verification run includes the compile-fail/UI
boundary suite in addition to the runtime tests.

## Residual Notes

No remaining in-scope Milestone 9 debt remains in the bulk ingest/transform
lane.

Future work still exists, but it belongs to later milestones rather than
Milestone 9 itself:

- retention / compaction / reclaim
- replication and capsule programs
- broader generic and domain certification expansion
- later budget and operational-envelope programs beyond the Milestone 9 bulk
  contract
