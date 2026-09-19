use std::collections::BTreeMap;

use worth_query::facade::domain;

use super::installed_operation_fixture::configured_runtime;
use super::operation_sharing::settle;

#[test]
fn settled_snapshot_exports_exact_foundational_rows_without_mutating_operational_truth() {
    let mut workspace = configured_runtime()
        .workspace("consumption-cost-export")
        .unwrap();
    let settled = settle(&mut workspace);
    let snapshot = settled.consumption_cost_snapshot();
    let before = operational_rows(&snapshot);
    assert_eq!(snapshot.rows().len(), 110);
    assert_eq!(
        before.len(),
        snapshot.rows().len(),
        "counter names must be unique"
    );
    assert_eq!(nonzero_rows(&snapshot), expected_nonzero_rows());
    assert_work_classes(&snapshot);

    let receipt = snapshot.materialize_foundational_receipt().unwrap();

    assert_eq!(before, operational_rows(&snapshot));
    assert_eq!(before.len(), snapshot.rows().len());
    assert_eq!(receipt.bundle().counter_specs().len(), before.len());
    assert_eq!(receipt.counter_rows().len(), before.len());
    let specs = receipt
        .bundle()
        .counter_specs()
        .iter()
        .map(|spec| (spec.name().as_str(), spec.expected_exact_count()))
        .collect::<BTreeMap<_, _>>();
    let rows = receipt
        .counter_rows()
        .iter()
        .map(|row| (row.name().as_str(), row.observed_count()))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(specs, rows);
    assert_eq!(rows, before);
    assert_eq!(
        receipt.bundle().claim().evidence_strength(),
        worth_foundational::FoundationalPerformanceEvidenceStrength::CounterBackedExecutionReceipt
    );
    assert_eq!(
        receipt.bundle().claim().access_pattern(),
        worth_foundational::FoundationalPerformanceAccessPatternPosture::TraversalLocal
    );
    assert_eq!(
        receipt.bundle().claim().included_work(),
        &[
            worth_foundational::FoundationalPerformanceWorkClass::AuthoritativeObservation,
            worth_foundational::FoundationalPerformanceWorkClass::ValidationPlanning,
        ]
    );

    let attached = worth_foundational::performance_api::lower_lane::reports::attach_counter_backed_performance_receipt(
        worth_foundational::FoundationalPerformanceAttachmentTargetKind::BoundaryReport,
        receipt,
    )
    .unwrap();
    let report = worth_foundational::performance_api::lower_lane::reports::plan_performance_report(
        worth_foundational::FoundationalPerformanceReportRequest {
            source: attached,
            profile: support_ready_profile(),
            include_layout_intent: false,
            include_contract_names: false,
            include_counter_specs: true,
            include_counter_rows: true,
            include_supporting_evidence_rows: false,
            include_budget_decisions: false,
            include_denied_work: false,
            include_widened_work: false,
        },
    )
    .materialize();
    assert_eq!(report.counter_rows().len(), before.len());
    assert_eq!(before, operational_rows(&snapshot));
}

fn assert_work_classes(snapshot: &domain::WorthQueryConsumptionCostSnapshot) {
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

fn nonzero_rows(snapshot: &domain::WorthQueryConsumptionCostSnapshot) -> BTreeMap<&str, u64> {
    operational_rows(snapshot)
        .into_iter()
        .filter(|(_, count)| *count != 0)
        .collect()
}

fn expected_nonzero_rows() -> BTreeMap<&'static str, u64> {
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

fn support_ready_profile() -> worth_foundational::FoundationalProfileSet {
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

fn operational_rows(snapshot: &domain::WorthQueryConsumptionCostSnapshot) -> BTreeMap<&str, u64> {
    snapshot
        .rows()
        .iter()
        .map(|row| (row.name(), row.observed_count()))
        .collect()
}
