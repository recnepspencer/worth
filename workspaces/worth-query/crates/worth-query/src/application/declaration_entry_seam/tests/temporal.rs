use crate::application::{
    WorthQueryDeclarationEntryInspectionInput, WorthQueryDeclarationEntryReadinessRequest,
    WorthQueryDeclarationEntryReadinessStatus,
};

use super::support::{
    handle, temporal_current_envelope, TemporalCurrentFamily, TemporalHistoricalFamily,
    TemporalInput, TemporalPreviewFamily,
};

#[test]
fn temporal_current_family_admits_bridge_readiness_once_runtime_temporal_support_closes() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = handle(request_execution, "temporal-current");
    let readiness = handle.declaration_entry_readiness::<TemporalInput<TemporalCurrentFamily>>();
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
        .declare_review_and_progress(TemporalInput::<TemporalCurrentFamily>::stale("edge:42"))
        .is_err()
    {
        panic!("runtime-backed temporal declaration should now progress");
    }
}

#[test]
fn temporal_preview_family_keeps_preview_truth_basis_invalid_and_localized() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = handle(request_execution, "temporal-preview");
    let readiness = handle.declaration_entry_readiness::<TemporalInput<TemporalPreviewFamily>>();
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
        "temporal declaration-entry readiness does not currently admit preview bridge truth context"
    );
    assert_eq!(
        signal_row.status(),
        WorthQueryDeclarationEntryReadinessStatus::InvalidBasis
    );
    assert_eq!(
        signal_row.reason(),
        "temporal declaration-entry readiness does not currently admit preview-derived signal basis families"
    );
}

#[test]
fn temporal_historical_family_keeps_historical_truth_basis_invalid_and_localized() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = handle(request_execution, "temporal-historical");
    let readiness = handle.declaration_entry_readiness::<TemporalInput<TemporalHistoricalFamily>>();
    let bridge_row = readiness
        .rows()
        .iter()
        .find(|row| row.crossing_row().bridge_continuation_family().is_some())
        .expect("bridge row should exist");

    assert_eq!(
        bridge_row.status(),
        WorthQueryDeclarationEntryReadinessStatus::InvalidBasis
    );
    assert_eq!(
        bridge_row.reason(),
        "temporal declaration-entry readiness does not currently admit historical bridge truth context"
    );
}

#[test]
fn retained_non_temporal_subject_does_not_inherit_family_level_temporal_debt() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = handle(request_execution, "temporal-current");
    let envelope = temporal_current_envelope(
        &handle,
        TemporalInput::<TemporalCurrentFamily>::plain("edge:42"),
    );

    let readiness = handle
        .try_declaration_entry_readiness::<TemporalInput<TemporalCurrentFamily>>(
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
                temporal_current_envelope(
                    &handle,
                    TemporalInput::<TemporalCurrentFamily>::plain("edge:43"),
                ),
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
