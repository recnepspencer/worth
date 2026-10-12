use super::super::support::*;
use super::consumer_support::routing::{route_consumer_stop_class, ConsumerStopRoute};
use super::consumer_support::runtime_errors::temporal_public_family_admission_error;

fn message_probe_matches_previous_support_wording(error: &WorthQueryRuntimeError) -> bool {
    error.to_string().contains("first temporal wording")
}

#[test]
fn public_api_family_admission_denial_surfaces_typed_family_status_posture_and_reason() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let error = temporal_public_family_admission_error(
        request_execution,
        "stop-class-consumer-public-admission",
        "support-gated temporal family stays closed",
    );

    match error.stop_class() {
        WorthQueryStopClass::FamilyAdmissionDenied {
            family,
            status,
            teaching_posture,
            reason,
        } => {
            assert_eq!(family, WorthQueryRuntimeFacadeFamily::Temporal);
            assert_eq!(status, WorthQueryRuntimeFamilySupportStatus::Supported);
            assert_eq!(
                teaching_posture,
                Some(WorthQueryRuntimeFamilyTeachingPosture::SupportGateOnly)
            );
            assert!(!reason.is_empty());
        }
        other => panic!("expected family-admission stop class, got {other:?}"),
    }
}

#[test]
fn consumer_router_handles_public_family_admission_without_string_matching() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let family_error = temporal_public_family_admission_error(
        request_execution,
        "stop-class-consumer-family-route",
        "support-gated temporal family stays closed",
    );

    assert_eq!(
        route_consumer_stop_class(&family_error),
        ConsumerStopRoute::FamilyAdmissionDenied {
            family: WorthQueryRuntimeFacadeFamily::Temporal,
            status: WorthQueryRuntimeFamilySupportStatus::Supported,
            teaching_posture: Some(WorthQueryRuntimeFamilyTeachingPosture::SupportGateOnly),
        }
    );
}

#[test]
fn typed_family_admission_matching_survives_message_rewording_while_string_probe_drifts() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let first_error = temporal_public_family_admission_error(
        request_execution,
        "stop-class-consumer-reword-first",
        "first temporal wording",
    );
    let second_error = temporal_public_family_admission_error(
        request_execution,
        "stop-class-consumer-reword-second",
        "second temporal wording",
    );

    assert_eq!(
        route_consumer_stop_class(&first_error),
        ConsumerStopRoute::FamilyAdmissionDenied {
            family: WorthQueryRuntimeFacadeFamily::Temporal,
            status: WorthQueryRuntimeFamilySupportStatus::Supported,
            teaching_posture: Some(WorthQueryRuntimeFamilyTeachingPosture::SupportGateOnly),
        }
    );
    assert_eq!(
        route_consumer_stop_class(&second_error),
        ConsumerStopRoute::FamilyAdmissionDenied {
            family: WorthQueryRuntimeFacadeFamily::Temporal,
            status: WorthQueryRuntimeFamilySupportStatus::Supported,
            teaching_posture: Some(WorthQueryRuntimeFamilyTeachingPosture::SupportGateOnly),
        }
    );

    assert!(
        message_probe_matches_previous_support_wording(&first_error),
        "the probe should match the original wording before drift"
    );
    assert!(
        !message_probe_matches_previous_support_wording(&second_error),
        "a consumer string probe should drift when presentation wording changes"
    );
}

#[test]
fn consumer_route_helper_uses_zero_string_matching_control_flow() {
    let router_source = include_str!("consumer_support/routing.rs");
    assert!(
        !router_source.contains("error.to_string()"),
        "consumer route helper must not route by formatting the runtime error"
    );
    assert!(
        !router_source.contains(".contains("),
        "consumer route helper must not route by probing message substrings"
    );
}
