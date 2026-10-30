pub use std::collections::BTreeMap;

pub use worth_query::facade::domain;

pub use super::installed_operation_fixture::configured_runtime;
pub use super::operation_sharing::settle;

pub fn assert_work_classes(snapshot: &domain::WorthQueryConsumptionCostSnapshot) {
    use worth_foundational::FoundationalPerformanceWorkClass as Class;
    for row in snapshot.rows() {
        let expected = if row.name().starts_with("query.execution.") {
            Class::AuthoritativeObservation
        } else {
            Class::ValidationPlanning
        };
        assert_eq!(row.work_class(), expected, "wrong class for {}", row.name());
    }
}

pub fn nonzero_rows(snapshot: &domain::WorthQueryConsumptionCostSnapshot) -> BTreeMap<&str, u64> {
    operational_rows(snapshot)
        .into_iter()
        .filter(|(_, count)| *count != 0)
        .collect()
}

pub fn expected_nonzero_rows() -> BTreeMap<&'static str, u64> {
    [
        ("query.lookup.authority_checks", 1),
        ("query.lookup.indexed_operation_lookups", 1),
        ("query.lookup.graph_binding_lookups", 1),
        ("query.binding.authority_checks", 1),
        ("query.binding.operation_lookups", 1),
        ("query.binding.graph_binding_lookups", 1),
        ("query.binding.conditional_lowering_lookups", 1),
        ("query.binding.planning_steps", 1),
        ("query.binding.authority_shape_admissions", 1),
        ("query.binding.commit_posture_classifications", 1),
        ("query.binding.executor_route_lookups", 1),
        ("query.binding.workflow_executor_route_lookups", 1),
        ("query.binding.parallel_admission_route_lookups", 1),
        ("query.support.installation_generation_checks", 1),
        ("query.support.mint_guard_checks", 1),
        ("query.support.dimensions_evaluated", 15),
        ("query.resource_admission.runtime_authority_checks", 1),
        ("query.resource_admission.input_contract_checks", 1),
        ("query.resource_admission.execution_contract_checks", 1),
        ("query.resource_admission.resource_contract_lookups", 1),
        ("query.resource_admission.support_snapshot_checks", 1),
        ("query.resource_admission.strategy_checks", 1),
        ("query.resource_admission.envelope_dimension_checks", 32),
        ("query.resource_admission.provider_session_mints", 1),
        ("query.execution.runtime_authority_checks", 1),
        ("query.execution.primary_read_contacts", 1),
        ("query.execution.executor_contacts", 1),
        ("query.execution.terminal_posture_checks", 1),
        ("query.execution.publication_checks", 1),
        ("query.execution.consumption_contacts", 1),
        ("query.dependency.semantic_contract_checks", 1),
        ("query.dependency.execution_receipt_checks", 2),
        ("query.dependency.graph_receipt_checks", 1),
        ("query.dependency.installed_definition_visits", 1),
        ("query.dependency.graph_read_role_visits", 1),
        ("query.dependency.native_projection_edges", 2),
        ("query.dependency.result_shape_edges", 1),
        ("query.dependency.replay_contract_edges", 1),
        ("query.dependency.lineage_contract_edges", 1),
        ("query.dependency.support_contract_edges", 1),
        ("query.dependency.realized_direct_output_edges", 1),
        ("query.dependency.canonical_traversal_edges", 8),
        ("query.dependency.uniqueness_hash_checks", 8),
        ("query.dependency.compiled_dependency_count", 8),
        ("query.dependency.closure_edges_traversed", 7),
        ("query.dependency.impact_index_entries", 4),
        ("query.dependency.impact_index_dependency_visits", 8),
    ]
    .into_iter()
    .collect()
}

pub fn support_ready_profile() -> worth_foundational::FoundationalProfileSet {
    worth_foundational::profiles()
        .set()
        .diagnostic_richness(worth_foundational::DiagnosticRichnessProfile::Standard)
        .support_posture(worth_foundational::SupportPostureProfile::SupportReady)
        .compatibility_posture(worth_foundational::CompatibilityPostureProfile::NativeOnly)
        .admission_readiness(worth_foundational::AdmissionReadinessProfile::Admitted)
        .retention_delivery(worth_foundational::RetentionDeliveryProfile::Retained)
        .certification_posture(worth_foundational::CertificationPostureProfile::Uncertified)
        .execution_objective(worth_foundational::ExecutionObjectiveProfile::Balanced)
        .observation_activation(worth_foundational::ObservationActivationProfile::Continuous)
        .compose()
        .unwrap()
}

pub fn operational_rows(
    snapshot: &domain::WorthQueryConsumptionCostSnapshot,
) -> BTreeMap<&str, u64> {
    snapshot
        .rows()
        .iter()
        .map(|row| (row.name(), row.observed_count()))
        .collect()
}
