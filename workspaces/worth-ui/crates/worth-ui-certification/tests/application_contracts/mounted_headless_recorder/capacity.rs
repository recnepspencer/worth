use super::*;
use worth_ui_test_support::WorthUiMountedIdentityCertificationExt;

#[test]
fn stale_binding_native_mode_and_capacity_deny_without_recording() {
    stale_binding_denies_at_runtime_authority_boundary();
    unsupported_native_mode_denies_before_recorder_effects();
    transcript_capacity_denies_before_recorder_effects();
    retained_capacity_recovers_after_drain();
}

#[test]
fn shutdown_releases_surface_capacity_for_reused_recorder() {
    let recorder = WorthUiHeadlessRecorder::new(UiHeadlessRecorderCapacity::new(1, 1, 4_096));
    let mut first = mounted_application_with_host("headless-release-first", recorder.clone())
        .launch()
        .unwrap();
    let first_surface = first.create_semantic_surface().unwrap();
    first
        .register_host_surface(
            first_surface,
            UiHostSurfacePresentationMode::RecordOnly,
            profile(1),
        )
        .unwrap();

    let shutdown = first.shutdown();
    assert!(matches!(
        shutdown.host_session_release(),
        Some(worth_ui_runtime::facade::host::UiHostSessionReleaseOutcome::Released(receipt))
            if receipt.released_surface_count() == 1
    ));

    let mut second = mounted_application_with_host("headless-release-second", recorder)
        .launch()
        .unwrap();
    let second_surface = second.create_semantic_surface().unwrap();
    assert!(second
        .register_host_surface(
            second_surface,
            UiHostSurfacePresentationMode::RecordOnly,
            profile(1),
        )
        .is_ok());
}

#[test]
fn dropped_session_releases_surface_capacity_for_reused_recorder() {
    let recorder = WorthUiHeadlessRecorder::new(UiHeadlessRecorderCapacity::new(1, 1, 4_096));
    let mut first = mounted_application_with_host("headless-drop-first", recorder.clone())
        .launch()
        .unwrap();
    let first_surface = first.create_semantic_surface().unwrap();
    first
        .register_host_surface(
            first_surface,
            UiHostSurfacePresentationMode::RecordOnly,
            profile(1),
        )
        .unwrap();
    drop(first);

    let mut second = mounted_application_with_host("headless-drop-second", recorder)
        .launch()
        .unwrap();
    let second_surface = second.create_semantic_surface().unwrap();
    assert!(second
        .register_host_surface(
            second_surface,
            UiHostSurfacePresentationMode::RecordOnly,
            profile(1),
        )
        .is_ok());
}

fn stale_binding_denies_at_runtime_authority_boundary() {
    let recorder = WorthUiHeadlessRecorder::default();
    let mut session = mounted_application_with_host("headless-stale-binding", recorder.clone())
        .launch()
        .unwrap();
    let surface = session.create_semantic_surface().unwrap();
    let stale = session
        .register_host_surface(
            surface,
            UiHostSurfacePresentationMode::RecordOnly,
            profile(1),
        )
        .unwrap()
        .binding_generation();
    let node = first_node(&session);
    session.mount_instance(node, surface).unwrap();
    let candidate = prepare(&mut session);
    session
        .rebind_host_surface(stale, UiHostSurfacePresentationMode::RecordOnly, profile(2))
        .unwrap();

    let outcome =
        session.present_prepared_mounted_frame(candidate, UiPresentationDeadline::at_tick(10), 0);
    match outcome {
        UiMountedFrameOutcome::AdmissionDenied(rejection) => assert_eq!(
            rejection.denial(),
            worth_ui_runtime::facade::mounted::UiMountedPresentationAdmissionDenial::PreparedFrameBasisChanged
        ),
        _ => panic!("stale binding basis must deny before host admission"),
    }
    assert!(recorder.observed_transcripts().is_empty());
}

fn unsupported_native_mode_denies_before_recorder_effects() {
    let recorder = WorthUiHeadlessRecorder::default();
    let mut session = mounted_application_with_host("headless-native-denial", recorder.clone())
        .launch()
        .unwrap();
    let surface = session.create_semantic_surface().unwrap();
    session
        .register_host_surface(
            surface,
            UiHostSurfacePresentationMode::NativeDisplay,
            profile(1),
        )
        .unwrap();
    let node = first_node(&session);
    session.mount_instance(node, surface).unwrap();
    let candidate = prepare(&mut session);

    assert_rejected(
        "unsupported native mode",
        session.present_prepared_mounted_frame(candidate, UiPresentationDeadline::at_tick(10), 0),
        UiHostSurfacePresentationDenial::UnsupportedPresentationMode(
            UiHostSurfacePresentationMode::NativeDisplay,
        ),
    );
    assert!(recorder.observed_transcripts().is_empty());
}

fn transcript_capacity_denies_before_recorder_effects() {
    let recorder = WorthUiHeadlessRecorder::new(UiHeadlessRecorderCapacity::new(4, 1, 0));
    let mut session = mounted_application_with_host("headless-capacity", recorder.clone())
        .launch()
        .unwrap();
    let surface = session.create_semantic_surface().unwrap();
    session
        .register_host_surface(
            surface,
            UiHostSurfacePresentationMode::RecordOnly,
            profile(1),
        )
        .unwrap();
    let node = first_node(&session);
    session.mount_instance(node, surface).unwrap();
    let candidate = prepare(&mut session);

    assert_rejected(
        "zero mechanic capacity",
        session.present_prepared_mounted_frame(candidate, UiPresentationDeadline::at_tick(10), 0),
        UiHostSurfacePresentationDenial::CapacityExceeded,
    );
    assert!(recorder.observed_transcripts().is_empty());
}

fn retained_capacity_recovers_after_drain() {
    let recorder = WorthUiHeadlessRecorder::new(UiHeadlessRecorderCapacity::new(4, 1, 4_096));
    let mut session = mounted_application_with_host("headless-retention", recorder.clone())
        .launch()
        .unwrap();
    let surface = session.create_semantic_surface().unwrap();
    session
        .register_host_surface(
            surface,
            UiHostSurfacePresentationMode::RecordOnly,
            profile(1),
        )
        .unwrap();
    let node = first_node(&session);
    let graph_node = node.graph_node_identity();
    let mounted = session.mount_instance(node, surface).unwrap();
    let first = prepare(&mut session);
    assert!(matches!(
        session.present_prepared_mounted_frame(first, UiPresentationDeadline::at_tick(10), 0,),
        UiMountedFrameOutcome::Published(_)
    ));
    session.unmount_instance(mounted).unwrap();
    let current_node = session
        .mounted_graph_node(graph_node)
        .expect("the remounted occurrence retains its graph node");
    session.mount_instance(current_node, surface).unwrap();
    let blocked = prepare(&mut session);
    assert_rejected(
        "retained transcript capacity",
        session.present_prepared_mounted_frame(blocked, UiPresentationDeadline::at_tick(20), 1),
        UiHostSurfacePresentationDenial::CapacityExceeded,
    );
    assert_eq!(recorder.observed_transcripts().len(), 1);
    assert_eq!(recorder.drain_transcripts().len(), 1);
    assert!(recorder.observed_transcripts().is_empty());
    let recovered = prepare(&mut session);
    assert!(matches!(
        session.present_prepared_mounted_frame(recovered, UiPresentationDeadline::at_tick(30), 2,),
        UiMountedFrameOutcome::Published(_)
    ));
}
