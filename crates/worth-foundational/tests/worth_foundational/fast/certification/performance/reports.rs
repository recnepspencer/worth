use worth_foundational::{
    performance, performance_api, profiles, CertificationPostureProfile, DiagnosticRichnessProfile,
    FoundationalPerformanceAccessPatternPosture, FoundationalPerformanceAttachmentDenial,
    FoundationalPerformanceAttachmentTargetKind, FoundationalPerformanceBoundary,
    FoundationalPerformanceBreadthLocalityPosture, FoundationalPerformanceCounterRow,
    FoundationalPerformanceEvidenceStrength, FoundationalPerformanceExecutionTemperature,
    FoundationalPerformanceFallbackDebtPosture, FoundationalPerformanceFreshnessRetentionPosture,
    FoundationalPerformanceLayoutIntent, FoundationalPerformanceReportMaterializationBoundary,
    FoundationalPerformanceReportSection, FoundationalPerformanceReportSectionDecisionCause,
    FoundationalPerformanceWorkClass, RetentionDeliveryProfile, SupportPostureProfile,
};

fn standard_profile() -> worth_foundational::FoundationalProfileSet {
    profiles()
        .set()
        .diagnostic_richness(DiagnosticRichnessProfile::Standard)
        .support_posture(SupportPostureProfile::SupportReady)
        .compatibility_posture(worth_foundational::CompatibilityPostureProfile::NativeOnly)
        .admission_readiness(worth_foundational::AdmissionReadinessProfile::Admitted)
        .retention_delivery(RetentionDeliveryProfile::Retained)
        .certification_posture(CertificationPostureProfile::Uncertified)
        .execution_objective(worth_foundational::ExecutionObjectiveProfile::Balanced)
        .observation_activation(worth_foundational::ObservationActivationProfile::Continuous)
        .compose()
        .expect("standard profile should compose")
}

fn operational_minimal_profile() -> worth_foundational::FoundationalProfileSet {
    profiles()
        .set()
        .diagnostic_richness(DiagnosticRichnessProfile::OperationalMinimal)
        .support_posture(SupportPostureProfile::InternalOnly)
        .compatibility_posture(worth_foundational::CompatibilityPostureProfile::NativeOnly)
        .admission_readiness(worth_foundational::AdmissionReadinessProfile::Admitted)
        .retention_delivery(RetentionDeliveryProfile::Retained)
        .certification_posture(CertificationPostureProfile::Uncertified)
        .execution_objective(worth_foundational::ExecutionObjectiveProfile::Throughput)
        .observation_activation(worth_foundational::ObservationActivationProfile::OnDemand)
        .compose()
        .expect("minimal profile should compose")
}

fn authoritative_bundle() -> worth_foundational::FoundationalPerformanceBundle<
    worth_foundational::FoundationalAuthoritativePerformanceClaim,
> {
    let claim = performance()
        .claim()
        .authoritative_execution()
        .boundary(FoundationalPerformanceBoundary::AuthoritativeExecution)
        .evidence_strength(FoundationalPerformanceEvidenceStrength::CounterBackedExecutionReceipt)
        .breadth_locality(FoundationalPerformanceBreadthLocalityPosture::PointLocal)
        .access_pattern(FoundationalPerformanceAccessPatternPosture::PointLookup)
        .execution_temperature(FoundationalPerformanceExecutionTemperature::HotPath)
        .freshness_retention(FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent)
        .fallback_debt(FoundationalPerformanceFallbackDebtPosture::Verified)
        .include_work(FoundationalPerformanceWorkClass::AuthoritativeMutation)
        .exclude_work(FoundationalPerformanceWorkClass::SupportReportAssembly)
        .finish()
        .expect("claim should build");

    let layout = performance().define_layout_intent(
        FoundationalPerformanceLayoutIntent::AoS,
        FoundationalPerformanceAccessPatternPosture::PointLookup,
        worth_foundational::FoundationalPerformanceAllocationPosture::ActionLocal,
    );
    let contract_name =
        worth_foundational::FoundationalPerformanceContractName::new("query.snapshot.targets")
            .expect("valid contract");
    let counter_name =
        worth_foundational::FoundationalPerformanceCounterName::new("authoritative.rows")
            .expect("valid counter");
    let support_code = worth_foundational::FoundationalPerformanceSupportingEvidenceCode::new(
        "support.snapshot.audit",
    )
    .expect("valid support code");

    performance_api::lower_lane::basis::performance_bundle(claim)
        .attach_layout_intent_claim(layout)
        .attach_contract_name(contract_name)
        .attach_counter_spec(worth_foundational::FoundationalPerformanceCounterSpec::new(
            counter_name,
            FoundationalPerformanceWorkClass::AuthoritativeMutation,
            3,
        ))
        .attach_supporting_evidence_row(
            worth_foundational::FoundationalPerformanceSupportingEvidenceRow::new(
                support_code,
                FoundationalPerformanceWorkClass::SupportReportAssembly,
            ),
        )
        .finish()
        .expect("bundle should build")
}

