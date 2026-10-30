#[allow(unused_imports)]
use crate::certification::ordinary_context::performance::grouped_surface::*;

#[test]
fn grouped_performance_surface_exposes_common_lower_and_stronger_lanes() {
    let common_front_door = worth_foundational::performance_api::common_path::performance();
    let claim = common_front_door
        .claim()
        .authoritative_execution()
        .boundary(worth_foundational::FoundationalPerformanceBoundary::AuthoritativeExecution)
        .evidence_strength(worth_foundational::FoundationalPerformanceEvidenceStrength::CounterBackedExecutionReceipt)
        .breadth_locality(worth_foundational::FoundationalPerformanceBreadthLocalityPosture::PointLocal)
        .access_pattern(worth_foundational::FoundationalPerformanceAccessPatternPosture::PointLookup)
        .execution_temperature(worth_foundational::FoundationalPerformanceExecutionTemperature::HotPath)
        .freshness_retention(worth_foundational::FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent)
        .fallback_debt(worth_foundational::FoundationalPerformanceFallbackDebtPosture::Verified)
        .include_work(worth_foundational::FoundationalPerformanceWorkClass::AuthoritativeMutation)
        .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::SupportReportAssembly)
        .finish()
        .expect("common-path claim should build");

    let layout = common_front_door.define_layout_intent(
        worth_foundational::FoundationalPerformanceLayoutIntent::AoS,
        worth_foundational::FoundationalPerformanceAccessPatternPosture::PointLookup,
        worth_foundational::FoundationalPerformanceAllocationPosture::ActionLocal,
    );

    let bundle = worth_foundational::performance_api::lower_lane::basis::performance_bundle(claim)
        .attach_layout_intent_claim(layout)
        .attach_counter_spec(worth_foundational::FoundationalPerformanceCounterSpec::new(
            worth_foundational::FoundationalPerformanceCounterName::new("authoritative.rows")
                .expect("valid counter name"),
            worth_foundational::FoundationalPerformanceWorkClass::AuthoritativeMutation,
            3,
        ))
        .finish()
        .expect("lower-lane bundle should build");
    let receipt = worth_foundational::performance_api::lower_lane::receipts::counter_backed_performance_receipt(bundle)
        .attach_counter_row(worth_foundational::FoundationalPerformanceCounterRow::new(
            worth_foundational::FoundationalPerformanceCounterName::new("authoritative.rows")
                .expect("valid counter name"),
            3,
        ))
        .finish()
        .expect("counter-backed receipt should build");
    let attached = worth_foundational::performance_api::lower_lane::reports::attach_counter_backed_performance_receipt(
        worth_foundational::FoundationalPerformanceAttachmentTargetKind::BoundaryReport,
        receipt,
    )
    .expect("receipt attachment should build");
    let plan = worth_foundational::performance_api::lower_lane::reports::plan_performance_report(
        worth_foundational::FoundationalPerformanceReportRequest {
            source: attached,
            profile: standard_profile(),
            include_layout_intent: true,
            include_contract_names: false,
            include_counter_specs: true,
            include_counter_rows: true,
            include_supporting_evidence_rows: false,
            include_budget_decisions: false,
            include_denied_work: false,
            include_widened_work: false,
        },
    );
    let report = plan.materialize();
    let readiness = worth_foundational::performance_api::stronger_lane::readiness::foundational_performance_milestone8_readiness_report();
    let readiness_certified =
        worth_foundational::performance_api::stronger_lane::readiness::certify_foundational_performance_milestone8_production_test_readiness();

    assert_eq!(report.counter_rows()[0].observed_count(), 3);
    assert!(report.layout_intent_claim().is_some());
    assert!(readiness.passes_readiness_checklist());
    assert!(std::ptr::eq(
        worth_foundational::performance_api::stronger_lane::readiness::require_foundational_performance_milestone8_production_test_readiness(
            &readiness_certified
        ),
        readiness_certified.payload()
    ));
    let _ = worth_foundational::performance_api::stronger_lane::certified::foundational_performance_certified_attachment_authority();
}
