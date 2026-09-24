use super::super::{UiNativeApplicationDriverCleanupCompletion, UiNativeDriverShutdownEvidence};

#[test]
fn immediate_and_retried_cleanup_emit_the_same_captured_shutdown_evidence() {
    let direct_query =
        crate::facade::entry::UiNativeApplicationQueryCloseObservation::empty_complete();
    let direct = shutdown_evidence().finalize(&direct_query);
    let deferred = UiNativeApplicationDriverCleanupCompletion {
        query_close: crate::facade::entry::UiNativeApplicationQueryCloseObservation::empty_complete(
        ),
        evidence: shutdown_evidence(),
    }
    .into_client_close();
    let worth_ui_host_native::UiNativeEventLoopClientClose::CompleteWithObservation(deferred) =
        deferred
    else {
        panic!("complete retried cleanup must retain its shutdown observation")
    };
    assert_eq!(direct, deferred);
    assert_eq!(deferred.observation_ingress().counts(), [2, 3, 5, 7, 11]);
    assert!(deferred.derived_state_reconstruction().is_some());
    assert_eq!(
        deferred
            .visual_snapshot()
            .map(|snapshot| snapshot.affinity()),
        Some([13, 17, 19, 23, 29, 31, 37])
    );
}

#[cfg(feature = "certification-support")]
#[test]
fn queued_host_readiness_overlaps_real_application_driver_shutdown() {
    use crate::certification_support::{ScriptedPresentationHost, ScriptedSurfaceCompletion};
    use crate::facade::mounted::{UiHostSurfaceCancellationOutcome, UiMountedFrameOutcome};
    use crate::runtime::tests::active_application_session_test_support::source_backed_component_app_with_host_and_viewport_allocation;

    let host = ScriptedPresentationHost::native_display();
    host.push_in_flight(
        vec![ScriptedSurfaceCompletion::Pending],
        UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
    );
    let mut shell = source_backed_component_app_with_host_and_viewport_allocation(host.clone())
        .launch_native_surface()
        .expect("native certification shell should launch");
    shell.observe_native_viewport_readiness([800, 600], 1_000, false);
    super::super::program_progress::layout::complete_program_layout(&mut shell).unwrap();
    let Ok(UiMountedFrameOutcome::InFlight(in_flight)) = shell.present_frame(2, 0) else {
        panic!("scripted host must retain a real in-flight presentation")
    };
    drop(in_flight);
    assert_eq!(host.native_in_flight_count(), 1);

    let driver = super::super::UiNativeApplicationDriver::from_launched_shell_for_test(shell);
    let certification = worth_ui_host_native::certify_client_close_with_queued_readiness(driver);

    assert!(certification.client_cleanup_complete());
    assert!(certification.readiness_closure_complete());
    assert!(certification
        .overlap()
        .crossed_queued_readiness_with_held_attempt());
    assert_eq!(
        certification
            .overlap()
            .queued_readiness_before_client_close(),
        1
    );
    let shutdown = certification
        .client_shutdown()
        .expect("application driver shutdown must report real client evidence");
    assert_eq!(shutdown.shutdown_attempts().len(), 1);
    assert_eq!(
        shutdown.shutdown_attempts()[0].disposition(),
        worth_ui_host_native::UiNativeClientShutdownAttemptDisposition::CancelledBeforeEffects
    );
    assert!(shutdown.managed_semantic_resources_complete());
    assert!(shutdown.intent_resources_empty());
    assert_eq!(shutdown.resources().terminal_mounted_layouts(), 0);
    assert_eq!(shutdown.resources().terminal_raster_cache_entries(), 0);
    assert_eq!(host.native_in_flight_count(), 0);
}

fn shutdown_evidence() -> UiNativeDriverShutdownEvidence {
    UiNativeDriverShutdownEvidence::captured(
        Some(
            worth_ui_host_native::UiNativeClientDerivedStateReconstructionObservation::reported(
                worth_ui_host_native::UiNativeClientDerivedStateLossClass::MountedLayouts,
                1,
                1,
                3,
                3,
            ),
        ),
        [2, 3, 5, 7, 11],
        Some(worth_ui_host_native::UiNativeClientVisualSnapshotObservation::reported(
            worth_ui_host_native::UiNativeClientVisualSnapshotInput {
                affinity: [13, 17, 19, 23, 29, 31, 37],
                relation: worth_ui_host_native::UiNativeClientVisualSnapshotRelation::Current,
                native_client_origin: [41, 43],
                client_physical_dimensions: [1, 1],
                viewport_logical_dimension_bits: [1.0_f32.to_bits(); 2],
                scale_bits: [1.0_f32.to_bits(); 2],
                translation_bits: [0.0_f32.to_bits(); 2],
                coordinate_orientation:
                    worth_ui_host_native::UiNativeClientVisualCoordinateOrientation::TopLeftOrigin,
                coordinate_rounding:
                    worth_ui_host_native::UiNativeClientVisualCoordinateRounding::PixelCenterNearest,
                pixel_dimensions: [1, 1],
                pixel_stride: 4,
                pixel_color_space:
                    worth_ui_host_native::UiNativeClientVisualPixelColorSpace::Srgb,
                pixel_bytes: Box::new([47, 53, 59, 61]),
                visible_region_count: 1,
                hit_test_region_count: 1,
                cost_counters: [0; 11],
            },
        )),
    )
}
