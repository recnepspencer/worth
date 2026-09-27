# Milestone 3 Closeout: WAL-Coordinated Durable Mode And Crash Recovery

## Status

Milestone 3 is closed as of 2026-04-14.

`worth-store` now has a real crash-safe durable-mode boundary instead of a
best-effort hosted runtime wrapper.

The semantic center shipped in this milestone is:

durable hosted mutations now cross one explicit publication chain, emit
append-only WAL artifacts, acknowledge only after authoritative publication is
recoverably complete, and restart through a typed recovery lane that converges
to the same retained truth as full rebuild from canonical authoritative
artifacts.

This is not "we added a log." The store now owns:

- explicit durable-mode recovery gating before writes resume
- append-only WAL families for admitted mutation intent, canonical result,
  publication progress, and recovery decisions
- durable mutation identity and retry-resolution surfaces
- a real publication subdomain with proof-bearing internal phase progression
- a real recovery planning and recovery execution subdomain
- typed crash-boundary recovery outcomes for discard, retain, finish, and
  duplicate suppression
- machine-checkable Milestone 3 certification bundles
- hostile crash-boundary and corruption coverage across JSON-file and SQLite
  persistence lanes

## Shipped Scope

Milestone 3 delivered:

- WAL artifact families and integrity verification in
  [wal/mod.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/wal/mod.rs)
- a dedicated publication subdomain with phase-bearing internal wrappers in
  [publication/mod.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/publication/mod.rs)
- recovery planning and recovery execution subdomains in
  [recovery/planning.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/recovery/planning.rs)
  and
  [recovery/execution.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/recovery/execution.rs)
- durable-mode lifecycle and recovery gating in
  [modes/durable.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/modes/durable.rs)
- WAL-aware backend support in
  [backend/engine.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/backend/engine.rs),
  [backend/records.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/backend/records.rs),
  and
  [backend/sqlite.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/backend/sqlite.rs)
- WAL state persistence and integrity verification inside the backend state and
  integrity layers in
  [backend/state/wal.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/backend/state/wal.rs)
  and
  [backend/integrity/verification.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/backend/integrity/verification.rs)
- Milestone 3 evidence surfaces in
  [evidence/milestone_3.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/evidence/milestone_3.rs)
  plus M3 counter extensions in
  [evidence/counters.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/evidence/counters.rs)
- typed recovery and durable-mode failure families in
  [failure/mod.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/failure/mod.rs)
- crash-boundary scenario coverage in
  [tests/wal_recovery.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/tests/wal_recovery.rs)
- Milestone 3 certification coverage in
  [tests/milestone_3_certification.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/tests/milestone_3_certification.rs)

## Acceptance Mapping

Milestone 3 is considered closed against the roadmap and
[test-requirements.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/test-requirements.md)
because the required acceptance surfaces are now covered directly.

### `WAL crash boundary exactness test`

Covered by:

- `tests::wal_recovery::crash_before_ack_discards_unpublished_intent`
- `tests::wal_recovery::crash_after_authoritative_publication_retains_truth_and_resolves_retry`
- `tests::wal_recovery::crash_after_acknowledgment_retains_truth_exactly_once`
- `tests::wal_recovery::repeated_crash_restart_loops_converge_to_same_truth_as_rebuild`

What is proven:

- crash before acknowledgment leaves no partially published authoritative truth
- crash after authoritative publication but before acknowledgment retains
  published truth and resolves retry through a typed equivalent-commit path
- crash after acknowledgment retains truth exactly once and collapses restart
  into duplicate suppression rather than replay
- repeated crash-restart loops converge to the same authoritative export as full
  rebuild from canonical authoritative artifacts
- restart becomes quiescent for already-closed durable mutations instead of
  reopening historical durable work forever

### `Machine-checkable Milestone 3 certification bundle`

Covered by:

