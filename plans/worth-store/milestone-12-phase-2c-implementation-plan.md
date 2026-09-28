# Worth Store Milestone 12 Phase 2C Implementation Plan

## Summary

Finish the missing manifest-publication and authoritative partial-truth pieces
of Phase 2 before moving deeper into derived rebuild, rolling upgrade, or
restore execution.

The current implementation has the compatibility catalog, manifest identity
types, decode quarantine, in-memory manifest index reconstruction, reader/write
admission, batch receipt reuse, adapter cost gates, typed rejection surfaces,
and first derived reuse planning. The next batch should harden the boundary
that every later phase depends on: a compatibility manifest must be published,
recoverable, digest-bound, and strong enough to reject decoded-but-semantically
unsafe authoritative artifacts before they can become checked or admitted.

This remains an in-memory/subsystem implementation batch. It should not add a
SQLite schema migration, facade read/write APIs, rolling upgrade execution,
restore publication execution, adapter execution, or a full certification
runner.

## Governing Constraint

An artifact that decodes structurally but lacks a recovered/published manifest,
has a stale or mismatched manifest digest, carries unknown authoritative meaning,
or depends on a relation that only exists by numeric proximity must fail with a
typed compatibility rejection before any semantic view or authoritative witness
can be constructed.

## Current State To Preserve

- `compatibility/catalog` declares the first-ship family catalog and registry
  snapshot.
- `compatibility/manifests` defines family/version/window/digest/publication
  vocabulary, but publication and recovery are shells.
- `compatibility/decoding` quarantines raw bytes and rejects malformed,
  truncated, and overlong frames.
- `compatibility/admission` performs in-memory manifest-index lookup, declared
  edge resolution, receipt reuse, adapter cost gating, and read/write receipt
  construction.
- `compatibility/derived` admits exact derived reuse only from a matching native
  derived receipt and turns non-native relations into rebuild-required posture.
- Compile-fail tests already prevent external construction of proof-bearing
  witnesses, checked artifacts, receipts, admission outcomes, and derived reuse
  proofs.

## Key Changes

### 1. Manifest Publication Ledger

Add a Phase 2C in-memory manifest publication ledger under
`crates/worth-store/src/compatibility/manifests/`.

Required types:

- `CompatibilityManifestPublicationRecord`
- `CompatibilityManifestFrontier`
- `CompatibilityManifestPublicationLedger`
- `CompatibilityManifestRecoveryPlan`
- `CompatibilityRecoveredManifestIndex`
- `CompatibilityManifestPublicationReceipt`

Rules:

- Manifest publication records are append-only.
- The ledger is keyed by `ArtifactFamilyId` plus manifest digest.
- A family may have a frontier, but old manifest records may not be mutated in
  place.
- Publication receipts are proof-bearing and crate-private to construct.
- Recovery can rebuild an index from publication records and registry
  declarations, but it may not invent an authoritative manifest from a decoded
  artifact.
- Publication units must bind family id, format window, semantic window,
  authority/derived classification, digest, and publication sequence.

### 2. Manifest Gap And Digest Drift Rejections

Extend admission so manifest lookup can distinguish these cases:

- manifest family undeclared
- manifest publication missing for a declared family
- manifest digest mismatch between artifact and recovered publication
- manifest window unsupported by format version
- manifest window unsupported by semantic version

Required rejection kinds:

- `MissingManifestPublication`
- `RecoveredManifestDigestMismatch`
- `RecoveredManifestWindowMismatch`

Map these into `StoreErrorKind` using the existing manifest/format/semantic
failure topology rather than string-only reasons.

### 3. Authoritative Partial-Truth Gate

Add a small authoritative admission layer in
`crates/worth-store/src/compatibility/authoritative/` that consumes a checked
artifact plus read/write receipt and produces an
`AuthoritativeCompatibilityWitness` only when the authoritative semantic surface
is explicitly admitted.

Required types:

- `AuthoritativeMeaningDeclaration`
- `AuthoritativeUnknownMeaning`
- `AuthoritativePartialTruthRejection`
- `AuthoritativeAdmissionReport`

Rules:

- Native, forward, and backward relations may be admitted only if the family has
  an explicit authoritative meaning declaration for that semantic version.
- `AdapterRequired` does not produce authoritative meaning unless adapter parity
  is explicitly witnessed. Phase 2C should reject this path, because adapter
  execution/parity is not implemented yet.
