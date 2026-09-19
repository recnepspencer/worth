use std::time::Duration;
use worth_ui::facade::observation_report::WorthUiHostObservationSessionExt;
use worth_ui_test_support::WorthUiFrameworkTurnCertificationExt;
use worth_ui_test_support::WorthUiMountedIdentityCertificationExt;

use worth_ui::facade::observation_report::{
    UiHostObservationLoss, UiHostObservationReportDenial, UiHostObservationReportOutcome,
};
use worth_ui::facade::source::{WorthUiFilesystemSourceProvider, WorthUiFilesystemSourceWatcher};
use worth_ui_certification::scenario::filesystem_application_lifecycle::FilesystemApplicationLifecycleScenario;
use worth_ui_host_headless::{UiHeadlessRecorderCapacity, WorthUiHeadlessRecorder};
use worth_ui_runtime::facade::mounted::{
    UiHostSurfacePresentationDenial, UiHostSurfacePresentationMode, UiMountedAllocationProjection,
    UiMountedEffectFamily, UiMountedFrameOutcome, UiMountedOmissionReason,
    UiMountedPresentationAdmissionDenial, UiPresentationDeadline, UiSurfaceBindingGeneration,
};
use worth_ui_test_support::{
    WorthUiMountedFrameExecutionCertificationExt, WorthUiMountedPublicationCertificationExt,
};

use super::mounted_successor::{
    admit_query_projection, all_lane_request, establish_first_allocation_catalog, mount_all_nodes,
    publish_all_lane_frame,
};
use crate::filesystem_contract_workspace::FilesystemContractWorkspace;
use crate::host_observation_fixture::{batch, pointer, report, source};
use crate::mounted_application_lifecycle::known_empty_surface_world::profile;
use crate::mounted_application_lifecycle::published_mounted_world::{
    presented_epoch, PresentedObservationBasis,
};

const SETTLEMENT_TIMEOUT: Duration = Duration::from_secs(5);
const SOURCE: &str = "app/main.wui";

#[test]
fn one_real_predecessor_survives_ordered_hostile_seams_until_each_is_resolved() {
    let workspace = FilesystemContractWorkspace::new("phase-10-hostile-mounted");
    let source_text = FilesystemApplicationLifecycleScenario::preview_cross_lane_source_text(false);
    workspace.write(SOURCE, &source_text);
    let mut watcher = WorthUiFilesystemSourceWatcher::start(WorthUiFilesystemSourceProvider::new(
        workspace.root(),
    ))
    .expect("production watcher registers the hostile real-file world");
    let recorder = WorthUiHeadlessRecorder::with_viewport_extent(
        UiHeadlessRecorderCapacity::production_default(),
        worth_ui::facade::measurement_exchange::UiViewportExtentObservation {
            width: 1024.0,
            height: 768.0,
        },
    );
    let mut scenario = FilesystemApplicationLifecycleScenario::new("phase-10-hostile-mounted");
    let capabilities = scenario.preview_cross_lane_capability_application(recorder.clone());
    let initial = watcher
        .take_initial_snapshot()
        .expect("watcher owns the initial settled bytes");
    let submission = FilesystemApplicationLifecycleScenario::lower_snapshot(
        initial,
        capabilities.capabilities(),
    );
    let mut session = scenario
        .prepare_preview_cross_lane_application_with_host(submission, recorder.clone())
        .launch()
        .expect("hostile real-file application launches");

    let (allocation_root, original_instance) = mount_all_nodes(&mut session);
    establish_first_allocation_catalog(&mut session);
    admit_query_projection(&mut scenario, &mut session);
    let predecessor = publish_all_lane_frame(&mut session);

    let stale = prepare_all_lane_frame(&mut session);
    assert!(stale.surfaces().iter().any(|surface| {
        surface.projection().nodes().iter().any(|node| {
            matches!(
                node.allocation(),
                UiMountedAllocationProjection::Known { .. }
                    | UiMountedAllocationProjection::PortalAnchorObservation { .. }
                    | UiMountedAllocationProjection::Omitted(
                        UiMountedOmissionReason::AllocationBoundsUnknown
                    )
            )
        })
    }));
    let surface = session
        .inspect_mounted_identity()
        .mounted_instances()
        .iter()
        .find(|entry| entry.identity() == original_instance)
        .expect("original mounted instance remains current")
        .basis()
        .semantic_surface_identity();
    let mut reordered = session
        .inspect_mounted_identity()
        .mounted_instances()
        .iter()
        .map(|entry| entry.identity())
        .collect::<Vec<_>>();
    reordered.reverse();
    session.reorder_mounted_instances(&reordered).unwrap();
    session.unmount_instance(original_instance).unwrap();
    let remounted = session
        .mount_instance(
            session.mounted_graph_node(allocation_root).unwrap(),
            surface,
        )
        .unwrap();
    assert_ne!(original_instance, remounted);
    assert_prepared_frame_stale(&mut session, stale, &predecessor);
    let after_remount = publish_all_lane_frame(&mut session);
    assert_eq!(after_remount.predecessor(), Some(predecessor.frame()));

    let active_binding =
        session.inspect_mounted_identity().surface_bindings()[0].binding_generation();
    let unsupported_surface = session.create_semantic_surface().unwrap();
    let unsupported_binding = session
        .register_host_surface(
            unsupported_surface,
            UiHostSurfacePresentationMode::NativeDisplay,
            profile(2),
        )
        .unwrap()
        .binding_generation();
    let unsupported_instance = session
        .mount_instance(
            session.mounted_graph_node(allocation_root).unwrap(),
            unsupported_surface,
        )
        .unwrap();
    let unsupported = prepare_all_lane_frame(&mut session);
    let transcript_count = recorder.observed_transcripts().len();
    let outcome =
        session.present_prepared_mounted_frame(unsupported, UiPresentationDeadline::at_tick(30), 2);
    let UiMountedFrameOutcome::RejectedBeforeEffects(rejected) = outcome else {
        panic!("unsupported headless native surface must reject before effects");
    };
    assert!(rejected.rejections().iter().any(|entry| {
        entry.denial()
            == UiHostSurfacePresentationDenial::UnsupportedEffect(
                UiMountedEffectFamily::CanvasSpatial,
            )
    }));
    assert_eq!(recorder.observed_transcripts().len(), transcript_count);
    assert_eq!(session.current_mounted_publication(), Some(&after_remount));
    session.unmount_instance(unsupported_instance).unwrap();
    assert_eq!(
        session
            .deregister_host_surface(unsupported_binding)
            .unwrap(),
        unsupported_surface
    );
    let after_surface_restore = publish_all_lane_frame(&mut session);
    assert_eq!(
        after_surface_restore.predecessor(),
        Some(after_remount.frame())
    );

    prove_observation_denials_are_terminal(
        &mut session,
        active_binding,
        remounted,
        &after_surface_restore,
    );
    assert_eq!(
        session.current_mounted_publication(),
        Some(&after_surface_restore)
    );

    workspace.write_atomic(SOURCE, "not a valid Worth UI declaration");
    let denied_source = watcher
        .settle(SETTLEMENT_TIMEOUT)
        .expect("invalid stable bytes remain observable source truth");
    assert!(denied_source
        .attempt_candidate_for_certification(session.capabilities())
        .is_err());
    assert_eq!(
        session.current_mounted_publication(),
        Some(&after_surface_restore)
    );
    workspace.write_atomic(SOURCE, &source_text);
    let restored_source = watcher
        .settle(SETTLEMENT_TIMEOUT)
        .expect("restored real bytes settle independently");
    assert!(restored_source
        .attempt_candidate_for_certification(session.capabilities())
        .is_ok());
    let live = publish_all_lane_frame(&mut session);
    assert_eq!(live.predecessor(), Some(after_surface_restore.frame()));

    drop(session.shutdown());
    let watcher_shutdown = watcher.shutdown().expect("watcher unregisters");
    assert!(watcher_shutdown.observed_notification_count() >= 2);
    workspace.close();
}

