#[allow(unused_imports)]
use crate::certification::ordinary_context::performance::reports::*;
use {
    worth_foundational::performance, worth_foundational::performance_api,
    worth_foundational::profiles, worth_foundational::CertificationPostureProfile,
    worth_foundational::DiagnosticRichnessProfile,
    worth_foundational::FoundationalPerformanceAccessPatternPosture,
    worth_foundational::FoundationalPerformanceAttachmentDenial,
    worth_foundational::FoundationalPerformanceAttachmentTargetKind,
    worth_foundational::FoundationalPerformanceBoundary,
    worth_foundational::FoundationalPerformanceBreadthLocalityPosture,
    worth_foundational::FoundationalPerformanceCounterRow,
    worth_foundational::FoundationalPerformanceEvidenceStrength,
    worth_foundational::FoundationalPerformanceExecutionTemperature,
    worth_foundational::FoundationalPerformanceFallbackDebtPosture,
    worth_foundational::FoundationalPerformanceFreshnessRetentionPosture,
    worth_foundational::FoundationalPerformanceLayoutIntent,
    worth_foundational::FoundationalPerformanceReportMaterializationBoundary,
    worth_foundational::FoundationalPerformanceReportSection,
    worth_foundational::FoundationalPerformanceReportSectionDecisionCause,
    worth_foundational::FoundationalPerformanceWorkClass,
    worth_foundational::RetentionDeliveryProfile, worth_foundational::SupportPostureProfile,
};

#[test]
fn attachment_targets_fail_closed_by_source_strength() {
    let bundle = authoritative_bundle();
    let denial =
        worth_foundational::performance_api::lower_lane::reports::attach_performance_bundle(
            worth_foundational::FoundationalPerformanceAttachmentTargetKind::BoundaryReceipt,
            bundle.clone(),
        );
    assert_eq!(
        denial,
        Err(worth_foundational::FoundationalPerformanceAttachmentDenial::ClaimBundlesCannotAttachToBoundaryReceipts)
    );

    let policy = policy_receipt();
    let denial =
        worth_foundational::performance_api::lower_lane::reports::attach_policy_admission_receipt(
            worth_foundational::FoundationalPerformanceAttachmentTargetKind::BoundarySummary,
            policy,
        );
    assert_eq!(
        denial,
        Err(worth_foundational::FoundationalPerformanceAttachmentDenial::PolicyReceiptsCannotAttachToBoundarySummaries)
    );

    let receipt = worth_foundational::performance_api::lower_lane::receipts::counter_backed_performance_receipt(bundle)
        .attach_counter_row(worth_foundational::FoundationalPerformanceCounterRow::new(
            worth_foundational::FoundationalPerformanceCounterName::new("authoritative.rows")
                .expect("valid counter"),
            3,
        ))
        .finish()
        .expect("counter-backed receipt should build");

    let denial = worth_foundational::performance_api::lower_lane::reports::attach_counter_backed_performance_receipt(
        worth_foundational::FoundationalPerformanceAttachmentTargetKind::CertificationBundle,
        receipt,
    );
    assert_eq!(
        denial,
        Err(
            worth_foundational::FoundationalPerformanceAttachmentDenial::CounterBackedReceiptsCannotAttachToCertificationBundles
        )
    );
}

#[test]
fn report_planning_elides_optional_support_rows_under_reduced_richness() {
    let attached =
        worth_foundational::performance_api::lower_lane::reports::attach_performance_bundle(
            worth_foundational::FoundationalPerformanceAttachmentTargetKind::BoundaryArtifact,
            authoritative_bundle(),
        )
        .expect("bundle attachment should build");

    let plan = worth_foundational::performance_api::lower_lane::reports::plan_performance_report(
        worth_foundational::FoundationalPerformanceReportRequest {
            source: attached,
            profile: operational_minimal_profile(),
            include_layout_intent: true,
            include_contract_names: true,
            include_counter_specs: false,
            include_counter_rows: false,
            include_supporting_evidence_rows: true,
            include_budget_decisions: false,
            include_denied_work: false,
            include_widened_work: false,
        },
    );

    assert_eq!(
        plan.materialization_boundary(),
        worth_foundational::FoundationalPerformanceReportMaterializationBoundary::ReportAssembly
    );
    assert!(plan
        .included_sections()
        .contains(&worth_foundational::FoundationalPerformanceReportSection::LayoutIntent));
    assert!(plan.excluded_sections().iter().any(|decision| {
        decision.section() == worth_foundational::FoundationalPerformanceReportSection::SupportingEvidenceRows
            && decision.cause() == worth_foundational::FoundationalPerformanceReportSectionDecisionCause::ProfileElided
    }));
}

