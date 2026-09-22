use std::any::TypeId;

use worth_ui::facade::app::WorthUi;
use worth_ui::facade::inspection::{
    UiClientPhysicalPixel, UiClientPhysicalRect, UiGeometryOnly, UiPixelsOptional,
    UiPixelsRequired, UiVisualArtifactPolicy, UiVisualCancellationPosture,
    UiVisualCaptureCancellation, UiVisualCaptureDeadline, UiVisualCapturePoll,
    UiVisualGrantLifetime, UiVisualGrantSurfaceScope, UiVisualHitTestOutcome,
    UiVisualInspectionAudience, UiVisualInspectionByteBudget, UiVisualInspectionCapacity,
    UiVisualInspectionPolicy, UiVisualInspectionRegionCapacity, UiVisualPixelRetentionDisposition,
    UiVisualSnapshotDenial, UiVisualSnapshotIndeterminate, UiVisualSnapshotOutcome,
    UiVisualSnapshotRequest, UiVisualVisibleOutcome,
};
use worth_ui_platform_pulse::visual_identity_pulse::{
    PlatformPulseVisualIdentityScenario, PLATFORM_PULSE_BACKGROUND_LOGICAL_POINT,
    PLATFORM_PULSE_CANONICAL_LOGICAL_EXTENT, PLATFORM_PULSE_IDENTITY_TARGET_AUTHORED_NAME,
    PLATFORM_PULSE_TARGET_LOGICAL_POINT, PLATFORM_PULSE_TARGET_RGB,
};
use worth_ui_runtime::facade::mounted::{
    UiMountedFrameOutcome, UiMountedInspectionReceipt, UiMountedInspectionRequest,
    UiPresentationDeadline,
};
use worth_ui_test_support::WorthUiMountedPublicationCertificationExt;

use super::mounted_application_lifecycle::in_flight_presentation_world::{
    mounted_session, prepared,
};
use super::mounted_host_protocol::scripted_host::ScriptedPresentationHost;

#[path = "visual_snapshot/comparison.rs"]
mod comparison;
#[path = "visual_snapshot/disclosure_evidence.rs"]
mod disclosure_evidence;
#[path = "visual_snapshot/phase_2_lifecycle.rs"]
mod phase_2_lifecycle;
#[path = "visual_snapshot/phase_2_outcomes.rs"]
mod phase_2_outcomes;
#[path = "visual_snapshot/resource_bounds.rs"]
mod resource_bounds;
#[path = "visual_snapshot/support.rs"]
mod support;
#[path = "visual_snapshot/target_lifecycle.rs"]
mod target_lifecycle;

use support::{
    complete_required_pixel_capture, current_target, immediate_pixel_capture, pending_host_capture,
};

#[test]
fn phase_1_public_contract_keeps_policy_coordinate_and_result_axes_distinct() {
    assert_eq!(
        TypeId::of::<<UiGeometryOnly as UiVisualArtifactPolicy>::CapturedPosture>(),
        TypeId::of::<UiGeometryOnly>()
    );
    assert_eq!(
        TypeId::of::<<UiPixelsOptional as UiVisualArtifactPolicy>::CapturedPosture>(),
        TypeId::of::<UiPixelsOptional>()
    );
    assert_eq!(
        TypeId::of::<<UiPixelsRequired as UiVisualArtifactPolicy>::CapturedPosture>(),
        TypeId::of::<UiPixelsRequired>()
    );
    assert_ne!(
        TypeId::of::<UiVisualVisibleOutcome>(),
        TypeId::of::<UiVisualHitTestOutcome>()
    );

    let region = UiClientPhysicalRect::new(48, 24, 112, 72).expect("valid half-open target");
    assert!(region.contains(UiClientPhysicalPixel::new(48, 24).unwrap()));
    assert!(region.contains(UiClientPhysicalPixel::new(111, 71).unwrap()));
    assert!(!region.contains(UiClientPhysicalPixel::new(112, 48).unwrap()));
    assert!(!region.contains(UiClientPhysicalPixel::new(80, 72).unwrap()));
}

