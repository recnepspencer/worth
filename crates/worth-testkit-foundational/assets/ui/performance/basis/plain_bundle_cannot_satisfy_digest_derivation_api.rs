use worth_foundational::canonicalization;
use worth_foundational::performance;
use worth_foundational::{
    CanonicalDigestAlgorithmId, FoundationalPerformanceAccessPatternPosture,
    FoundationalPerformanceBoundary, FoundationalPerformanceBreadthLocalityPosture,
    FoundationalPerformanceEvidenceStrength, FoundationalPerformanceExecutionTemperature,
    FoundationalPerformanceFallbackDebtPosture, FoundationalPerformanceFreshnessRetentionPosture,
    FoundationalPerformanceWorkClass,
};

fn main() {
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
        .unwrap();
    let bundle = worth_foundational::performance_api::lower_lane::basis::performance_bundle(claim)
        .finish()
        .unwrap();

    let _ = canonicalization()
        .digest()
        .for_sequence(bundle, CanonicalDigestAlgorithmId::sha256());
}
