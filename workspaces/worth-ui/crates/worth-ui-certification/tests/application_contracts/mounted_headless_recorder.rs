use worth_ui::facade::app::WorthUiVisibleRange;
use worth_ui::facade::source::WorthUiFilesystemSourceProvider;
use worth_ui_certification::scenario::filesystem_application_lifecycle::FilesystemApplicationLifecycleScenario;
use worth_ui_host_headless::{
    UiHeadlessNodePaintMechanic, UiHeadlessRecorderCapacity, WorthUiHeadlessRecorder,
};
use worth_ui_runtime::facade::host::{WorthUiHostCapability, WorthUiOperationalHostAdapter};
use worth_ui_runtime::facade::mounted::{
    UiHostSurfacePresentationDenial, UiHostSurfacePresentationMode, UiMountedAllocationProjection,
    UiMountedEffectFamily, UiMountedFrameOutcome, UiMountedFrameRequest, UiMountedOmissionReason,
    UiMountedParticipationStatus, UiPresentationDeadline,
};
use worth_ui_test_support::WorthUiMountedIdentityCertificationExt;
use worth_ui_test_support::{
    WorthUiActiveSessionCertificationExt, WorthUiFrameworkTurnCertificationExt,
};
use worth_ui_test_support::{
    WorthUiMountedFrameExecutionCertificationExt, WorthUiMountedPublicationCertificationExt,
};

use super::filesystem_contract_workspace::FilesystemContractWorkspace;
use super::mounted_application_lifecycle::known_empty_surface_world::{
    first_node, mounted_application_with_host, profile,
};

#[path = "mounted_headless_recorder/capacity.rs"]
mod capacity;
#[path = "mounted_headless_recorder/measurement.rs"]
mod measurement;
#[path = "mounted_headless_recorder/reconstruction.rs"]
mod reconstruction;

#[test]
fn real_wui_record_only_presentation_emits_post_translation_mechanics() {
    let recorder = WorthUiHeadlessRecorder::default();
    let mut session = mounted_application_with_host("mounted-headless-recording", recorder.clone())
        .launch()
        .expect("real file-authored application launches");
    let capabilities = recorder.operational_capability_report();
    assert_eq!(
        capabilities.observed_capabilities(),
        &[
            WorthUiHostCapability::MountedFrameRecording,
            WorthUiHostCapability::SemanticFocusPlacement,
        ]
    );
    let surface = session.create_semantic_surface().unwrap();
    let binding = session
        .register_host_surface(
            surface,
            UiHostSurfacePresentationMode::RecordOnly,
            profile(1),
        )
        .unwrap()
        .binding_generation();
    let node = first_node(&session);
    let mounted_instance = session.mount_instance(node, surface).unwrap();
    let frame = prepare(&mut session);

    let publication =
        match session.present_prepared_mounted_frame(frame, UiPresentationDeadline::at_tick(10), 0)
        {
            UiMountedFrameOutcome::Published(receipt) => receipt,
            _ => panic!("record-only headless translation must publish"),
        };
    let transcripts = recorder.observed_transcripts();
    assert_eq!(transcripts.len(), 1);
    assert_ordinary_transcript(&transcripts[0], &publication, binding, mounted_instance);
}

fn assert_ordinary_transcript(
    transcript: &worth_ui_host_headless::UiHeadlessMountedFrameTranscript,
    publication: &worth_ui_runtime::facade::mounted::UiMountedFramePublicationReceipt,
    binding: worth_ui_runtime::facade::mounted::UiSurfaceBindingGeneration,
    mounted_instance: worth_ui_runtime::facade::mounted::UiMountedInstanceIdentity,
) {
    assert_eq!(transcript.mode(), UiHostSurfacePresentationMode::RecordOnly);
    assert_eq!(transcript.attempt(), publication.attempt());
    assert_eq!(transcript.frame(), publication.frame());
    assert_eq!(transcript.binding(), binding);
    assert_eq!(transcript.nodes().len(), 1);
    assert_eq!(transcript.nodes()[0].mounted_instance(), mounted_instance);
    assert_eq!(
        transcript.nodes()[0].paint(),
        UiHeadlessNodePaintMechanic::Omitted(UiMountedOmissionReason::NotProducedByExecutedLane)
    );
    assert_eq!(
        transcript.nodes()[0].allocation(),
        UiMountedAllocationProjection::Omitted(UiMountedOmissionReason::NoCommittedAllocation)
    );
    assert_eq!(
        transcript.nodes()[0].participation().paint().status(),
        UiMountedParticipationStatus::Deferred
    );
    assert!(transcript
        .paint_batches()
        .windows(2)
        .all(|pair| paint_order(&pair[0]) <= paint_order(&pair[1])));
    assert_eq!(
        transcript.unperformed_effects()[0],
        worth_ui_host_headless::UiHeadlessUnperformedEffect::NativePaint {
            filled_rect_count: transcript.filled_rects().len() as u32,
            portal_overlay_count: transcript.portal_overlays().len() as u32,
            semantic_text_count: transcript.semantic_text().len() as u32,
            preview_node_count: 0,
        }
    );
}

