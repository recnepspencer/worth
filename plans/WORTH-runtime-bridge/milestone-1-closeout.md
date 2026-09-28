# Milestone 1 Closeout: Patch-To-Invalidation And Snapshot Evaluation

## Status

Milestone 1 is closed as of 2026-04-05.

The runtime bridge now exists as a real protocol boundary rather than host glue
between `worth-relational` and `worth-signal`.

The semantic center shipped in this milestone is:

committed truth enters once through a canonical bridge envelope, routing lowers
once into a deterministic invalidation artifact, signal-facing evaluation reads
stable snapshot-backed truth only, and replay/diagnostics consume canonical
bridge artifacts rather than rediscovering semantics from host state.

This is not "relational changes wake up signal somehow." The bridge now owns:

- a dedicated crate and facade boundary
- canonical committed-patch ingestion
- frozen bridge-owned mapping truth
- deterministic route planning and lowering
- bounded replay-safe artifact identity
- bridge-owned snapshot-backed evaluation preparation
- canonical route records, replay records, and derived explanations
- typed bridge failure classes and certification evidence

## Shipped Scope

Milestone 1 delivered:

- one public bridge facade in `worth-runtime-bridge`
- canonical committed truth envelope normalization, validation, and ingestion
- a narrow relational adapter for committed patch loading and snapshot opening
- a narrow signal sink contract that accepts bridge invalidation artifacts only
- frozen mapping registration with deterministic ordering and build-time overlap
  rejection
- deterministic route planning with canonical packet derivation, route
  summaries, and counters
- canonical invalidation artifacts with bounded SHA-256-based route identity,
  invalidation identity, and snapshot token values
- bridge-owned delivery that validates snapshot identity and exact packet/result
  correspondence before signal delivery
- a separate bridge-owned signal evaluation preparation surface carrying a
  lowered artifact, the planned read packet, and a bound snapshot context
- canonical route records, versioned canonical replay records, replay parity
  checks, and derived explanation reconstruction
- hostile harness coverage for deterministic routing, snapshot stability,
  diagnostics-tier parity, replay drift detection, and explicit failure
  behavior

## Acceptance Mapping

Milestone 1 is considered closed against the roadmap because the required
acceptance surfaces are now covered directly.

### `Deterministic patch-to-invalidation routing`

Covered by:

- `harness::tests::bridge_route_identity_is_stable_when_patch_items_arrive_out_of_order_with_duplicates`
- `harness::tests::bridge_artifact_identities_are_bounded_and_stable_for_identical_patchsets`
- `harness::tests::bridge_routes_registered_fallback_deterministically`

What is proven:

- canonical patch normalization collapses duplicates before routing
- identical canonical patchsets produce identical route and invalidation
  identities
- artifact identity is bounded and digest-backed rather than basis-string-shaped
- registered fallback routing is deterministic and bridge-owned

### `Snapshot-stable bridge-backed evaluation`

Covered by:

- `harness::tests::bridge_snapshot_delivery_remains_stable_after_newer_truth_arrives`
- `harness::tests::bridge_delivery_keeps_preplanned_snapshot_after_newer_truth_arrives_during_delivery`
- `harness::tests::bridge_prepares_signal_evaluation_with_snapshot_context_without_sink_delivery`
- `harness::tests::bridge_prepared_signal_evaluation_keeps_preplanned_snapshot_after_newer_truth_arrives`
- `harness::tests::bridge_delivery_fails_when_newer_truth_arrives_without_required_snapshot`
- `harness::tests::bridge_snapshot_identity_mismatch_fails_explicitly`
- `harness::tests::bridge_snapshot_contract_rejects_missing_required_reads`

What is proven:

- bridge delivery and prepared evaluation stay pinned to the planned snapshot
- newer truth does not silently change the truth view of an already planned
  route
- signal-facing evaluation receives a bridge snapshot context rather than raw
  truth access
- snapshot acquisition, identity mismatch, and read-contract violations fail
  explicitly

