use worth_foundational::profiles_api::stronger_lane::readiness;

fn requires_certified_readiness(
    _: &worth_foundational::FoundationalProfileProductionTestReadyArtifact,
) {
}

fn main() {
    let report = readiness::foundational_profile_milestone3_readiness_report();
    requires_certified_readiness(&report);
}