#[test]
fn real_cross_lane_recording_preserves_exact_unperformed_external_mechanics() {
    let recorder = WorthUiHeadlessRecorder::default();
    let (mut scenario, workspace, mut session) = launch_cross_lane(recorder.clone());
    let (frame, binding) = prepare_cross_lane(&mut scenario, &mut session);
    assert!(matches!(
        session.present_prepared_mounted_frame(frame, UiPresentationDeadline::at_tick(10), 0,),
        UiMountedFrameOutcome::Published(_)
    ));

    let transcripts = recorder.observed_transcripts();
    assert_exact_external_mechanics(&transcripts[0]);
    session
        .rebind_host_surface(
            binding,
            UiHostSurfacePresentationMode::NativeDisplay,
            profile(2),
        )
        .unwrap();
    let native_frame = execute_cross_lane_frame(&mut session);
    assert_rejected(
        "cross-lane native presentation",
        session.present_prepared_mounted_frame(
            native_frame,
            UiPresentationDeadline::at_tick(20),
            1,
        ),
        UiHostSurfacePresentationDenial::UnsupportedEffect(UiMountedEffectFamily::CanvasSpatial),
    );
    assert_eq!(recorder.observed_transcripts().len(), 1);
    let _ = session.shutdown();
    workspace.close();
}

fn launch_cross_lane(
    recorder: WorthUiHeadlessRecorder,
) -> (
    FilesystemApplicationLifecycleScenario,
    FilesystemContractWorkspace,
    worth_ui::facade::app::WorthUiActiveApplicationSession,
) {
    let scenario = FilesystemApplicationLifecycleScenario::new("headless-cross-lane");
    let workspace = FilesystemContractWorkspace::new("headless-cross-lane");
    workspace.write(
        "app/main.wui",
        &FilesystemApplicationLifecycleScenario::cross_lane_source_text(),
    );
    let capabilities = scenario.cross_lane_capability_application(recorder.clone());
    let submission = FilesystemApplicationLifecycleScenario::lower_snapshot(
        WorthUiFilesystemSourceProvider::new(workspace.root())
            .read()
            .unwrap(),
        capabilities.capabilities(),
    );
    let session = scenario
        .prepare_cross_lane_application_with_host(submission, recorder)
        .launch()
        .unwrap();
    (scenario, workspace, session)
}

fn prepare_cross_lane(
    scenario: &mut FilesystemApplicationLifecycleScenario,
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
) -> (
    worth_ui_runtime::facade::mounted::UiPreparedMountedFrame,
    worth_ui_runtime::facade::mounted::UiSurfaceBindingGeneration,
) {
    let surface = session.create_semantic_surface().unwrap();
    let binding = session
        .register_host_surface(
            surface,
            UiHostSurfacePresentationMode::RecordOnly,
            profile(1),
        )
        .unwrap()
        .binding_generation();
    let nodes = session.graph().node_identities().collect::<Vec<_>>();
    for node in nodes {
        let handle = session.mounted_graph_node(node).unwrap();
        session.mount_instance(handle, surface).unwrap();
    }
    let projection = scenario.settled_query_projection();
    let link = session.query_fact_link("inspector.measurements").unwrap();
    drop(
        session
            .execute_framework_turn(|turn| {
                turn.query_projection(|source| {
                    source.admit_settled(projection).unwrap();
                    source.submit_settled(&link).unwrap();
                });
            })
            .expect("no mounted presentation lease is active")
            .into_completion(),
    );
    (execute_cross_lane_frame(session), binding)
}