### `Diagnostics-tier invariance and explanation`

Covered by:

- `harness::tests::bridge_harness_parity_proves_routing_truth_is_invariant_across_diagnostics_tiers`
- `harness::tests::bridge_certification_matrix_reports_diagnostics_for_candidate_profiles`
- `harness::tests::bridge_route_explanation_reconstructs_patch_to_invalidation_mapping`

What is proven:

- diagnostics tier changes richness only, not route truth
- certification profiles preserve the same bridge semantics
- canonical route records can reconstruct patch-to-invalidation explanation
  without becoming the routing authority

### `Replay parity from canonical artifacts`

Covered by:

- `harness::tests::bridge_replay_capture_exposes_last_route_record`
- `harness::tests::bridge_replay_accepts_versioned_canonical_route_record`
- `harness::tests::bridge_replay_rejects_incompatible_canonical_route_record_version`
- `harness::tests::bridge_replay_detects_route_drift_after_restart_shaped_truth_change`

What is proven:

- canonical route records are captured by the bridge itself
- replay proceeds through a versioned canonical artifact boundary
- unsupported canonical artifact versions fail explicitly
- replay detects route drift after restart instead of reconstructing a
  convenient new truth

### `Explicit failure behavior`

Covered by:

- `harness::tests::bridge_rejects_unmapped_surface_without_registration`
- `harness::tests::bridge_snapshot_identity_mismatch_fails_explicitly`
- `harness::tests::bridge_snapshot_contract_rejects_missing_required_reads`
- `harness::tests::bridge_delivery_fails_when_newer_truth_arrives_without_required_snapshot`
- `harness::tests::bridge_replay_rejects_incompatible_canonical_route_record_version`
- `harness::tests::bridge_replay_detects_route_drift_after_restart_shaped_truth_change`

What is proven:

- missing mapping registration fails explicitly
- snapshot failures remain typed and phase-specific
- canonical replay compatibility failures remain typed and explicit
- replay mismatch is reported as bridge replay failure rather than raw host
  error leakage

## Additional Hardening Added Before Close

Milestone 1 closeout includes these extra hardening outcomes beyond the minimum
phase labels:

- the signal sink was narrowed so it cannot act as a second bridge executor
- ingress was separated from planning so planning consumes validated canonical
  envelope truth rather than raw host loading paths
- route and invalidation identities were converted from unbounded basis strings
  into bounded digest-backed identities
- `BridgePlannedRoute` was made move-only so one plan cannot be lowered through
  multiple public paths
- canonical replay was tightened to a versioned bridge-owned artifact rather
  than leaving raw in-memory replay as the authoritative public surface
- snapshot packet construction was made bridge-internal so prepared evaluation
  cannot widen snapshot breadth by minting ad hoc packets

These changes were made because the closeout bar was certifiable runtime
behavior, not MVP plausibility.

## Explicit Deferrals

Milestone 1 intentionally does not include:

- fine-grained aspect/lens subscription semantics beyond coarse milestone 1
  routing
- branch-aware bridge policy propagation
- speculative preview flows
- merge-aware routing or merge-aware replay
- bridge-mediated writeback into truth
- generalized reactive source protocol as a product surface

Those remain future roadmap work and were not faked early in Milestone 1.

## Verification Baseline

At closeout, the crate verification baseline is:

- `cargo test -p worth-runtime-bridge`

This passes cleanly and includes:

- 35 unit and harness tests
- compile-fail boundary tests for private construction and phase seams
- replay compatibility and replay drift lanes
- diagnostics-tier parity and certification-profile comparison lanes

## Operational Conclusion

Milestone 1 is now closed at the bridge level.

The bridge no longer depends on host-local routing glue, live truth reads, or
diagnostics-by-convention to function honestly. It now has a real causal
boundary, a canonical proof chain, typed failure behavior, bounded replay-safe
artifacts, and bridge-native trust evidence.
