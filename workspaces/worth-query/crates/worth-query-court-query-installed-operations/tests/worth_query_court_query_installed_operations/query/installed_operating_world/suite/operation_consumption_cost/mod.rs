pub(crate) use worth_query_testkit_query::facade::query::suite::operation_consumption_cost::*;

#[test]
fn settled_snapshot_exports_exact_foundational_rows_without_mutating_operational_truth() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = configured_runtime()
        .workspace(request_execution, "consumption-cost-export")
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
