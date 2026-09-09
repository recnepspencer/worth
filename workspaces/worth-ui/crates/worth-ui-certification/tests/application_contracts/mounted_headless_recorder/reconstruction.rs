use worth_ui::facade::measurement_exchange::UiViewportExtentObservation;
use worth_ui_host_headless::{
    UiHeadlessMountedFrameTranscript, UiHeadlessRecorderCapacity, WorthUiHeadlessRecorder,
};
use worth_ui_runtime::facade::mounted::{
    UiHostSurfacePresentationMode, UiMountedFrameOutcome, UiMountedFrameRequest,
    UiPresentationDeadline,
};
use worth_ui_test_support::{
    WorthUiFrameworkTurnCertificationExt, WorthUiMountedFrameExecutionCertificationExt,
    WorthUiMountedIdentityCertificationExt, WorthUiMountedPublicationCertificationExt,
};

use super::super::mounted_application_lifecycle::known_empty_surface_world::profile;
use super::super::mounted_static_paint::{establish_allocation, launch_and_mount_pulse};

#[test]
fn missing_surface_state_reconstructs_from_mounted_authority_then_returns_to_local_delta() {
    let recorder = WorthUiHeadlessRecorder::with_viewport_extent(
        UiHeadlessRecorderCapacity::production_default(),
        UiViewportExtentObservation {
            width: 160.0,
            height: 96.0,
        },
    );
    let (mut session, _, _, _, _) = launch_and_mount_pulse(recorder.clone());
    establish_allocation(&mut session);
    let initial = execute(
        &mut session,
        10,
        UiMountedFrameRequest::all_bound_surfaces(),
        2,
    );
    assert_eq!(initial.cost_report().adapter().draw_list_mutations(), 0);
    let first = one(recorder.drain_transcripts());

    let second_surface = session.create_semantic_surface().unwrap();
    let second_binding = session
        .register_host_surface(
            second_surface,
            UiHostSurfacePresentationMode::RecordOnly,
            profile(2),
        )
        .unwrap()
        .binding_generation();
    let nodes = session.graph().node_identities().collect::<Vec<_>>();
    let second_instances = nodes
        .iter()
        .map(|node| {
            let handle = session.mounted_graph_node(*node).unwrap();
            session.mount_instance(handle, second_surface).unwrap()
        })
        .collect::<Vec<_>>();
    let reconstructed = execute(
        &mut session,
        20,
        UiMountedFrameRequest::exact_surfaces(vec![second_surface]),
        2,
    );
    let reconstruction_cost = reconstructed.cost_report().adapter();
    assert_eq!(reconstruction_cost.draw_list_mutations(), 2);
    assert_eq!(reconstruction_cost.order_mutations(), 2);
    assert_eq!(reconstruction_cost.logical_damage_regions(), 2);
    assert_eq!(reconstruction_cost.retained_command_scans(), 0);
    assert_eq!(reconstruction_cost.retained_command_clones(), 0);
    let rebuilt = one(recorder.drain_transcripts());
    assert_eq!(rebuilt.binding(), second_binding);
    assert_same_semantic_paint(&first, &rebuilt);

    let removed = rebuilt.filled_rects()[0].mounted_instance();
    assert!(second_instances.contains(&removed));
    session.unmount_instance(removed).unwrap();
    let delta = execute_after_removal(&mut session, second_surface);
    let delta_cost = delta.cost_report().adapter();
    assert_eq!(delta_cost.draw_list_mutations(), 1);
    assert_eq!(delta_cost.order_mutations(), 1);
    assert_eq!(delta_cost.retained_command_scans(), 0);
    assert_eq!(delta_cost.retained_command_clones(), 0);
    let local = one(recorder.drain_transcripts());
    assert_eq!(local.binding(), second_binding);
    assert_eq!(local.filled_rects().len(), 1);
    let _ = session.shutdown();
}

