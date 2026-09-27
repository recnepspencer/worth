# Milestone 4 Closeout: Snapshot Persistence And Point-In-Time Restore

## Status

Milestone 4 is closed as of 2026-04-14.

`worth-store` now has a real snapshot substrate instead of an optimistic
"serialize some state" path.

The semantic center shipped in this milestone is:

immutable snapshots are now persisted as derived basis-plus-image artifact
families, point-in-time reads are basis-explicit, snapshot-plus-tail restore
uses the persisted snapshot as a real prefix instead of disguising full replay,
and deleted snapshot images can be rebuilt from canonical authoritative
artifacts without promoting snapshots into authority.

This is not "we added checkpoints." The store now owns:

- explicit snapshot basis, identity, version, and non-authority boundaries
- immutable snapshot publication with basis and image records
- point-in-time pure-snapshot and snapshot-plus-tail read surfaces
- real prefix-plus-suffix restore planning and execution
- snapshot rebuild from authoritative artifacts
- snapshot integrity verification across JSON-file and SQLite persistence lanes
- machine-checkable Milestone 4 certification bundles
- hostile snapshot corruption, publication-gap, and version-mismatch coverage

## Shipped Scope

Milestone 4 delivered:

- a dedicated public snapshot subdomain in
  [snapshot/mod.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/snapshot/mod.rs)
- snapshot basis and image records in
  [backend/records.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/backend/records.rs)
- snapshot state decomposition by responsibility in
  [backend/state/snapshots/basis.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/backend/state/snapshots/basis.rs),
  [backend/state/snapshots/image.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/backend/state/snapshots/image.rs),
  [backend/state/snapshots/read.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/backend/state/snapshots/read.rs),
  and
  [backend/state/snapshots/restore.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/backend/state/snapshots/restore.rs)
- backend snapshot orchestration in
  [backend/engine.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/backend/engine.rs)
  and
  [backend/facade.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/backend/facade.rs)
- SQLite snapshot persistence in
  [backend/sqlite.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/backend/sqlite.rs)
- snapshot integrity verification in
  [backend/integrity/snapshot_records.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/backend/integrity/snapshot_records.rs)
- shared authoritative export rebuild support in
  [backend/export.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/backend/export.rs)
- Milestone 4 evidence and counter extensions in
  [evidence/milestone_4.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/evidence/milestone_4.rs)
  and
  [evidence/counters.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/evidence/counters.rs)
- public facade support in
  [facade.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/facade.rs)
- snapshot scenario coverage in
  [tests/snapshots.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/tests/snapshots.rs)
- Milestone 4 certification coverage in
  [tests/milestone_4_certification.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/tests/milestone_4_certification.rs)

## Acceptance Mapping

Milestone 4 is considered closed against the roadmap and
[test-requirements.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/test-requirements.md)
because the required acceptance surfaces are now covered directly.

### `Snapshot-plus-tail restore equivalence test`

Covered by:

- `tests::snapshots::snapshot_plus_tail_restore_matches_direct_point_in_time_read`
- `tests::snapshots::snapshot_rebuild_uses_basis_when_image_is_missing`
- `tests::milestone_4_certification::milestone_4_certification_bundle_proves_restore_and_rebuild_equivalence`
- `tests::milestone_4_certification::milestone_4_certification_bundle_matches_across_backend_variation_and_delete_rebuild_lane`

What is proven:

- snapshot-plus-tail restore converges to the same truth-visible image as a
  direct point-in-time snapshot-tail read for the same frontier
- delete-and-rebuild restores the same truth-visible snapshot image rather than
  depending on retained snapshot-local bytes
- backend variation across in-memory and SQLite lanes does not change
  snapshot-visible truth, restore truth, rebuild truth, or artifact digest
- rebuilt snapshots remain derived from authoritative artifacts instead of
  becoming shadow authority

### `Typed snapshot failure localization`

Covered by:

- `tests::snapshots::snapshot_restore_rejects_pre_snapshot_target`
- `tests::milestone_4_certification::sqlite_snapshot_corruption_fails_typed_on_reopen`
- `tests::milestone_4_certification::sqlite_missing_snapshot_image_fails_with_publication_gap_on_reopen`
- `tests::milestone_4_certification::sqlite_snapshot_version_mismatch_fails_typed_on_reopen`

