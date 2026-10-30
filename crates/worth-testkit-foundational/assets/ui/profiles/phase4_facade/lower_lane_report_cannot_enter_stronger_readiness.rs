use worth_foundational::profiles_api::lower_lane::materialization;

fn requires_stronger_readiness(
    _: &worth_foundational::FoundationalProfileProductionTestReadyArtifact,
) {}

fn main() {
    let _ = materialization::FoundationalObservationDisposition::Inactive;
    let report = worth_foundational::foundational_profile_milestone10_readiness_report();
    requires_stronger_readiness(&report);
}