fn execute(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
    tick: u64,
    request: UiMountedFrameRequest,
    expected_rects: usize,
) -> worth_ui_runtime::facade::mounted::UiMountedFramePublicationReceipt {
    let prepared = prepare(session, request);
    assert_eq!(prepared.surfaces().len(), 1);
    assert_eq!(
        prepared.surfaces()[0]
            .projection()
            .filled_rects()
            .rows()
            .len(),
        expected_rects,
    );
    publish(session, prepared, tick)
}

fn execute_after_removal(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
    changed_surface: worth_ui_runtime::facade::mounted::UiSemanticSurfaceIdentity,
) -> worth_ui_runtime::facade::mounted::UiMountedFramePublicationReceipt {
    let prepared = prepare(
        session,
        UiMountedFrameRequest::exact_surfaces(vec![changed_surface]),
    );
    assert_eq!(prepared.surfaces().len(), 1);
    let changed = prepared
        .surfaces()
        .iter()
        .find(|surface| surface.projection().surface() == changed_surface)
        .expect("all-bound frame contains the reconstructed surface");
    assert_eq!(changed.projection().filled_rects().rows().len(), 1);
    publish(session, prepared, 30)
}

fn prepare(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
    request: UiMountedFrameRequest,
) -> worth_ui_runtime::facade::mounted::UiPreparedMountedFrame {
    crate::mounted_geometry_fixture::install_current_occurrence_geometry(session);
    session
        .execute_framework_turn(|_| {})
        .unwrap()
        .into_execution()
        .unwrap_or_else(|_| panic!("empty source turn permits mounted execution"))
        .prepare_mounted_frame(request)
        .unwrap()
}

fn publish(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
    prepared: worth_ui_runtime::facade::mounted::UiPreparedMountedFrame,
    tick: u64,
) -> worth_ui_runtime::facade::mounted::UiMountedFramePublicationReceipt {
    match session.present_prepared_mounted_frame(prepared, UiPresentationDeadline::at_tick(tick), 0)
    {
        UiMountedFrameOutcome::Published(publication) => publication,
        UiMountedFrameOutcome::Unchanged(_) => panic!("reconstruction journey was unchanged"),
        UiMountedFrameOutcome::Reconciled(_) => panic!("reconstruction journey reconciled"),
        UiMountedFrameOutcome::RejectedBeforeEffects(rejected) => {
            panic!(
                "reconstruction journey was rejected before effects: {:?}",
                (tick, rejected.rejections())
            )
        }
        UiMountedFrameOutcome::InFlight(_) => panic!("reconstruction journey remained in flight"),
        UiMountedFrameOutcome::PresentationIndeterminate(_) => {
            panic!("reconstruction journey became indeterminate")
        }
        UiMountedFrameOutcome::Superseded(_) => panic!("reconstruction journey was superseded"),
        UiMountedFrameOutcome::RetentionDenied(_) => {
            panic!("reconstruction journey was denied by retention")
        }
        UiMountedFrameOutcome::AdmissionDenied(_) => {
            panic!("reconstruction journey was denied at admission")
        }
        UiMountedFrameOutcome::CompletionDenied(_) => {
            panic!("reconstruction journey was denied at completion")
        }
    }
}

fn one(transcripts: Box<[UiHeadlessMountedFrameTranscript]>) -> UiHeadlessMountedFrameTranscript {
    assert_eq!(transcripts.len(), 1);
    transcripts.into_vec().pop().unwrap()
}

fn assert_same_semantic_paint(
    first: &UiHeadlessMountedFrameTranscript,
    rebuilt: &UiHeadlessMountedFrameTranscript,
) {
    let rows = |transcript: &UiHeadlessMountedFrameTranscript| {
        transcript
            .filled_rects()
            .iter()
            .map(|row| {
                (
                    row.bounds(),
                    row.clip_bounds(),
                    row.color(),
                    row.layer_semantic_order(),
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(rows(first), rows(rebuilt));
    assert_eq!(first.paint_order().len(), rebuilt.paint_order().len());
}