fn policy_receipt() -> worth_foundational::FoundationalPolicyAdmissionReceipt {
    let claim = performance()
        .claim()
        .policy_admission()
        .boundary(FoundationalPerformanceBoundary::AuthoritativeExecution)
        .evidence_strength(FoundationalPerformanceEvidenceStrength::RuntimePolicyAdmission)
        .breadth_locality(FoundationalPerformanceBreadthLocalityPosture::DeltaBound)
        .access_pattern(FoundationalPerformanceAccessPatternPosture::PointLookup)
        .execution_temperature(FoundationalPerformanceExecutionTemperature::HotPath)
        .freshness_retention(FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent)
        .fallback_debt(FoundationalPerformanceFallbackDebtPosture::Verified)
        .include_work(FoundationalPerformanceWorkClass::ValidationPlanning)
        .exclude_work(FoundationalPerformanceWorkClass::SupportReportAssembly)
        .finish()
        .expect("policy claim should build");

    performance()
        .policy_admission_receipt(claim)
        .budget_decision(
            worth_foundational::FoundationalPerformanceBudgetKind::Breadth,
            4,
            4,
        )
        .finish()
        .expect("policy receipt should build")
}

#[test]
fn phase6_attachment_targets_fail_closed_by_source_strength() {
    let bundle = authoritative_bundle();
    let denial = performance_api::lower_lane::reports::attach_performance_bundle(
        FoundationalPerformanceAttachmentTargetKind::BoundaryReceipt,
        bundle.clone(),
    );
    assert_eq!(
        denial,
        Err(FoundationalPerformanceAttachmentDenial::ClaimBundlesCannotAttachToBoundaryReceipts)
    );

    let policy = policy_receipt();
    let denial = performance_api::lower_lane::reports::attach_policy_admission_receipt(
        FoundationalPerformanceAttachmentTargetKind::BoundarySummary,
        policy,
    );
    assert_eq!(
        denial,
        Err(FoundationalPerformanceAttachmentDenial::PolicyReceiptsCannotAttachToBoundarySummaries)
    );

    let receipt = performance_api::lower_lane::receipts::counter_backed_performance_receipt(bundle)
        .attach_counter_row(FoundationalPerformanceCounterRow::new(
            worth_foundational::FoundationalPerformanceCounterName::new("authoritative.rows")
                .expect("valid counter"),
            3,
        ))
        .finish()
        .expect("counter-backed receipt should build");

    let denial = performance_api::lower_lane::reports::attach_counter_backed_performance_receipt(
        FoundationalPerformanceAttachmentTargetKind::CertificationBundle,
        receipt,
    );
    assert_eq!(
        denial,
        Err(
            FoundationalPerformanceAttachmentDenial::CounterBackedReceiptsCannotAttachToCertificationBundles
        )
    );
}

#[test]
fn report_planning_elides_optional_support_rows_under_reduced_richness() {
    let attached = performance_api::lower_lane::reports::attach_performance_bundle(
        FoundationalPerformanceAttachmentTargetKind::BoundaryArtifact,
        authoritative_bundle(),
    )
    .expect("bundle attachment should build");

    let plan = performance_api::lower_lane::reports::plan_performance_report(
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
        FoundationalPerformanceReportMaterializationBoundary::ReportAssembly
    );
    assert!(plan
        .included_sections()
        .contains(&FoundationalPerformanceReportSection::LayoutIntent));
    assert!(plan.excluded_sections().iter().any(|decision| {
        decision.section() == FoundationalPerformanceReportSection::SupportingEvidenceRows
            && decision.cause() == FoundationalPerformanceReportSectionDecisionCause::ProfileElided
    }));
}

