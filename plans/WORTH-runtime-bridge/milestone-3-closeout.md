# Milestone 3 Closeout: Lineage-Aware Subscription Continuity

## Status

Milestone 3 is closed as of 2026-04-06.

The runtime bridge now preserves subscription continuity as a first-class,
replay-safe bridge contract when truth identity evolves through replace,
split, admitted merge-like continuation, branch-local divergence, or explicit
continuity rejection.

The semantic center shipped in this milestone is:

canonical prior subscription slices are lifted from canonical route truth into
one planned continuity request set, continuity planning consults only explicit
branch and snapshot authority plus truth-owned lineage exports, canonical
resolution classifies each prior slice into one closed-world outcome, lowering
produces one deterministic remap artifact or typed rejection surface, and
replay/diagnostics consume those same continuity artifacts rather than
rediscovering lineage behavior from latest IDs, live truth, or host-local glue.

This is not "lineage happened to keep subscriptions mostly working."
Milestone 3 made continuity itself a bridge-owned, proof-carrying artifact.

The bridge now owns:

- closed-world continuity request, class, outcome, and rejection vocabulary
- explicit continuity authority basis over canonical route truth, source
  branch, and source snapshot
- planned historical lineage packets derived from prior subscription slices
- deterministic continuity classification for single-successor, split,
  admitted merge-like, ambiguous rejection, no-authority rejection, and
  unsupported rejection paths
- canonical remapped subscription-slice artifacts with continuity counters and
  digest-backed identity
- canonical continuity records, continuity replay, and continuity explanation
  reconstruction
- harness certification proving continuity parity, branch divergence behavior,
  replay parity, and explicit rejection behavior

## Shipped Scope

Milestone 3 delivered:

- a dedicated bridge continuity subdomain with bridge-owned authority,
  taxonomy, request planning, lineage packet, resolution, lowering, counters,
  and diagnostics surfaces
- explicit `BridgeContinuityAuthorityBasis` and `BridgeLineageContext` wiring
  so continuity depends on canonical route truth rather than ad hoc side
  context
- a narrow truth-owned lineage adapter seam for historical resolution and
  continuity planning
- canonical continuity request-set derivation from prior route truth and prior
  subscription slices
- planned historical lineage packet derivation and typed lineage-source error
  mapping
- deterministic continuity classification into:
  `ContinuesAsSingleSuccessor`,
  `ContinuesAsSplitSuccessors`,
  `ContinuesViaTruthLoweredCanonicalMergeSuccessor`,
  `RejectedAmbiguousSuccessor`,
  `RejectedNoAuthoritativeSuccessor`,
  `RejectedUnsupportedContinuityClass`
- canonical continuity lowering into remapped subscription-slice artifacts and
  continuity identities
- canonical continuity replay records, replay mismatch detection, and
  explanation reconstruction over canonical continuity truth
- retained continuity diagnostics and continuity explanations alongside route
  diagnostics
- a runtime-backed relational bridge source that now resolves committed bridge
  patches from canonical commit envelopes and resolves bridge snapshot
  authority from commit/version truth rather than mutable latest-publication
  state
- continuity-aware harness fixtures, adapter behavior, parity suites, and
  certification matrices covering replace, split, branch divergence,
  ambiguity rejection, unsupported continuity, and replay parity

## Acceptance Mapping

Milestone 3 is considered closed against the roadmap and the engineering spec
because the required acceptance surfaces are now covered directly.

### `Truth identity evolution preserves or rejects subscription continuity deterministically`

Covered by:

- `harness::tests::planning::bridge_resolved_lineage_continuity_lowers_single_successor_artifact`
- `harness::tests::planning::bridge_resolved_lineage_continuity_lowers_split_successor_artifact`
- `harness::tests::planning::bridge_resolved_lineage_continuity_lowers_merge_like_successor_artifact`
- `harness::tests::planning::bridge_resolved_lineage_continuity_rejects_ambiguous_successor_sets`
- `harness::tests::planning::bridge_resolved_lineage_continuity_rejects_no_authoritative_successor`
- `continuity::resolution::tests::resolution_classifies_single_record_with_multiple_lineages_as_merge_like`
- `continuity::resolution::tests::resolution_rejects_competing_successor_sets_as_ambiguous`

What is proven:

- identical continuity inputs lower to identical canonical continuity outcomes
- replace-style continuity remains distinct from split-style continuity
- the one admitted Milestone 3 merge-like class is explicit and deterministic
- ambiguous and unsupported continuity do not degrade into accidental remaps
- every prior slice receives one typed continuity outcome

### `Topology-style replace/split flows remain traceable through bridge diagnostics`

Covered by:

- `harness::tests::diagnostics::bridge_diagnostics_retain_canonical_continuity_records`
- `harness::tests::explanations::bridge_continuity_explanation_reconstructs_canonical_continuity_truth`
- `harness::tests::planning::bridge_historical_lineage_packet_uses_planned_continuity_requests`
- `harness::tests::planning::bridge_continuity_planning_requires_explicit_lineage_context`
- `harness::tests::planning::bridge_continuity_planning_rejects_branch_mismatch_against_route_truth`

What is proven:

- continuity explanation is derived from canonical continuity truth rather than
  becoming a second continuity authority
- lineage planning is explicit, planned, and route-derived
- branch and snapshot authority remain part of continuity truth
- continuity diagnostics reconstruct continuation and rejection without
  widening into live history rediscovery

### `Replayed lineage-aware routing matches original continuity behavior`

Covered by:

- `harness::tests::replay::bridge_continuity_replay_matches_original_canonical_artifact`
- `harness::tests::replay::bridge_continuity_replay_rejects_artifact_drift`
- `harness::tests::replay::bridge_continuity_replay_rejects_incompatible_canonical_record_version`
- `facade::bridge::tests::runtime_bridge_replays_historical_commit_after_newer_publication_arrives`
- `harness::tests::replay::bridge_replay_detects_route_drift_after_restart_shaped_truth_change`