#[test]
fn phase_1_request_and_product_scenario_preserve_explicit_inputs() {
    let request = UiVisualSnapshotRequest::for_local_development_unredacted_frame(7_u8)
        .artifacts(UiPixelsRequired::policy())
        .deadline(UiVisualCaptureDeadline::at_tick(41))
        .cancellation(UiVisualCaptureCancellation::new(19));
    assert_eq!(request.target(), &7);
    assert_eq!(request.capture_deadline().unwrap().tick(), 41);
    assert_eq!(
        request
            .cancellation_posture()
            .unwrap()
            .diagnostic_identity(),
        19
    );

    let scenario = PlatformPulseVisualIdentityScenario::canonical();
    assert_eq!(
        scenario.target_authored_name(),
        PLATFORM_PULSE_IDENTITY_TARGET_AUTHORED_NAME
    );
    // The scenario carries the product's own canonical geometry. Restating
    // those numbers here only proved that two copies agreed until the product
    // world changed extent, and then the copy outlived the world it described.
    assert_eq!(
        scenario.logical_extent(),
        PLATFORM_PULSE_CANONICAL_LOGICAL_EXTENT
    );
    assert_eq!(
        scenario.background_logical_point(),
        PLATFORM_PULSE_BACKGROUND_LOGICAL_POINT
    );
    assert_eq!(
        scenario.target_logical_point(),
        PLATFORM_PULSE_TARGET_LOGICAL_POINT
    );
    assert!(scenario.background_logical_point() != scenario.target_logical_point());
    assert!(
        scenario.target_logical_point()[0] < scenario.logical_extent()[0]
            && scenario.target_logical_point()[1] < scenario.logical_extent()[1],
        "the adjudicated target point must fall inside the captured extent"
    );
    assert_eq!(PLATFORM_PULSE_TARGET_RGB, [172, 103, 242]);
}

#[test]
fn phase_2_retained_frame_preserves_exact_host_presentation_epoch() {
    let host = ScriptedPresentationHost::default();
    let (mut session, bindings) = mounted_session(host.clone(), "visual-epoch-retention", 1);
    host.push_presented();
    let frame = prepared(&mut session);
    let outcome =
        session.present_prepared_mounted_frame(frame, UiPresentationDeadline::at_tick(10), 0);
    let published = match outcome {
        UiMountedFrameOutcome::Published(receipt) => receipt,
        _ => panic!("scripted presentation must publish"),
    };
    let inspected = match session.inspect_mounted_frame(UiMountedInspectionRequest::current()) {
        UiMountedInspectionReceipt::Available(frame) => frame,
        other => panic!("published frame must be retained, got {other:?}"),
    };
    assert_eq!(inspected.frame(), published.frame());
    assert_eq!(inspected.presentation().attempt(), published.attempt());
    assert_eq!(inspected.presentation().surfaces().len(), 1);
    let surface = &inspected.presentation().surfaces()[0];
    assert_eq!(surface.binding(), bindings[0]);
    assert_eq!(surface.epoch().diagnostic_value(), 1);
}