#[test]
fn explicit_materialization_carries_requested_counter_and_support_rows() {
    let receipt = performance_api::lower_lane::receipts::counter_backed_performance_receipt(
        authoritative_bundle(),
    )
    .attach_counter_row(FoundationalPerformanceCounterRow::new(
        worth_foundational::FoundationalPerformanceCounterName::new("authoritative.rows")
            .expect("valid counter"),
        3,
    ))
    .finish()
    .expect("counter-backed receipt should build");
    let attached = performance_api::lower_lane::reports::attach_counter_backed_performance_receipt(
        FoundationalPerformanceAttachmentTargetKind::BoundaryReport,
        receipt,
    )
    .expect("counter-backed receipt attachment should build");

    let plan = performance_api::lower_lane::reports::plan_performance_report(
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
        FoundationalPerformanceReportMaterializationBoundary::SupportExpansion
    );

    let report = plan.materialize();
    assert_eq!(
        report.target(),
        FoundationalPerformanceAttachmentTargetKind::BoundaryReport
    );
    assert_eq!(report.counter_rows()[0].observed_count(), 3);
    assert_eq!(report.counter_specs().len(), 1);
    assert_eq!(report.supporting_evidence_rows().len(), 1);
    assert!(report.layout_intent_claim().is_some());
}

#[test]
fn claim_inspection_only_reports_do_not_leak_optional_sections() {
    let attached = performance_api::lower_lane::reports::attach_performance_bundle(
        FoundationalPerformanceAttachmentTargetKind::BoundaryArtifact,
        authoritative_bundle(),
    )
    .expect("bundle attachment should build");

    let plan = performance_api::lower_lane::reports::plan_performance_report(
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
        FoundationalPerformanceReportMaterializationBoundary::ClaimInspectionOnly
    );
    assert_eq!(
        plan.included_sections(),
        vec![FoundationalPerformanceReportSection::Claim]
    );

    let report = plan.materialize();
    assert_eq!(
        report.materialization_boundary(),
        FoundationalPerformanceReportMaterializationBoundary::ClaimInspectionOnly
    );
    assert!(report.layout_intent_claim().is_none());
    assert!(report.contract_names().is_empty());
    assert!(report.counter_specs().is_empty());
    assert!(report.counter_rows().is_empty());
    assert!(report.supporting_evidence_rows().is_empty());
}

#[test]
fn policy_receipt_reports_keep_unavailable_sections_explicit_and_budget_sections_materialized() {
    let attached = performance_api::lower_lane::reports::attach_policy_admission_receipt(
        FoundationalPerformanceAttachmentTargetKind::BoundaryReport,
        policy_receipt(),
    )
    .expect("policy receipt attachment should build");

    let plan = performance_api::lower_lane::reports::plan_performance_report(
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
        FoundationalPerformanceReportMaterializationBoundary::ReportAssembly
    );
    assert!(plan.excluded_sections().iter().any(|decision| {
        decision.section() == FoundationalPerformanceReportSection::LayoutIntent
            && decision.cause()
                == FoundationalPerformanceReportSectionDecisionCause::UnavailableFromSource
    }));
    assert!(plan.excluded_sections().iter().any(|decision| {
        decision.section() == FoundationalPerformanceReportSection::CounterRows
            && decision.cause()
                == FoundationalPerformanceReportSectionDecisionCause::UnavailableFromSource
    }));
    assert!(plan
        .included_sections()
        .contains(&FoundationalPerformanceReportSection::BudgetDecisions));

    let report = plan.materialize();
    assert!(report.layout_intent_claim().is_none());
    assert!(report.contract_names().is_empty());
    assert!(report.counter_specs().is_empty());
    assert!(report.counter_rows().is_empty());
    assert_eq!(report.budget_decisions().len(), 1);
    assert!(report.denied_work().is_empty());
    assert!(report.widened_work().is_empty());
}