What is proven:

- illegal restore targets fail explicitly as target or basis mismatch failures
- corrupted snapshot images fail on reopen through typed snapshot integrity
  failure families instead of broadening into fallback authority
- missing snapshot images with retained basis records fail as publication gaps
  instead of being tolerated as partial publication
- unsupported snapshot family/version lanes fail explicitly and typed

### `Performance and boundedness proof surface`

Covered by:

- `tests::snapshots::snapshot_plus_tail_restore_matches_direct_point_in_time_read`
- `tests::milestone_4_certification::milestone_4_certification_bundle_proves_restore_and_rebuild_equivalence`

What is proven:

- snapshot-tail reads now consume the persisted snapshot as a real prefix and
  expose only the admitted suffix width through counters instead of quietly
  rebuilding the whole target image from authority
- `snapshot_read_tail_commit_count` and
  `snapshot_read_tail_replay_count` exactly match representative suffix width
- `snapshot_restore_tail_commit_count` and
  `snapshot_restore_tail_replay_count` exactly match representative restore
  suffix width
- capture, read, restore, and rebuild work are machine-checkable through the
  counter snapshot rather than hidden behind a cheap-looking API

### `Machine-checkable Milestone 4 certification bundle`

Covered by:

- `tests::milestone_4_certification::milestone_4_certification_bundle_proves_restore_and_rebuild_equivalence`
- `tests::milestone_4_certification::milestone_4_certification_bundle_matches_across_backend_variation_and_delete_rebuild_lane`

What is proven:

- Milestone 4 emits `truth_digest`, `restore_digest`, `artifact_digest`,
  `rebuild_digest`, and `counter_snapshot`
- certification output remains deterministic and machine-checkable
- equivalent backend and rebuild lanes converge to identical semantic digests
  while still preserving lane-local operational counters

## Additional Hardening Added Before Close

Milestone 4 closeout includes these extra hardening outcomes beyond the minimum
roadmap labels:

- `SnapshotRestorePlan` was hardened into a real proof-bearing plan boundary
  instead of a caller-synthesizable bag of fields
- the old monolithic snapshot state module was split by responsibility so basis
  selection, image persistence, reads, and restore no longer drift together
- snapshot integrity verification was tightened to prove semantic
  rebuildability, not just outer image digest shape
- explicit snapshot family, basis, and image format versions were frozen into
  the family boundary
- snapshot-tail read and restore stopped using a mechanically dishonest
  full-target rebuild path and now consume the persisted snapshot image as a
  true prefix
- Milestone 4 certification was expanded from one clean in-memory lane to
  backend variation plus delete-and-rebuild parity lanes

These changes were made because the bar for `worth-store` is production-grade
derived durability, not "snapshots seemed to make restore faster once."

## Explicit Deferrals

Milestone 4 intentionally does not include:

- multi-resolution or partial materialization families
- branch delta layering and delta-stack rewrite policy
- retention-driven snapshot pruning policy beyond honest non-authority
  boundaries
- replication capsules and snapshot shipping across machines
- live-query continuation over snapshot bases
- analysis checkpoint families
- snapshot profitability heuristics beyond explicit counters and basis honesty

Those remain later roadmap milestones and were not implied early here.

## Verification Baseline

At closeout, the crate verification baseline is:

- `cargo test -p worth-store`

This passes cleanly and includes:

- 47 runtime tests
- 1 compile-fail harness
- 3 compile-fail UI boundary cases
- point-in-time snapshot read and snapshot-plus-tail restore parity
- delete-and-rebuild snapshot parity
- backend variation across in-memory and SQLite lanes
- hostile snapshot corruption, publication-gap, and version-mismatch lanes
- existing Milestone 1, Milestone 2, and Milestone 3 parity and integrity
  coverage

## Operational Conclusion

Milestone 4 is now closed at the store level.

`worth-store` no longer treats snapshots as a soft convenience or a hidden
truth shortcut. It now has explicit snapshot basis and version contracts,
immutable snapshot publication, point-in-time read surfaces, real
snapshot-plus-tail restore, rebuild from canonical authority, typed hostile
failure localization, machine-checkable Milestone 4 certification evidence,
and a cost surface that is honest about suffix work instead of concealing full
reconstruction behind a cheaper name.
