# worth-signal Milestone 2 Field Classification Inventory

> **Status:** Published closure inventory for Milestone 2
>
> **Primary spec:** [milestone-2.md](./milestone-2.md)
>
> **Primary implementation surfaces:**
> - [trace.rs](../../crates/worth-signal/src/data/trace.rs)
> - [effect.rs](../../crates/worth-signal/src/data/graph/runtime/effect.rs)
> - [prepared_apply.rs](../../crates/worth-signal/src/logic/evaluation/engine/prepared_apply.rs)
> - [recorder.rs](../../crates/worth-signal/src/diagnostics/runtime/recorder.rs)
> - [resolver.rs](../../crates/worth-signal/src/logic/explain/resolver.rs)

## Purpose

This inventory closes the Milestone 2 requirement to publish the authority
classification, boundedness, replay stability, and branch-stability rationale
for the remaining artifact-side surfaces.

The standard is strict:

- hot fields must justify their operational residency
- cold fields must justify their retention shape
- read-time surfaces must declare whether absence is expected under lower
  retention policies

## RuntimeArtifactState

| Field | Class | Bounded | Replay stable | Branch stable | Why it remains hot |
| --- | --- | --- | --- | --- | --- |
| `output_hash` | Hot authority | Yes | Yes | Yes | Operational output identity for fast equality, merge comparability, and replay continuity. |
| `output_identity` | Hot derived but required | Yes | Yes | Yes | Required by suppression and merge comparability; compact compared to retained explain detail. |
| `continuity_token: ContinuityAuthorityToken` | Hot authority | Yes | Yes | Yes | Lineage/replay/restore continuity truth. Wrapped to make authority role explicit. |
| `output_change` | Hot authority | Yes | Yes | Yes | Required for invalidation, lineage transition class, and merge semantics. |
| `recomputed` | Hot authority | Yes | Yes | Yes | Distinguishes direct compute from reuse/suppression semantics. |
| `dependency_count` | Hot derived but required | Yes | Yes | Yes | Used by hot telemetry and compact dependency fingerprint comparison. |
| `meaningful_input_changes` | Hot authority | Yes | Yes | Yes | Required for recomputation breadth and replay reporting. |
| `changed_partition_count` | Hot derived but required | Yes | Yes | Yes | Compact locality proof used by runtime telemetry and lineage transition classification. |
| `propagation_suppressed` | Hot authority | Yes | Yes | Yes | Directly drives downstream invalidation behavior. |
| `changed_scopes: CompactChangedScopeProof` | Hot derived but required | Bounded by changed scope set | Yes | Yes | Required by partition-aware invalidation and replay-safe trace reconstruction when cold regions are absent. |
| `memoized_origin` | Hot authority | Yes | Yes | Yes | Direct execution-origin truth used by planner/reporting semantics. |
| `reuse_basis: ReuseOperationalBasis` | Hot authority | Yes | Yes | Yes | Compact operational truth for how the current artifact became current. Wrapped to prevent cold-rich reuse drift. |
| `reuse_origin` | Hot authority | Yes | Yes | Yes | Required for deterministic lineage transition semantics. |
| `reuse_boundary_authority` | Hot authority | Yes | Yes | Yes | Compact digest-based reuse boundary proof used by certification and lineage recording. |
| `lineage_artifact_id: ArtifactTransitionKey` | Hot authority | Yes | Yes | Branch-local by design | Current artifact continuity handle used by lineage recorder and merge/runtime state. Wrapped to make continuity role explicit. |
| `merge_authority` | Hot authority | Yes | Yes | Branch-local by design | Required for branch reconciliation decisions. |

## ColdArtifactIntent

| Field | Class | Bounded | Replay stable | Branch stable | Notes |
| --- | --- | --- | --- | --- | --- |
| `changed_regions` | Cold retained authority seed | Canonical vector | Yes | Yes | Only emitted when cold retention is not fully omitted. |
| `labels` | Cold retained authority seed | Bounded `SmallVec<[String; 4]>` | Yes | Yes | Label count is explicitly capped during hot emission. |
| `keyed_family` | Cold retained authority seed | Yes | Yes | Yes | Diagnostic/read-time assembly only. |
| `keyed_key` | Cold retained authority seed | Yes | Yes | Yes | Diagnostic/read-time assembly only. |
| `reuse_certification` | Cold retained authority seed | Bounded by certification payload | Yes | Yes | Optional; retained only when present on the effect. |
| `reuse_boundary_context` | Cold retained authority seed | Strategy-gated | Yes | Yes | Only emitted for cross-identity/partial-splice strategies under retaining policies. |

## RetainedDiagnosticArtifact / ColdArtifactRecord

| Field | Class | Bounded | Replay stable | Branch stable | Notes |
| --- | --- | --- | --- | --- | --- |
| `changed_regions` | Cold retained authority | Canonical vector | Yes | Yes | Canonical region richness for diagnostics/explain. |
| `labels` | Cold retained authority | Policy-bounded at emission time | Yes | Yes | Read-time only. |
| `keyed_family` | Cold retained authority | Yes | Yes | Yes | Read-time only. |
| `keyed_key` | Cold retained authority | Yes | Yes | Yes | Read-time only. |
| `reuse_certification` | Cold retained authority | Bounded proof payload | Yes | Yes | Read-time only. |
| `reuse_boundary_context` | Cold retained authority | Strategy-gated | Yes | Yes | Rich proof only when explicitly retained. |

## Read-Time Assembly Surfaces

| Surface | Class | Parity contract | Notes |
| --- | --- | --- | --- |
| `HistoricalArtifactRecord` | Cold derivable from canonical | Participates in `SemanticArtifactParity` | Assembled from hot runtime truth plus retained cold authority. |
| `TraceSummary` | Cold derivable from canonical | Participates in `SemanticArtifactParity` | `reuse_boundary_context` is present only when retained cold detail exists. |
| Explanation artifact | Mixed read-time assembly | Participates indirectly through record/summary parity and explicit explanation tests | Upstream/causal links remain canonicalized at read time. |
| Provenance artifact | Mixed read-time assembly | Not reduced to one parity token, but tested for retained-vs-reconstructed equivalence | Carries graph-shaped read-time structure. |

## TraceSummary Behavior Note

`TraceSummary.reuse_boundary_context` is intentionally no longer a hot-lane mirror.
It is a read-time assembly field backed by retained cold detail.

That means:

- consumers still compile against the same field name
- under operational / omit-style policies the field may be `None`
- that `None` does not mean reuse authority is unavailable
- it means only the compact hot authority was kept hot and the rich cold detail
  was not retained

Consumers that need authoritative runtime reuse truth must use
`RuntimeArtifactState.reuse_boundary_authority`, not
`TraceSummary.reuse_boundary_context`.

## Audit Closure Notes

This inventory is the formal justification for the three audit concerns that
were documentation-driven rather than code-driven:

- why `RuntimeArtifactState` still has multiple hot fields
- why those remaining hot fields are still allowed
- why `TraceSummary.reuse_boundary_context` can legitimately be absent under
  lower-retention policy without implying semantic loss of hot continuity truth
