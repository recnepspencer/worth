use std::time::Duration;
use worth_ui_test_support::WorthUiMountedIdentityCertificationExt;
use worth_ui_test_support::{
    WorthUiActiveSessionCertificationExt, WorthUiFrameworkTurnCertificationExt,
    WorthUiMountedPublicationCertificationExt,
};

use worth_ui::facade::app::{
    WorthUiMountedApplicationReplacementOutcome, WorthUiMountedReplacementPreparationOutcome,
};
use worth_ui::facade::measurement_exchange::UiViewportExtentObservation;
use worth_ui::facade::source::{WorthUiFilesystemSourceProvider, WorthUiFilesystemSourceWatcher};
use worth_ui_certification::scenario::application_authority_closure::candidate_catalog::admit_candidate_catalog;
use worth_ui_certification::scenario::filesystem_application_lifecycle::FilesystemApplicationLifecycleScenario;
use worth_ui_host_headless::{UiHeadlessRecorderCapacity, WorthUiHeadlessRecorder};
use worth_ui_runtime::facade::mounted::{
    UiMountedFrameOutcome, UiMountedLaneParticipation, UiPresentationDeadline,
    UiRequiredLaneContributionStatus,
};
use worth_ui_runtime::facade::{WorthUiMountedPreviewDisposition, WorthUiMountedPreviewOutcome};

use super::mixed_real_world::{
    admit_query_projection, all_lane_request, establish_first_allocation_catalog, mount_all_nodes,
    publish_all_lane_frame,
};
use crate::filesystem_contract_workspace::FilesystemContractWorkspace;
use crate::mounted_application_lifecycle::adapter_projection_world::{
    retire_query, submit_preview,
};

const SETTLEMENT_TIMEOUT: Duration = Duration::from_secs(5);
const SOURCE: &str = "app/main.wui";

#[path = "mounted_successor_transcript_assertions.rs"]
mod transcript_assertions;

use transcript_assertions::assert_translated_cross_lane_frame;

#[test]
fn real_file_mount_measure_preview_and_watcher_edit_publish_one_mounted_successor() {
    let workspace = FilesystemContractWorkspace::new("phase-10-mounted-successor");
    workspace.write(
        SOURCE,
        &FilesystemApplicationLifecycleScenario::preview_cross_lane_source_text(false),
    );
    let mut watcher = WorthUiFilesystemSourceWatcher::start(WorthUiFilesystemSourceProvider::new(
        workspace.root(),
    ))
    .expect("production watcher registers the real .wui tree");
    let recorder = WorthUiHeadlessRecorder::with_viewport_extent(
        UiHeadlessRecorderCapacity::production_default(),
        UiViewportExtentObservation {
            width: 1024.0,
            height: 768.0,
        },
    );
    let mut scenario = FilesystemApplicationLifecycleScenario::new("phase-10-mounted-successor");
    let capabilities = scenario.preview_cross_lane_capability_application(recorder.clone());
    let initial = watcher
        .take_initial_snapshot()
        .expect("watcher owns the initial settled file bytes");
    let submission = FilesystemApplicationLifecycleScenario::lower_snapshot(
        initial,
        capabilities.capabilities(),
    );
    let mut session = scenario
        .prepare_preview_cross_lane_application_with_host(submission, recorder.clone())
        .launch()
        .expect("real file-authored splitter cross-lane application launches");

    let (preview_target, preview_instance) = mount_all_nodes(&mut session);
    establish_first_allocation_catalog(&mut session);
    admit_query_projection(&mut scenario, &mut session);
    let preview = publish_preview(&mut session, preview_target, preview_instance);
    let ordinary = publish_all_lane_frame(&mut session);
    assert_eq!(ordinary.predecessor(), Some(preview.frame()));
    assert_translated_cross_lane_frame(&recorder, ordinary.frame());
    let rust_authored =
        publish_equivalent_rust_authored_successor(&mut scenario, &mut session, &ordinary);
    assert_translated_cross_lane_frame(&recorder, rust_authored.frame());

    workspace.write_atomic(
        SOURCE,
        &FilesystemApplicationLifecycleScenario::preview_cross_lane_source_text(true),
    );
    let settled = watcher
        .settle(SETTLEMENT_TIMEOUT)
        .expect("real operating-system watcher settles the atomic edit");
    let submission =
        FilesystemApplicationLifecycleScenario::lower_snapshot(settled, session.capabilities());
    let mut prepared = session
        .prepare_replacement(submission)
        .expect("watcher successor prepares through the public session");
    prepared
        .admit_candidate_settled_query_projection(scenario.settled_query_projection())
        .expect("candidate independently admits its exact settled Query projection");
    let catalog = admit_candidate_catalog(&session, &mut prepared);
    let lowered = session
        .lower_prepared_replacement(*prepared)
        .expect("watcher successor lowers");
    let pending = session
        .stage_prepared_replacement(lowered)
        .expect("watcher successor stages");
    let boundary = session
        .execute_framework_turn(|_| {})
        .expect("no mounted presentation is active")
        .into_execution()
        .unwrap_or_else(|_| panic!("empty turn yields replacement activation authority"))
        .into_activation_boundary();
    let replacement = match session
        .prepare_mounted_replacement(pending, catalog, boundary, None, all_lane_request())
        .expect("mounted watcher successor prepares")
    {
        WorthUiMountedReplacementPreparationOutcome::Prepared(replacement) => replacement,
        WorthUiMountedReplacementPreparationOutcome::SemanticNoOp(_) => {
            panic!("the authored successor component changes application meaning")
        }
    };
    assert_all_execution_lanes(replacement.frame());
    let (application, mounted) = match replacement.present(UiPresentationDeadline::at_tick(40), 3) {
        WorthUiMountedApplicationReplacementOutcome::Published {
            application,
            mounted,
        } => (application, mounted),
        _ => panic!("complete headless presentation must publish one mounted successor"),
    };

    assert_eq!(
        application.active_generation(),
        session.generation_identity()
    );
    assert_eq!(mounted.generation(), session.generation_identity());
    assert_eq!(mounted.predecessor(), Some(rust_authored.frame()));
    assert_eq!(session.current_mounted_publication(), Some(&mounted));
    assert!(session
        .inspect_mounted_identity()
        .mounted_instances()
        .iter()
        .any(|receipt| receipt.identity() == preview_instance));
    assert_translated_cross_lane_frame(&recorder, mounted.frame());

    retire_query(&mut scenario, application.into_operation_live_retirement());
    retire_query(
        &mut scenario,
        session.shutdown().into_operation_live_retirement(),
    );
    let watcher_shutdown = watcher
        .shutdown()
        .expect("production watcher unregisters independently");
    assert!(watcher_shutdown.observed_notification_count() > 0);
    workspace.close();
}

