# Milestone 1 Closeout: Canonical Commit Persistence And Artifact Authority

> **Note:** The legacy root crate `crates/worth-store` was removed on 2026-09-27. Paths into it below name its files as they were; they are no longer links. The store now lives in [`workspaces/worth-store`](../../workspaces/worth-store/README.md).

## Status

Milestone 1 is closed as of 2026-04-13.

`worth-store` now has a real authoritative durability boundary instead of a
placeholder persistence wrapper.

The semantic center shipped in this milestone is:

runtime-produced canonical commit meaning enters through one proof-bearing
append path, persists as explicit authoritative artifact families, verifies on
fetch and reopen, and can be exported and rebuilt without consulting
backend-local layout.

This is not "commits save somewhere." The store now owns:

- one public `worth-store` facade and builder surface
- one canonical append and fetch pipeline with sealed proof stages
- explicit authoritative artifact families for commits, parent edges, branch
  records, branch heads, and artifact digests
- canonicalization-versioned digest identity for every authoritative family
- one production-grade embedded backend baseline in both in-memory and
  file-backed configurations
- one structurally distinct SQLite backend family with the same authoritative
  contract
- startup and fetch integrity verification
- rebuild from canonical authoritative export
- exact Milestone 1 counter surfaces
- machine-checkable Milestone 1 certification bundles

## Shipped Scope

Milestone 1 delivered:

- the `worth-store` crate wired into the workspace
- a narrow public facade in
  `crates/worth-store/src/facade.rs`
- proof-bearing authority types, canonicalization, and authoritative export
  basis under
  `crates/worth-store/src/authority`
- a store-owned backend contract, embedded backend baseline, and SQLite backend
  family in
  `crates/worth-store/src/backend/mod.rs`
  plus
  `crates/worth-store/src/backend/embedded.rs`
  and
  `crates/worth-store/src/backend/sqlite.rs`
- normalized authoritative record families in
  `crates/worth-store/src/backend/records.rs`
- separated state mutation, integrity, and persistence subsystems under
  `crates/worth-store/src/backend/state`
  and
  `crates/worth-store/src/backend/integrity`
- exact counters and machine-checkable evidence bundles under
  `crates/worth-store/src/evidence`
- typed failure families under
  `crates/worth-store/src/failure`
- hostile test coverage for append legality, branch-head authority, corruption
  rejection, export/rebuild parity, and backend parity under
  `crates/worth-store/src/tests`

## Acceptance Mapping

Milestone 1 is considered closed against the roadmap and
[test-requirements.md](./test-requirements.md)
because the required acceptance surfaces are now covered directly.

### `Durable artifact authority equivalence test`

Covered by:

- `tests::certification::durable_artifact_authority_equivalence_bundle_matches_across_backend_and_rebuild_lanes`
- `tests::certification::authoritative_export_and_certification_json_are_identical_across_equivalent_lanes`
- `tests::export_rebuild::canonical_export_rebuild_preserves_authoritative_truth`
- `tests::export_rebuild::rebuilt_store_continues_local_commit_sequence_monotonically`
- `tests::persistence::file_backed_backend_reloads_authoritative_commits`
- `tests::persistence::sqlite_backend_reloads_authoritative_commits`

What is proven:

- in-memory, file-backed, and SQLite backend-family lanes converge to the same
  `truth_digest`, `history_digest`, `branch_heads_digest`, `artifact_digest`,
  and `replay_digest`
- rebuild from authoritative export converges to the same authoritative truth
  as the original store
- backend-local persistence layout differences do not change authoritative
  conclusions
- local append ordering resumes coherently after rebuild instead of resetting
  hidden local state

### `Canonical envelope and authority integrity`

Covered by:

- `tests::authority_append::append_and_fetch_preserves_authoritative_commit_truth`
- `tests::authority_append::identical_duplicate_append_is_idempotent`
- `tests::authority_append::conflicting_duplicate_commit_identity_is_rejected`
- `tests::authority_append::orphan_parent_reference_is_rejected`
- `tests::canonicalization::unsupported_canonicalization_version_is_rejected`
- `tests::canonicalization::duplicate_parent_lists_are_rejected_as_noncanonical`

What is proven:

- append and fetch preserve canonical commit meaning exactly
- digest-equivalent duplicate append is idempotent
- conflicting duplicate identity fails explicitly
- orphan parent references fail explicitly
- unsupported canonicalization versions fail explicitly
- non-canonical parent ordering inputs fail explicitly before persistence

### `Branch-head legality and replay-stable branch authority`

Covered by:

- `tests::branch_lifecycle::branch_append_requires_registered_branch_and_then_fast_forwards`
- `tests::branch_lifecycle::non_root_commit_is_rejected_for_empty_branch`

What is proven:

- unknown branches cannot silently gain authority through append
- branch creation is a typed authority action distinct from commit append
- branch heads advance through explicit branch authority rather than inferred
  max-sequence heuristics
- non-root append on an empty branch fails explicitly

### `Integrity verification and corruption localization`

Covered by:

- `tests::persistence::corrupted_persisted_digest_record_is_rejected_on_open`
- `tests::persistence::branch_head_digest_drift_is_rejected_on_open`
- `tests::persistence::sqlite_corrupted_digest_record_is_rejected_on_open`
- `tests::persistence::sqlite_branch_head_digest_drift_is_rejected_on_open`
- `tests::persistence::sqlite_missing_parent_row_is_rejected_on_open`
- `tests::persistence::sqlite_malformed_envelope_payload_is_rejected_on_open`
- `tests::export_rebuild::duplicate_authoritative_export_records_are_rejected`

What is proven:

- corrupted authoritative digest records fail at reopen with typed integrity
  failure
- branch-head digest drift is localized to the authoritative branch-head path
- SQLite row corruption and missing authoritative rows also fail at reopen with
  typed integrity failure rather than backend-local bluffing
- duplicated authoritative export records are rejected rather than collapsed
  silently during rebuild

### `Complexity and counter proof obligations`

Covered by:

- `tests::counters::append_and_fetch_counters_match_admitted_work`
- `tests::counters::duplicate_idempotent_append_does_not_increment_authoritative_append_counters_twice`

What is proven:

- append and fetch counters are exact for a representative Milestone 1
  authority path
- canonicalization work remains observable
- fetch verification failures remain separately countable from successful fetch
  verification

## Additional Hardening Added Before Close

Milestone 1 closeout includes these extra hardening outcomes beyond the minimum
roadmap labels:

- the backend was decomposed by authority mutation, integrity verification, and
  persistence responsibility instead of remaining one mixed module
- authoritative export import was hardened to reject duplicate records instead
  of quietly last-write-wins collapsing them
- branch creation now returns the exact branch it created instead of relying on
  indirect lookup
- rebuild now restores local append and head-update sequence continuity rather
  than resetting hidden local ordering state
- machine-checkable certification bundles were added so the milestone has named
  proof artifacts instead of only implicit equality assertions
- the crate root was restructured into `authority`, `backend`, `evidence`, and
  `failure` subdomains so Milestone 2 does not inherit a flat root skeleton
- SQLite was added as a true second backend family, not just a second
  configuration of the same state engine
- SQLite hostile reopen lanes were added so the second backend family is proven
  under corruption rather than only parity-tested under success paths

These changes were made because the bar for `worth-store` is certifiable
durability authority, not MVP persistence plausibility.

## Explicit Deferrals

Milestone 1 intentionally does not include:

- WAL durability and crash-boundary recovery
- store-owned durable-mode runtime lifecycle
- snapshots or snapshot-plus-tail restore
- branch delta layering and structural-block storage
- schema/lineage/cursor durable families beyond forward-compatible reservation
- live-query continuation
- replication capsules
- retention, compaction, tiering, or budget admission control
- blob or object storage

Those remain later roadmap milestones and were not faked early here.

## Verification Baseline

At closeout, the crate verification baseline is:

- `cargo test -p worth-store`

This passes cleanly and includes:

- 24 unit tests
- backend-family parity and rebuild parity lanes
- corruption-localization lanes for both JSON-file and SQLite persistence
- branch-head legality and duplicate-identity rejection lanes
- exact counter certification for the representative authority path

## Operational Conclusion

Milestone 1 is now closed at the store level.

`worth-store` no longer depends on backend-local row shape, reload luck, or
diagnostics-by-convention to preserve committed truth honestly. It now has a
real authority pipeline, explicit authoritative artifact families, typed
failure behavior, rebuildable canonical export, backend parity evidence, and a
machine-checkable Milestone 1 certification bundle surface.
