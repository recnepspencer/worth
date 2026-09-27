# Milestone 2 Closeout: Aspect Mapping And Fine-Grained Subscriptions

## Status

Milestone 2 is closed as of 2026-04-05.

The runtime bridge now preserves truth-side precision as a first-class bridge
contract rather than widening every meaningful change into coarse whole-object
invalidation.

The semantic center shipped in this milestone is:

committed truth enters through a compatibility-checked canonical envelope,
normalizes once into canonical truth-delta surfaces, bridge-owned mapping truth
classifies and lowers those surfaces once into canonical subscription slices,
signal delivery consumes slice-native invalidation artifacts over a stable
snapshot, and replay/diagnostics consume the same proof chain rather than
rediscovering semantics from raw spellings, host order, or live truth.

This is not "field changes happen to wake up more targeted signal work."
Milestone 2 made precision itself a replay-safe, bridge-owned causal artifact.

The bridge now owns:

- canonical truth-delta surface normalization and surface identity
- bridge-owned aspect mapping and fine-grained subscription slice identity
- deterministic match classification including suppression, fallback, and
  unsupported-path handling
- slice-native lowered invalidation artifacts and slice-shaped read packets
- replay-safe planning and lowering provenance for fine-grained routes
- diagnostics and explanations that expose why a truth-side surface reached a
  specific subscription slice
- hardened envelope, planning, lowering, and delivery seams suitable for later
  lineage, historical, and bulk-routing milestones

## Shipped Scope

Milestone 2 delivered:

- bridge-owned aspect registration and fine-grained subscription taxonomy with
  deterministic freeze ordering and build-time overlap rejection
- normalized truth-delta surface derivation from committed patch envelopes,
  including canonical surface identity and surface-kind handling
- deterministic fine-grained match classification for matched, suppressed, and
  fallback-admitted surfaces
- canonical subscription-slice lowering with stable slice identity and
  slice-driven snapshot read packets
- slice-native signal delivery artifacts carrying canonical invalidation
  targets, canonical subscription slices, and route contract proof
- typed planning, lowering, prepared-delivery, snapshot-admission, and
  validated-read proof phases rather than loose state threading
- envelope hardening with producer metadata, compatibility checks, sealed
  normalized/validated phases, and normalized-only route identity
- planning and lowering provenance plus validated lowered-plan execution
  admission
- unified identity, match, error, and diagnostics abstractions that removed
  duplicate structural types and copy-projected proof surfaces
- explicit mapping-context, route-scope, diagnostics-sink, snapshot-pool, and
  clone/digest budgeting extension seams for future milestones
- bounded, indexed diagnostics retention and replay-safe canonical route
  records with fine-grained match and slice evidence
- hostile harness coverage for fine-grained precision, replay parity, delivery
  stability, diagnostics-tier invariance, fallback honesty, proof-surface
  visibility, and future-proofing seams

## Acceptance Mapping

Milestone 2 is considered closed against the roadmap because the required
acceptance surfaces are now covered directly.

### `Fine-grained truth changes invalidate only the intended derived surfaces`

Covered by:

- `harness::tests::bridge_route_identity_is_stable_across_equivalent_surface_spellings`
- `harness::tests::bridge_slice_identity_is_stable_for_identical_slice_sets`
- `harness::tests::bridge_route_record_captures_slice_counters_and_slice_entries`
- `routing::surfaces::tests::derives_default_field_surface_without_prefix`
- `routing::surfaces::tests::derives_prefixed_region_surface`
- `routing::matching::tests::classify_surface_as_matched_when_direct_registration_exists`

What is proven:

- semantically equivalent producer spellings normalize to the same routing
  truth
- identical canonical slice sets produce identical slice identity
- slice entries and slice counters are preserved as canonical route truth
- field- and region-shaped truth deltas are normalized structurally rather than
  treated as loose strings
- direct fine-grained matches remain explicit and deterministic

### `Coarse and fine subscription routes remain parity-safe with bridge diagnostics`

Covered by:

- `harness::tests::bridge_harness_parity_proves_routing_truth_is_invariant_across_diagnostics_tiers`
- `harness::tests::bridge_harness_parity_proves_fine_grained_slice_truth_is_invariant_across_diagnostics_tiers`
- `harness::tests::bridge_routes_registered_fallback_deterministically`
- `harness::tests::bridge_route_explanation_exposes_fine_grained_match_status`

What is proven:

- diagnostics richness changes explanation only, not route truth
- fine-grained slice truth remains invariant across diagnostics tiers
- coarse fallback remains explicit, deterministic, and bridge-owned rather than
  hidden widening
- route explanation reconstructs fine-grained match status from canonical route
  truth instead of becoming a second routing authority

### `Aspect mapping behavior is replayable and explainable`

Covered by:

- `harness::tests::bridge_replay_preserves_canonical_route_outcome_for_delivered_patch`
- `harness::tests::bridge_replay_rejects_subscription_slice_drift`
- `harness::tests::bridge_replay_accepts_versioned_canonical_route_record`
- `harness::tests::bridge_route_explanation_reconstructs_patch_to_invalidation_mapping`
- `harness::tests::bridge_replay_capture_exposes_last_route_record`

What is proven:

- replay preserves canonical route identity, invalidation identity, and slice
  identity for successful fine-grained routes
- replay rejects slice-level drift explicitly rather than quietly re-lowering a
  convenient new route
- versioned canonical route records remain the replay authority
- explanation surfaces can reconstruct the path from normalized truth surface
  to lowered invalidation artifact from canonical bridge artifacts alone

### `Stable snapshot-backed reads for fine-grained subscriptions`

Covered by:

- `harness::tests::bridge_snapshot_delivery_remains_stable_after_newer_truth_arrives`
- `harness::tests::bridge_delivery_keeps_preplanned_snapshot_after_newer_truth_arrives_during_delivery`
- `harness::tests::bridge_prepares_signal_evaluation_with_snapshot_context_without_sink_delivery`
- `harness::tests::bridge_prepared_signal_evaluation_keeps_preplanned_snapshot_after_newer_truth_arrives`
- `harness::tests::bridge_snapshot_identity_mismatch_fails_explicitly`
- `harness::tests::bridge_snapshot_contract_rejects_missing_required_reads`

What is proven:

- fine-grained routing still evaluates against the planned snapshot rather than
  drifting live truth
- prepared evaluation and direct delivery use the same snapshot-stable truth
  model
- snapshot identity mismatch and read-contract violations remain typed,
  explicit, and bridge-native

### `Explicit ambiguity, unsupported-path, and failure behavior`

Covered by:

- `harness::tests::bridge_rejects_unmapped_surface_without_registration`
- `routing::surfaces::tests::rejects_unknown_surface_prefix`
- `mapping::aspects::tests::freeze_rejects_same_rank_overlap_for_same_surface_kind`
- `mapping::aspects::tests::freeze_rejects_duplicate_registration_ids`
- `harness::tests::bridge_sink_rejection_records_failure_diagnostics_with_slice_identity`

What is proven:

- unmapped and unsupported fine-grained truth surfaces fail explicitly
- ambiguous registration overlap is rejected during freeze rather than deferred
  to hot-path delivery
- sink-side rejection is captured in failure diagnostics with slice identity
- failure surfaces remain typed and carry bridge-owned context rather than
  leaking raw host behavior

### `Future-proofed planning and lowering contract surfaces`

Covered by:

- `harness::tests::bridge_delivery_and_result_surfaces_expose_planning_and_lowering_proof_contracts`
- `harness::tests::bridge_prepared_delivery_is_equivalent_to_one_shot_delivery`
- `harness::tests::bridge_empty_mapping_context_is_equivalent_to_default_planning_path`
- `harness::tests::bridge_snapshot_reader_pool_is_used_when_configured`
- `harness::tests::bridge_counters_expose_digest_input_bytes`
- `builder::tests::build_accepts_custom_diagnostics_sink`

What is proven:

- planning and lowering provenance are first-class public bridge truth
- prepared delivery and one-shot delivery remain parity-safe
- the mapping-context seam is structurally present and a true no-op today
- snapshot pooling, diagnostics-sink injection, and digest-budget visibility
  exist as real extension seams rather than retrofit notes
- structural cost accounting is visible on route results rather than hidden in
  internal timers

## Additional Hardening Added Before Close

Milestone 2 closeout includes these extra hardening outcomes beyond the minimum
phase labels:

- envelope ingress was hardened from a repair-by-convention bag into a
  compatibility-checked proof chain with sealed normalized and validated
  phases
- route identity was rebased on normalized truth and resolved mapping proof so
  raw producer spelling can no longer drift canonical route truth
- mapping context, planning provenance, lowering provenance, and route contract
  proof were made explicit canonical artifacts so later continuity/history work
  has a real substrate
- the internal pipeline was refactored around typed phase packets rather than
  wide bags and positional tuples
- diagnostics were upgraded from unbounded retained vectors into bounded,
  indexed, sink-backed state with replay/failure capture aligned to bridge
  contract truth
- identity, error, and match surfaces were unified through generic or aliased
  abstractions so future milestones do not have to re-thread structurally
  identical types across the crate
- canonical payloads such as invalidation targets, slice sets, and route record
  entries were moved onto shared ownership so diagnostics and results can share
  one canonical artifact instead of copying richer payloads per route
- clone-budget markers, digest-budget counters, snapshot pooling seams, and
  route-scope markers were added before later milestones can silently turn
  today's cheap paths into hidden structural cost centers

These changes were made because the closeout bar was not "fine-grained routing
works in the happy path." The closeout bar was production-grade proof,
canonicality, replay parity, and a substrate strong enough to carry Milestones
3-5 without architectural surgery.

## Explicit Deferrals

Milestone 2 intentionally does not include:

- lineage-aware subscription continuity across replace, split, merge-like, or
  structural remapping flows
- branch-aware or historical evaluation beyond the snapshot-pinned truth
  contract already inherited from Milestone 1
- bulk-routing, parallel-admission, or full scale-path planning beyond the
  future-proofing seams added here
- speculative preview flows or bridge-mediated writeback
- merge-aware bridge semantics

Those remain later roadmap work and were not smuggled into Milestone 2 under
imprecise names.

## Verification Baseline

At closeout, the crate verification baseline is:

- `cargo test -p worth-runtime-bridge`

This passes cleanly and includes:

- 65 unit and harness tests
- one compile-fail boundary test crate
- eight trybuild/UI phase-boundary tests
- replay parity, replay drift, and replay compatibility lanes
- diagnostics-tier parity and retention-budget lanes
- fine-grained slice identity, slice explanation, and slice failure diagnostics
  lanes

## Operational Conclusion

Milestone 2 is now closed at the bridge level.

The runtime bridge no longer treats precision as diagnostics-only metadata
attached to a coarse invalidation story. It now owns a real fine-grained
causal contract: canonical truth-delta surfaces, canonical slice identity,
typed precision/fallback classification, slice-native lowered artifacts,
snapshot-stable delivery, replay-safe proof chains, and bridge-native
certification evidence.

That is the right substrate for Milestone 3. Lineage continuity can now extend
an already explicit precision model instead of trying to recover one later.