#[test]
fn phase_2_launch_seals_the_application_declared_visual_policy() {
    let policy = UiVisualInspectionPolicy::bounded(
        worth_ui::facade::inspection::UiVisualInspectionDisclosure::redacted(
            UiVisualInspectionAudience::DiagnosticAgent,
        ),
        UiVisualInspectionCapacity::bounded(3, 7, 11),
        UiVisualInspectionRegionCapacity::bounded(17, 19),
        UiVisualInspectionByteBudget::bounded(4_096, 8_192, 16_384, 32_768),
    )
    .expect("the scenario policy is valid");
    let session = WorthUi::app()
        .with_change_profile(worth_ui::facade::rebind::UiChangeProfile::platform_pulse())
        .with_visual_inspection_policy(policy)
        .freeze()
        .map(
            worth_ui_runtime::facade::entry::WorthUiCertificationApplicationTransition::activate_headless,
        )
        .expect("the application prepares")
        .launch()
        .expect("the application launches");

    assert_eq!(session.visual_inspection_authority().policy(), policy);
    assert_eq!(
        session.visual_inspection_authority().policy().audience(),
        UiVisualInspectionAudience::DiagnosticAgent
    );
    let pixel_scope = session
        .visual_inspection_authority()
        .issue_pixel_grant()
        .scope();
    assert_eq!(
        pixel_scope.audience(),
        UiVisualInspectionAudience::DiagnosticAgent
    );
    assert_eq!(
        pixel_scope.surfaces(),
        UiVisualGrantSurfaceScope::RegisteredApplicationSurfaces
    );
    assert_eq!(pixel_scope.lifetime(), UiVisualGrantLifetime::ActiveSession);
    assert_eq!(pixel_scope.maximum_snapshot_count(), 3);
    assert_eq!(pixel_scope.maximum_capture_bytes(), 4_096);
    assert_eq!(pixel_scope.maximum_retained_pixel_bytes(), 8_192);
    assert_eq!(
        pixel_scope.maximum_retained_structural_bytes_per_receipt(),
        16_384
    );
    assert_eq!(
        pixel_scope.maximum_retained_structural_bytes_per_session(),
        32_768
    );
    assert_eq!(pixel_scope.maximum_visible_region_records(), 17);
    assert_eq!(pixel_scope.maximum_hit_test_region_records(), 19);
}

#[test]
fn phase_2_capture_pins_and_reads_the_exact_presented_surface() {
    let host = ScriptedPresentationHost::default();
    host.set_visual_capture_capability(worth_ui_host_contract::UiHostCaptureCapability::Pixels {
        maximum_bytes: 1_024,
        exact_presentation_epoch: true,
    });
    let (mut session, _) = mounted_session(host.clone(), "visual-capture-exact-surface", 1);
    host.push_presented();
    let frame = prepared(&mut session);
    let published =
        match session.present_prepared_mounted_frame(frame, UiPresentationDeadline::at_tick(10), 0)
        {
            UiMountedFrameOutcome::Published(receipt) => receipt,
            _ => panic!("the scripted world publishes"),
        };
    let inspected = match session.inspect_mounted_frame(UiMountedInspectionRequest::current()) {
        UiMountedInspectionReceipt::Available(frame) => frame,
        other => panic!("the published frame is inspectable, got {other:?}"),
    };
    let target = inspected
        .current_visual_target()
        .expect("one current presented surface is unambiguous");
    let receipt = complete_required_pixel_capture(&mut session, &host, target);
    assert_eq!(
        receipt.affinity().frame(),
        published.frame().diagnostic_value()
    );
    assert_eq!(receipt.pixel_artifact().bytes().len(), 8);
    assert_eq!(receipt.coordinates().native_client_origin(), [17, 23]);
    assert_eq!(receipt.coordinates().scale(), [1.6, 2.0]);
    let calls = host.visual_capture_calls();
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0], calls[1]);
    assert_eq!(calls[0].frame(), published.frame());
    let disposed = session.dispose_visual_snapshot(receipt);
    assert!(disposed.released_registered_resource());
    assert_eq!(
        session
            .mounted_retention_report()
            .class(worth_ui_runtime::facade::mounted::UiMountedRetentionClass::VisualSnapshot)
            .active_leases(),
        0
    );
}

