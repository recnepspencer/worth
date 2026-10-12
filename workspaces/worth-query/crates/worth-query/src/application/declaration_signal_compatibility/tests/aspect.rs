use crate::application::{
    assert_declaration_aspect_projections, WorthQueryDeclarationSignalCompatibilityChecked,
};

use super::support::domain::{
    handle, ConflictingAspectFamily, ExpandedAspectFamily, Input, MissingAspectFamily,
    RuntimeFamily,
};

#[test]
fn signal_compatibility_exposes_dependency_and_produced_aspects() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let compatibility = handle(request_execution, "primary")
        .declare_review_progress_describe_plan_receipt_envelope_and_check_signal_compatibility(
            Input::<RuntimeFamily>::new("edge:42"),
        )
        .unwrap_or_else(|_| panic!("signal compatibility should succeed"));

    assert_eq!(
        compatibility.aspect_coverage_basis(),
        crate::application::WorthQueryDeclarationAspectCoverageBasis::EnvelopePublishedCoverage
    );
    assert_eq!(
        compatibility.aspect_fit(),
        crate::application::WorthQueryDeclarationAspectFit::CompatibleSuperset
    );
    assert!(!compatibility.dependency_aspects().required().is_empty());
    assert!(!compatibility.produced_aspects().required().is_empty());
}

#[test]
fn signal_compatibility_denies_missing_and_conflicting_dependency_slices() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = handle(request_execution, "primary");
    let missing_envelope = handle
        .envelope_routes_from_progressed(
            handle
                .declare_review_and_progress(Input::<MissingAspectFamily>::new("edge:42"))
                .unwrap_or_else(|_| panic!("progression should admit")),
        )
        .unwrap_or_else(|_| panic!("envelope should succeed"));
    match handle.signal_compatibility_checked(
        crate::application::WorthQueryDeclarationSignalCompatibilityInput::enveloped(
            missing_envelope,
        ),
    ) {
        WorthQueryDeclarationSignalCompatibilityChecked::Denied(denial) => assert_eq!(
            denial.cause(),
            crate::application::WorthQueryDeclarationSignalCompatibilityDenialCause::AuthorityAspectGap
        ),
        _ => panic!("missing signal dependency slice should deny"),
    }

    let conflicting_envelope = handle
        .envelope_routes_from_progressed(
            handle
                .declare_review_and_progress(Input::<ConflictingAspectFamily>::new("edge:42"))
                .unwrap_or_else(|_| panic!("progression should admit")),
        )
        .unwrap_or_else(|_| panic!("envelope should succeed"));
    match handle.signal_compatibility_checked(
        crate::application::WorthQueryDeclarationSignalCompatibilityInput::enveloped(
            conflicting_envelope,
        ),
    ) {
        WorthQueryDeclarationSignalCompatibilityChecked::Denied(denial) => assert_eq!(
            denial.cause(),
            crate::application::WorthQueryDeclarationSignalCompatibilityDenialCause::AuthorityAspectGap
        ),
        _ => panic!("conflicting signal dependency slice should deny"),
    }
}

#[test]
fn signal_compatibility_digest_changes_with_dependency_or_produced_aspects() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let base = handle(request_execution, "primary")
        .declare_review_progress_describe_plan_receipt_envelope_and_check_signal_compatibility(
            Input::<RuntimeFamily>::new("edge:42"),
        )
        .unwrap_or_else(|_| panic!("signal compatibility should succeed"));
    let expanded = handle(request_execution, "primary")
        .declare_review_progress_describe_plan_receipt_envelope_and_check_signal_compatibility(
            Input::<ExpandedAspectFamily>::new("edge:42"),
        )
        .unwrap_or_else(|_| panic!("expanded signal compatibility should succeed"));

    assert_ne!(
        base.signal_compatibility_digest(),
        expanded.signal_compatibility_digest()
    );
}

#[test]
fn signal_support_rows_expose_dependency_and_produced_aspects() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let support =
        handle(request_execution, "primary").signal_compatibility_support::<Input<RuntimeFamily>>();
    let row = &support.rows()[0];

    assert_declaration_aspect_projections(
        row.required_dependency_aspects().required(),
        &["selection.active_face", "signal.dependency.runtime_inputs"],
    );
    assert_declaration_aspect_projections(
        row.produced_aspects().required(),
        &["signal.produced.derived_face_preview"],
    );
}

#[test]
fn signal_support_rows_expose_authority_mismatch_for_missing_dependencies() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let support = handle(request_execution, "primary")
        .signal_compatibility_support::<Input<MissingAspectFamily>>();
    let row = &support.rows()[0];

    assert_eq!(
        row.aspect_mismatch(),
        Some(crate::application::WorthQueryDeclarationAuthorityAspectMismatch::AuthorityAspectGap)
    );
}
