# Milestone 6 Closeout: Aspect-Aware Physical Layout And Content-Addressed Structural Blocks

Status: Completed on 2026-04-17

Parent spec: [milestone-6.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-6.md)

Roadmap: [worth_store_roadmap.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/worth_store_roadmap.md)

## Summary

Milestone 6 is closed.

`worth-store` now supports admitted aspect-layout planning, typed narrow-read
and explicit fallback surfaces, persisted layout materializations, durable
scope/block/chunk derived families, semantic structural-block identity across
equivalent branch publications, deterministic chunk-model export, authority-
rooted rebuild of Milestone 6 derived artifacts, compile-time proof boundaries,
an explicit three-lane layout-support model, and a named certification suite
proving narrowing parity, rebuild parity, chunk determinism, lane honesty, and
typed corruption failure across backend variation.

The core closure claim is:

- admitted aspect-layout acceleration stays subordinate to canonical authority
  and explicit branch-delta control truth
- persisted scope/block/chunk families make the published Milestone 6 read and
  export paths `Verified`, not just "fast in practice"
- layout-support posture is explicit and policy-resolved rather than ambient:
  callers can choose proof-only, on-demand materialized, or policy-eager
  materialized lanes without silent promotion
- structural block reuse is semantic and cross-branch stable rather than
  branch-local storage coincidence
- chunk export is deterministic, non-authoritative, and stable across rebuild
- deleting Milestone 6 derived families does not change truth; rebuild restores
  the same meaning from authority-coupled seeds

## What Shipped

- admitted Milestone 6 scope vocabulary for:
  - `SingleEntityAspectScope`
  - `EntitySetUniformAspectScope`
  - `CdcTouchedAspectScope`
- typed `Admitted`, `Fallback`, and `Rejected` aspect-layout planning surfaces
- compile-time sealed witnesses for:
  - admitted layout reads
  - structural block reuse
  - frozen chunk layout
  - Milestone 7 layout references
  - Milestone 9 physical chunk references
- persisted Milestone 6 layout materializations
- durable derived access-structure families for:
  - scope-to-slice membership
  - structural-block identity and membership
  - chunk membership
- semantic `StructuralBlockId` coalescing equivalent cross-branch publications
- real execution surfaces for:
  - `execute_aspect_layout_read`
  - `structural_block_lookup`
  - `execute_dedup_backed_read`
  - `export_milestone_6_chunk_model`
- explicit branch-delta control truth surface for parity comparison
- explicit layout-support lanes:
  - `ProofOnly`
  - `OnDemandMaterialized`
  - `PolicyEagerMaterialized`
- explicit resolved layout-support postures:
  - `ProofOnly`
  - `OnDemandMaterialized`
  - `PolicyEagerMaterializedPublished`
  - `PolicyEagerMaterializedReuseExisting`
- explicit first-ship layout-support policy surface:
  - `materialize_hot_branch_reads`
  - `materialize_repeated_scope_reads`
  - `repeated_scope_threshold`
- authority-rooted rebuild of Milestone 6 materializations and derived families
  from commit-coupled layout seeds
- commit-coupled layout seed publication under commit support summaries
- migration-safe SQLite transition to the commit-coupled layout seed table
- machine-checkable `Milestone6CertificationBundle`
- machine-checkable `Milestone6AccessStructureVerification`
- machine-checkable `Milestone6ComplexitySurface`
- machine-checkable `Milestone6CounterContract`

## Acceptance Evidence

The closeout bundle now emits:

- `truth_digest`
- `artifact_digest`
- `diagnostics_digest`
- `counter_snapshot`

The Milestone 6 evidence surface now explicitly carries:

- layout read report
- physical layout report
- requested layout support lane
- resolved layout support lane
- layout support publication disposition
- certification origin
- layout materialization report where persisted support exists
- access-structure contract and open-time verification
- per-path complexity status
- counter contract
- certification summary

## Certification Result

The Milestone 6 named suite required by
[test-requirements.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/test-requirements.md)
is now implemented in
[crates/worth-store/src/tests/milestone_6_certification.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/tests/milestone_6_certification.rs).

It covers:

- admitted layout truth parity across backend variation
- admitted counter-contract parity
- admitted artifact parity
- proof-only lane remaining unpublished
- on-demand lane publishing exactly once then reusing
- policy-eager resolving to proof-only when no trigger is active
- policy-eager resolving to materialized when hot-branch or repeated-scope
  policy triggers activate
- requested-vs-resolved lane evidence and publication disposition honesty
- authority rebuild preserving layout identity
- authority rebuild preserving execution surfaces
- overlapping-branch dedup/control parity
- chunk-export rebuild parity
- SQLite legacy seed migration parity
- scope-shape truth divergence
- generalized-scope explicit fallback
- commit-coupled seed corruption typed failure
- chunk-export corruption typed failure
- chunk-export boundary mismatch typed failure

## Verification

The final verification run used:

- `cargo test -p worth-store milestone_6_certification -- --nocapture`
- `cargo test -p worth-store --lib layout -- --nocapture`
- `cargo test -p worth-store layout_counters -- --nocapture`
- `cargo test -p worth-store --test phase_boundaries_compile_fail -- --nocapture`

All passed.

## Concurrency Boundary With Milestone 7 And Milestone 9

Milestone 6 closed without absorbing Milestone 7 or Milestone 9 authority.

The maintained boundary is:

- Milestone 6 owns aspect-aware physical layout, structural-block identity,
  dedup reuse, chunk identity, and rebuild/certification of those derived
  families
- Milestone 7 still owns schema, lineage, cursor, checkpoint, and support-
  artifact semantic meaning
- Milestone 7-facing references remain branch/frontier typed and do not expose
  layout internals
- Milestone 9 may consume the frozen chunk model and physical chunk references,
  but Milestone 6 still owns chunk identity and the non-authority contract
- bulk orchestration, resume, checkpoint, and bounded-memory semantics remain
  Milestone 9 work rather than leaking back into the Milestone 6 substrate

## Residual Notes

No in-scope Milestone 6 debt remains on the published/materialized path.

The important honest boundary is:

- live, unpublished proof-only layout paths still report `Debt` until real
  materialized scope/block/chunk families are explicitly chosen for that lane
- on-demand materialized Milestone 6 paths certify as `Verified`
- policy-eager lanes are now first-class, but they only certify as `Verified`
  when the resolved posture actually materializes or reuses durable support
- proof-only lanes never silently auto-promote just because matching durable
  support already exists elsewhere in the store

Future work still exists, but it belongs to later milestones:

- Milestone 8 live-query continuation on top of the Milestone 6 substrate
- Milestone 9 bulk orchestration and resumable chunk execution on top of the
  frozen chunk model
- later retention, compaction, replication, and tiering programs that consume
  Milestone 6 derived families without redefining authority
