# Worth Store Milestone 12 Phase 3B Implementation Plan

## Summary

Finish the Phase 3 derived/support-family compatibility surface before moving
to rolling upgrades or restore publication.

The current implementation can classify exact derived reuse, invalidate a
derived artifact when its observed format/semantic version drifts from a
required window, preserve rebuild debt when rebuild is deferred, and require
retained-authority plus maintenance-admission witnesses before a rebuild plan
is admitted. That is the right proof shape, but it is still too generic: it
does not yet encode the first-ship derived/support family lanes strongly enough
to prove snapshots, deltas, layout/chunk records, live-query basis records,
bulk progress records, retention/rebuild records, maintenance summaries, and
tier manifests all preserve their specific domain laws.

This batch should make derived compatibility practical and catalog-wide. It
should still not execute rebuild work, schedule maintenance jobs, add SQLite
schema, run restore publication, implement rolling upgrades, or expose new
facade methods.

## Governing Constraint

A derived/support artifact that decodes successfully must not be reusable
merely because it is a known family. It must be admitted through the correct
family-specific compatibility lane. If its lane requires retained authority,
bulk basis parity, maintenance admission, or tier non-authority preservation,
that requirement must be explicit in the type system and visible in counters.

## Current State To Preserve

- `compatibility/catalog` declares 13 first-ship families and classifies
  snapshots, deltas, Milestone 6, Milestone 8, Milestone 9, Milestone 10,
  Milestone 11, and Milestone 13 records as derived/support families.
- `compatibility/derived` has generic:
  - `DerivedCompatibilityReusePlan`
  - `DerivedBasisCompatibilityPlan`
  - `DerivedInvalidationPlan`
  - `DerivedRebuildRequirement`
  - `CompatibilityRebuildDebt`
  - `RetainedAuthorityCompatibilityWitness`
  - `CompatibilityMaintenanceAdmissionWitness`
  - `DerivedRebuildCompatibilityPlan`
- Read/write admission, manifest recovery, receipt basis binding, adapter cost
  gates, and authoritative partial-truth gates already exist.
- Compile-fail tests already prevent external construction of derived reuse,
  invalidation, rebuild-debt, retained-authority, and maintenance-admission
  proof types.

## Key Changes

### 1. Derived Family Lane Model

Add first-class derived/support lane declarations under
`crates/worth-store/src/compatibility/derived.rs` or a new
`compatibility/derived/lane.rs` module if the file gets too large.

Required types:

- `DerivedCompatibilityLane`
- `DerivedCompatibilityLaneKind`
- `DerivedCompatibilityLaneDeclaration`
- `DerivedCompatibilityLaneRegistry`
- `DerivedCompatibilityLaneSnapshot`

Required lane kinds:

- `SnapshotReuse`
- `BranchDeltaReuse`
- `LayoutBlockChunkReuse`
- `LiveBasisContinuationReuse`
- `BulkResumeReuse`
- `RetentionRebuildSupport`
- `MaintenanceSummarySupport`
- `TierManifestSupport`

Rules:

- Every `CompatibilityFamilyKind` with derived classification must map to
  exactly one lane kind.
- Adding a derived family kind must require updating the lane mapping.
- Lane declarations must expose:
  - family id
  - catalog family kind
  - required artifact compatibility window
  - whether retained authority is required
  - whether maintenance admission is required
  - whether the artifact is exact acceleration or support metadata
  - whether tier/placement non-authority must be preserved
  - counter lane id
  - certification lane id
- Lane declarations are data; they must not execute rebuilds, reads, writes, or
  tier moves.

### 2. Family-Specific Derived Basis Planning

Extend `plan_derived_basis_compatibility` so callers plan through a
`DerivedCompatibilityLaneDeclaration`, not only a generic
`DerivedFamilyDeclaration`.

Required behavior:

- Snapshot and branch-delta lanes may produce exact reuse only on native
  relation plus matching lane window.
- Layout/block/chunk lanes must reject basis drift explicitly; they should not
  silently become generic rebuilds if the lane says the basis is incompatible.
- Live-basis/continuation lanes must preserve Milestone 8 continuation meaning
  by requiring a continuation-support lane declaration rather than pretending
  the store owns live-query semantics.
- Bulk resume lanes must reject non-native resume interpretation before any
  resume-ready support artifact can be considered valid.
- Retention/rebuild-support lanes must preserve Milestone 10 rebuild-debt
  identity when invalidation is deferred.
- Maintenance-summary lanes must require Milestone 11 maintenance admission for
  rebuild or rewrite work.
- Tier-manifest lanes must preserve Milestone 13 non-authority: compatibility
  can reject or invalidate tier records, but cannot convert placement evidence
  into semantic authority.

Required types:

- `DerivedBasisCompatibilityInput`
- `DerivedLaneCompatibilityPlan`
- `DerivedLaneReuseAdmission`
- `DerivedLaneInvalidation`
- `DerivedLaneRebuildRequirement`
- `DerivedLaneRejection`

The plan can wrap the existing generic derived plan types, but the public API
should make the lane semantics visible.

### 3. Tier Non-Authority Compatibility Posture

Add a narrow type surface for Milestone 13 preservation.

Required types:

- `TierCompatibilityNonAuthorityPosture`
- `TierManifestCompatibilityPlan`
- `TierManifestCompatibilityRejection`

Rules:

- Tier compatibility plans may admit the tier record as placement/cost support
  only.
- They may not produce `AuthoritativeCompatibilityWitness`.
- They may not be accepted as retained-authority proof for derived rebuild.
- They must expose whether the tier artifact was reused, invalidated, or
  rejected.

