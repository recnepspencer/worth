use super::*;

pub(crate) mod support;

#[path = "activation/activation_staging_boundary_tests.rs"]
mod activation_staging_boundary_tests;
#[path = "activation/activation_staging_test_support.rs"]
pub(crate) mod activation_staging_test_support;
#[path = "source_ingress/active_application_candidate_catalog_test_support.rs"]
mod active_application_candidate_catalog_test_support;
#[path = "source_ingress/active_application_replacement_tests.rs"]
mod active_application_cutover_tests;
#[path = "source_ingress/active_application_evidence_authority_tests.rs"]
mod active_application_evidence_authority_tests;
#[path = "source_ingress/active_application_launch_tests.rs"]
mod active_application_launch_tests;
#[path = "source_ingress/active_application_replacement_storm_tests.rs"]
mod active_application_replacement_storm_tests;
#[path = "source_ingress/active_application_replacement_authority_tests.rs"]
mod active_application_replacement_tests;
#[path = "source_ingress/active_application_session_test_support.rs"]
pub(crate) mod active_application_session_test_support;
#[path = "source_ingress/active_host_session_tests.rs"]
mod active_host_session_tests;
#[path = "lifecycle/active_runtime_authority_tests.rs"]
mod active_runtime_authority_tests;
#[path = "planning/allocation_catalog_test_support.rs"]
pub(crate) mod allocation_catalog_test_support;
#[path = "planning/allocation_constraint_boundary_tests.rs"]
mod allocation_constraint_boundary_tests;
#[path = "planning/allocation_planning_boundary_tests.rs"]
mod allocation_planning_boundary_tests;
#[path = "planning/allocation_planning_certification_tests.rs"]
mod allocation_planning_certification_tests;
#[path = "planning/allocation_planning_evidence_lifecycle_tests.rs"]
mod allocation_planning_evidence_lifecycle_tests;
#[path = "planning/allocation_planning_inspection_boundary_tests.rs"]
mod allocation_planning_inspection_boundary_tests;
#[path = "planning/allocation_planning_test_support.rs"]
pub(crate) mod allocation_planning_test_support;
#[path = "planning/allocation_truth_boundary_tests.rs"]
mod allocation_truth_boundary_tests;
#[path = "source_ingress/appearance_component_session_test_support.rs"]
pub(crate) mod appearance_component_session_test_support;
#[path = "source_ingress/appearance_owner_availability_test_support.rs"]
pub(crate) mod appearance_owner_availability_test_support;
#[path = "source_ingress/appearance_owner_availability_tests.rs"]
mod appearance_owner_availability_tests;
#[path = "source_ingress/appearance_theme_test_support.rs"]
pub(crate) mod appearance_theme_test_support;
#[path = "source_ingress/application_semantic_no_op_tests.rs"]
mod application_semantic_no_op_tests;
#[path = "replacement/artifact_equivalence_boundary_tests.rs"]
mod artifact_equivalence_boundary_tests;
#[path = "replacement/candidate_admission_boundary_tests.rs"]
mod candidate_admission_boundary_tests;
#[path = "source_ingress/candidate_composition_preparation_tests.rs"]
mod candidate_composition_preparation_tests;
#[path = "planning/canvas_spatial_region_reuse_tests.rs"]
mod canvas_spatial_region_reuse_tests;
#[path = "activation/committed_allocation_activation_boundary_tests.rs"]
mod committed_allocation_activation_boundary_tests;
#[path = "planning/complete_equivalence_schema_tests.rs"]
mod complete_equivalence_schema_tests;
#[path = "replacement/dependency_impact_narrowing_boundary_tests.rs"]
mod dependency_impact_narrowing_boundary_tests;
#[path = "replacement/dependency_impact_narrowing_test_support.rs"]
pub(crate) mod dependency_impact_narrowing_test_support;
#[path = "replacement/durable_resize_input_boundary_tests.rs"]
pub(crate) mod durable_resize_input_boundary_tests;
#[path = "replacement/durable_state_inventory_boundary_tests.rs"]
mod durable_state_inventory_boundary_tests;
#[path = "replacement/durable_state_inventory_test_support.rs"]
mod durable_state_inventory_test_support;
#[path = "replacement/durable_state_reconciliation_boundary_tests.rs"]
mod durable_state_reconciliation_boundary_tests;
#[path = "replacement/durable_state_reconciliation_test_support.rs"]
pub(crate) mod durable_state_reconciliation_test_support;
#[path = "planning/execution_plan_input_boundary_tests.rs"]
mod execution_plan_input_boundary_tests;
#[path = "replacement/file_rust_replacement_parity_boundary_tests.rs"]
mod file_rust_replacement_parity_boundary_tests;
#[path = "replacement/file_rust_replacement_parity_test_support.rs"]
mod file_rust_replacement_parity_test_support;
#[path = "activation/frame_activation_gate_boundary_tests.rs"]
mod frame_activation_gate_boundary_tests;
#[path = "execution/handle_allocation_boundary_tests.rs"]
mod handle_allocation_boundary_tests;
#[path = "replacement/identity_match_graph_boundary_tests.rs"]
mod identity_match_graph_boundary_tests;
#[path = "replacement/identity_match_graph_report_boundary_tests.rs"]
mod identity_match_graph_report_boundary_tests;
#[path = "replacement/identity_match_graph_test_support.rs"]
mod identity_match_graph_test_support;
#[path = "host_observation/identity_state_query_certification_boundary_tests.rs"]
mod identity_state_query_certification_boundary_tests;
#[path = "host_observation/identity_state_query_certification_test_support.rs"]
mod identity_state_query_certification_test_support;
#[path = "execution/lane_admission_boundary_tests.rs"]
mod lane_admission_boundary_tests;
#[path = "execution/lane_admission_fixture.rs"]
mod lane_admission_fixture;
#[path = "activation/lane_change_activation_test_support.rs"]
mod lane_change_activation_test_support;
#[path = "execution/lane_frame_cost_certification_boundary_tests.rs"]
mod lane_frame_cost_certification_boundary_tests;
#[path = "execution/lane_frame_cost_certification_canvas_fixture.rs"]
mod lane_frame_cost_certification_canvas_fixture;
#[path = "execution/lane_frame_cost_certification_scale_fixture.rs"]
mod lane_frame_cost_certification_scale_fixture;
#[path = "execution/lane_frame_cost_certification_test_support.rs"]
mod lane_frame_cost_certification_test_support;
#[path = "execution/lane_meaning_parity_boundary_tests.rs"]
mod lane_meaning_parity_boundary_tests;
#[path = "execution/lane_meaning_parity_test_support.rs"]
mod lane_meaning_parity_test_support;
#[path = "lifecycle/lifecycle_path_parity.rs"]
mod lifecycle_path_parity;
#[path = "execution/measurement_boundary_tests.rs"]
mod measurement_boundary_tests;
#[path = "source_ingress/multi_removal_activation_tests.rs"]
mod multi_removal_activation_tests;
#[path = "source_ingress/native_pointer_observation_test_support.rs"]
pub(crate) mod native_pointer_observation_test_support;
#[path = "replacement/node_replacement_classification_boundary_tests.rs"]
mod node_replacement_classification_boundary_tests;
#[path = "replacement/node_replacement_classification_test_support.rs"]
mod node_replacement_classification_test_support;
#[path = "execution/ordinary_lane_boundary_tests.rs"]
mod ordinary_lane_boundary_tests;
#[path = "execution/ordinary_lane_test_support.rs"]
mod ordinary_lane_test_support;
#[path = "planning/ordinary_owner_bundle_scale_tests.rs"]
mod ordinary_owner_bundle_scale_tests;
#[path = "planning/ordinary_plan_structure_hostile_tests.rs"]
mod ordinary_plan_structure_hostile_tests;
#[path = "activation/phase_10_scroll_owned_activation_tests.rs"]
mod phase_10_scroll_owned_activation_tests;
#[path = "activation/phase_11_portal_allocation_tests.rs"]
mod phase_11_portal_allocation_tests;
#[path = "activation/phase_11_portal_test_support.rs"]
mod phase_11_portal_test_support;
#[path = "activation/phase_11_substrate_readiness_tests.rs"]
mod phase_11_substrate_readiness_tests;
#[path = "planning/plan_equivalence_boundary_tests.rs"]
mod plan_equivalence_boundary_tests;
#[path = "planning/plan_equivalence_topology_test_support.rs"]
mod plan_equivalence_topology_test_support;
#[path = "planning/plan_inspection_boundary_tests.rs"]
mod plan_inspection_boundary_tests;
#[path = "planning/plan_inspection_expected_provenance.rs"]
mod plan_inspection_expected_provenance;
#[path = "planning/plan_region_storage_boundary_tests.rs"]
mod plan_region_storage_boundary_tests;
#[path = "planning/plan_topology_boundary_tests.rs"]
mod plan_topology_boundary_tests;
#[path = "planning/plan_topology_identity_boundary_tests.rs"]
mod plan_topology_identity_boundary_tests;
#[path = "planning/plan_topology_test_support.rs"]
mod plan_topology_test_support;
#[path = "activation/production_catalog_activation_test_support.rs"]
pub(crate) mod production_catalog_activation_test_support;
#[path = "replacement/query_binding_comparison_boundary_tests.rs"]
mod query_binding_comparison_boundary_tests;
#[path = "replacement/query_binding_comparison_test_support.rs"]
mod query_binding_comparison_test_support;
#[path = "replacement/query_succession_boundary_tests.rs"]
mod query_succession_boundary_tests;
#[path = "replacement/query_topology_impact_boundary_tests.rs"]
mod query_topology_impact_boundary_tests;
#[path = "execution/realtime_overlay_lane_boundary_tests.rs"]
mod realtime_overlay_lane_boundary_tests;
#[path = "execution/realtime_overlay_lane_test_support.rs"]
mod realtime_overlay_lane_test_support;
#[path = "activation/regional_activation_failure_tests.rs"]
mod regional_activation_failure_tests;
#[path = "activation/regional_activation_test_support.rs"]
mod regional_activation_test_support;
#[path = "activation/regional_activation_transaction_tests.rs"]
mod regional_activation_transaction_tests;
#[path = "execution/reload_counter_boundary_tests.rs"]
mod reload_counter_boundary_tests;
#[path = "execution/reload_counter_test_support.rs"]
mod reload_counter_test_support;
#[path = "host_observation/reload_failure_boundary_tests.rs"]
mod reload_failure_boundary_tests;
#[path = "host_observation/reload_failure_test_support.rs"]
mod reload_failure_test_support;
#[path = "host_observation/reload_storm_certification_boundary_tests.rs"]
mod reload_storm_certification_boundary_tests;
#[path = "host_observation/reload_storm_certification_test_support.rs"]
mod reload_storm_certification_test_support;
#[path = "replacement/replacement_candidate_boundary_tests.rs"]
mod replacement_candidate_boundary_tests;
#[path = "replacement/replacement_impact_boundary_tests.rs"]
mod replacement_impact_boundary_tests;
#[path = "replacement/replacement_impact_test_support.rs"]
pub(crate) mod replacement_impact_test_support;
#[path = "diagnostics/runtime_diagnostics_boundary_tests.rs"]
mod runtime_diagnostics_boundary_tests;
#[path = "diagnostics/runtime_diagnostics_projection_boundary_tests.rs"]
mod runtime_diagnostics_projection_boundary_tests;
#[path = "diagnostics/runtime_diagnostics_projection_test_support.rs"]
mod runtime_diagnostics_projection_test_support;
#[path = "source_ingress/source_backed_graph_identity_boundary_tests.rs"]
mod source_backed_graph_identity_boundary_tests;
#[path = "source_ingress/source_backed_package_boundary_test_support.rs"]
mod source_backed_package_boundary_test_support;
#[path = "source_ingress/source_backed_package_boundary_tests.rs"]
mod source_backed_package_boundary_tests;
#[path = "source_ingress/source_ingress_boundary_test_support.rs"]
pub(crate) mod source_ingress_boundary_test_support;
#[path = "source_ingress/source_ingress_boundary_tests.rs"]
mod source_ingress_boundary_tests;
#[path = "source_ingress/source_ingress_test_support.rs"]
pub(crate) mod source_ingress_test_support;
#[path = "execution/steady_frame_counter_boundary_tests.rs"]
mod steady_frame_counter_boundary_tests;
#[path = "execution/steady_frame_counter_forbidden_work_tests.rs"]
mod steady_frame_counter_forbidden_work_tests;
#[path = "execution/steady_frame_counter_schema_tests.rs"]
mod steady_frame_counter_schema_tests;
#[path = "execution/steady_frame_report_planning_tests.rs"]
mod steady_frame_report_planning_tests;
#[path = "support/touch_origin_receipt_boundary_tests.rs"]
mod touch_origin_receipt_boundary_tests;
#[path = "execution/virtualized_data_lane_boundary_tests.rs"]
mod virtualized_data_lane_boundary_tests;
#[path = "execution/virtualized_data_lane_test_support.rs"]
mod virtualized_data_lane_test_support;
