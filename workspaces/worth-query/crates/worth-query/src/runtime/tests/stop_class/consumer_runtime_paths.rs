use super::super::support::*;
use super::completeness_support::runtime_paths::{
    intent_commit_denied_error, intent_execution_routing_failed_error,
    preview_promotion_atomic_batch_unsupported_error, preview_promotion_rebinding_required_error,
    preview_promotion_stale_basis_error, preview_promotion_write_failed_error,
};
use super::consumer_support::routing::{route_consumer_stop_class, ConsumerStopRoute};
use super::consumer_support::runtime_errors::temporal_public_family_admission_error;

#[test]
fn consumer_router_handles_runtime_generated_stop_classes_without_string_matching() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let public_family_error = temporal_public_family_admission_error(
        request_execution,
        "consumer-stop-class-public-family",
        "runtime-backed temporal family stays support-gated here",
    );
    assert_eq!(
        route_consumer_stop_class(&public_family_error),
        ConsumerStopRoute::FamilyAdmissionDenied {
            family: WorthQueryRuntimeFacadeFamily::Temporal,
            status: WorthQueryRuntimeFamilySupportStatus::Supported,
            teaching_posture: Some(WorthQueryRuntimeFamilyTeachingPosture::SupportGateOnly),
        }
    );

    let intent_error = intent_commit_denied_error(request_execution);
    assert_eq!(
        route_consumer_stop_class(&intent_error),
        ConsumerStopRoute::IntentCommitDenied
    );

    let preview_cases = [
        (
            preview_promotion_stale_basis_error(request_execution),
            WorthQueryPreviewPromotionDenialKind::StaleBasis,
        ),
        (
            preview_promotion_atomic_batch_unsupported_error(request_execution),
            WorthQueryPreviewPromotionDenialKind::AtomicBatchUnsupported,
        ),
        (
            preview_promotion_rebinding_required_error(request_execution),
            WorthQueryPreviewPromotionDenialKind::RebindingRequired,
        ),
        (
            preview_promotion_write_failed_error(request_execution),
            WorthQueryPreviewPromotionDenialKind::WriteFailed,
        ),
    ];
    for (preview_error, expected_kind) in preview_cases {
        assert_eq!(
            route_consumer_stop_class(&preview_error),
            ConsumerStopRoute::PreviewPromotionDenied(expected_kind)
        );
    }

    let routing_error = intent_execution_routing_failed_error(request_execution);
    assert_eq!(
        route_consumer_stop_class(&routing_error),
        ConsumerStopRoute::IntentExecutionRoutingFailed(
            WorthQueryRuntimeDeclarationFailureKind::LiveSubscriptionInstallation,
        )
    );
}
