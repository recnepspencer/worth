#[path = "relational_certification/baseline_layers.rs"]
pub(crate) mod baseline_layers;

#[allow(unused_imports)]
pub(crate) use crate::exhaustive::relational_certification::basis_cost;

#[path = "relational_certification/basis/observation.rs"]
pub(crate) mod basis_observation;

#[path = "relational_certification/basis/read_cutover.rs"]
pub(crate) mod basis_read_cutover;

#[path = "relational_certification/basis/readmission.rs"]
pub(crate) mod basis_readmission;

#[path = "relational_certification/basis/retention.rs"]
pub(crate) mod basis_retention;

#[path = "relational_certification/comparison_contracts.rs"]
pub(crate) mod comparison_contracts;

#[path = "relational_certification/delta_contracts.rs"]
pub(crate) mod delta_contracts;

#[path = "relational_certification/field_contracts.rs"]
pub(crate) mod field_contracts;

#[path = "../fixtures/relational_certification/field_values.rs"]
pub(crate) mod field_values;

#[path = "../fixtures/relational_certification/invariants/admission/graph_composition_probe.rs"]
pub(crate) mod graph_composition_probe;

#[path = "../fixtures/relational_certification/invariants/admission/graph_selected_state_probe.rs"]
pub(crate) mod graph_selected_state_probe;

#[path = "relational_certification/root/selection/branch_invariant.rs"]
pub(crate) mod invariant_branch_selection;

#[path = "relational_certification/invariants/uniqueness/global.rs"]
pub(crate) mod invariant_global_uniqueness;

#[path = "../fixtures/relational_certification/invariants/selected_state/oracle_expectations.rs"]
pub(crate) mod invariant_oracle_expectations;

#[path = "relational_certification/invariants/selected_state/proposed_state.rs"]
pub(crate) mod invariant_proposed_state;

#[allow(unused_imports)]
pub(crate) use crate::exhaustive::relational_certification::invariant_structural_selection;

#[path = "../fixtures/relational_certification/invariants/uniqueness/assertion.rs"]
pub(crate) mod invariant_uniqueness_assertion;

#[path = "relational_certification/mvcc/archive_retention.rs"]
pub(crate) mod mvcc_archive_retention;

#[path = "../fixtures/relational_certification/mvcc/branch_fork_fixture.rs"]
pub(crate) mod mvcc_branch_fork_fixture;

#[path = "relational_certification/mvcc/cancellation.rs"]
pub(crate) mod mvcc_cancellation;

#[path = "relational_certification/mvcc/cancellation_publication_boundaries.rs"]
pub(crate) mod mvcc_cancellation_publication_boundaries;

#[allow(unused_imports)]
pub(crate) use crate::exhaustive::relational_certification::mvcc_model_sequences;

#[path = "relational_certification/mvcc/owner_phase_locality.rs"]
pub(crate) mod mvcc_owner_phase_locality;

#[path = "../fixtures/relational_certification/mvcc/owner_phase_locality_court.rs"]
pub(crate) mod mvcc_owner_phase_locality_court;

#[path = "relational_certification/mvcc/preparation_owner.rs"]
pub(crate) mod mvcc_preparation_owner;

#[path = "relational_certification/mvcc/retention.rs"]
pub(crate) mod mvcc_retention;

#[allow(unused_imports)]
pub(crate) use crate::exhaustive::relational_certification::mvcc_seeded_model;

#[path = "relational_certification/observation_contracts.rs"]
pub(crate) mod observation_contracts;

#[allow(unused_imports)]
pub(crate) use crate::exhaustive::relational_certification::oracle_ancestry;

#[allow(unused_imports)]
pub(crate) use crate::exhaustive::relational_certification::oracle_application;

#[path = "relational_certification/owner_service_completion.rs"]
pub(crate) mod owner_service_completion;

#[allow(unused_imports)]
pub(crate) use crate::exhaustive::relational_certification::preservation_recovery;

#[path = "relational_certification/production_failures.rs"]
pub(crate) mod production_failures;

#[path = "relational_certification/production_snapshot_failures.rs"]
pub(crate) mod production_snapshot_failures;