- `tests::milestone_3_certification::milestone_3_certification_bundle_proves_recovery_and_rebuild_equivalence`
- `tests::milestone_3_certification::milestone_3_certification_bundle_captures_typed_recovery_failure`

What is proven:

- Milestone 3 emits `truth_digest`, `replay_digest`, `restore_digest`,
  `failure_digest`, and `counter_snapshot`
- a clean recovery lane emits a deterministic empty failure digest instead of a
  meaningless non-empty placeholder
- a corrupted WAL lane emits a typed failure that flows into the certification
  bundle through `ObservedRecoveryFailure`
- certification output remains machine-checkable rather than relying on log
  reading or stringly success checks

### `Durable recovery gating and retry resolution`

Covered by:

- `tests::wal_recovery::crash_before_ack_discards_unpublished_intent`
- `tests::wal_recovery::crash_after_authoritative_publication_retains_truth_and_resolves_retry`
- `tests::wal_recovery::repeated_crash_restart_loops_converge_to_same_truth_as_rebuild`

What is proven:

- durable restart enters an explicit recovery handle before mutation admission
- `DurableRecoveryHandle::plan()` exposes pending durable work rather than a
  ceremonial empty plan
- retry resolution distinguishes acknowledged equivalent commit, not previously
  published, and higher-level-policy cases through a typed surface

### `WAL integrity and typed failure localization`

Covered by:

- `tests::milestone_3_certification::milestone_3_certification_bundle_captures_typed_recovery_failure`
- existing persistence corruption lanes in
  [tests/persistence.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/tests/persistence.rs)

What is proven:

- corrupted WAL digests fail explicitly as `WalDigestMismatch`
- SQLite-backed durable stores reject damaged WAL state before recovery is
  allowed to proceed
- authoritative corruption and WAL corruption both fail through store-owned
  typed errors rather than backend-driver jargon

## Additional Hardening Added Before Close

Milestone 3 closeout includes these extra hardening outcomes beyond the minimum
roadmap labels:

- durable publication was extracted into its own subdomain instead of remaining
  buried inside the durable-mode handle
- the durable publication path now uses internal phase-bearing wrapper types so
  the order `admit -> canonical result -> authoritative publication ->
  acknowledgment` is carried structurally instead of only by local variable
  discipline
- recovery planning and recovery execution were separated so the restart lane is
  no longer one mixed helper path
- recovery planning was tightened so already-closed durable mutations stop
  re-entering restart work forever
- Milestone 3 certification was hardened with a real after-ack crash lane and a
  real typed failure-digest lane rather than non-empty-digest placeholders

These changes were made because the bar for `worth-store` is certifiable crash
exactness, not "restart usually seems to work."

## Explicit Deferrals

Milestone 3 intentionally does not include:

- snapshot capture or snapshot-plus-tail restore
- branch delta layering and delta-stack rewrite policy
- retention-driven WAL archival or compaction beyond what is needed for honest
  crash recovery
- live-query continuation
- replication capsules or import/export
- bulk-ingest resumability beyond one durable commit path
- budget admission control
- higher-level operator policy surfaces beyond typed recovery failure and retry
  resolution

Those remain later roadmap milestones and were not implied early here.

## Verification Baseline

At closeout, the crate verification baseline is:

- `cargo test -p worth-store`

This passes cleanly and includes:

- 38 runtime tests
- 1 compile-fail harness
- 3 compile-fail UI boundary cases
- before-ack, after-publish, and after-ack crash lanes
- repeated crash-restart versus rebuild parity
- typed WAL corruption certification
- existing Milestone 1 and Milestone 2 parity and integrity coverage

## Operational Conclusion

Milestone 3 is now closed at the store level.

`worth-store` no longer depends on process luck or startup convention to make
durable mode believable. It now has an explicit durable publication chain, a
typed WAL artifact model, crash recovery and rebuild control lanes, typed retry
resolution, typed WAL corruption failure, machine-checkable Milestone 3
certification evidence, and restart behavior that converges instead of drifting.