What is proven:

- continuity replay proceeds from canonical continuity records rather than live
  in-memory lineage state
- continuity replay rejects request, resolution, or artifact drift explicitly
- unsupported canonical record versions fail explicitly
- older bridge route truth remains reconstructable after newer publication
  arrives
- continuity no longer depends on latest-publication coincidence

### `Branch-local identity evolution remains explicit and diagnostics-tier-invariant`

Covered by:

- `harness::tests::certification::bridge_harness_branch_divergence_changes_continuity_outcome_explicitly`
- `harness::tests::parity::bridge_harness_parity_proves_continuity_truth_is_invariant_across_diagnostics_tiers`
- `harness::tests::certification::bridge_continuity_certification_matrix_reports_candidate_profile_parity`
- `harness::tests::certification::bridge_harness_continuity_certifies_ambiguous_rejection_explicitly`

What is proven:

- branch-local lineage differences produce explicit continuity differences
  instead of accidental cross-branch reuse
- diagnostics richness changes explanation only, not continuity truth
- certification profiles preserve the same continuity semantics
- ambiguous continuity remains a typed first-class result under hostile
  certification pressure

### `Continuity planning uses explicit authority rather than hidden breadth`

Covered by:

- `facade::bridge::tests::runtime_bridge_lineage_source_resolves_real_relational_history`
- `adapter::tests::historical_lineage_authority_digest_is_canonical_for_same_inputs`
- `adapter::tests::historical_lineage_authority_rejects_noncanonical_inputs`
- `harness::tests::planning::bridge_historical_lineage_packet_rejects_mismatched_returned_authority_basis`
- `facade::bridge::tests::runtime_bridge_relational_source_drives_public_bridge_delivery_with_canonical_snapshot_authority`

What is proven:

- the truth-owned lineage export is canonicalized and branch/snapshot-bound
- the bridge rejects lineage authority mismatches explicitly
- relational continuity resolution uses snapshot-safe successor resolution
  rather than broad visible-snapshot scans
- the runtime-backed relational source serves bridge continuity and replay from
  authoritative commit/version truth, not mutable latest-publication state

## Additional Hardening Added Before Close

Milestone 3 closeout includes these extra hardening outcomes beyond the minimum
phase labels:

- continuity authority basis was tightened so source branch is preserved as
  canonical route truth rather than trusted from injected side context
- lineage digest construction was moved into bridge-canonical authority rather
  than adapter spelling, preventing continuity identity drift from host-local
  string formation
- route and continuity replay artifacts were versioned and made explicit
  schema-bearing canonical records
- merge-like and ambiguous continuity were implemented as real classification
  paths rather than taxonomy-only names
- rejected ambiguous continuity was prevented from carrying successor remaps
  forward
- the relational continuity seam was narrowed from visible-snapshot scans to
  lineage-local, snapshot-safe successor resolution
- the real runtime-backed relational source was lifted off `latest_bundle()`
  assumptions so historical replay remains reconstructable after newer
  publication
- continuity harness support was promoted from local fixtures to first-class
  parity and certification surfaces with branch-divergence and rejection lanes

These changes were made because the closeout bar was not "continuity seems to
work in the happy path." The closeout bar was production-grade proof,
canonicality, replay safety, and an honest substrate for later historical,
merge-aware, and scale-path milestones.

## Signal Integration Note

Milestone 3 is operationally end-to-end across `worth-relational`,
`worth-runtime-bridge`, and `worth-signal`, but the new architectural weight
landed almost entirely in relational and bridge.

That is intentional.

In this milestone:

- `worth-relational` remains the authority for lineage meaning, historical
  resolution, and commit/version truth
- `worth-runtime-bridge` remains the authority for continuity planning,
  classification, remapping, replay, and diagnostics
- `worth-signal` remains the authority for derived node identity and execution,
  and continues to receive canonical bridge invalidation surfaces rather than
  relational lineage internals

So Milestone 3 does tie into signal operationally, but it does not expand
signal into a second continuity authority.

## Explicit Deferrals

Milestone 3 intentionally does not include:

- full merge-aware bridge semantics across general multi-parent truth history
- structural-identity-assisted remapping
- branch-aware evaluation as a full user-facing product surface beyond the
  continuity-local branch context required here
- speculative preview continuity
- bridge-mediated writeback
- generalized host-facing lineage tooling outside the continuity contract

Those remain later roadmap work and were not smuggled into Milestone 3 under
vague names.

## Verification Baseline

At closeout, the verification baseline for the milestone implementation is:

- `cargo test -p worth-runtime-bridge -- --nocapture`
- `cargo test -p worth-relational facade::bridge -- --nocapture`

This passes cleanly and includes:

- the full bridge unit, harness, replay, parity, certification, and compile-fail boundary suite
- relational bridge-source tests for committed patch loading, snapshot reads,
  continuity lineage resolution, runtime-backed public delivery, and historical
  replay after newer publication arrival
- canonical continuity replay, drift rejection, schema compatibility, branch
  divergence, ambiguity rejection, and diagnostics-tier invariance lanes

## Operational Conclusion

Milestone 3 is now closed at the bridge level.

The runtime bridge no longer treats subscription continuity as a best-effort
effect of latest identity or coincident current truth. It now owns a real
continuity contract: canonical continuity authority basis, planned lineage
packets, deterministic continuity classification, typed rejection, canonical
remap artifacts, replay-safe continuity records, retained explanations, and
certification evidence strong enough to carry later historical, merge-aware,
and scale-path work without reopening the identity substrate.
