use worth_foundational::{
    FoundationalAuthoritativePerformanceClaim, FoundationalCounterBackedPerformanceReceipt,
    FoundationalPerformanceReportPlan,
};

fn require_counter_backed_receipt(
    _receipt: &FoundationalCounterBackedPerformanceReceipt<FoundationalAuthoritativePerformanceClaim>,
) {
}

fn main() {
    let claim = worth_foundational::performance()
        .claim()
        .authoritative_execution()
        .boundary(worth_foundational::FoundationalPerformanceBoundary::AuthoritativeExecution)
        .evidence_strength(
            worth_foundational::FoundationalPerformanceEvidenceStrength::CounterBackedExecutionReceipt,
        )
        .breadth_locality(worth_foundational::FoundationalPerformanceBreadthLocalityPosture::PointLocal)
        .access_pattern(worth_foundational::FoundationalPerformanceAccessPatternPosture::PointLookup)
        .execution_temperature(worth_foundational::FoundationalPerformanceExecutionTemperature::HotPath)
        .freshness_retention(
            worth_foundational::FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent,
        )
        .fallback_debt(worth_foundational::FoundationalPerformanceFallbackDebtPosture::Verified)
        .include_work(worth_foundational::FoundationalPerformanceWorkClass::AuthoritativeMutation)
        .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::SupportReportAssembly)
        .finish()
        .unwrap();
    let bundle = worth_foundational::performance_bundle(claim).finish().unwrap();
    let attached = worth_foundational::attach_performance_bundle(
        worth_foundational::FoundationalPerformanceAttachmentTargetKind::BoundaryArtifact,
        bundle,
    )
    .unwrap();
    let plan: FoundationalPerformanceReportPlan<_> =
        worth_foundational::plan_performance_report(worth_foundational::FoundationalPerformanceReportRequest {
            source: attached,
            profile: worth_foundational::profiles()
                .set()
                .diagnostic_richness(worth_foundational::DiagnosticRichnessProfile::Standard)
                .support_posture(worth_foundational::SupportPostureProfile::SupportReady)
                .compatibility_posture(worth_foundational::CompatibilityPostureProfile::NativeOnly)
                .admission_readiness(worth_foundational::AdmissionReadinessProfile::Admitted)
                .retention_delivery(worth_foundational::RetentionDeliveryProfile::Retained)
                .certification_posture(worth_foundational::CertificationPostureProfile::Uncertified)
                .compose()
                .unwrap(),
            include_layout_intent: false,
            include_contract_names: false,
            include_counter_specs: false,
            include_counter_rows: false,
            include_supporting_evidence_rows: false,
            include_budget_decisions: false,
            include_denied_work: false,
            include_widened_work: false,
        });

    require_counter_backed_receipt(&plan);
}
