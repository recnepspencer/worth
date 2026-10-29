use worth_foundational::{
    performance_api::common_path as performance_common, FoundationalPerformanceBoundary,
    FoundationalPerformanceBreadthLocalityPosture, FoundationalPerformanceEvidenceStrength,
    FoundationalPerformanceExecutionTemperature, FoundationalPerformanceFallbackDebtPosture,
    FoundationalPerformanceFreshnessRetentionPosture, FoundationalPerformanceWorkClass,
};

fn requires_authoritative(_: worth_foundational::FoundationalAuthoritativePerformanceClaim) {}

fn main() {
    let support_claim = performance_common::performance()
        .claim()
        .support_derived()
        .boundary(FoundationalPerformanceBoundary::SupportAssembly)
        .evidence_strength(FoundationalPerformanceEvidenceStrength::SupportDerivedPerformanceClaim)
        .breadth_locality(FoundationalPerformanceBreadthLocalityPosture::BranchLocal)
        .access_pattern(worth_foundational::FoundationalPerformanceAccessPatternPosture::TraversalLocal)
        .execution_temperature(FoundationalPerformanceExecutionTemperature::SupportOnly)
        .freshness_retention(FoundationalPerformanceFreshnessRetentionPosture::StaleSupport)
        .fallback_debt(FoundationalPerformanceFallbackDebtPosture::Debt)
        .include_work(FoundationalPerformanceWorkClass::SupportReportAssembly)
        .exclude_work(FoundationalPerformanceWorkClass::AuthoritativeMutation)
        .finish()
        .unwrap();

    requires_authoritative(support_claim);
}
