use worth_foundational::{
    performance_api::common_path as performance_common, FoundationalPerformanceBoundary,
    FoundationalPerformanceEvidenceStrength, FoundationalPerformanceExecutionTemperature,
    FoundationalPerformanceFallbackDebtPosture, FoundationalPerformanceFreshnessRetentionPosture,
};

fn main() {
    let front_door = performance_common::performance();
    let _ = front_door.evaluate_primitive_legality(
        FoundationalPerformanceBoundary::AuthoritativeExecution,
        FoundationalPerformanceEvidenceStrength::CounterBackedExecutionReceipt,
        FoundationalPerformanceExecutionTemperature::HotPath,
        FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent,
        FoundationalPerformanceFallbackDebtPosture::Verified,
        &["authoritative_mutation"],
        &["replay_reconstruction"],
    );
}