- `DerivedRebuildRequired` and `Incompatible` may never admit authoritative
  meaning.
- Unknown authoritative fields/meaning fail with
  `CompatibilityAuthoritativePartialTruthRejected`.
- The witness constructor remains `pub(crate)`.

### 4. Receipt Invalidity Basis

Strengthen `CompatibilityAdmissionReceipt` and `ReceiptKey` so receipt reuse is
invalidated by:

- registry snapshot identity
- recovered manifest frontier identity
- artifact family
- observed semantic version
- target semantic version
- manifest digest
- admission path

This should stay in-memory for now. The goal is not durable receipt persistence;
the goal is to prevent batch-local proof reuse from outliving the manifest and
registry basis it actually proved.

### 5. Counter And Evidence Surface

Extend `CompatibilityAdmissionCounters` and `Milestone12AdmissionReport` with:

- `manifest_publication_count`
- `manifest_recovery_record_count`
- `manifest_publication_gap_count`
- `manifest_digest_mismatch_count`
- `manifest_window_mismatch_count`
- `authoritative_partial_truth_rejection_count`
- `receipt_basis_mismatch_count`

Update `MILESTONE_12_COUNTER_NAMES` so every new counter has exact vocabulary.

### 6. Module Decomposition

Move toward the spec's intended domain topology without a risky full rewrite.

Recommended split:

- keep `compatibility/mod.rs` as facade and test host for now
- split `compatibility/manifests.rs` into `compatibility/manifests/mod.rs`,
  `identity.rs`, `publication.rs`, and `recovery.rs`
- split `compatibility/authoritative.rs` into
  `compatibility/authoritative/mod.rs`, `meaning.rs`, and `admission.rs`

Do not split `admission.rs` in this batch unless the change is local and
mechanical. The higher-value boundary is manifest/authoritative separation.

## Tests

Add unit tests for:

- manifest publication records are append-only and deterministic
- recovered manifest index is built from publication records and registry
  declarations, not artifact rows
- admission rejects a declared family with no manifest publication
- admission rejects artifact manifest digest drift after recovery
- admission rejects unsupported recovered format window
- admission rejects unsupported recovered semantic window
- receipt reuse succeeds inside the same registry/manifest frontier basis
- receipt reuse rejects after manifest frontier identity changes
- authoritative witness is produced only after checked artifact plus admitted
  authoritative meaning declaration
- native read without authoritative meaning declaration rejects as partial
  truth
- adapter-required authoritative admission rejects until adapter parity witness
  exists
- partial-truth rejection increments the exact counter

Add compile-fail tests under `crates/worth-store/tests/ui/` for:

- external code cannot construct `CompatibilityManifestPublicationReceipt`
- external code cannot construct `CompatibilityRecoveredManifestIndex`
- external code cannot construct `AuthoritativeMeaningDeclaration` if it is
  proof-bearing
- external code cannot construct `AuthoritativePartialTruthRejection`
- external code cannot construct `AuthoritativeCompatibilityWitness`
  through the new admission path
- semantic artifact view cannot be obtained from a checked artifact without the
  authoritative witness/admission path

Register the new trybuild fixtures in the existing
`phase_boundaries_compile_fail` harness.

## Explicit Non-Goals

- no durable SQLite manifest schema
- no runtime store facade read/write compatibility API
- no rolling-upgrade admission execution
- no restore publication execution
- no backup manifest scanning
- no adapter execution
- no maintenance scheduling integration
- no full Milestone 12 certification runner

## Verification

Run:

- `cargo fmt -p worth-store`
- `cargo test -p worth-store compatibility --lib`
- `cargo test -p worth-store milestone_12 --lib`
- `cargo test -p worth-store --test phase_boundaries_compile_fail -- --test-threads=1`

## Exit Criteria

- A decoded artifact cannot pass admission unless its manifest was published or
  recovered through the compatibility manifest ledger.
- A manifest gap, digest drift, or recovered window mismatch has a typed
  failure and exact counter evidence.
- Authoritative meaning exposure requires a checked artifact, admitted receipt,
  and explicit authoritative meaning declaration.
- Batch receipt reuse is invalidated by registry and manifest frontier basis.
- The new proof-bearing publication/recovery/authoritative types cannot be
  forged by external code.
- No runtime read/write, rolling, restore, adapter execution, or durable
  persistence behavior changes in this batch.
