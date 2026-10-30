use worth_foundational::performance;
use worth_foundational::{
    FoundationalCounterBackedPerformanceReceipt, FoundationalPerformanceAccessPatternPosture,
    FoundationalPerformanceBoundary, FoundationalPerformanceBreadthLocalityPosture,
    FoundationalPerformanceBudgetKind, FoundationalPerformanceEvidenceStrength,
    FoundationalPerformanceExecutionTemperature, FoundationalPerformanceFallbackDebtPosture,
    FoundationalPerformanceFreshnessRetentionPosture, FoundationalPerformanceWorkClass,
};

fn require_counter_backed_receipt(
    _receipt: &FoundationalCounterBackedPerformanceReceipt<
        worth_foundational::FoundationalAuthoritativePerformanceClaim,
    >,
) {
}

fn main() {
    let claim = performance()
        .claim()
        .policy_admission()
        .boundary(FoundationalPerformanceBoundary::AuthoritativeExecution)
        .evidence_strength(FoundationalPerformanceEvidenceStrength::RuntimePolicyAdmission)
        .breadth_locality(FoundationalPerformanceBreadthLocalityPosture::PointLocal)
        .access_pattern(FoundationalPerformanceAccessPatternPosture::PointLookup)
        .execution_temperature(FoundationalPerformanceExecutionTemperature::HotPath)
        .freshness_retention(FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent)
        .fallback_debt(FoundationalPerformanceFallbackDebtPosture::Verified)
        .include_work(FoundationalPerformanceWorkClass::ValidationPlanning)
        .exclude_work(FoundationalPerformanceWorkClass::SupportReportAssembly)
        .finish()
        .unwrap();

    let receipt = performance()
        .policy_admission_receipt(claim)
        .budget_decision(FoundationalPerformanceBudgetKind::Breadth, 1, 1)
        .finish()
        .unwrap();

    require_counter_backed_receipt(&receipt);
}
