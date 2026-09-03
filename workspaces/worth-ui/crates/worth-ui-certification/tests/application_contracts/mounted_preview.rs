use worth_ui_certification::scenario::filesystem_application_lifecycle::FilesystemApplicationLifecycleScenario;
use worth_ui_host_headless::WorthUiHeadlessRecorder;
use worth_ui_runtime::facade::mounted::{
    UiHostSurfacePresentationMode, UiMountedLaneParticipation, UiMountedPreviewProjection,
    UiPresentationDeadline, UiRequiredLaneContributionStatus,
};
use worth_ui_runtime::facade::runtime_handoff::{
    UiAllocationReplanTransactionCommitDenial, UiAllocationReplanTransactionOutcome,
    UiPreviewPaintIsolationOutcome, UiResizeLogicalExtent, UiResizePreviewSample,
    WorthUiFrameworkTurnCompletion,
};
use worth_ui_runtime::facade::{
    WorthUiMountedPreviewDisposition, WorthUiMountedPreviewOutcome,
    WorthUiMountedPreviewPreparationDenial,
};
use worth_ui_test_support::WorthUiFrameworkTurnCertificationExt;
use worth_ui_test_support::WorthUiMountedIdentityCertificationExt;

use super::mounted_application_lifecycle::adapter_projection_world::{
    cell_status, preview_application_from_sources, preview_application_with_host, preview_target,
    retire_query, submit_preview,
};
use super::mounted_application_lifecycle::known_empty_surface_world::profile;

#[test]
fn real_wui_preview_records_and_publishes_through_the_mounted_contract() {
    let recorder = WorthUiHeadlessRecorder::default();
    let (mut session, workspace, mut scenario) =
        preview_application_with_host("mounted-preview-record", recorder.clone());
    let surface = session.create_semantic_surface().unwrap();
    session
        .register_host_surface(
            surface,
            UiHostSurfacePresentationMode::RecordOnly,
            profile(1),
        )
        .unwrap();
    let target = preview_target(&session);
    let handle = session.mounted_graph_node(target).unwrap();
    let instance = session.mount_instance(handle, surface).unwrap();

    let pending = submit_preview(&mut session, target, 320.0);
    let prepared = match pending.prepare(instance) {
        Ok(prepared) => prepared,
        Err(_) => panic!("matching mounted instance must prepare preview"),
    };
    let frame = prepared.frame();
    assert_eq!(
        cell_status(frame, UiMountedLaneParticipation::Preview),
        UiRequiredLaneContributionStatus::Admitted
    );
    assert!(frame
        .manifest()
        .lane_contributions()
        .iter()
        .filter(|cell| cell.lane() != UiMountedLaneParticipation::Preview)
        .all(|cell| cell.status() == UiRequiredLaneContributionStatus::ExplicitEmpty));
    assert_eq!(
        frame
            .manifest()
            .lane_contributions()
            .iter()
            .filter(|cell| cell.lane() == UiMountedLaneParticipation::Preview)
            .count(),
        1,
        "the existing mounted-preview contract has one preview lane"
    );
    assert!(matches!(
        frame.surfaces()[0].projection().nodes()[0].preview(),
        UiMountedPreviewProjection::Resize {
            extent_subpixels,
            candidate_count: 1,
            all_candidates_admitted: true,
            ..
        } if extent_subpixels == UiResizeLogicalExtent::try_from_logical_pixels(320.0).unwrap().subpixels()
    ));

    let resolved = match prepared.present(UiPresentationDeadline::at_tick(10), 0) {
        WorthUiMountedPreviewOutcome::Resolved(resolved) => resolved,
        _ => panic!("record-only preview must resolve synchronously"),
    };
    let publication = match resolved.disposition() {
        WorthUiMountedPreviewDisposition::Published(publication) => publication,
        _ => panic!("record-only mounted preview must publish"),
    };
    assert!(matches!(
        resolved.isolation(),
        UiPreviewPaintIsolationOutcome::Verified(_)
    ));
    let transcripts = recorder.observed_transcripts();
    assert_eq!(transcripts.len(), 1);
    assert_eq!(transcripts[0].frame(), publication.frame());
    assert_eq!(transcripts[0].nodes()[0].mounted_instance(), instance);
    assert!(matches!(
        transcripts[0].nodes()[0].preview(),
        UiMountedPreviewProjection::Resize {
            extent_subpixels,
            candidate_count: 1,
            all_candidates_admitted: true,
            ..
        } if extent_subpixels == UiResizeLogicalExtent::try_from_logical_pixels(320.0).unwrap().subpixels()
    ));
    assert_eq!(
        session.inspect_mounted_identity().current_frame(),
        Some(publication.frame())
    );
    retire_query(
        &mut scenario,
        session.shutdown().into_operation_live_retirement(),
    );
    workspace.close();
}

