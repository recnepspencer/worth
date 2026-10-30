use worth_signal::facade::{
    CompletionDenialClass, ResourceBoundaryPerformanceEnvelope, ResourceCertificationFamily,
    ResourceMilestoneBScenarioEvidenceKind, ResourceMilestoneBScenarioId,
    ResourceMilestoneBScenarioRow,
};

fn forged_performance() -> ResourceBoundaryPerformanceEnvelope {
    loop {}
}

fn main() {
    let _forged = ResourceMilestoneBScenarioRow {
        id: ResourceMilestoneBScenarioId::LifecycleReplayParity,
        evidence_kind: ResourceMilestoneBScenarioEvidenceKind::CertificationFamily,
        certification_family: Some(ResourceCertificationFamily::AsyncResourceLifecycleParity),
        completion_denial_class: Some(CompletionDenialClass::Malformed),
        evidence_digest: String::new(),
        performance: forged_performance(),
        passed: true,
    };
}
