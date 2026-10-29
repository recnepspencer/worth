use worth_signal::facade::{
    ResourceBoundaryPerformanceEnvelope, ResourceMilestoneCPolicyCertificationFamily,
    ResourceMilestoneCPolicyScenarioEvidenceKind, ResourceMilestoneCPolicyScenarioId,
    ResourceMilestoneCPolicyScenarioRow, ResourcePolicyRestoreCompatibilityDenialClass,
    ResourceRetryDenialClass, ResourceTimeoutHeartbeatExtensionDenialClass,
};

fn forged_performance() -> ResourceBoundaryPerformanceEnvelope {
    loop {}
}

fn main() {
    let _forged = ResourceMilestoneCPolicyScenarioRow {
        id: ResourceMilestoneCPolicyScenarioId::RetryBudgetExhaustionRejected,
        evidence_kind: ResourceMilestoneCPolicyScenarioEvidenceKind::ReplayCompatibilityDenial,
        certification_family: Some(
            ResourceMilestoneCPolicyCertificationFamily::AsyncRetryBudgetAndBackoffCertification,
        ),
        policy_provenance_digest: Some(String::new()),
        retry_denial_class: Some(ResourceRetryDenialClass::RetryBudgetExhausted),
        timeout_heartbeat_denial_class: Some(
            ResourceTimeoutHeartbeatExtensionDenialClass::NonActiveRequest,
        ),
        replay_restore_denial_class: Some(
            ResourcePolicyRestoreCompatibilityDenialClass::MissingDescriptor,
        ),
        evidence_digest: String::new(),
        performance: forged_performance(),
        passed: true,
    };
}
