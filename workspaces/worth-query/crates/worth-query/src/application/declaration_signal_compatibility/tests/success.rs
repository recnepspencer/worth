use crate::application::{
    WorthQueryDeclarationPrimaryAuthorityFamily, WorthQueryDeclarationSignalCompatibilityChecked,
    WorthQueryDeclarationSignalCompatibilityDenialCause,
    WorthQueryDeclarationSignalExecutionFamily,
};
use crate::basis_lifecycle::BasisFamily;

use super::support::{
    domain::{handle, HistoricalFamily, Input, MixedFamily, PreviewFamily, RuntimeFamily},
    proof::{compatibility_from_envelope_input, envelope_checked_for},
};

#[test]
fn common_lane_emits_signal_compatibility_artifact() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let compatibility = handle(request_execution, "common")
        .declare_review_progress_describe_plan_receipt_envelope_and_check_signal_compatibility(
            Input::<RuntimeFamily>::new("edge:42"),
        )
        .unwrap_or_else(|_| panic!("signal compatibility should admit"));

    assert_eq!(
        compatibility.execution_family(),
        WorthQueryDeclarationSignalExecutionFamily::RuntimeDerivedExecution
    );
    assert_eq!(compatibility.basis_families(), &[BasisFamily::CurrentHead]);
}

#[test]
fn advanced_lane_envelope_input_routes_without_checked_wrapper_loss() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let compatibility = compatibility_from_envelope_input(
        &handle(request_execution, "advanced"),
        Input::<RuntimeFamily>::new("edge:42"),
    );

    assert_eq!(
        compatibility.execution_family(),
        WorthQueryDeclarationSignalExecutionFamily::RuntimeDerivedExecution
    );
}

#[test]
fn equivalent_retained_envelope_truth_yields_identical_digest() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = handle(request_execution, "digest");

    let left = handle
        .signal_compatibility_from_progressed(
            handle
                .declare_review_and_progress(Input::<RuntimeFamily>::new("edge:42"))
                .unwrap_or_else(|_| panic!("progression should admit")),
        )
        .unwrap_or_else(|_| panic!("signal compatibility should admit"));
    let right = handle
        .signal_compatibility_from_progressed(
            handle
                .declare_review_and_progress(Input::<RuntimeFamily>::new("edge:42"))
                .unwrap_or_else(|_| panic!("progression should admit")),
        )
        .unwrap_or_else(|_| panic!("signal compatibility should admit"));

    assert_eq!(
        left.signal_compatibility_digest(),
        right.signal_compatibility_digest()
    );
}

#[test]
fn admitted_world_identity_changes_digest() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let left = handle(request_execution, "left")
        .declare_review_progress_describe_plan_receipt_envelope_and_check_signal_compatibility(
            Input::<RuntimeFamily>::new("edge:42"),
        )
        .unwrap_or_else(|_| panic!("signal compatibility should admit"));
    let right = handle(request_execution, "right")
        .declare_review_progress_describe_plan_receipt_envelope_and_check_signal_compatibility(
            Input::<RuntimeFamily>::new("edge:42"),
        )
        .unwrap_or_else(|_| panic!("signal compatibility should admit"));

    assert_ne!(
        left.signal_compatibility_digest(),
        right.signal_compatibility_digest()
    );
}

#[test]
fn mixed_authority_stays_modifier_not_peer_authority() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let compatibility = handle(request_execution, "mixed")
        .declare_review_progress_describe_plan_receipt_envelope_and_check_signal_compatibility(
            Input::<MixedFamily>::new("edge:42"),
        )
        .unwrap_or_else(|_| panic!("signal compatibility should admit"));

    assert_eq!(
        compatibility.primary_authority_family(),
        WorthQueryDeclarationPrimaryAuthorityFamily::MixedAuthority
    );
    assert_eq!(
        compatibility.execution_family(),
        WorthQueryDeclarationSignalExecutionFamily::MixedDerivedExecution
    );
}

#[test]
fn basis_family_differences_stay_distinct() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = handle(request_execution, "basis");

    let historical = handle
        .declare_review_progress_describe_plan_receipt_envelope_and_check_signal_compatibility(
            Input::<HistoricalFamily>::new("edge:42"),
        )
        .unwrap_or_else(|_| panic!("signal compatibility should admit"));
    let preview = handle
        .declare_review_progress_describe_plan_receipt_envelope_and_check_signal_compatibility(
            Input::<PreviewFamily>::new("edge:42"),
        )
        .unwrap_or_else(|_| panic!("signal compatibility should admit"));

    assert_eq!(
        historical.basis_families(),
        &[BasisFamily::HistoricalSnapshot]
    );
    assert_eq!(preview.basis_families(), &[BasisFamily::PreviewDerived]);
    assert_ne!(
        historical.signal_compatibility_digest(),
        preview.signal_compatibility_digest()
    );
}

#[test]
fn wrong_handle_envelope_is_denied_before_signal_classification() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let source = handle(request_execution, "source");
    let target = handle(request_execution, "target");
    let checked = target.signal_compatibility_checked(
        crate::application::WorthQueryDeclarationSignalCompatibilityInput::envelope_checked(
            envelope_checked_for(&source, Input::<RuntimeFamily>::new("edge:42")),
        ),
    );

    match checked {
        WorthQueryDeclarationSignalCompatibilityChecked::Denied(denied) => {
            assert_eq!(
                denied.cause(),
                WorthQueryDeclarationSignalCompatibilityDenialCause::SignalCompatibilityMismatch
            );
        }
        _ => panic!("wrong-handle envelopes should deny before signal classification"),
    }
}
