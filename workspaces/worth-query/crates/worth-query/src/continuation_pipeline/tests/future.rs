use crate::application::WorthQueryDeclarationFamilyMarker;
use crate::continuation_pipeline::{
    prepare_continuation_from_signal_checked_on_handle, WorthQueryPreparedContinuationOutcome,
};
use crate::future_signal_test_support::{
    checked_signal_public_posture, future_signal_admitted_handle, future_signal_bridge_request,
    AsyncFutureSignalFamily, FutureSignalInput, OrdinaryFutureSignalFamily,
    TemporalFutureSignalFamily,
};

#[test]
fn ordinary_prepared_continuation_keeps_future_projection_visible() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = future_signal_admitted_handle(request_execution, "future");
    let checked = checked_signal_public_posture(
        &handle,
        FutureSignalInput::<OrdinaryFutureSignalFamily>::new("face-a"),
    );
    let outcome = prepare_continuation_from_signal_checked_on_handle(
        &handle,
        checked,
        future_signal_bridge_request(),
        OrdinaryFutureSignalFamily::aspect_contract(),
    )
    .into_checked()
    .into_outcome();

    match outcome {
        WorthQueryPreparedContinuationOutcome::Prepared(prepared) => {
            assert_eq!(prepared.future_projection().class().as_str(), "ordinary");
            assert!(!prepared.basis_lifecycle_support_digest().is_empty());
            assert!(prepared.signal_compatibility_digest().is_some());
        }
        _ => panic!("ordinary compatible signal posture should prepare"),
    }
}

#[test]
fn temporal_and_async_signal_debt_stop_before_prepared_continuation_exists() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = future_signal_admitted_handle(request_execution, "future-public");

    let temporal = prepare_continuation_from_signal_checked_on_handle(
        &handle,
        checked_signal_public_posture(
            &handle,
            FutureSignalInput::<TemporalFutureSignalFamily>::new("face-a"),
        ),
        future_signal_bridge_request(),
        TemporalFutureSignalFamily::aspect_contract(),
    )
    .into_checked()
    .into_outcome();
    let async_signal = prepare_continuation_from_signal_checked_on_handle(
        &handle,
        checked_signal_public_posture(
            &handle,
            FutureSignalInput::<AsyncFutureSignalFamily>::new("face-a"),
        ),
        future_signal_bridge_request(),
        AsyncFutureSignalFamily::aspect_contract(),
    )
    .into_checked()
    .into_outcome();

    match temporal {
        WorthQueryPreparedContinuationOutcome::Prepared(prepared) => {
            assert_eq!(prepared.future_projection().class().as_str(), "temporal");
            assert!(prepared.signal_compatibility_digest().is_some());
        }
        _ => panic!("temporal future signal posture should prepare under the public runtime"),
    }
    match async_signal {
        WorthQueryPreparedContinuationOutcome::Prepared(prepared) => {
            assert_eq!(
                prepared.future_projection().class().as_str(),
                "async_resource"
            );
            assert!(prepared.signal_compatibility_digest().is_some());
        }
        _ => panic!("async future signal posture should prepare under the public runtime"),
    }
}
