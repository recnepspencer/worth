use worth_foundational::{
    performance_api::common_path as performance_common, FoundationalPerformanceBoundary,
    FoundationalPerformanceBreadthLocalityPosture, FoundationalPerformanceEvidenceStrength,
    FoundationalPerformanceExecutionTemperature, FoundationalPerformanceFallbackDebtPosture,
    FoundationalPerformanceFreshnessRetentionPosture, FoundationalPerformanceWorkClass,
};

fn requires_replay(_: worth_foundational::FoundationalReplayMaterializationPerformanceClaim) {}

fn main() {
    let policy_claim = performance_common::performance()
        .claim()
        .policy_admission()
        .boundary(FoundationalPerformanceBoundary::MaintenancePlanning)
        .evidence_strength(FoundationalPerformanceEvidenceStrength::RuntimePolicyAdmission)
        .breadth_locality(FoundationalPerformanceBreadthLocalityPosture::BasisLocalBatch)
        .access_pattern(worth_foundational::FoundationalPerformanceAccessPatternPosture::ScanHeavy)
        .execution_temperature(FoundationalPerformanceExecutionTemperature::WarmPath)
        .freshness_retention(FoundationalPerformanceFreshnessRetentionPosture::HistoricalRetained)
        .fallback_debt(FoundationalPerformanceFallbackDebtPosture::Deferred)
        .include_work(FoundationalPerformanceWorkClass::ValidationPlanning)
        .exclude_work(FoundationalPerformanceWorkClass::ForensicParity)
        .finish()
        .unwrap();

    requires_replay(policy_claim);
}