#[test]
fn preview_target_mismatch_is_typed_and_supersession_has_no_host_effect() {
    let recorder = WorthUiHeadlessRecorder::default();
    let (mut session, workspace, mut scenario) =
        preview_application_with_host("mounted-preview-mismatch", recorder.clone());
    let surface = session.create_semantic_surface().unwrap();
    session
        .register_host_surface(
            surface,
            UiHostSurfacePresentationMode::RecordOnly,
            profile(1),
        )
        .unwrap();
    let target = preview_target(&session);
    let other = session
        .graph()
        .node_identities()
        .find(|identity| *identity != target)
        .expect("the real application supplies a distinct mounted mismatch target");
    let wrong_instance = session
        .mount_instance(session.mounted_graph_node(other).unwrap(), surface)
        .unwrap();
    let rejection = match submit_preview(&mut session, target, 240.0).prepare(wrong_instance) {
        Ok(_) => panic!("a different mounted graph node cannot receive the preview"),
        Err(rejection) => rejection,
    };
    assert_eq!(
        rejection.denial(),
        &WorthUiMountedPreviewPreparationDenial::PreviewTargetMismatch
    );
    let resolved = rejection.supersede();
    assert!(matches!(
        resolved.disposition(),
        WorthUiMountedPreviewDisposition::Superseded
    ));
    assert!(recorder.observed_transcripts().is_empty());
    assert!(session.inspect_mounted_identity().current_frame().is_none());
    retire_query(
        &mut scenario,
        session.shutdown().into_operation_live_retirement(),
    );
    workspace.close();
}

#[test]
fn resizable_surface_with_only_scroll_state_cannot_mint_preview_authority() {
    let recorder = WorthUiHeadlessRecorder::default();
    let (mut session, workspace, mut scenario) = preview_application_from_sources(
        "mounted-preview-non-splitter",
        recorder.clone(),
        FilesystemApplicationLifecycleScenario::resizable_non_splitter_source_text(false),
        FilesystemApplicationLifecycleScenario::resizable_non_splitter_source_text(true),
    );
    let surface = session.create_semantic_surface().unwrap();
    session
        .register_host_surface(
            surface,
            UiHostSurfacePresentationMode::RecordOnly,
            profile(1),
        )
        .unwrap();
    let target = preview_target(&session);
    let handle = session.mounted_graph_node(target).unwrap();
    let instance = session.mount_instance(handle, surface).unwrap();
    let extent = UiResizeLogicalExtent::try_from_logical_pixels(300.0).unwrap();
    let completion = session
        .execute_framework_turn(|turn| {
            turn.resize_preview(|source| {
                source
                    .admit_and_submit(UiResizePreviewSample::new(target, extent))
                    .unwrap();
            });
        })
        .expect("no mounted presentation lease is active");
    let completion = match completion.into_mounted_preview() {
        Ok(_) => panic!("unrelated durable scroll state must not mint splitter preview authority"),
        Err(completion) => completion,
    };
    assert!(matches!(
        completion.into_completion(),
        WorthUiFrameworkTurnCompletion::AllocationInvalidationsNarrowed {
            transaction: UiAllocationReplanTransactionOutcome::Denied(
                UiAllocationReplanTransactionCommitDenial::ResizeBasisDenied,
            ),
            ..
        }
    ));
    assert!(session
        .mounted_instances_for(handle)
        .unwrap()
        .contains(&instance));
    assert!(recorder.observed_transcripts().is_empty());
    assert!(session.inspect_mounted_identity().current_frame().is_none());
    retire_query(
        &mut scenario,
        session.shutdown().into_operation_live_retirement(),
    );
    workspace.close();
}
