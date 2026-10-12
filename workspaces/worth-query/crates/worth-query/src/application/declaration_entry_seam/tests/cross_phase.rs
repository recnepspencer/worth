use crate::application::{
    WorthQueryDeclarationEntryInspectionInput, WorthQueryDeclarationEntryReadinessStatus,
};
use crate::runtime::{
    runtime_state_snapshot_basis_label_identity,
    runtime_state_snapshot_result_shape_label_identity, WorthQueryInspection,
    WorthQueryRuntimeStateKind,
};

use super::support::{
    async_current_envelope, temporal_current_envelope, workspace_and_handle, AsyncCurrentFamily,
    AsyncInput, TemporalCurrentFamily, TemporalInput,
};

#[test]
fn retained_world_basis_and_subject_aware_temporal_readiness_stay_stitched() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (workspace, handle) = workspace_and_handle(request_execution, "cross-phase-temporal");
    let world_basis = handle.retained_world_basis();

    let world_state = workspace
        .state(request_execution, &world_basis)
        .expect("retained world basis should snapshot");
    let world_inspection = workspace
        .inspect(request_execution, &world_basis)
        .expect("retained world basis should inspect");

    assert_eq!(world_state.kind(), WorthQueryRuntimeStateKind::Ready);
    assert_eq!(
        world_state.basis_for_reporting(),
        runtime_state_snapshot_basis_label_identity(
            world_basis.basis_lifecycle_support_identity(),
        )
        .as_str()
    );
    assert_eq!(
        world_state.result_shape_for_reporting(),
        runtime_state_snapshot_result_shape_label_identity(world_basis.handle_identity(),).as_str()
    );

    match world_inspection {
        WorthQueryInspection::BasisLifecycle(inspection) => {
            assert_eq!(inspection.subject_label(), "admitted_world_basis");
            assert_eq!(inspection.state_kind(), WorthQueryRuntimeStateKind::Ready);
            assert_eq!(
                inspection.support_digest(),
                Some(world_basis.support_snapshot_digest())
            );
            assert_eq!(inspection.basis_digest(), world_state.basis_for_reporting());
            assert_eq!(
                inspection.shape_digest(),
                world_state.result_shape_for_reporting()
            );
        }
        other => panic!("expected basis lifecycle inspection, got {other:?}"),
    }

    let plain_envelope = temporal_current_envelope(
        &handle,
        TemporalInput::<TemporalCurrentFamily>::plain("edge:plain"),
    );
    let plain_inspection = handle
        .inspect_declaration_entry(WorthQueryDeclarationEntryInspectionInput::envelope_checked(
            crate::application::WorthQueryDeclarationEnvelopeChecked::Enveloped(plain_envelope),
        ))
        .unwrap_or_else(|_| panic!("plain temporal-family envelope should inspect"));
    let bridge_row = plain_inspection
        .readiness()
        .rows()
        .iter()
        .find(|row| row.crossing_row().bridge_continuation_family().is_some())
        .expect("bridge row should exist");
    assert_eq!(
        bridge_row.status(),
        WorthQueryDeclarationEntryReadinessStatus::Admitted
    );

    let stale_envelope = temporal_current_envelope(
        &handle,
        TemporalInput::<TemporalCurrentFamily>::stale("edge:stale"),
    );
    let stale_inspection = handle
        .inspect_declaration_entry(WorthQueryDeclarationEntryInspectionInput::envelope_checked(
            crate::application::WorthQueryDeclarationEnvelopeChecked::Enveloped(stale_envelope),
        ))
        .unwrap_or_else(|_| panic!("stale temporal-family envelope should inspect"));
    let stale_bridge_row = stale_inspection
        .readiness()
        .rows()
        .iter()
        .find(|row| row.crossing_row().bridge_continuation_family().is_some())
        .expect("stale bridge row should exist");
    assert_eq!(
        stale_bridge_row.status(),
        WorthQueryDeclarationEntryReadinessStatus::Admitted
    );
}

#[test]
fn retained_world_basis_and_subject_aware_async_readiness_stay_stitched() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (workspace, handle) = workspace_and_handle(request_execution, "cross-phase-async");
    let world_basis = handle.retained_world_basis();

    let world_inspection = workspace
        .inspect(request_execution, &world_basis)
        .expect("retained world basis should inspect");
    match world_inspection {
        WorthQueryInspection::BasisLifecycle(inspection) => {
            assert_eq!(inspection.subject_label(), "admitted_world_basis");
            assert_eq!(
                inspection.support_digest(),
                Some(world_basis.support_snapshot_digest())
            );
        }
        other => panic!("expected basis lifecycle inspection, got {other:?}"),
    }

    let plain_envelope = async_current_envelope(
        &handle,
        AsyncInput::<AsyncCurrentFamily>::plain("edge:plain"),
    );
    let plain_inspection = handle
        .inspect_declaration_entry(WorthQueryDeclarationEntryInspectionInput::envelope_checked(
            crate::application::WorthQueryDeclarationEnvelopeChecked::Enveloped(plain_envelope),
        ))
        .unwrap_or_else(|_| panic!("plain async-family envelope should inspect"));
    let bridge_row = plain_inspection
        .readiness()
        .rows()
        .iter()
        .find(|row| row.crossing_row().bridge_continuation_family().is_some())
        .expect("bridge row should exist");
    assert_eq!(
        bridge_row.status(),
        WorthQueryDeclarationEntryReadinessStatus::Admitted
    );

    let blocking_envelope = async_current_envelope(
        &handle,
        AsyncInput::<AsyncCurrentFamily>::bridge_blocking("edge:blocking"),
    );
    let blocking_inspection = handle
        .inspect_declaration_entry(WorthQueryDeclarationEntryInspectionInput::envelope_checked(
            crate::application::WorthQueryDeclarationEnvelopeChecked::Enveloped(blocking_envelope),
        ))
        .unwrap_or_else(|_| panic!("blocking async-family envelope should inspect"));
    let blocking_bridge_row = blocking_inspection
        .readiness()
        .rows()
        .iter()
        .find(|row| row.crossing_row().bridge_continuation_family().is_some())
        .expect("blocking bridge row should exist");
    assert_eq!(
        blocking_bridge_row.status(),
        WorthQueryDeclarationEntryReadinessStatus::Admitted
    );
}
