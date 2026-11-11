# Classifications requested in integrate_pending.md (P-C, 2026-11-10)

## 1. `authored_index_writes.rs:71` "recovered authored index writes disagree with original source" on journal replay

HAPPY product defect, FIXED in `54b4fa230d`:
`crates/worth-relational/src/durability/authority/runtime_rebuild/index_catalog.rs::readmit` now reinstalls the
installed index routing after a non-empty journal catalog delta. Cause: a definition insert clears routing
(`runtime/state/subsystems/indexing/definition_catalog.rs:91`); live `IndexAuthority::register` reinstalls it,
checkpoint restore reinstalls it, journal-delta readmission did not, so replay derived index writes as
Unavailable where the original commit had Complete.
Evidence: both Host courts H8 reported PASS in the first Query Tier 2 (`fable-repair-query-tier2-b.log`):
`journal_only_killed_home_reopens_original_inputs_and_commits_without_initializing` and
`close_checkpoint_exhaustion_reports_failure_and_preserves_journal_reopen`.
Store `partial_released_key_admission_retries_same_home_without_duplicate_retention`: see the Store
selected-failure lane (`fable-repair-store-focused.log`) when it finishes.
Not covered by this fix (separate paths, Root family F3, Query family Q1): routing/selection loss after a
checkpoint or catalog restore; P-C `2dce9c178c` addresses the Complete-checkpoint selection half.

## 2. Close checkpoint's 60-second maintenance deadline ("CheckpointRecoveryRequired" under a starved machine)

Classification: **C2 (cost and bounds)**, with a track-H court rule.

- Mechanism: `ApplicationHomeCheckpointLimits::new` sets `maintenance_timeout = 60 s`
  (`workspaces/worth-store/crates/worth-store-application/src/checkpoint/limits.rs:20`). The maintenance worker
  turns it into a wall-clock deadline on a child request (`checkpoint/maintenance/worker.rs:154-161`), and the
  close path runs its checkpoint through that worker (`Purpose::Close`). When it elapses, close refuses with the
  typed `ApplicationHomeCloseDenial::CheckpointRecoveryRequired` and keeps custody
  (`world_section/close/checkpoint.rs:35`, `close/posture.rs:293-309`).
- So the failure is typed and loses nothing: no correctness hole, no data at risk. What is wrong is that the
  outcome of a graceful close is decided by machine speed, which is a timing trigger.
- The bound already has a deterministic twin: `maintenance_work` (1 << 30) and `maintenance_memory_bytes` are
  declared in the same policy. The principled end state is that close is bounded by the declared work and memory
  allowance plus the caller's own deadline and cancellation, and never by a self-imposed wall clock. That changes
  the public scheduling policy (`with_maintenance_allowance(work, memory, timeout)`), so it is a C2 bounds item,
  not a graceful-path repair.
- Track H rule until C2 lands: graceful-close courts must not pass or fail on wall time. A court that flakes on
  this deadline is not a product regression. It is recorded against C2 with its name; it is never retried until
  green, and never given a larger timeout to hide the trigger.
- Spec action at the Landing 2a spec update: add to C2 "close-path maintenance is bounded by declared work and
  memory and the caller's deadline; remove the internal wall-clock maintenance timeout from the close decision".
