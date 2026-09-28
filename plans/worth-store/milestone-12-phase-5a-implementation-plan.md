# Worth Store Milestone 12 Phase 5A Implementation Plan

## Summary

Start the Milestone 12 certification foundation now that the compatibility
subsystem has enough proof-bearing surfaces to certify.

This batch should not implement a full artifact-format evolution runtime,
durable backup persistence, restore execution, rolling publication execution,
SQLite schema changes, facade read/write APIs, or adapter execution. It should
build the machine-checkable certification shell that assembles the lanes we
already implemented into an honest Milestone 12 evidence bundle.

The goal is to stop treating compatibility tests as scattered unit coverage.
Phase 5A should create the certification vocabulary, lane inputs, lane
outcomes, and summary bundle that later Phase 5 work can widen into the full
`Artifact Format Evolution And Rolling Compatibility Test`.

## Governing Constraint

Milestone 12 is not complete because individual old/new read, derived, rolling,
or restore tests pass. It is complete only when each admitted or rejected
compatibility lane emits machine-checkable evidence that explains:

- what artifact family was tested
- what source and target versions were compared
- what manifest and registry basis was used
- which declared edge was selected or missing
- whether semantic meaning was admitted, rebuilt, invalidated, or rejected
- what proof-bearing value carried the admission
- what counters prove the breadth and cost of the check

Phase 5A must make that certification shape real without pretending to run the
full milestone certification suite yet.

## Current State To Preserve

- Phase 1 vocabulary/catalog is implemented under
  `crates/worth-store/src/compatibility/`.
- Phase 2 read/write/manifest admission exists with:
  - recovered manifest frontier identity
  - registry snapshot identity
  - receipt basis binding
  - declared edge registry
  - adapter cost-class gates
  - typed admission outcomes
- Phase 3 derived/support lanes exist with:
  - derived lane registry/snapshot
  - lane-specific reuse/invalidation/rebuild requirements
  - bulk resume skew rejection
  - maintenance-lane admission binding
  - tier non-authority preservation
- Phase 4A rolling admission exists with:
  - first-ship two-capability policy
  - declared edge requirement
  - relation preservation in admitted plans
  - multi-writer, multi-version, missing-edge, adapter-edge, and skew
    rejection lanes
- Phase 4B restore/backup planning exists with:
  - backup-scope restore admission
  - declared edge requirement
  - publication conflict rejection
  - out-of-scope scan rejection
  - disaster-recovery truth-vs-derived window classification
- Compile-fail coverage protects the important proof-bearing constructors.

## Key Changes

## Implementation Order

Implement Phase 5A in this order so each step leaves the crate compiling and
the evidence surface grows monotonically:

1. Add `compatibility/certification.rs` with the lane ID, lane kind, lane
   input, lane status, lane rejection, and lane outcome vocabulary only.
   Export read-only public types through `compatibility/mod.rs`; keep accepted
   outcome constructors `pub(crate)`.
2. Add deterministic mandatory lane enumeration for the Phase 5A lane list.
   Unit-test that labels are stable and that every mandatory lane has a unique
   ID.
3. Add compatibility matrix assembly over lane outcomes. It should reject
   duplicate lanes, missing mandatory lanes, and mismatched lane IDs before
   constructing a matrix.
4. Add the evidence bundle shell in `evidence/milestone_12.rs`. The bundle
   should require a matrix, lane outcomes, admission report, version-skew
   report, and complexity surface. It should aggregate only; it must not rerun
   compatibility admission.
5. Add counter-contract validation against `Milestone12AdmissionReport`. The
   first version can be explicit and mechanical: every field that represents a
   counter must have a corresponding counter-name constant.
6. Add conversion helpers from existing typed outcomes into certification lane
   outcomes. Start with rolling and restore because they are the newest and
   most likely to regress, then add authoritative read/write and derived lanes.
7. Add compile-fail fixtures for synthetic accepted certification outcomes,
   synthetic evidence bundles, rejected-lane-to-restore-witness misuse,
   rolling-lane-to-upgrade-witness misuse, and derived-lane-to-retained-
   authority misuse.
8. Run the focused test gates and update this plan or the milestone spec if any
   certification obligation turns out to require a different type shape.

### 1. Certification Lane Vocabulary

Add a certification module under the compatibility subsystem, preferably:

```text
crates/worth-store/src/compatibility/certification.rs
```

Required types:

- `Milestone12CertificationLaneKind`
- `Milestone12CertificationLaneId`
- `Milestone12CertificationLaneInput`
- `Milestone12CertificationLaneOutcome`
- `Milestone12CertificationLaneStatus`
- `Milestone12CertificationLaneRejection`
- `Milestone12CertificationRunSummary`

Required lane kinds for Phase 5A:

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

- Lane IDs must be stable strings.
- Lane input must include family id, source semantic version, target semantic
  version, and expected relation or rejection kind where applicable.
- Lane outcome must distinguish accepted, rejected, invalidated, rebuild
  required, and evidence-only lanes.
