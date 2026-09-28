# Worth Store Milestone 12 Phase 5B Implementation Plan

## Summary

Implement the first real Milestone 12 certification runner for the compatibility
lanes that already exist.

Phase 5A created the lane vocabulary, matrix shell, outcome wrappers, and
evidence bundle shape. Phase 5B should stop using synthetic lane outcomes in
tests and start assembling certification evidence by invoking the actual
compatibility admission, derived, rolling, restore, manifest, and disaster
recovery planning functions.

This batch should still not implement durable manifest persistence, facade
read/write APIs, restore execution, rolling writer publication, adapter
execution, or rebuild execution. Its job is to create a trustworthy named
certification harness for the implemented compatibility surfaces and expose the
remaining runtime/persistence gaps honestly.

## Governing Constraint

Milestone 12 cannot close on scattered unit tests or synthetic certification
rows.

The certification runner must prove that each implemented compatibility lane is
backed by the actual proof-bearing operation it claims to certify. A lane that
claims "rolling missing edge rejected" must run the rolling planner and preserve
its `MissingCompatibilityEdge` rejection. A lane that claims "restore scoped
backup admitted" must run restore compatibility planning and preserve the
selected relation. A lane that claims "derived bulk resume rejected" must run
the derived/bulk compatibility surface and preserve the typed rejection. The
runner may aggregate evidence; it may not fabricate admission, rejection, or
counter posture.

## Current State To Preserve

- Phase 5A certification vocabulary exists in
  `crates/worth-store/src/compatibility/certification.rs`.
- `Milestone12CertificationEvidenceBundle` aggregates matrix, lane outcomes,
  admission report, version-skew report, complexity surface, and rolling,
  restore, and derived summaries.
- Existing compatibility planning surfaces already cover:
  - read/write admission and typed rejection
  - manifest publication and recovered index reconstruction
  - authoritative partial-truth rejection
  - derived reuse, invalidation, rebuild-required, bulk skew rejection,
    maintenance-lane admission, and tier non-authority preservation
  - first-ship rolling admission/rejection
  - restore scope/publication-conflict/missing-edge/incompatible-edge planning
  - disaster-recovery truth versus derived acceleration classification
- Compile-fail coverage already protects proof-bearing constructors and the
  Phase 5A certification evidence wrappers.

## Key Changes

### 1. Add A Certification Runner Module

Add a focused runner module under compatibility:

```text
crates/worth-store/src/compatibility/certification_runner.rs
```

Required public or crate-visible types:

- `Milestone12ArtifactFormatEvolutionCertification`
- `Milestone12CertificationRunner`
- `Milestone12CertificationScenario`
- `Milestone12CertificationFixture`
- `Milestone12CertificationDigestSet`
- `Milestone12CertificationDiagnostics`

Rules:

- The runner must assemble lane outcomes from actual compatibility operations,
  not from manually constructed accepted/rejected outcomes.
- Fixture construction may be deterministic and in-memory for this batch.
- The runner should return the existing `Milestone12CertificationEvidenceBundle`
  plus certification-specific digests and diagnostics.
- Runner construction should be crate-owned; external code should consume
  read-only evidence only.

### 2. Replace Synthetic Lane Tests With Real Lane Assembly

Move the Phase 5A test fixture from synthetic outcome construction toward a
real runner path.

The runner must produce all mandatory Phase 5A lanes:

- `CatalogCompleteness`
- `AuthoritativeNativeRead`
- `AuthoritativeForwardRead`
- `AuthoritativeBackwardRead`
- `AuthoritativeMissingEdgeRejected`
- `AuthoritativeIncompatibleEdgeRejected`
- `DerivedSnapshotReuseAccepted`
- `DerivedLayoutBasisRejected`
- `DerivedBulkResumeRejected`
- `TierManifestNonAuthorityPreserved`
- `RollingTwoCapabilityAdmitted`
- `RollingMultiWriterRejected`
- `RollingMissingEdgeRejected`
- `RollingAdapterEdgeRejected`
- `RestoreScopedBackupAdmitted`
- `RestoreOutOfScopeRejected`
- `RestorePublicationConflictRejected`
- `RestoreMissingEdgeRejected`
- `DisasterRecoveryTruthWindow`
- `DisasterRecoveryDerivedWindow`

Rules:

- Each lane outcome must carry counters emitted by the operation it ran.
- Each rejected lane must preserve the operation's `CompatibilityRejectionKind`
  or lane-specific rejection posture.