#[path = "relational_certification/production_world.rs"]
pub(crate) mod production_world;

#[path = "relational_certification/profile_contracts.rs"]
pub(crate) mod profile_contracts;

#[path = "relational_certification/read_contracts.rs"]
pub(crate) mod read_contracts;

#[path = "../fixtures/relational_certification/reference/attempt_evidence.rs"]
pub(crate) mod reference_attempt_evidence;

#[path = "relational_certification/reference/compatibility.rs"]
pub(crate) mod reference_compatibility;

#[path = "relational_certification/reference/cost.rs"]
pub(crate) mod reference_cost;

#[path = "relational_certification/reference/fork.rs"]
pub(crate) mod reference_fork;

#[path = "relational_certification/reference/owner_binding.rs"]
pub(crate) mod reference_owner_binding;

#[path = "relational_certification/reference/retention.rs"]
pub(crate) mod reference_retention;

#[allow(unused_imports)]
pub(crate) use crate::exhaustive::relational_certification::root_authoritative_accounting;

#[path = "relational_certification/root/copy_on_write/branch.rs"]
pub(crate) mod root_branch_copy_on_write;

#[path = "relational_certification/root/selection/branch_isolation.rs"]
pub(crate) mod root_branch_isolation;

#[path = "relational_certification/root/selection/traversal.rs"]
pub(crate) mod root_branch_traversal_isolation;

#[allow(unused_imports)]
pub(crate) use crate::exhaustive::relational_certification::root_cost_scale_axes;

#[allow(unused_imports)]
pub(crate) use crate::exhaustive::relational_certification::root_cost_scopes;

#[path = "relational_certification/root/delta/lowering_contracts.rs"]
pub(crate) mod root_delta_lowering_contracts;

#[path = "relational_certification/root/accounting/derived_cache.rs"]
pub(crate) mod root_derived_cache_accounting;

#[path = "relational_certification/root/sharing/fork.rs"]
pub(crate) mod root_fork_sharing;

#[path = "relational_certification/root/inspection/boundaries.rs"]
pub(crate) mod root_inspection_boundaries;

#[path = "relational_certification/root/copy_on_write/named_delta.rs"]
pub(crate) mod root_named_delta_cow;

#[allow(unused_imports)]
pub(crate) use crate::exhaustive::relational_certification::root_persistent_path_accounting;

#[path = "relational_certification/root/copy_on_write/region_reuse.rs"]
pub(crate) mod root_region_reuse;

#[path = "relational_certification/root/sharing/metric_truth_lanes.rs"]
pub(crate) mod root_sharing_metric_truth_lanes;

#[path = "../fixtures/relational_certification/root/sharing/observation.rs"]
pub(crate) mod root_sharing_observation;

#[allow(unused_imports)]
pub(crate) use crate::scale::relational_certification::scale_invariant_admission;

#[path = "relational_certification/schema_contracts.rs"]
pub(crate) mod schema_contracts;

#[path = "relational_certification/schema_transition_cancellation.rs"]
pub(crate) mod schema_transition_cancellation;

#[path = "relational_certification/schema_transition_replay.rs"]
pub(crate) mod schema_transition_replay;

#[path = "relational_certification/schema_transition_target.rs"]
pub(crate) mod schema_transition_target;

#[path = "relational_certification/invariants/admission/standard_graph_composition.rs"]
pub(crate) mod standard_graph_composition;

#[allow(unused_imports)]
pub(crate) use crate::exhaustive::relational_certification::trace_replay;

#[path = "../fixtures/relational_certification/world/mod.rs"]
pub(crate) mod world;

#[test]
fn supply_chain_semantic_world_target_is_present() {
    let scale = world::supply_chain::SupplyChainScale::court();
    let baseline = world::supply_chain::SupplyChainBaseline::operating(scale);
    assert_eq!(baseline.scale.name, world::supply_chain::ScaleName::Court);
    assert_eq!(baseline.definition.entities.len(), 244);
}

#[allow(unused_imports)]
pub(crate) use crate::shared_relational_certification_test_execution_policy as test_execution_policy;
