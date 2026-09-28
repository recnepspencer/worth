# Milestone 7 Closeout: Durable Schema, Lineage, Cursor, And Checkpoint Artifacts

> **Note:** The legacy root crate `crates/worth-store` was removed on 2026-09-27. Paths into it below name its files as they were; they are no longer links. The store now lives in [`workspaces/worth-store`](../../workspaces/worth-store/README.md).

## Status

Milestone 7 is closed as of 2026-04-15.

This milestone is not "metadata persistence."

`worth-store` now durably preserves the support-truth families that restart,
historical identity resolution, durable cursor continuation, and embedded
checkpoint re-entry actually depend on, without promoting those families above
canonical commit authority.

The center of the milestone that shipped is:

- schema-boundary truth is durably queryable as its own support family
- lineage truth is durably queryable as its own support family
- durable cursor identity and subscriber checkpoint progress are explicit,
  monotonic, and restart-visible
- embedded checkpoints are persisted through proof-bearing basis and
  classification boundaries instead of loose optional-field conventions
- restart and recovery classify support-family gaps explicitly instead of
  bluffing clean continuation
- Milestone 7 now emits machine-checkable support-truth, diagnostics,
  complexity, and counter evidence

## What Shipped

Milestone 7 delivered:

- explicit schema-boundary support persistence and fetch surfaces in
  `crates/worth-store/src/backend/integrity/support_records.rs`,
  `crates/worth-store/src/backend/engine.rs`,
  and
  `crates/worth-store/src/facade.rs`
- explicit lineage support persistence and first-class historical identity
  resolution through `HistoricalIdentityRequest`,
  `HistoricalIdentityResolution`, and
  `WorthStore::fetch_lineage_history(...)` in
  `crates/worth-store/src/authority/proofs.rs`,
  `crates/worth-store/src/backend/engine.rs`,
  and
  `crates/worth-store/src/facade.rs`
- commit-coupled support publication and deterministic support identities in
  `crates/worth-store/src/backend/state/commit_append.rs`
  and
  `crates/worth-store/src/backend/integrity/support_records.rs`
- durable cursor identity persistence, checkpoint persistence, monotonic
  acknowledgment, resume planning, and witness-bearing advance flow in
  `crates/worth-store/src/authority/proofs.rs`,
  `crates/worth-store/src/backend/engine.rs`,
  and
  `crates/worth-store/src/facade.rs`
- proof-bearing embedded checkpoint admission and persistence in
  `crates/worth-store/src/modes/embedded.rs`
- durable restart and support-gap recovery classification in
  `crates/worth-store/src/recovery/support.rs`,
  `crates/worth-store/src/recovery/report.rs`,
  and
  `crates/worth-store/src/modes/durable.rs`
- backend-open access-structure verification and backend-family-specific
  complexity proof surfaces in
  `crates/worth-store/src/evidence/milestone_7.rs`
  and
  `crates/worth-store/src/backend/engine.rs`
- Milestone 7 counter extensions and named hot-path accounting in
  `crates/worth-store/src/evidence/counters.rs`
- compile-fail checkpoint boundary enforcement in
  `crates/worth-store/tests/phase_boundaries_compile_fail.rs`
  and
  `crates/worth-store/tests/ui`
- milestone-grade evidence and certification in
  `crates/worth-store/src/evidence/milestone_7.rs`
  and
  `crates/worth-store/src/tests/milestone_7_certification.rs`

## Acceptance Mapping

Milestone 7 is considered closed against
[milestone-7.md](./milestone-7.md)
and
[test-requirements.md](./test-requirements.md)
because the required acceptance surfaces are now directly mapped to code and
tests.

### `Schema/Lineage/Cursor Durability Test`

Covered by:

- `crates/worth-store/src/tests/milestone_7_certification.rs`
- `crates/worth-store/src/tests/cursor_support.rs`

What is proven:

- restart parity across primary and control lanes uses explicit support-truth
  digests rather than narrative comparison
- typed support-gap classification remains backend-stable across local-file and
  SQLite recovery lanes
- durable cursor resume survives reopen and remains keyed to explicit cursor
  identity
- historical identity resolution is durable, commit-scoped, branch-scoped, and
  typed when no matching lineage neighborhood exists
- exactly-once support publication survives duplicate append and restore/replay
  equivalence lanes

### `Typed degraded recovery instead of ambient continuation`

Covered by:

- `tests::cursor_support::durable_recovery_reports_cursor_checkpoint_gap_as_support_rebuild`
- `tests::cursor_support::durable_recovery_reports_checkpoint_shape_violation_as_support_quarantine`
- `tests::milestone_7_certification::milestone_7_support_gap_bundle_captures_typed_rebuild_classification`

What is proven:

- cursor/checkpoint/schema/lineage support gaps become typed rebuild or
  quarantine outcomes
- support-summary failures localize the affected support family rather than
  collapsing into a generic gap bucket
- operator-facing recovery disposition remains explicit and machine-readable

### `Compile-time checkpoint shape enforcement`

Covered by:

- `crates/worth-store/tests/ui/raw_external_checkpoint_envelope_rejected.rs`
- `crates/worth-store/tests/ui/basis_free_checkpoint_rejects_basis_binding.rs`
- `crates/worth-store/src/modes/embedded.rs`

What is proven:

- raw external checkpoint envelopes are not admitted across the public
  persistence boundary
- basis-bearing and basis-free checkpoint shapes are distinct compile-visible
  forms
- checkpoint persistence now depends on `VerifiedEmbeddedCheckpoint` rather
  than caller-synthesized loose records

### `Performance and complexity honesty`

Covered by:

- `crates/worth-store/src/evidence/milestone_7.rs`
- `crates/worth-store/src/evidence/counters.rs`
- `tests::milestone_7_certification::milestone_7_access_structure_verification_degrades_to_debt_when_cursor_index_is_corrupted`
- `tests::milestone_7_certification::milestone_7_schema_access_structure_degrades_to_debt_when_index_is_corrupted`
- `tests::milestone_7_certification::milestone_7_lineage_access_structure_degrades_to_debt_when_index_is_corrupted`
- `tests::milestone_7_certification::milestone_7_support_publication_degrades_to_debt_when_summary_index_is_corrupted`
- `tests::milestone_7_certification::milestone_7_cursor_identity_admission_degrades_to_debt_when_checkpoint_index_is_corrupted`
- `tests::milestone_7_certification::milestone_7_embedded_checkpoint_access_structure_degrades_to_debt_when_index_is_corrupted`

What is proven:

- every named Milestone 7 hot path publishes a machine-checkable
  `Verified` or `Debt` status
- complexity claims are backend-family-specific rather than generic
- access structures are verified at open, not merely declared on paper
- adversarial corruption can force honest `Debt` instead of leaving the bundle
  falsely green

### `Machine-checkable Milestone 7 certification bundle`

Covered by:

- `tests::milestone_7_certification::milestone_7_bundle_proves_clean_support_artifact_restart_parity`
- `tests::milestone_7_certification::milestone_7_backend_contracts_are_family_specific`
- `tests::milestone_7_certification::milestone_7_bundle_proves_exactly_once_support_publication_under_duplicate_append`

What is proven:

- `Milestone7CertificationBundle` emits:
  - `history_digest`
  - `artifact_digest`
  - `replay_digest`
  - `support_truth_digest`
  - `diagnostics_digest`
  - `support_artifact_recovery_report`
  - `certification_summary`
  - `access_structure_contract`
  - `access_structure_verification`
  - `complexity_status`
  - `counter_contract`
  - `counter_snapshot`
- `support_truth_digest` remains the semantic parity surface across equivalent
  lanes
- `diagnostics_digest` remains the telemetry-bearing surface and may diverge
  when lane-local work differs

## Additional Hardening Added Before Close

Milestone 7 closeout includes hardening that materially improved the milestone
beyond the first implementation pass:

- missing Milestone 7 failure taxonomy was made explicit and store-owned
- historical identity resolution became a first-class public store surface
  instead of an implied future capability
- checkpoint shape moved from runtime-only convention to compile-visible
  typestate with compile-fail proof coverage
- proof vocabulary was made explicit through
  `CommitCoupledSupportAppendWitness`, `ResumeAdmittedCursor`,
  `AdvanceCursorWitness`, and `BasisBoundCheckpointWitness`
- complexity/debt evidence stopped being static declaration and became
  backend-open verification plus counter-backed proof basis
- the adversarial `Debt` matrix was expanded to every named hot path in the
  milestone
- the public schema boundary surface and engine support fetch logic were
  de-duplicated to remove semantic fork risk before freeze

These changes matter because the bar for `worth-store` is production-grade
support durability, not "resume worked once on a happy path."

## Explicit Deferrals

Milestone 7 intentionally does not include:

- Milestone 8 live-query execution, query narrowing, or delivery semantics
- replication capsules, cross-machine cursor shipping, or checkpoint shipping
- Milestone 5 / 6 physical delta layout as part of support-artifact meaning
- richer historical identity analysis surfaces beyond the durable
  commit-scoped lineage neighborhood required here
- snapshot profitability heuristics or later checkpoint families beyond the
  durability substrate needed for restart-safe embedded re-entry
- future deepening of proof chains where the current milestone boundary is
  already honest and machine-checked

Those remain later roadmap work and are not hidden incompleteness inside
Milestone 7.

## Verification Baseline

At closeout, the verification baseline is:

- `cargo test -p worth-store cursor_support -- --nocapture`
- `cargo test -p worth-store milestone_7_certification -- --nocapture`
- `cargo test -p worth-store`

This passes cleanly and includes:

- `134` runtime tests
- `1` compile-fail harness
- `9` compile-fail UI boundary tests
- Milestone 7 restart parity, typed support-gap recovery, historical identity,
  cursor durability, complexity/debt, and backend-family proof coverage
- the earlier milestone certification suites that Milestone 7 now depends on

## Operational Conclusion

Milestone 7 is now closed at the store level.

`worth-store` no longer relies on ambient session memory, replay-only lineage
reconstruction, hidden support extraction, or loosely shaped embedded
checkpoints to survive restart honestly. It now has explicit durable schema,
lineage, cursor, subscriber-checkpoint, and embedded-checkpoint support
families; commit-coupled exactly-once publication; typed degraded recovery;
proof-bearing checkpoint and cursor boundaries; backend-specific complexity
contracts; adversarial `Debt` certification for corrupted access structures;
and machine-checkable Milestone 7 closeout evidence.
