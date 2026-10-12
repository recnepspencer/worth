//! truth view helpers keep current and historical meaning distinct.
use super::*;

#[test]
fn truth_view_helpers_keep_current_and_historical_meaning_distinct() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "main");
    let current_helper = handle
        .geometry_helpers()
        .prepare_current_truth_view_for_active_face_selection_checked(progressed(
            &handle, "face-b",
        ));
    let historical_helper = handle
        .geometry_helpers()
        .prepare_historical_truth_view_for_active_face_selection_checked(progressed(
            &handle, "face-c",
        ));
    let current_generic = handle.orchestrate_signal_compatibility_checked(
        generic_signal_orchestration_input(&handle, progressed(&handle, "face-b"))
            .with_bridge_request(
                crate::application::WorthQueryDeclarationBridgeContinuationRequest::new(
                    crate::application::WorthQueryDeclarationBridgeContinuationMode::TruthView,
                    crate::application::WorthQueryDeclarationBridgeTruthContext::Current,
                ),
            ),
    );
    let historical_generic = handle.orchestrate_signal_compatibility_checked(
        generic_signal_orchestration_input(&handle, progressed(&handle, "face-c"))
            .with_bridge_request(
                crate::application::WorthQueryDeclarationBridgeContinuationRequest::new(
                    crate::application::WorthQueryDeclarationBridgeContinuationMode::TruthView,
                    crate::application::WorthQueryDeclarationBridgeTruthContext::Historical,
                ),
            ),
    );

    assert_eq!(
        current_helper.orchestration_digest(),
        current_generic.orchestration_digest()
    );
    assert_eq!(
        historical_helper.orchestration_digest(),
        historical_generic.orchestration_digest()
    );
    assert_ne!(
        current_helper.orchestration_digest(),
        historical_helper.orchestration_digest()
    );
    if let WorthQuerySignalCompatibilityOrchestrationOutcome::Bound(
        WorthQuerySignalCompatibilityOrchestration::Prepared(prepared),
    ) = current_helper.outcome()
    {
        assert_eq!(
            prepared.signal_execution_family(),
            Some(WorthQueryDeclarationSignalExecutionFamily::RuntimeDerivedExecution)
        );
    }
}
