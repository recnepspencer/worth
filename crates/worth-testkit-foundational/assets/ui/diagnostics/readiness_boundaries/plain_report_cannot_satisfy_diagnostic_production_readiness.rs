fn require_readiness(
    _value: worth_foundational::FoundationalDiagnosticProductionTestReadyArtifact,
) {
}

fn main() {
    let report: worth_foundational::FoundationalDiagnosticProductionReadinessReport =
        worth_foundational::foundational_diagnostic_milestone6_readiness_report();
    require_readiness(report);
}
