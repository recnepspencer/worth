#[path = "relational_certification/baseline_layers.rs"]
mod baseline_layers;
#[path = "relational_certification/basis/cost.rs"]
mod basis_cost;
#[path = "relational_certification/basis/observation.rs"]
mod basis_observation;
#[path = "relational_certification/basis/read_cutover.rs"]
mod basis_read_cutover;
#[path = "relational_certification/basis/readmission.rs"]
mod basis_readmission;
#[path = "relational_certification/basis/retention.rs"]
mod basis_retention;
#[path = "relational_certification/comparison_contracts.rs"]
mod comparison_contracts;
#[path = "relational_certification/delta_contracts.rs"]
mod delta_contracts;
#[path = "relational_certification/field_contracts.rs"]
mod field_contracts;
#[path = "relational_certification/field_values.rs"]
mod field_values;
#[path = "relational_certification/invariants/admission/graph_composition_probe.rs"]
mod graph_composition_probe;
#[path = "relational_certification/invariants/admission/graph_selected_state_probe.rs"]
mod graph_selected_state_probe;
#[path = "relational_certification/root/selection/branch_invariant.rs"]
mod invariant_branch_selection;
#[path = "relational_certification/invariants/uniqueness/global.rs"]
mod invariant_global_uniqueness;
#[path = "relational_certification/invariants/selected_state/oracle_expectations.rs"]
mod invariant_oracle_expectations;
#[path = "relational_certification/invariants/selected_state/proposed_state.rs"]
mod invariant_proposed_state;
#[path = "relational_certification/invariants/selected_state/structural_selection.rs"]
mod invariant_structural_selection;
#[path = "relational_certification/invariants/uniqueness/assertion.rs"]
mod invariant_uniqueness_assertion;
#[path = "relational_certification/mvcc/archive_retention.rs"]
mod mvcc_archive_retention;
#[path = "relational_certification/mvcc/branch_fork_fixture.rs"]
mod mvcc_branch_fork_fixture;
#[path = "relational_certification/mvcc/cancellation.rs"]
mod mvcc_cancellation;
#[cfg(feature = "test-operation-control")]
#[path = "relational_certification/mvcc/cancellation_publication_boundaries.rs"]
mod mvcc_cancellation_publication_boundaries;
#[path = "relational_certification/mvcc/model_sequences.rs"]
mod mvcc_model_sequences;
#[cfg(feature = "test-operation-control")]
#[path = "relational_certification/mvcc/owner_phase_locality.rs"]
mod mvcc_owner_phase_locality;
#[cfg(feature = "test-operation-control")]
#[path = "relational_certification/mvcc/owner_phase_locality_court.rs"]
mod mvcc_owner_phase_locality_court;
#[path = "relational_certification/mvcc/preparation_owner.rs"]
mod mvcc_preparation_owner;
#[path = "relational_certification/mvcc/retention.rs"]
mod mvcc_retention;
#[path = "relational_certification/mvcc/seeded_model.rs"]
mod mvcc_seeded_model;
#[path = "relational_certification/observation_contracts.rs"]
mod observation_contracts;
#[path = "relational_certification/oracle_ancestry.rs"]
mod oracle_ancestry;
#[path = "relational_certification/oracle_application.rs"]
mod oracle_application;
#[path = "relational_certification/owner_service_completion.rs"]
mod owner_service_completion;
#[path = "relational_certification/preservation/recovery.rs"]
mod preservation_recovery;
#[path = "relational_certification/production_failures.rs"]
mod production_failures;
#[path = "relational_certification/production_snapshot_failures.rs"]
mod production_snapshot_failures;
#[path = "relational_certification/production_world.rs"]
mod production_world;
#[path = "relational_certification/profile_contracts.rs"]
mod profile_contracts;
#[path = "relational_certification/read_contracts.rs"]
mod read_contracts;
#[path = "relational_certification/reference/attempt_evidence.rs"]
mod reference_attempt_evidence;
#[path = "relational_certification/reference/compatibility.rs"]
mod reference_compatibility;
#[path = "relational_certification/reference/cost.rs"]
mod reference_cost;
#[path = "relational_certification/reference/fork.rs"]
mod reference_fork;
#[path = "relational_certification/reference/owner_binding.rs"]
mod reference_owner_binding;
#[path = "relational_certification/reference/retention.rs"]
mod reference_retention;
#[path = "relational_certification/root/accounting/authoritative.rs"]
mod root_authoritative_accounting;
#[path = "relational_certification/root/copy_on_write/branch.rs"]
mod root_branch_copy_on_write;
#[path = "relational_certification/root/selection/branch_isolation.rs"]
mod root_branch_isolation;
#[path = "relational_certification/root/selection/traversal.rs"]
mod root_branch_traversal_isolation;
#[path = "relational_certification/root/cost/scale_axes.rs"]
mod root_cost_scale_axes;
#[path = "relational_certification/root/cost/scopes.rs"]
mod root_cost_scopes;
#[path = "relational_certification/root/delta/lowering_contracts.rs"]
mod root_delta_lowering_contracts;
#[path = "relational_certification/root/accounting/derived_cache.rs"]
mod root_derived_cache_accounting;
#[path = "relational_certification/root/sharing/fork.rs"]
mod root_fork_sharing;
#[path = "relational_certification/root/inspection/boundaries.rs"]
mod root_inspection_boundaries;
#[path = "relational_certification/root/copy_on_write/named_delta.rs"]
mod root_named_delta_cow;
#[path = "relational_certification/root/accounting/persistent_path.rs"]
mod root_persistent_path_accounting;
#[path = "relational_certification/root/copy_on_write/region_reuse.rs"]
mod root_region_reuse;
#[path = "relational_certification/root/sharing/metric_truth_lanes.rs"]
mod root_sharing_metric_truth_lanes;
#[path = "relational_certification/root/sharing/observation.rs"]
mod root_sharing_observation;
#[path = "relational_certification/invariants/admission/scale_profile.rs"]
mod scale_invariant_admission;
#[path = "relational_certification/schema_contracts.rs"]
mod schema_contracts;
#[cfg(feature = "test-operation-control")]
#[path = "relational_certification/schema_transition_cancellation.rs"]
mod schema_transition_cancellation;
#[path = "relational_certification/schema_transition_replay.rs"]
mod schema_transition_replay;
#[path = "relational_certification/schema_transition_target.rs"]
mod schema_transition_target;
#[path = "relational_certification/invariants/admission/standard_graph_composition.rs"]
mod standard_graph_composition;
#[path = "relational_certification/trace_replay.rs"]
mod trace_replay;
#[path = "relational_certification/world/mod.rs"]
mod world;

#[test]
fn supply_chain_semantic_world_target_is_present() {
    let scale = world::supply_chain::SupplyChainScale::court();
    let baseline = world::supply_chain::SupplyChainBaseline::operating(scale);
    assert_eq!(baseline.scale.name, world::supply_chain::ScaleName::Court);
    assert_eq!(baseline.definition.entities.len(), 244);
}
