use worth_foundational::performance;
use worth_foundational::{
    FoundationalAuthoritativePerformanceClaim, FoundationalPerformanceAccessPatternPosture,
    FoundationalPerformanceBoundary, FoundationalPerformanceBreadthLocalityPosture,
    FoundationalPerformanceBudgetKind, FoundationalPerformanceEvidenceStrength,
    FoundationalPerformanceExecutionTemperature, FoundationalPerformanceFallbackDebtPosture,
    FoundationalPerformanceFreshnessRetentionPosture, FoundationalPerformanceWorkClass,
};

fn require_authoritative_claim(_claim: &FoundationalAuthoritativePerformanceClaim) {}

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

    require_authoritative_claim(&receipt);
}
