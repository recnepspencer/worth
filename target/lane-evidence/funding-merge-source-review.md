# Pending merge source review

Read-only source review of the pending merge with HEAD
`25619ac5d1ae6092ab5a91bc4b7123be6042475c` and MERGE_HEAD
`14b16fa292e96ff0f5d96b55bd88c338a290cd87`. No Cargo, test, lint,
formatting or boundary command was run by this reviewer, and no source was
edited. This is source review before qualification, not final lane clearance.

Reading boundary: the named conflict clusters in Complete branch/root capture,
append frontier, Head/Full history materialization and kind reconstruction;
direct consumers in Complete preparation/grouping/emission/admission projection,
component capture and the adjacency backing owner. Both parents were compared
independently. The merge plan was not used as proof. Governing sources were
AGENTS.md, coding guidelines, milestone-3.md D12, and the approved funding design
records (with their later corrections).

## Findings

1. **P1 — Production request reconstruction calls a test-only reset method.**
   `crates/worth-relational/src/storage/residency/kind_projection/reconstruction/installation.rs:35`
   calls `adjacency.reset_historical_kind_buckets()`. Its only owner definition
   in `storage/partition/adjacency.rs:147` is gated by `#[cfg(test)]`. Therefore
   ordinary production compilation cannot resolve the method. This new request
   installation adapter retained the pre-cutover call while integration made
   custody explicit. The independently auto-merged `kind_projection/full.rs:172`
   already uses the correct production door,
   `reset_historical_kind_buckets_with_custody(None)`.

   Correction: cut this call over to that same production door. `None` is
   appropriate here because each subsequently installed request path/value owns
   its actual request reservation; do not restore a production alias for the
   retired convenience method. Required verification: production owner compile
   plus the selected Head/Full reconstruction and last-owner/refund courts.
   Law: N/A as a distinct architectural defect; this violates the required
   production build contract and prevents the promised materialization path
   from existing. MENTALITY 4 requires implementation to match its declared
   construction and lifecycle contracts.

2. **P1 — Auto-merged content capture still treats an actual root as FoldedRoot.**
   `crates/worth-relational/src/durability/authority/checkpoint_complete/regions/emission.rs:134`
   calls `source.root.retained_content()`. `source` now comes from
   `CapturedRoots::get` (`checkpoint_complete/roots.rs:46`), whose return type is
   `Option<&Arc<RelationalBranchRoot>>`; `RelationalBranchRoot` has no `root`
   field. This integration content addition was auto-merged against the funding
   parent's `FoldedRoots` removal without its necessary caller cutover.

   Correction: obtain the same immutable content directory directly from
   `source.retained_content()`. Keep `root_capture::capture` in place: deleting
   that call would drop integration's content preservation. Required
   verification: production owner compile, Complete recapture with retained
   content and selected cold reopen/continuation evidence. Law: N/A as a
   distinct architectural defect; this violates the production build contract
   and blocks integration's retained-content emission. The replacement must
   preserve Architecture 18 and MENTALITY 9's authority/derived distinction by
   reading the actual captured root, not reconstructing a substitute.

## Source conclusions and limits

- Actual root/component capture preserves the funding design's state boundary:
  `FrozenCompleteMetadata::capture` obtains branch DATA and component custody
  under the same selection/configuration cut. `capture_complete` validates
  current snapshot/lifecycle pairs and retained owner descriptors and retains
  their roots and genuine obligations. `BranchImages` classifies against that
  same captured metadata. Removal of historical branch/working-state folds is
  intentional D12 behavior, not an omitted replay obligation.
- `source::read_canonical` derives `covered` from the selected base and demands
  only `artifacts.iter().enumerate().skip(covered)`. Complete preparation uses
  actual captured roots. No covered canonical replay was reintroduced in the
  reviewed cluster; no budget/default/fixture allowance change is needed to
  correct the two findings.
- The frontier merge retains the integration parent's `CanonicalDigestId`
  boundaries while retaining the funding parent's request-aware allocation.
  The explicit `.bytes()` conversions feed the same hashes. Sibling path,
  exhausted key, opaque-history coverage and future-append checks remain.
- The new projection request driver shares `clone_claim`, owned historical
  reconstruction, sorted claim equality and installation membership checks
  with the controlled driver. It preserves duplicate, contradictory kind,
  owned generation, peer membership and current bucket validation. Its private
  stopped-cause slot restores the original request refusal before returning;
  malformed-content outcomes remain region-scoped storage failures.
- Ignoring the controlled callback's work/memory estimate in the request
  adapter does not by itself make the request lane unfunded. Head loading holds
  preflight `quote.wire` in `MaterializationFunding`; Full/resident reconstruction
  retains projection and current-image scratch funding. Actual adjacency paths
  are installed through `get_mut_for_request` and `index_*_for_request`, with
  reservations on the copied map/value backing. The controlled driver keeps
  its separate explicit custody callbacks. No supported additional funding
  laundering or semantic divergence was found in that seam.

Remaining evidence is operational: after the two source cutovers, rerun the
required owner/integration/refusal/custody lanes and constitutional gates on the
final merged tree. In particular, source tracing cannot replace the zero
covered-body-read, work/memory denial, retry and last-owner refund observations.