fn prepare_all_lane_frame(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
) -> worth_ui_runtime::facade::mounted::UiPreparedMountedFrame {
    crate::mounted_geometry_fixture::install_current_occurrence_geometry(session);
    session
        .execute_framework_turn(|_| {})
        .expect("no presentation lease is active")
        .into_execution()
        .unwrap_or_else(|_| panic!("hostile world reaches mounted execution"))
        .prepare_mounted_frame(all_lane_request())
        .expect("hostile world prepares a complete all-lane frame")
}

fn assert_prepared_frame_stale(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
    stale: worth_ui_runtime::facade::mounted::UiPreparedMountedFrame,
    predecessor: &worth_ui_runtime::facade::mounted::UiMountedFramePublicationReceipt,
) {
    let outcome =
        session.present_prepared_mounted_frame(stale, UiPresentationDeadline::at_tick(20), 1);
    assert!(matches!(
        outcome,
        UiMountedFrameOutcome::AdmissionDenied(rejection)
            if rejection.denial()
                == UiMountedPresentationAdmissionDenial::PreparedFrameBasisChanged
    ));
    assert_eq!(session.current_mounted_publication(), Some(predecessor));
}

fn prove_observation_denials_are_terminal(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
    binding: UiSurfaceBindingGeneration,
    instance: worth_ui_runtime::facade::mounted::UiMountedInstanceIdentity,
    publication: &worth_ui_runtime::facade::mounted::UiMountedFramePublicationReceipt,
) {
    let inspection = session.inspect_mounted_identity();
    let receipt = inspection
        .frame_receipts()
        .iter()
        .find(|entry| {
            entry.frame_identity() == publication.frame()
                && entry.mounted_instance_identity() == instance
        })
        .expect("remounted instance has a current frame receipt")
        .node_receipt_identity();
    let basis = PresentedObservationBasis {
        frame: publication.frame(),
        epoch: presented_epoch(session, publication.frame(), binding),
        instance,
        receipt,
    };
    drop(inspection);
    let first = batch(
        source(session, binding, &basis),
        (1, 1),
        UiHostObservationLoss::Complete,
        vec![report(1, pointer(1, 10), &basis)],
    );
    assert!(matches!(
        session.validate_host_observation_batch(first.clone()),
        UiHostObservationReportOutcome::Validated(_)
    ));
    assert!(matches!(
        session.validate_host_observation_batch(first),
        UiHostObservationReportOutcome::Duplicate(_)
    ));
    assert_valid_observation(session, binding, &basis, 2);

    let foreign = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let foreign_batch = batch(
        source(session, foreign, &basis),
        (3, 3),
        UiHostObservationLoss::Complete,
        vec![report(3, pointer(3, 30), &basis)],
    );
    assert_eq!(
        session.validate_host_observation_batch(foreign_batch),
        UiHostObservationReportOutcome::Denied(UiHostObservationReportDenial::BindingNotPresented)
    );
    assert_valid_observation(session, binding, &basis, 3);
}

fn assert_valid_observation(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
    binding: UiSurfaceBindingGeneration,
    basis: &PresentedObservationBasis,
    sequence: u64,
) {
    let raw = batch(
        source(session, binding, basis),
        (sequence, sequence),
        UiHostObservationLoss::Complete,
        vec![report(sequence, pointer(sequence, 10), basis)],
    );
    assert!(matches!(
        session.validate_host_observation_batch(raw),
        UiHostObservationReportOutcome::Validated(_)
    ));
}
