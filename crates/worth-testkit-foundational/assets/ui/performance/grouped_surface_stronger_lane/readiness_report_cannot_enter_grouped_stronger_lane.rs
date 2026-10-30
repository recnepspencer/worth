use worth_foundational::performance_api::stronger_lane::readiness;

fn requires_certified_readiness(
    _: &worth_foundational::FoundationalPerformanceProductionTestReadyArtifact,
) {
}

fn main() {
    let report = readiness::foundational_performance_milestone8_readiness_report();
    requires_certified_readiness(&report);
}
