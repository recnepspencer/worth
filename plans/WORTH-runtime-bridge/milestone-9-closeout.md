# Milestone 9 Closeout: Merge-Aware Bridge Semantics And Multi-Parent History Consumption

## Status

Milestone 9 is closed as of 2026-04-08.

The runtime bridge now treats merge-bearing truth history as a first-class,
replay-safe, bridge-owned consumption protocol rather than as adapter folklore,
branch-topology guesswork, or structural convenience under pressure.

The semantic center shipped in this milestone is:

one canonical merge declaration lowers through admitted merge contract truth
into one validated merge phase, that phase lowers through one explicit
precedence pipeline over canonical ordered-parent authority, lineage, causal
frontier, schema-policy outcome, and structural advisory inputs, the lowered
packet set reduces into one typed merge routing outcome before any publication
occurs, publication remains split between continuity, advisory remap, and
explanation artifacts, and canonical merge records, replay, diagnostics, and
certification all consume those same bridge-owned artifacts rather than
rediscovering merge meaning from latest truth, host iteration order, patch
shape coincidence, or branch-local folklore.

This is not "the bridge understands merges now."
Milestone 9 made ordered parent meaning, merge-class provenance, denial-stage
localization, replay sufficiency, and causal explanation explicit, typed, and
certifiable.

The bridge now owns:

- a canonical merge declaration, validation, contract, lowering, routing,
  publication, record, and explanation vocabulary
- one canonical bridge-to-relational ontology mapping surface with retained
  version and provenance identity
- explicit parent-order proof and parent-order digest basis rather than
  incidental parent lists
- a typed precedence chain for merge admission, lineage authority,
  deletion/topology gating, causal admissibility, schema-policy outcome
  admissibility, structural advisory refinement, and publication
- typed merge outcomes for continuity candidate, denial, and structural
  contradiction
- replay-safe merge records and merge replay certification bundles retaining
  canonical lowering, routing, publication, and explanation truth
- merge diagnostics and explanations derived from canonical records rather
  than host logs
- harness certification for parent-order determinism, unsupported and denied
  merge classes, replay parity, diagnostics-tier invariance, ontology-lowering
  parity, and topology-rewire hostility

## Shipped Scope

Milestone 9 delivered:

- a dedicated bridge-owned `merge/` subsystem split across taxonomy,
  declaration, validation, contracts, counters, lowering, routing,
  publication, explanation, and replay
- canonical `MergeHistoryDeclaration`,
  `ValidatedMergeHistoryDeclaration`,
  `AdmittedMergeHistoryContract`,
  `LoweredMergeHistoryPacketSet`,
  `ReducedMergeRoutingArtifact`,
  `PublishedMergeContinuityArtifact`,
  `PublishedMergeRemapArtifact`,
  `PublishedMergeExplanationArtifact`, and
  `MergeReplayCertificationBundle` surfaces
- builder-time merge registration, canonical merge registry freezing,
  duplicate and ambiguous declaration rejection, and explicit admission through
  the frozen registry
- bridge-owned merge lowering over ordered parent proof, ontology provenance,
  authoritative lineage, causal frontier, schema-policy outcome, and
  structural advisory inputs
- deterministic reduction into typed merge routing outcomes rather than
  host-local merge interpretation
- explicit publication rules that preserve structural advisory-only status and
  prevent deletion/topology classes from fabricating continuity
- canonical merge records retained in bridge diagnostics and replay from
  canonical merge records rather than ambient latest truth
- widened merge counter surfaces covering declaration, contract, parent count,
  class support, stage-local denial, structural consult width, remap
  publication, explanation request, replay request, replay mismatch, and
  discovery-work accounting
- typed merge publication failures for deletion, topology, causal-truncation,
  policy-rejection, continuity denial, and structural contradiction boundaries
- harness-grade merge certification bundles and merge harness targets for
  suites 10 through 12

## Acceptance Mapping