fn publish_equivalent_rust_authored_successor(
    scenario: &mut FilesystemApplicationLifecycleScenario,
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
    predecessor: &worth_ui_runtime::facade::mounted::UiMountedFramePublicationReceipt,
) -> worth_ui_runtime::facade::mounted::UiMountedFramePublicationReceipt {
    crate::mounted_geometry_fixture::install_current_occurrence_geometry(session);
    let submission = FilesystemApplicationLifecycleScenario::preview_cross_lane_rust_submission(
        session.capabilities(),
    );
    let mut prepared = session
        .prepare_replacement(submission)
        .expect("equivalent Rust-authored candidate prepares");
    let active_nodes = session.graph().node_identities().collect::<Vec<_>>();
    let candidate_nodes = prepared
        .candidate_graph()
        .node_identities()
        .collect::<Vec<_>>();
    assert_eq!(
        session
            .graph()
            .compare_to(prepared.candidate_graph())
            .kind(),
        worth_ui::facade::graph::UiGraphWorldDifferenceKind::SameWorldSuccessor,
        "equivalent Rust meaning is rebased as the direct graph successor; \
         active={active_nodes:?}, candidate={candidate_nodes:?}"
    );
    prepared
        .admit_candidate_settled_query_projection(scenario.settled_query_projection())
        .expect("equivalent Rust candidate admits the same settled Query projection");
    let catalog = admit_candidate_catalog(&*session, &mut prepared);
    let lowered = session
        .lower_prepared_replacement(*prepared)
        .expect("equivalent Rust candidate lowers");
    let pending = session
        .stage_prepared_replacement(lowered)
        .expect("equivalent Rust candidate stages");
    let boundary = session
        .execute_framework_turn(|_| {})
        .expect("no mounted presentation is active")
        .into_execution()
        .unwrap_or_else(|_| panic!("empty turn yields replacement activation authority"))
        .into_activation_boundary();
    let outcome = session
        .prepare_mounted_replacement(pending, catalog, boundary, None, all_lane_request())
        .expect("equivalent Rust candidate reaches one mounted decision");
    let WorthUiMountedReplacementPreparationOutcome::Prepared(replacement) = outcome else {
        panic!("mounted eligibility makes the equivalent authored graph a prepared successor")
    };
    assert_all_execution_lanes(replacement.frame());
    let (application, mounted) = match replacement.present(UiPresentationDeadline::at_tick(30), 2) {
        WorthUiMountedApplicationReplacementOutcome::Published {
            application,
            mounted,
        } => (application, mounted),
        _ => panic!("equivalent Rust-authored mounted successor must publish"),
    };
    assert_eq!(mounted.predecessor(), Some(predecessor.frame()));
    assert_eq!(mounted.generation(), session.generation_identity());
    retire_query(scenario, application.into_operation_live_retirement());
    mounted
}

fn publish_preview(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
    target: worth_ui::facade::graph::UiGraphNodeIdentity,
    instance: worth_ui_runtime::facade::mounted::UiMountedInstanceIdentity,
) -> worth_ui_runtime::facade::mounted::UiMountedFramePublicationReceipt {
    crate::mounted_geometry_fixture::install_current_occurrence_geometry(session);
    // A preview paints over accepted layout, so the host shows it first.
    let layout = session
        .prepare_application_presentation_frame(all_lane_request())
        .expect("the installed layout prepares an ordinary frame");
    assert!(matches!(
        session.present_prepared_mounted_frame(layout, UiPresentationDeadline::at_tick(5), 0),
        UiMountedFrameOutcome::Published(_)
    ));
    let prepared = submit_preview(session, target, 320.0)
        .prepare(instance)
        .unwrap_or_else(|e| panic!("mounted splitter prepares preview: {:?}", e.denial()));
    let resolved = match prepared.present(UiPresentationDeadline::at_tick(10), 0) {
        WorthUiMountedPreviewOutcome::Resolved(resolved) => resolved,
        _ => panic!("headless preview resolves synchronously"),
    };
    match resolved.disposition() {
        WorthUiMountedPreviewDisposition::Published(publication) => publication.clone(),
        _ => panic!("mounted preview publishes before the first edit"),
    }
}

fn assert_all_execution_lanes(frame: &worth_ui_runtime::facade::mounted::UiPreparedMountedFrame) {
    for expected in [
        UiMountedLaneParticipation::Ordinary,
        UiMountedLaneParticipation::Virtualized,
        UiMountedLaneParticipation::CanvasSpatial,
        UiMountedLaneParticipation::Realtime,
    ] {
        assert!(frame.manifest().lane_contributions().iter().any(|cell| {
            cell.lane() == expected && cell.status() == UiRequiredLaneContributionStatus::Admitted
        }));
    }
}
