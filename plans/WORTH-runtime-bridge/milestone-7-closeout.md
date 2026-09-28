# Milestone 7 Closeout: Reactive Source Protocol And Clean Host Surfaces

## Status

Milestone 7 is closed as of 2026-04-08.

The runtime bridge now treats truth-backed source reads as a first-class,
replay-safe bridge protocol rather than as incidental reach-through into
relational storage and host-specific adapter folklore.

The semantic center shipped in this milestone is:

one canonical source declaration lowers through admitted source contract truth
into one validated source phase, that phase lowers into one bridge-planned
truth-view packet set, the packet set materializes only through a bridge-owned
source adapter seam that must preserve planned packet identity and order, and
canonical source materialization, replay, diagnostics, and certification all
consume those same bridge-owned artifacts rather than rediscovering read
meaning from latest truth, builder order, or adapter convenience.

This is not "the bridge can read truth from a source."
Milestone 7 made read authority, source capability, packet breadth, and source
causality explicit, typed, and certifiable.

The bridge now owns:

- a canonical source declaration, contract, validation, planning,
  materialization, failure, and record vocabulary
- explicit source capability admission for snapshot, historical, branch, and
  replay-compatible read surfaces
- a proof-bearing validated source phase derived from admitted contract truth
- packet-set honest source planning and materialization rather than scalar
  convenience reads pretending to be the contract
- canonical source materialization records and canonical source failure records
- replay from canonical source artifacts instead of ambient latest truth
- diagnostics and explanations derived from retained source truth rather than
  host logs or harness folklore
- harness certification for multi-host parity, capability rejection, builder
  swap parity, replay parity, and hostile adapter behavior

## Shipped Scope

Milestone 7 delivered:

- a dedicated bridge-owned `source/` subdomain split across declaration,
  capabilities, contracts, validation, planning, materialization, records, and
  failures
- canonical `SourceDeclaration`, `AdmittedSourceContract`,
  `ValidatedSourceDeclaration`, `PlannedSourceReadPacketSet`,
  `MaterializedTruthViewPacketSet`, `SourceMaterializationRecord`, and
  `SourceFailureRecord` surfaces
- builder-time source registration, canonical source registry freezing,
  duplicate and ambiguous declaration rejection, capability mismatch rejection,
  and explicit missing/multiple source-adapter rejection
- runtime source admission through the frozen registry rather than ambient
  adapter capability discovery
- public source packet-set planning and materialization entrypoints, including
  batch packet planning and batch source materialization
- a bridge-owned source adapter seam that materializes planned packets while
  preserving planned packet identity and order
- canonical source materialization records retaining packet-set truth,
  authority-basis digests, snapshot identities, materialization paths, and
  source counters
- typed source failure classification for contract mismatch, materialization
  rejection, selector mismatch, and adapter drift
- retained source diagnostics, source explanations, and source replay from
  canonical source records
- harness-grade source certification bundles covering control, hostile, and
  replay lanes plus named parity matrices for multi-host and builder-swap
  equivalence
- hostile runtime and harness coverage for unregistered declarations, adapter
  snapshot-open rejection, adapter snapshot-identity drift, and adapter packet
  reordering drift

## Acceptance Mapping

Milestone 7 is considered closed against the roadmap, the engineering spec,
and `test-requirements.md` because the required acceptance surfaces are now
covered directly.

### `Multiple host-shaped source implementations satisfy the same bridge contract`

Covered by:

- `harness::tests::source::certification::multi_host_adapters_preserve_canonical_truth_view_results`
- `harness::tests::source::certification::multi_host_adapters_preserve_canonical_truth_view_results_for_batch_source_materialization`
- `facade::tests::policy_and_materialization::source_diagnostics_richness_preserves_source_truth`

What is proven:

- direct and wrapped host adapter shapes produce identical source truth and
  identical retained source bundles
- packet-set materialization preserves the same canonical truth-view result
  across host adapter shapes
- diagnostics richness changes explanation only, not source truth

### `Source-backed evaluation remains parity-safe across admitted read modes`

Covered by:

- `facade::tests::policy_and_materialization::runtime_materializes_registered_source_packet`
- `facade::tests::policy_and_materialization::runtime_materializes_registered_source_packet_set`
- `facade::tests::policy_and_materialization::runtime_replays_registered_source_materialization_record`
- `facade::tests::policy_and_materialization::runtime_replays_multi_packet_source_materialization_record`
- `harness::tests::source::certification::bridge_harness_source_control_hostile_and_replay_lanes_preserve_certification_truth`

What is proven:

- admitted source declarations lower into bridge-owned truth-view packets rather
  than bypassing the truth-view planner
- single-packet and packet-set source reads both remain canonical and replayable
- replay proceeds from canonical source artifacts and reconstructs the same
  packet-set truth instead of consulting latest truth
- control and replay lanes preserve the same truth-view and source-contract
  digests

### `Bridge setup remains explicit and comprehensible at construction sites`

Covered by:

- `builder::tests::build_rejects_duplicate_source_declarations`
- `builder::tests::build_rejects_source_declarations_without_source_adapter`
- `builder::tests::build_rejects_multiple_source_adapters`
- `builder::tests::build_rejects_source_capability_mismatch_before_runtime_construction`
- `builder::tests::build_source_registry_digest_is_order_invariant`
- `facade::tests::policy_and_materialization::source_builder_order_does_not_change_materialized_source_truth`
- `harness::tests::source::certification::source_builder_swap_parity_is_harness_certified`