Compile-fail coverage must prove a tier compatibility plan cannot satisfy an
authoritative or retained-authority parameter.

### 4. Bulk Resume Compatibility Posture

Add a narrow type surface for Milestone 9 bulk support records.

Required types:

- `BulkResumeCompatibilityPlan`
- `BulkResumeCompatibilityRejection`
- `BulkResumeInterpretation`

Rules:

- Bulk resume compatibility must be based on declared semantic relation, not
  chunk ordinal proximity or digest shape alone.
- Non-native relation should reject or require explicit restart/rebuild posture;
  it must not silently resume a bulk program under changed interpretation.
- This remains a compatibility plan only. It must not call bulk execution,
  publish checkpoints, or mark a program resume-ready.

### 5. Maintenance Admission Boundary Hardening

The previous batch added `CompatibilityMaintenanceAdmissionWitness` but it is
still a compatibility-local shell. This batch should make the intended
Milestone 11 boundary more explicit without scheduling real work.

Required types:

- `CompatibilityMaintenanceLaneRequirement`
- `CompatibilityMaintenanceLaneAdmission`
- `CompatibilityMaintenanceLaneRejection`

Rules:

- Derived rebuild planning may require a specific maintenance lane class such
  as derived-family rebuild, snapshot refresh, maintenance audit, or tier
  placement/move support.
- A maintenance admission witness must bind:
  - artifact family id
  - compatibility lane id
  - maintenance lane id
  - expected maintenance work class label
- Mismatched family, lane, or maintenance work class rejects typed.
- The witness constructor remains `pub(crate)`.
- No Milestone 11 queue insertion or execution starts in this batch.

### 6. Counter And Evidence Surface

Extend `CompatibilityAdmissionCounters`,
`Milestone12AdmissionReport`, and `MILESTONE_12_COUNTER_NAMES`.

Required counters:

- `compatibility.derived.lane_plan_count`
- `compatibility.derived.lane_reuse_count`
- `compatibility.derived.lane_invalidation_count`
- `compatibility.derived.lane_rejection_count`
- `compatibility.derived.snapshot_reuse_count`
- `compatibility.derived.delta_reuse_count`
- `compatibility.derived.layout_basis_rejection_count`
- `compatibility.derived.bulk_resume_rejection_count`
- `compatibility.derived.maintenance_summary_rebuild_count`
- `compatibility.tier.non_authority_preserved_count`
- `compatibility.tier.manifest_rejection_count`
- `compatibility.maintenance.lane_mismatch_rejection_count`

Add certification matrix rows for:

- `derived_snapshot_reuse_accepted`
- `derived_delta_reuse_accepted`
- `layout_basis_skew_rejected`
- `bulk_resume_skew_rejected`
- `maintenance_summary_rebuild_admitted`
- `tier_manifest_non_authority_preserved`
- `tier_manifest_skew_rejected`

## Tests

Add unit tests for:

- every derived first-ship family maps to exactly one derived compatibility
  lane
- lane snapshot is deterministic and immutable
- snapshot lane admits exact native reuse
- branch-delta lane admits exact native reuse
- layout/block/chunk lane rejects basis drift with a lane-specific rejection
- live-basis/continuation lane produces support posture without redefining
  continuation semantics
- bulk resume lane rejects non-native semantic relation
- retention/rebuild lane preserves rebuild-debt count when rebuild is deferred
- maintenance-summary lane requires matching maintenance lane admission
- maintenance-summary lane rejects mismatched maintenance work class
- tier-manifest lane preserves placement non-authority
- tier-manifest lane rejects semantic drift without producing authority
- new counters project into `Milestone12AdmissionReport`
- counter contract includes every new counter name

Add compile-fail tests under `crates/worth-store/tests/ui/` for:

- external code cannot construct `DerivedCompatibilityLaneSnapshot`
- external code cannot construct `DerivedLaneReuseAdmission`
- external code cannot construct `DerivedLaneRebuildRequirement`
- external code cannot construct `CompatibilityMaintenanceLaneAdmission`
- tier compatibility plan cannot be passed as retained-authority proof
- tier compatibility plan cannot be passed as authoritative witness
- bulk resume compatibility plan cannot be passed as a resume-ready bulk
  program or checkpoint publication proof

## Explicit Non-Goals

- no durable SQLite schema
- no facade read/write compatibility APIs
- no actual derived rebuild execution
- no maintenance queue insertion or worker execution
- no bulk resume execution or checkpoint publication
- no tier move, recall, placement proposal, or residency mutation
- no rolling-upgrade admission
- no backup/restore publication
- no full Milestone 12 certification runner

## Verification

Run:

- `cargo fmt -p worth-store`
- `cargo test -p worth-store compatibility --lib`
- `cargo test -p worth-store milestone_12 --lib`
- `cargo test -p worth-store --test phase_boundaries_compile_fail -- --test-threads=1`

If lane-specific tests become numerous, add a focused filter:

- `cargo test -p worth-store derived --lib`

## Exit Criteria

- Every first-ship derived/support family has a concrete compatibility lane.
- Derived reuse, invalidation, rebuild, and rejection decisions are
  lane-specific instead of generic version drift guesses.
- Bulk resume compatibility rejects changed interpretation before resume
  posture is exposed.
- Maintenance-summary rebuilds require a compatibility maintenance lane
  admission proof that binds family, lane, and work-class labels.
- Tier compatibility preserves Milestone 13 placement non-authority at the type
  boundary.
- New counters and certification row names make derived, bulk, maintenance,
  and tier behavior machine-checkable.
- All new proof-bearing constructors remain sealed from external crates.