- Each admitted lane must preserve the selected `CompatibilityRelation` where
  applicable.
- Evidence-only lanes must still carry nonzero evidence counters.

### 3. Add Machine-Checkable Digest Surfaces

Add digest/report shells required by the named M12 suite:

- `artifact_digest`
- `failure_digest`
- `compatibility_matrix`
- `version_skew_report`
- `diagnostics_digest`
- `counter_snapshot`

Rules:

- Digests may be deterministic string digests over existing in-memory evidence
  for this phase.
- Digests must be derived from lane evidence, not from hardcoded labels alone.
- Failure digest must include typed rejection kinds.
- Diagnostics digest must change when diagnostics/counter/evidence posture
  changes without changing admitted semantic meaning.
- Counter snapshot must be a real aggregation of lane reports.

### 4. Implement Lane Families In Priority Order

Implement runner lanes in this order:

1. Catalog and manifest/index lanes.
2. Authoritative read/write admission lanes.
3. Derived/support lanes.
4. Rolling lanes.
5. Restore and disaster-recovery lanes.
6. Cross-lane aggregate digests and summaries.

Reason:

- Catalog/manifest evidence is the basis for every other compatibility claim.
- Authoritative admission is the core truth gate.
- Derived, rolling, and restore lanes should consume already-proven admission
  vocabulary instead of rebuilding their own proof language.

### 5. Certification Runner Tests

Add focused unit tests for the runner:

- runner emits every mandatory Phase 5A lane exactly once
- runner matrix is deterministic across repeated runs
- catalog completeness lane is backed by the first-ship registry snapshot
- authoritative missing-edge lane is backed by an actual missing-edge admission
  outcome
- authoritative incompatible edge lane is backed by actual incompatible edge
  rejection
- derived snapshot reuse lane is backed by actual derived reuse planning
- derived layout basis lane is backed by actual basis drift planning
- derived bulk resume lane is backed by actual bulk interpretation rejection
- tier manifest lane is backed by actual tier non-authority preservation
- rolling admitted lane preserves the selected relation from the rolling plan
- rolling rejected lanes preserve multi-writer, missing-edge, and adapter-edge
  failure posture
- restore admitted lane preserves selected relation and publication conflict
  count
- restore rejected lanes preserve out-of-scope, publication-conflict, and
  missing-edge failure posture
- disaster-recovery lanes distinguish truth and derived acceleration counters
- aggregate counter snapshot equals the sum of lane reports
- artifact/failure/diagnostics digests are deterministic and change when their
  source evidence changes

### 6. Compile-Fail Coverage

Add compile-fail fixtures only where Phase 5B introduces new public surfaces.

Required fixtures:

- external code cannot construct the certification runner's admitted fixture
  internals directly
- external code cannot fabricate `Milestone12CertificationDigestSet` with
  arbitrary digest strings if the type claims runner-produced evidence
- external code cannot call crate-only lane assembly helpers
- external code cannot treat certification diagnostics as runtime compatibility
  admission proof

If the new types remain entirely crate-private and no new public misuse surface
exists, document that no new trybuild fixture is needed and keep the existing
Phase 5A compile-fail coverage.

## Explicit Non-Goals

- No durable manifest persistence or SQLite schema changes.
- No durable certification bundle persistence.
- No facade read/write/restore APIs.
- No restore execution or visibility publication.
- No rolling writer publication or replica transfer.
- No adapter execution.
- No rebuild execution or Milestone 11 queue insertion.
- No generated historical artifact corpus yet.
- No claim that Milestone 12 is closed after Phase 5B.

## Tests To Run

```text
cargo fmt -p worth-store
cargo test -p worth-store compatibility --lib
cargo test -p worth-store milestone_12 --lib
cargo test -p worth-store artifact_format_evolution --lib
cargo test -p worth-store --test phase_boundaries_compile_fail -- --test-threads=1
```

If `artifact_format_evolution` does not exist yet, Phase 5B should create that
test filter name through runner test names or a dedicated test module.

## Exit Condition

Phase 5B is complete when `worth-store` has a deterministic
`Artifact Format Evolution And Rolling Compatibility` certification runner that
assembles every mandatory Phase 5A lane from real compatibility operations,
emits machine-checkable digest/counter/matrix evidence, and makes the remaining
runtime, persistence, restore-publication, rolling-publication, adapter, and
rebuild-execution gaps explicit rather than hiding them behind synthetic
evidence.
