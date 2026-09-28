# Milestone 6 Implementation Parity

## Purpose

This note records the implementation parity for Milestone 6 after the Phase 5
closeout pass. It distinguishes:

- shipped milestone substance
- shipped but intentionally narrow areas
- explicit deferred debt that must not be mistaken for completed support

## Shipped

- `worth-query` now owns a sealed `query_context` subdomain for:
  - basis declaration and binding
  - admitted current/branch/historical/preview-derived basis contexts
  - query-context execution artifacts
  - diff/comparison admission and query-shaped change-set shaping
  - basis and diff metadata
  - result bundles and replay-safe bundle digests
  - query-context support truth
- admitted basis families are frozen and explicit:
  - `CurrentBranchHead`
  - `BranchHead`
  - `HistoricalSnapshot`
  - `HistoricalCommit`
  - `PreviewDerivedHistorical`
- admitted comparison families are frozen and explicit:
  - `BranchToBranch`
  - `CurrentToHistorical`
  - `HistoricalToHistorical`
  - `PreviewToAuthoritative`
- basis and diff artifacts now preserve:
  - `query_digest`
  - `basis_digest`
  - `basis_family`
  - `comparison_basis_family` where relevant
  - `materialization_path_identity` where relevant
  - `preview_provenance_identity` where relevant
  - `result_digest`
  - `replay_digest`
  - `counter_snapshot_digest`
- runtime-backed basis execution is query-owned rather than host-shaped
- diff stays query-shaped and explicitly denies broad collection comparison
  rather than widening into hidden collection or raw-delta semantics
- historical reconstruction lanes that would broaden beyond the admitted narrow
  result shape now deny typed and early instead of pretending reconstruction
  parity already exists for those shapes
- the unified application facade now exposes Milestone 6 through:
  - explicit query-context capability witnesses
  - explicit basis and diff result bundles
  - query-context-specific support-profile truth derived from the same support
    authority path used by the facade
- milestone-native certification now closes against the real requirement suite
  name: `Historical / Diff / Basis Parity Test`
- compile-fail coverage now seals the remaining Milestone 6 proof artifacts:
  - `QueryBasisMetadata`
  - `DiffQueryMetadata`
  - `QueryContextExecutionArtifact`
  - `QueryContextSupportProfile`
  - raw historical materialization metadata and raw preview foundation artifacts
    cannot stand in for query-owned result bundles

## Shipped But Still Narrow

- historical execution is closed for the admitted runtime-backed paths only
- diff execution is intentionally narrow and denies broad collection-style
  comparison rather than pretending view or lineage semantics are already
  available
- query-context support truth honestly advertises deferred scope markers for:
  - `StoreBackedHistorical`
  - `StoreBackedDiff`
  - `BroadCollectionDiff`
- the certification harness remains under
  `historical_diff_certification/`, but it now proves the full Milestone 6
  closeout surface rather than an earlier phase subset

## Explicit Deferred Debt

- store-backed historical parity
- store-backed diff parity
- richer diff expression and presentation/view semantics beyond the current
  query-result-shaped admitted surface

No hidden basis substitution, host historical reconstruction, or raw delta
leakage debt remains inside the admitted Milestone 6 boundary.

## What Later Milestones May Assume

- basis variation is already query-owned and sealed
- historical materialization-path identity is already visible in ordinary
  query-context execution and result bundles
- diff artifacts are already query-shaped and replay-safe
- the unified facade already exposes the normative daily-driver Milestone 6
  surface
- the Milestone 6 certification slice already proves exact row coverage, exact
  denial coverage, exact replay/counter posture, result-shape identity
  explicitness, and zero rediscovery on admitted lanes

Later milestones must not assume:

- store-backed historical parity is already available
- store-backed diff parity is already available
- broad collection diff is admitted
- Milestone 7 lineage semantics or Milestone 8 view semantics are already
  implied by the current diff surface