What is proven:

- builder order does not redefine source meaning
- source declarations and adapter wiring fail early and typed when incomplete or
  conflicting
- capability admission is frozen before runtime construction
- equivalent builder setups produce identical source registry and source record
  truth

### `Unsupported or hostile source paths fail explicitly before semantic drift`

Covered by:

- `facade::tests::policy_and_materialization::runtime_rejects_unregistered_source_declaration`
- `facade::tests::policy_and_materialization::runtime_records_source_materialization_rejection_when_adapter_cannot_open_snapshot`
- `facade::tests::policy_and_materialization::runtime_records_adapter_capability_drift_when_adapter_binds_wrong_snapshot`
- `facade::tests::policy_and_materialization::runtime_rejects_source_packet_set_reordering_from_adapter`
- `harness::tests::source::certification::source_capability_rejection_matrix_is_harness_certified`
- `harness::tests::source::certification::source_adapter_open_rejection_is_typed_and_leaves_zero_false_success_residue`
- `harness::tests::source::certification::source_adapter_identity_drift_is_typed_and_leaves_zero_false_success_residue`

What is proven:

- unregistered declarations fail as source-contract mismatch rather than
  degrading into ambient reads
- adapter snapshot-open rejection and snapshot-identity drift remain typed and
  retained as canonical source failure truth
- host adapters cannot reorder or substitute bridge-planned packet truth
  without explicit typed rejection
- hostile certification lanes assert exact zero-residue counter behavior, not
  just failure presence

### `Milestone 7 certification bundles are machine-checkable and offline-auditable`

Covered by:

- `harness::tests::source::certification::bridge_harness_source_control_hostile_and_replay_lanes_preserve_certification_truth`
- `harness::tests::source::certification::multi_host_adapters_preserve_canonical_truth_view_results`
- `harness::tests::source::certification::source_capability_rejection_matrix_is_harness_certified`
- `harness::tests::source::certification::source_builder_swap_parity_is_harness_certified`

What is proven:

- source certification emits canonical bundle fields for `truth_view_digest`,
  `source_contract_digest`, `routing_digest`, `diagnostics_digest`,
  `failure_digest`, `replay_digest`, and `counter_snapshot`
- control, hostile, and replay lanes are all present where required
- parity and rejection judgments can be made from canonical bundle truth rather
  than live host logs
- exact zero counters are asserted where the milestone claims no fallback or no
  false-success residue

## Additional Hardening Added Before Close

Milestone 7 closeout includes these extra hardening outcomes beyond the minimum
phase labels:

- the source pipeline was split into real proof-bearing phases rather than
  leaving source work as facade-local convenience methods
- `ValidatedSourceDeclaration` was tightened to derive from admitted contract
  truth rather than rewrapping raw declarations cosmetically
- the source adapter seam was hardened so packet-set materialization must
  preserve bridge-planned packet identity and order exactly
- source failure retention was promoted into canonical bridge-owned failure
  truth instead of leaving hostile certification to harness-synthesized
  summaries
- source records were widened from singleton-shaped summaries to full packet-set
  truth with explicit planned/materialized set digests
- public batch source planning and materialization were added so packet-set
  truth is a real API surface rather than an internal representational trick
- source diagnostics and explanations were widened to packet-set scope so
  explanation does not collapse back into first-packet folklore
- the in-memory source harness fixture stopped taking the write lock twice for
  each committed-patch insert, removing unnecessary coordination churn from the
  certification substrate

These changes were made because the closeout bar was not "source reads work in
the happy path." The closeout bar was production-grade truth ownership,
causality preservation, packet-set honesty, replay safety, and certification
evidence strong enough to support later remapping, merge, and policy milestones
without reopening the read boundary.

## Explicit Deferrals

Milestone 7 intentionally does not include:

- structural-identity-aware remapping logic
- merge-aware source semantics over multi-parent truth history
- policy provenance propagation beyond source admission and diagnostics surfaces
- speculative preview or scheduler-owned downstream execution semantics
- bridge-mediated writeback or source-driven commit strategies
- generalized host-facing truth tooling outside the bridge-owned source
  contract

Those remain later roadmap work and were not smuggled into Milestone 7 under
ambiguous names.

## Verification Baseline

At closeout, the verification baseline for the milestone implementation is:

- `cargo test -p worth-runtime-bridge -- --nocapture`

This passes cleanly and includes:

- 238 unit and harness tests
- 1 no-`inc.rs` structural test
- 1 compile-fail boundary test crate
- 8 trybuild/UI phase-boundary tests
- source runtime, replay, diagnostics, hostile-adapter, parity, and
  certification coverage

## Operational Conclusion

Milestone 7 is now closed at the bridge level.

The runtime bridge no longer treats truth-backed reads as convenience access to
whatever a relational host happens to expose. It now owns a real source
contract: canonical declaration and contract identity, proof-bearing validation,
explicit capability admission, packet-set truthful planning and materialization,
typed retained failures, replay-safe source records, strict adapter obedience,
and certification evidence strong enough to carry Milestone 8 and later work
without reopening the source boundary.