fn execute_cross_lane_frame(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
) -> worth_ui_runtime::facade::mounted::UiPreparedMountedFrame {
    crate::mounted_geometry_fixture::install_current_occurrence_geometry(session);
    session
        .execute_framework_turn(|_| {})
        .expect("no mounted presentation lease is active")
        .into_execution()
        .unwrap_or_else(|_| panic!("cross-lane execution is admitted"))
        .prepare_mounted_frame(
            UiMountedFrameRequest::all_bound_surfaces()
                .with_virtualized_range(WorthUiVisibleRange::rows(0, 1).unwrap()),
        )
        .unwrap()
}

fn assert_exact_external_mechanics(
    transcript: &worth_ui_host_headless::UiHeadlessMountedFrameTranscript,
) {
    let effects = transcript.unperformed_effects();
    assert!(transcript.paint_batches().iter().any(|batch| {
        batch.primitive_kind()
            == worth_ui_runtime::facade::mounted::UiMountedPaintPrimitiveKind::CanvasSpatialBatch
    }));
    assert!(transcript.paint_batches().iter().any(|batch| {
        batch.primitive_kind()
            == worth_ui_runtime::facade::mounted::UiMountedPaintPrimitiveKind::RealtimeBatch
    }));
    assert!(effects.contains(
        &worth_ui_host_headless::UiHeadlessUnperformedEffect::CanvasSpatial {
            batch_index: 0,
            primitive_count: 64,
            hit_region_count: 0,
            overlay_row_count: 0,
            tool_state_row_count: 0,
        }
    ));
    assert!(effects.contains(
        &worth_ui_host_headless::UiHeadlessUnperformedEffect::Realtime {
            batch_index: 0,
            overlay_row_count: 2,
        }
    ));
}

fn prepare(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
) -> worth_ui_runtime::facade::mounted::UiPreparedMountedFrame {
    crate::mounted_geometry_fixture::install_current_occurrence_geometry(session);
    session
        .execute_framework_turn(|_| {})
        .expect("no mounted presentation lease is active")
        .into_execution()
        .unwrap_or_else(|_| panic!("empty source turn permits mounted execution"))
        .prepare_mounted_frame(UiMountedFrameRequest::all_bound_surfaces())
        .unwrap()
}

fn assert_rejected(
    context: &str,
    outcome: UiMountedFrameOutcome,
    expected: UiHostSurfacePresentationDenial,
) {
    let rejected = match outcome {
        UiMountedFrameOutcome::RejectedBeforeEffects(rejected) => rejected,
        UiMountedFrameOutcome::Published(_) => panic!("{context} unexpectedly published"),
        UiMountedFrameOutcome::Unchanged(_) => panic!("{context} unexpectedly reported unchanged"),
        UiMountedFrameOutcome::Reconciled(_) => panic!("{context} unexpectedly reconciled"),
        UiMountedFrameOutcome::InFlight(_) => panic!("{context} unexpectedly remained in flight"),
        UiMountedFrameOutcome::PresentationIndeterminate(_) => {
            panic!("{context} unexpectedly became indeterminate")
        }
        UiMountedFrameOutcome::RetentionDenied(_) => {
            panic!("{context} unexpectedly denied retention")
        }
        UiMountedFrameOutcome::AdmissionDenied(_) => {
            panic!("{context} unexpectedly denied runtime admission")
        }
        UiMountedFrameOutcome::CompletionDenied(_) => {
            panic!("{context} unexpectedly denied completion")
        }
        UiMountedFrameOutcome::Superseded(_) => panic!("{context} unexpectedly superseded"),
    };
    assert_eq!(rejected.rejections().len(), 1);
    assert_eq!(rejected.rejections()[0].denial(), expected);
}

fn paint_order(batch: &worth_ui_host_headless::UiHeadlessPaintBatchMechanic) -> (u8, u32, u16) {
    match batch.layer() {
        worth_ui_host_headless::UiHeadlessLayerMechanic::Ordered { semantic_order, .. } => {
            (0, semantic_order, batch.batch_index())
        }
        worth_ui_host_headless::UiHeadlessLayerMechanic::Omitted(_) => {
            (1, u32::MAX, batch.batch_index())
        }
    }
}
