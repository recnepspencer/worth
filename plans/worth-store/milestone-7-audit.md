# Milestone 7 Audit

Date: 2026-04-15

## Verdict

Milestone 7 is now freeze-ready against the current spec at a serious platform-grade bar.

The current implementation now closes the prior spec-completeness gaps:

- typed Milestone 7 failure taxonomy exists
- historical identity resolution is a first-class public surface
- embedded checkpoint shape is compiler-visible and enforced with compile-fail tests
- proof-carrying witness vocabulary is explicit across support append, cursor resume/advance, and checkpoint admission

## Closed Gaps

### Failure taxonomy

The Milestone 7-specific failure families are now present and routed:

- `SchemaBoundaryVersionUnsupported`
- `SupportAuthorityTaxonomyViolation`
- `HistoricalIdentityResolutionGap`

Primary surfaces:

- `crates/worth-store/src/failure/mod.rs`
- `crates/worth-store/src/backend/integrity/support_records.rs`
- `crates/worth-store/src/backend/engine.rs`

### Historical identity public surface

The store now exposes a first-class, store-owned historical identity contract:

- `HistoricalIdentityRequest`
- `HistoricalIdentityResolution`
- `WorthStore::fetch_lineage_history(...)`

Primary surfaces:

- `crates/worth-store/src/authority/proofs.rs`
- `crates/worth-store/src/facade.rs`
- `crates/worth-store/src/tests/cursor_support.rs`

### Compile-time checkpoint shape proof

The embedded checkpoint seam is no longer just a runtime convention. The public surface now uses:

- `BasisFreeCheckpoint`
- `BasisBoundCheckpoint`
- kind markers for `DerivedDurable` and `Ephemeral`
- contained-commit shape markers
- `BasisBoundCheckpointWitness`
- `VerifiedEmbeddedCheckpoint`

The compile boundary is now proven by trybuild tests:

- `tests/ui/raw_external_checkpoint_envelope_rejected.rs`
- `tests/ui/basis_free_checkpoint_rejects_basis_binding.rs`

Primary surfaces:

- `crates/worth-store/src/modes/embedded.rs`
- `crates/worth-store/tests/phase_boundaries_compile_fail.rs`
- `crates/worth-store/tests/ui/`

### Proof-carrying witness vocabulary

The representative Milestone 7 witness vocabulary is now explicit and store-owned:

- `CommitCoupledSupportAppendWitness`
- `ResumeAdmittedCursor`
- `AdvanceCursorWitness`
- `BasisBoundCheckpointWitness`
- `PersistedEmbeddedCheckpoint`

Primary surfaces:

- `crates/worth-store/src/authority/proofs.rs`
- `crates/worth-store/src/backend/state/commit_append.rs`
- `crates/worth-store/src/facade.rs`
- `crates/worth-store/src/modes/embedded.rs`

## Residual Notes

There is still room for future refinement, but not in a way that blocks honest Milestone 7 freeze:

- some convenience surfaces remain alongside stricter proof-bearing ones for compatibility
- the support/cursor/checkpoint proof chain can still be deepened in later milestones without invalidating the current boundary

These are evolution opportunities, not unresolved Milestone 7 contract gaps.

## Freeze Basis

Milestone 7 can now be closed out honestly because:

1. the public semantic contract named by the spec is present in code
2. the complexity/debt certification surface is machine-checkable
3. adversarial corruption degrades to typed debt or typed recovery states
4. compile-time and runtime proof boundaries now both exist where the spec requires them

## Verification

Verified with:

- `cargo test -p worth-store`