Milestone 9 is considered closed against the roadmap, the engineering spec,
and `test-requirements.md` because the required acceptance surfaces are now
covered directly.

### `Ordered parent meaning remains canonical and deterministic`

Covered by:

- `merge::declaration::tests::merge_history_declaration_is_canonical_for_same_inputs`
- `merge::lowering::tests::lowered_merge_packet_set_is_canonical_for_same_contract`
- `harness::tests::merge::ordered_parent_history_remains_deterministic_under_adapter_variation`
- `harness::tests::merge::merge_harness_parity_proves_truth_is_invariant_across_diagnostics_tiers`

What is proven:

- ordered parent lists are carried as explicit proof-bearing merge authority,
  not incidental metadata
- equivalent merge declarations lower to identical parent-order digests and
  identical canonical result bundles
- diagnostics-tier changes do not create a second merge meaning
- host-order variation does not perturb parent-order truth or replay output

### `Bridge merge classes remain losslessly attributable to relational merge ontology`

Covered by:

- `merge::declaration::tests::merge_ontology_mapping_surface_is_canonical_for_same_inputs`
- `merge::validation::tests::validation_accepts_lossless_many_to_one_bridge_class_lowering`
- `harness::tests::merge::merge_ontology_lowering_remains_lossless_under_many_to_one_bridge_class_mapping`

What is proven:

- ontology mapping is bridge-owned, versioned, and canonical
- many-to-one lowerings are admitted only when provenance remains lossless
- bridge-level merge class meaning remains attributable to canonical
  relational merge ontology during replay and certification

### `Unsupported and denied merge paths fail closed without structural convenience`

Covered by:

- `facade::tests::merge::runtime_denies_deletion_merge_at_deletion_topology_stage`
- `facade::tests::merge::runtime_denies_causal_truncation_at_causal_stage`
- `facade::tests::merge::runtime_localizes_structural_contradiction_without_reopening_continuity`
- `facade::tests::merge::runtime_rejects_merge_publication_with_typed_denial_kind`
- `harness::tests::merge::unsupported_merge_classes_fail_without_branch_reconciliation_fallback`
- `harness::tests::merge::merge_harness_denial_localizes_stage_without_reopening_continuity`
- `harness::tests::merge::topology_rewire_denial_is_typed_and_keeps_counter_scope_local`
- `harness::tests::merge::merge_harness_topology_rewire_lane_emits_canonical_denial_bundle`

What is proven:

- deletion and topology-rewire classes fail at the explicit
  deletion/topology gate rather than degrading into branch reconciliation
- causal-frontier truncation fails at the explicit causal admissibility stage
- structural contradiction remains typed and localized; it does not reopen
  continuity or remap after authoritative denial
- merge publication failures now expose typed denial kinds rather than a
  generic mismatch bucket
- denial bundles retain provenance and machine-checkable stage-local evidence

### `Replay validates full merge result bundles rather than explanation alone`

Covered by:

- `facade::tests::merge::runtime_replay_merge_history_certifies_full_bundle`
- `facade::tests::merge::runtime_replays_canonical_merge_record`
- `facade::tests::merge::runtime_replay_rejects_incompatible_merge_record_version`
- `harness::tests::merge::merge_replay_preserves_routing_and_explanation_parity`
- `harness::tests::merge::merge_harness_replay_remains_parity_safe_across_candidate_profiles`

What is proven:

- replay reproduces the same full merge bundle from canonical records
- incompatible merge record versions fail explicitly
- replay retains merge-aware routing, continuity, remap, and explanation
  parity rather than reconstructing only the narrative surface
- replay work is mechanically visible through replay and explanation counters

### `Milestone 9 certification bundles are machine-checkable and offline-auditable`

Covered by:

- `harness::tests::merge::merge_harness_certification_matrix_reports_candidate_profile_parity`
- `harness::tests::merge::ordered_parent_history_remains_deterministic_under_adapter_variation`
- `harness::tests::merge::unsupported_merge_classes_fail_without_branch_reconciliation_fallback`
- `harness::tests::merge::merge_replay_preserves_routing_and_explanation_parity`
- `harness::tests::merge::merge_harness_topology_rewire_lane_emits_canonical_denial_bundle`

What is proven:

- merge certification emits canonical bundle fields for
  `merge_history_digest`, `merge_ontology_mapping_report`,
  `parent_order_report`, `routing_digest`, `result_bundle_digest`,
  `merge_support_matrix`, `merge_denial_stage_report`, `failure_digest`,
  `diagnostics_digest`, `continuity_digest`, `explanation_digest`,
  `replay_digest`, and `counter_snapshot`
- equivalent merge runs compare equal across independently produced bundles
- denial lanes remain typed and mechanically attributable
- representative merge workloads prove bounded discovery work on default
  admitted paths instead of hiding widened scans

## Additional Hardening Added Before Close

Milestone 9 closeout includes these extra hardening outcomes beyond the minimum
phase labels:

- the merge precedence chain was hardened into explicit typed stages so future
  continuity pressure cannot smuggle heuristic behavior into routing
- the merge proof surface was widened to include remap publication,
  explanation request, replay request, replay mismatch, and widened-scan
  accounting in the core merge counter type rather than only in harness-local
  summaries
- merge publication failures were widened from a generic mismatch class into
  stage-meaningful denial kinds so canonical merge failure topology is queryable
- merge replay and explanation accounting were corrected so bundle-level
  counter truth reports the actual work performed rather than only artifact
  local snapshots
- ontology validation was hardened to admit lossless many-to-one lowering while
  still rejecting non-canonical duplicate relational entries, matching the
  suite 10 provenance requirement instead of over-restricting the mapping
  surface
- topology-rewire hostility was promoted from representative direct-runtime
  behavior into real harness-level certification bundles with exact bounded-cost
  assertions
- the Milestone 9 QA loop surfaced and corrected proof-surface weaknesses
  before closeout instead of leaving them as â€œtests still passâ€ debt

These changes were made because the closeout bar was not "merge cases replay on
fixtures." The closeout bar was deterministic ordered-parent truth
consumption, replay-safe causal explanation, typed denial boundaries, honest
cost accounting, and certification evidence strong enough to carry later
speculative and system-level work without reopening the merge authority
boundary.

## Explicit Deferrals

Milestone 9 intentionally does not include:

- speculative preview or discard coordination
- cross-runtime merge-policy propagation beyond consuming canonical truth-side
  outcomes
- bridge-mediated writeback or merge-producing commit strategies
- automatic admission of new merge ontology beyond what truth authority
  already defines
- system-level certification of the entire WORTH runtime stack

Those remain later roadmap or system-level work and were not smuggled into
Milestone 9 under merge-shaped names.

## Verification Baseline

At closeout, the verification baseline for the milestone implementation is:

- `cargo fmt --package worth-runtime-bridge`
- `cargo test -p worth-runtime-bridge`

This passes cleanly and includes:

- 321 unit and harness tests
- 1 no-`inc.rs` structural test
- 1 compile-fail boundary test crate
- 8 trybuild/UI phase-boundary tests
- merge runtime, replay, diagnostics, parity, certification, and hostile-lane
  coverage

## Operational Conclusion

Milestone 9 is now closed at the bridge level.

The runtime bridge no longer treats merge-bearing history as a host convenience
or an explanation-only afterthought. It now owns a real merge protocol:
canonical declaration and contract identity, proof-bearing ordered-parent
authority, typed precedence lowering, deterministic merge-aware routing,
explicit continuity/remap/explanation publication, replay-safe merge records,
typed denial surfaces, honest discovery-work counters, and certification
evidence strong enough to carry Milestone 10 and later work without reopening
the merge authority boundary.