- Lane outcomes must carry counters or a counter snapshot.
- Certification lane outcomes are evidence, not runtime admission receipts.
  They must not produce semantic views or facade-visible restore/rolling
  behavior.

### 2. Certification Matrix Assembly

Extend the existing `Milestone12CompatibilityMatrixRow` use into a concrete
matrix shell.

Required types:

- `Milestone12CompatibilityMatrix`
- `Milestone12CompatibilityMatrixEntry`
- `Milestone12CompatibilityMatrixStatus`

Rules:

- The matrix must include every Phase 5A lane kind.
- Missing mandatory lanes should be a typed certification failure.
- Duplicate lane IDs should reject typed.
- The matrix must preserve row order deterministically.
- Matrix entries should report the exact `Milestone12CompatibilityMatrixRow`
  or `Milestone12CertificationLaneKind` label, not an ad hoc string.

Tests:

- the Phase 5A matrix contains every mandatory lane exactly once
- duplicate lane IDs reject
- missing lane rejects
- matrix ordering is deterministic

### 3. Evidence Bundle Shell

Extend or add to `crates/worth-store/src/evidence/milestone_12.rs`.

Required types:

- `Milestone12ArtifactFormatEvolutionEvidence`
- `Milestone12RollingCompatibilityEvidence`
- `Milestone12RestoreCompatibilityEvidence`
- `Milestone12DerivedCompatibilityEvidence`
- `Milestone12CertificationEvidenceBundle`

Rules:

- Evidence bundle construction must require:
  - compatibility admission report
  - compatibility matrix
  - version skew report
  - complexity surface
  - lane outcomes
- Bundle construction should be deterministic.
- Bundle must expose read-only summaries only.
- Bundle construction must not execute compatibility checks itself. It
  aggregates already-produced lane evidence.

Tests:

- bundle construction preserves all lane outcomes
- bundle construction rejects matrix/lane mismatch
- bundle summary reports accepted/rejected lane counts
- bundle summary exposes restore/rolling/derived counts separately

### 4. Counter Contract Tightening

Phase 5A should harden counter vocabulary and make missing counters fail loudly.

Required changes:

- Add a `Milestone12CounterContract::validate_report` style method or
  equivalent typed check.
- Ensure every implemented counter in `CompatibilityAdmissionCounters` has a
  named counter in `MILESTONE_12_COUNTER_NAMES`.
- Ensure every Phase 5A lane outcome carries either:
  - a full admission report snapshot, or
  - a narrow lane counter summary with exact fields.

Tests:

- counter contract names every public report field
- certification lane outcomes cannot omit counter evidence
- missing restore, rolling, derived, or edge counters reject typed

### 5. Existing Lane Adapters Into Certification Evidence

Add evidence conversion helpers for implemented planning surfaces only.

Required conversion helpers:

- read admission outcome -> certification lane outcome
- derived lane plan/rejection -> certification lane outcome
- rolling admission/rejection -> certification lane outcome
- restore admission/rejection -> certification lane outcome
- disaster recovery plan -> certification lane outcome

Rules:

- These helpers must not rerun admission.
- They consume or borrow existing outcomes and counters.
- They must preserve selected relation for admitted read/rolling/restore lanes.
- They must preserve rejection kind for rejected lanes.
- They must preserve scope counters for restore out-of-scope lanes.

Tests:

- rolling admitted outcome preserves selected relation in certification
- restore admitted outcome preserves selected relation and publication-conflict
  count
- missing-edge rejections preserve `MissingCompatibilityEdge`
- derived rebuild/invalidated outcomes do not masquerade as acceptance

### 6. Compile-Fail Coverage

Add compile-fail fixtures under `crates/worth-store/tests/ui/`.

Required fixtures:

- external code cannot construct `Milestone12CertificationEvidenceBundle`
  without matrix/counter evidence
- external code cannot construct an accepted certification lane outcome without
  using a typed compatibility outcome
- external code cannot turn a rejected lane outcome into a restore publication
  witness
- external code cannot turn a rolling certification lane into an
  `UpgradeAdmissionWitness`
- external code cannot use a derived certification lane as retained authority

## Tests To Run

```text
cargo fmt -p worth-store
cargo test -p worth-store compatibility --lib
cargo test -p worth-store milestone_12 --lib
cargo test -p worth-store --test phase_boundaries_compile_fail -- --test-threads=1
```

## Explicit Non-Goals

- No full Phase 5 certification runner yet.
- No durable certification bundle persistence.
- No artifact decoding corpus or generated old/new fixture files.
- No SQLite schema changes.
- No facade read/write/restore APIs.
- No restore execution or visibility publication.
- No rolling writer publication or replica transfer.
- No adapter execution.
- No rebuild execution or Milestone 11 queue insertion.

## Exit Condition

Phase 5A is complete when Milestone 12 has a deterministic,
machine-checkable certification evidence shape that can represent every
implemented compatibility lane, reject missing/duplicate/mismatched lane
evidence, preserve selected relations and rejection kinds, and expose exact
counter evidence without executing new runtime behavior.