#[test]
fn explicit_materialization_carries_requested_counter_and_support_rows() {
    let receipt = worth_foundational::performance_api::lower_lane::receipts::counter_backed_performance_receipt(
        authoritative_bundle(),
    )
    .attach_counter_row(worth_foundational::FoundationalPerformanceCounterRow::new(
        worth_foundational::FoundationalPerformanceCounterName::new("authoritative.rows")
            .expect("valid counter"),
        3,
    ))
    .finish()
    .expect("counter-backed receipt should build");
    let attached = worth_foundational::performance_api::lower_lane::reports::attach_counter_backed_performance_receipt(
        worth_foundational::FoundationalPerformanceAttachmentTargetKind::BoundaryReport,
        receipt,
    )
    .expect("counter-backed receipt attachment should build");

    let plan = worth_foundational::performance_api::lower_lane::reports::plan_performance_report(
        worth_foundational::FoundationalPerformanceReportRequest {
            source: attached,
            profile: standard_profile(),
            include_layout_intent: true,
            include_contract_names: false,
            include_counter_specs: true,
            include_counter_rows: true,
            include_supporting_evidence_rows: true,
            include_budget_decisions: false,
            include_denied_work: false,
            include_widened_work: false,
        },
    );
    assert_eq!(
        plan.materialization_boundary(),
        worth_foundational::FoundationalPerformanceReportMaterializationBoundary::SupportExpansion
    );

    let report = plan.materialize();
    assert_eq!(
        report.target(),
        worth_foundational::FoundationalPerformanceAttachmentTargetKind::BoundaryReport
    );
    assert_eq!(report.counter_rows()[0].observed_count(), 3);
    assert_eq!(report.counter_specs().len(), 1);
    assert_eq!(report.supporting_evidence_rows().len(), 1);
    assert!(report.layout_intent_claim().is_some());
}

#[test]
fn claim_inspection_only_reports_do_not_leak_optional_sections() {
    let attached =
        worth_foundational::performance_api::lower_lane::reports::attach_performance_bundle(
            worth_foundational::FoundationalPerformanceAttachmentTargetKind::BoundaryArtifact,
            authoritative_bundle(),
        )
        .expect("bundle attachment should build");

    let plan = worth_foundational::performance_api::lower_lane::reports::plan_performance_report(
        worth_foundational::FoundationalPerformanceReportRequest {
            source: attached,
            profile: standard_profile(),
            include_layout_intent: false,
            include_contract_names: false,
            include_counter_specs: false,
            include_counter_rows: false,
            include_supporting_evidence_rows: false,
            include_budget_decisions: false,
            include_denied_work: false,
            include_widened_work: false,
        },
    );

    assert_eq!(
        plan.materialization_boundary(),
        worth_foundational::FoundationalPerformanceReportMaterializationBoundary::ClaimInspectionOnly
    );
    assert_eq!(
        plan.included_sections(),
        vec![worth_foundational::FoundationalPerformanceReportSection::Claim]
    );

    let report = plan.materialize();
    assert_eq!(
        report.materialization_boundary(),
        worth_foundational::FoundationalPerformanceReportMaterializationBoundary::ClaimInspectionOnly
    );
    assert!(report.layout_intent_claim().is_none());
    assert!(report.contract_names().is_empty());
    assert!(report.counter_specs().is_empty());
    assert!(report.counter_rows().is_empty());
    assert!(report.supporting_evidence_rows().is_empty());
}

#[test]
fn policy_receipt_reports_keep_unavailable_sections_explicit_and_budget_sections_materialized() {
    let attached =
        worth_foundational::performance_api::lower_lane::reports::attach_policy_admission_receipt(
            worth_foundational::FoundationalPerformanceAttachmentTargetKind::BoundaryReport,
            policy_receipt(),
        )
        .expect("policy receipt attachment should build");

    let plan = worth_foundational::performance_api::lower_lane::reports::plan_performance_report(
        worth_foundational::FoundationalPerformanceReportRequest {
            source: attached,
            profile: standard_profile(),
            include_layout_intent: true,
            include_contract_names: true,
            include_counter_specs: true,
            include_counter_rows: true,
            include_supporting_evidence_rows: true,
            include_budget_decisions: true,
            include_denied_work: true,
            include_widened_work: true,
        },
    );

    assert_eq!(
        plan.materialization_boundary(),
        worth_foundational::FoundationalPerformanceReportMaterializationBoundary::ReportAssembly
    );
    assert!(plan.excluded_sections().iter().any(|decision| {
        decision.section() == worth_foundational::FoundationalPerformanceReportSection::LayoutIntent
            && decision.cause()
                == worth_foundational::FoundationalPerformanceReportSectionDecisionCause::UnavailableFromSource
    }));
    assert!(plan.excluded_sections().iter().any(|decision| {
        decision.section() == worth_foundational::FoundationalPerformanceReportSection::CounterRows
            && decision.cause()
                == worth_foundational::FoundationalPerformanceReportSectionDecisionCause::UnavailableFromSource
    }));
    assert!(plan
        .included_sections()
        .contains(&worth_foundational::FoundationalPerformanceReportSection::BudgetDecisions));

    let report = plan.materialize();
    assert!(report.layout_intent_claim().is_none());
    assert!(report.contract_names().is_empty());
    assert!(report.counter_specs().is_empty());
    assert!(report.counter_rows().is_empty());
    assert_eq!(report.budget_decisions().len(), 1);
    assert!(report.denied_work().is_empty());
    assert!(report.widened_work().is_empty());
}
