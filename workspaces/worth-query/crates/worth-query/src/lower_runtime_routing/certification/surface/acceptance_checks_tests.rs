use super::acceptance_checks::{
    required_concrete_seam_coverage_digest, synthetic_tail_exactness_digest,
};
use super::{
    worth_query_lower_runtime_representative_surface,
    WorthQueryLowerRuntimeRepresentativeEvidenceSource,
};
use crate::lower_runtime_routing::WorthQueryLowerRuntimeSeamKey;

#[test]
fn required_phase_six_concrete_seams_are_enforced_hostilely() {
    let host_request = worth_runtime_bridge::facade::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let surface = worth_query_lower_runtime_representative_surface(resource_request)
        .with_evidence_source_override(
            WorthQueryLowerRuntimeSeamKey::SubscriptionActivation,
            WorthQueryLowerRuntimeRepresentativeEvidenceSource::InventorySynthesized,
        );

    let panic = std::panic::catch_unwind(|| required_concrete_seam_coverage_digest(&surface))
        .expect_err("required concrete seam fallback must fail acceptance");
    let message = panic_message(panic);

    assert!(message.contains("subscription-activation"));
}

#[test]
fn synthetic_tail_exactness_rejects_unexpected_runtime_backing_drift() {
    let host_request = worth_runtime_bridge::facade::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let surface = worth_query_lower_runtime_representative_surface(resource_request)
        .with_evidence_source_override(
            WorthQueryLowerRuntimeSeamKey::ComposeRead,
            WorthQueryLowerRuntimeRepresentativeEvidenceSource::InventorySynthesized,
        );

    let panic = std::panic::catch_unwind(|| synthetic_tail_exactness_digest(&surface))
        .expect_err("synthetic tail drift must fail acceptance");
    let message = panic_message(panic);

    assert!(message.contains("synthetic surface width drifted"));
}

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<String>() {
        return message.clone();
    }
    if let Some(message) = payload.downcast_ref::<&str>() {
        return (*message).to_string();
    }
    "non-string panic payload".to_string()
}
