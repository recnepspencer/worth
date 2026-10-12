use crate::application::WorthQueryDeclarationFamilyMarker;
use crate::future_signal_test_support::{
    checked_signal_public_posture, checked_signal_supported_runtime_test_posture,
    future_signal_admitted_handle, future_signal_bridge_request, AsyncFutureSignalFamily,
    FutureSignalInput, TemporalFutureSignalFamily,
};
use crate::signal_compatibility_orchestration::WorthQuerySignalCompatibilityOrchestrationOutcome;

use super::super::transcript::orchestrated_outcome_from_signal_checked_on_handle;

#[test]
fn future_checked_signal_compatibility_stays_future_typed_when_orchestration_stops_at_compatible() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = future_signal_admitted_handle(request_execution, "future-compatible");

    let temporal = orchestrated_outcome_from_signal_checked_on_handle(
        &handle,
        TemporalFutureSignalFamily::aspect_contract(),
        None,
        checked_signal_supported_runtime_test_posture(
            &handle,
            FutureSignalInput::<TemporalFutureSignalFamily>::new("face-a"),
        ),
    );
    let async_signal = orchestrated_outcome_from_signal_checked_on_handle(
        &handle,
        AsyncFutureSignalFamily::aspect_contract(),
        None,
        checked_signal_supported_runtime_test_posture(
            &handle,
            FutureSignalInput::<AsyncFutureSignalFamily>::new("face-a"),
        ),
    );

    match temporal {
        WorthQuerySignalCompatibilityOrchestrationOutcome::Bound(value) => {
            assert_eq!(value.future_projection().class().as_str(), "temporal");
        }
        _ => panic!(
            "temporal future subject should stay compatible under supported runtime test posture"
        ),
    }

    match async_signal {
        WorthQuerySignalCompatibilityOrchestrationOutcome::Bound(value) => {
            assert_eq!(value.future_projection().class().as_str(), "async_resource");
        }
        _ => panic!(
            "async future subject should stay compatible under supported runtime test posture"
        ),
    }
}

#[test]
fn public_future_signal_posture_still_stops_before_preparation_under_orchestration() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = future_signal_admitted_handle(request_execution, "future-public");

    let temporal = orchestrated_outcome_from_signal_checked_on_handle(
        &handle,
        TemporalFutureSignalFamily::aspect_contract(),
        Some(future_signal_bridge_request()),
        checked_signal_public_posture(
            &handle,
            FutureSignalInput::<TemporalFutureSignalFamily>::new("face-a"),
        ),
    );
    let async_signal = orchestrated_outcome_from_signal_checked_on_handle(
        &handle,
        AsyncFutureSignalFamily::aspect_contract(),
        Some(future_signal_bridge_request()),
        checked_signal_public_posture(
            &handle,
            FutureSignalInput::<AsyncFutureSignalFamily>::new("face-a"),
        ),
    );

    match temporal {
        WorthQuerySignalCompatibilityOrchestrationOutcome::Bound(value) => {
            assert_eq!(value.future_projection().class().as_str(), "temporal");
        }
        _ => panic!("temporal orchestration should bind under the public runtime"),
    }
    match async_signal {
        WorthQuerySignalCompatibilityOrchestrationOutcome::Bound(value) => {
            assert_eq!(value.future_projection().class().as_str(), "async_resource");
        }
        _ => panic!("async orchestration should bind under the public runtime"),
    }
}
