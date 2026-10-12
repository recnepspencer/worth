use crate::application::{
    WorthQueryAsyncLegalityDenialKind, WorthQueryDeclarationAdmissionOrLegalityError,
    WorthQueryDeclarationEntryInspectionInput, WorthQueryDeclarationEntryProgressionError,
    WorthQueryDeclarationEntryReadinessRequest, WorthQueryDeclarationEntryReadinessStatus,
};

use super::support::{
    async_current_envelope, handle, AsyncCurrentFamily, AsyncInput, AsyncPreviewFamily,
    DeferredAsyncFamily,
};

#[test]
fn async_current_family_admits_bridge_readiness_once_runtime_async_support_closes() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = handle(request_execution, "async-current");
    let readiness = handle.declaration_entry_readiness::<AsyncInput<AsyncCurrentFamily>>();
    let bridge_row = readiness
        .rows()
        .iter()
        .find(|row| row.crossing_row().bridge_continuation_family().is_some())
        .expect("bridge row should exist");

    assert_eq!(
        bridge_row.status(),
        WorthQueryDeclarationEntryReadinessStatus::Admitted
    );
    assert_eq!(
        bridge_row.reason(),
        "the declaration lowers into a bridge runtime route request"
    );
    if handle
        .declare_review_and_progress(AsyncInput::<AsyncCurrentFamily>::bridge_blocking("edge:42"))
        .is_err()
    {
        panic!("runtime-backed async declaration should now progress");
    }
}

#[test]
fn deferred_async_family_projects_family_level_async_debt_before_runtime_row_meaning() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = handle(request_execution, "async-deferred-family");
    let readiness = handle.declaration_entry_readiness::<AsyncInput<DeferredAsyncFamily>>();
    let bridge_row = readiness
        .rows()
        .iter()
        .find(|row| row.crossing_row().bridge_continuation_family().is_some())
        .expect("bridge row should exist");

    assert_eq!(
        bridge_row.status(),
        WorthQueryDeclarationEntryReadinessStatus::Deferred
    );
    assert_eq!(
        bridge_row.reason(),
        "async declaration-entry readiness remains deferred until this family's async declaration surface is admitted"
    );
}

#[test]
fn async_preview_family_keeps_preview_basis_invalid_and_localized() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = handle(request_execution, "async-preview");
    let readiness = handle.declaration_entry_readiness::<AsyncInput<AsyncPreviewFamily>>();
    let bridge_row = readiness
        .rows()
        .iter()
        .find(|row| row.crossing_row().bridge_continuation_family().is_some())
        .expect("bridge row should exist");
    let signal_row = readiness
        .rows()
        .iter()
        .find(|row| row.crossing_row().signal_execution_family().is_some())
        .expect("signal row should exist");

    assert_eq!(
        bridge_row.status(),
        WorthQueryDeclarationEntryReadinessStatus::InvalidBasis
    );
    assert_eq!(
        bridge_row.reason(),
        "async declaration-entry readiness does not currently admit preview bridge truth context"
    );
    assert_eq!(
        signal_row.status(),
        WorthQueryDeclarationEntryReadinessStatus::InvalidBasis
    );
    assert_eq!(
        signal_row.reason(),
        "async declaration-entry readiness does not currently admit preview-derived signal basis families"
    );
}

#[test]
fn retained_non_async_subject_does_not_inherit_family_level_async_debt() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = handle(request_execution, "async-current");
    let envelope =
        async_current_envelope(&handle, AsyncInput::<AsyncCurrentFamily>::plain("edge:42"));

    let readiness = handle
        .try_declaration_entry_readiness::<AsyncInput<AsyncCurrentFamily>>(
            WorthQueryDeclarationEntryReadinessRequest::base().for_retained_subject(
                crate::application::WorthQueryDeclarationEntryRetainedSubjectInput::envelope_checked(
                    crate::application::WorthQueryDeclarationEnvelopeChecked::Enveloped(
                        envelope,
                    ),
                ),
            ),
        )
        .unwrap_or_else(|_| panic!("subject-aware readiness should succeed"));
    let bridge_row = readiness
        .rows()
        .iter()
        .find(|row| row.crossing_row().bridge_continuation_family().is_some())
        .expect("bridge row should exist");
    assert_eq!(
        bridge_row.status(),
        WorthQueryDeclarationEntryReadinessStatus::Admitted
    );

    let inspection = handle
        .inspect_declaration_entry(WorthQueryDeclarationEntryInspectionInput::envelope_checked(
            crate::application::WorthQueryDeclarationEnvelopeChecked::Enveloped(
                async_current_envelope(&handle, AsyncInput::<AsyncCurrentFamily>::plain("edge:43")),
            ),
        ))
        .unwrap_or_else(|_| panic!("inspection should succeed"));
    let inspected_bridge_row = inspection
        .readiness()
        .rows()
        .iter()
        .find(|row| row.crossing_row().bridge_continuation_family().is_some())
        .expect("bridge row should exist");
    assert_eq!(
        inspected_bridge_row.status(),
        WorthQueryDeclarationEntryReadinessStatus::Admitted
    );
}

#[test]
fn unsupported_async_source_is_rejected_before_route_planning() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = handle(request_execution, "async-current");

    assert!(matches!(
        handle.declare_review_and_progress(AsyncInput::<AsyncCurrentFamily>::external_refresh("edge:99")),
        Err(WorthQueryDeclarationEntryProgressionError::Entry(
            WorthQueryDeclarationAdmissionOrLegalityError::Legality(
                crate::application::WorthQueryDeclarationLegalityDenial::AsyncProjectionUnsupported {
                    kind: WorthQueryAsyncLegalityDenialKind::UnsupportedSourceFamily(
                        crate::application::WorthQueryAsyncSourceFamily::ExternalResource,
                    ),
                    ..
                }
            )
        ))
    ));
}