#[test]
fn phase_2_registry_enforces_one_capture_per_surface_before_host_effects() {
    let host = ScriptedPresentationHost::default();
    host.set_visual_capture_capability(worth_ui_host_contract::UiHostCaptureCapability::Pixels {
        maximum_bytes: 1_024,
        exact_presentation_epoch: true,
    });
    let (mut session, _) = mounted_session(host.clone(), "visual-one-in-flight", 1);
    host.push_presented();
    let frame = prepared(&mut session);
    let _ = session.present_prepared_mounted_frame(frame, UiPresentationDeadline::at_tick(10), 0);
    let target = current_target(&session);
    let second_target = current_target(&session);
    let grant = session.visual_inspection_authority().issue_pixel_grant();
    let pending = session
        .begin_visual_pixel_snapshot(
            &grant,
            UiVisualSnapshotRequest::for_local_development_unredacted_frame(target)
                .artifacts(UiPixelsRequired::policy()),
        )
        .expect("the first capture registers");
    let denial = match session.begin_visual_pixel_snapshot(
        &grant,
        UiVisualSnapshotRequest::for_local_development_unredacted_frame(second_target)
            .artifacts(UiPixelsRequired::policy()),
    ) {
        Err(denial) => denial,
        Ok(_) => panic!("one surface cannot own two in-flight captures"),
    };
    assert_eq!(denial, UiVisualSnapshotDenial::CapacityExceeded);
    assert!(host.visual_capture_calls().is_empty());
    let cancelled = session.cancel_visual_snapshot(pending);
    assert_eq!(cancelled.host_readback_began(), Some(false));
}

#[test]
fn phase_2_shutdown_invalidates_registered_pixel_resources() {
    let (session, receipt) = immediate_pixel_capture("visual-shutdown-disposal");
    let evidence = receipt.evidence();
    assert_eq!(
        evidence.disclosure(),
        worth_ui::facade::inspection::UiVisualInspectionDisclosure::local_development_unredacted()
    );
    assert_eq!(receipt.pixel_artifact().bytes().len(), 8);
    let retained_structural_bytes = evidence.cost().retained_structural_bytes();
    let shutdown = session.shutdown();
    assert_eq!(shutdown.visual_capture().disposed_snapshot_count(), 1);
    assert_eq!(shutdown.visual_capture().disposed_pixel_bytes(), 8);
    assert_eq!(
        shutdown.visual_capture().disposed_structural_bytes(),
        retained_structural_bytes
    );
    assert!(receipt.pixel_artifact().bytes().is_empty());
    assert_eq!(
        receipt.evidence(),
        evidence,
        "managed shutdown invalidates bytes, not immutable correlation evidence"
    );
    assert_eq!(
        receipt.pixel_artifact().retention(),
        UiVisualPixelRetentionDisposition::Disposed
    );
}

#[test]
fn phase_2_cancellation_after_host_request_consumes_the_exact_request() {
    let (mut session, host, pending) = pending_host_capture("visual-cancel-after-request");
    host.set_visual_cancellation_outcome(
        worth_ui_host_contract::UiHostCaptureCancellationOutcome::ReadbackMayHaveBegun,
    );
    let capture_request = host.visual_capture_calls()[0];
    let cancelled = session.cancel_visual_snapshot(pending);

    assert_eq!(
        cancelled.posture(),
        UiVisualCancellationPosture::ReadbackMayHaveBegun
    );
    assert_eq!(cancelled.host_readback_began(), Some(true));
    assert_eq!(host.visual_cancellation_calls(), vec![capture_request]);
    assert_eq!(
        session
            .mounted_retention_report()
            .class(worth_ui_runtime::facade::mounted::UiMountedRetentionClass::VisualSnapshot)
            .active_leases(),
        0
    );
}

#[test]
fn phase_2_timeout_after_host_request_cleans_up_the_exact_request() {
    let (mut session, host, pending) = pending_host_capture("visual-timeout-after-request");
    let capture_request = host.visual_capture_calls()[0];
    let outcome = match session.poll_visual_snapshot(pending, 6) {
        UiVisualCapturePoll::Completed(outcome) => outcome,
        UiVisualCapturePoll::Pending(_) => panic!("the elapsed deadline terminalizes capture"),
    };
    assert!(matches!(
        outcome,
        UiVisualSnapshotOutcome::Indeterminate(
            UiVisualSnapshotIndeterminate::TimeoutAfterHostRequest
        )
    ));
    assert_eq!(host.visual_cancellation_calls(), vec![capture_request]);
}
