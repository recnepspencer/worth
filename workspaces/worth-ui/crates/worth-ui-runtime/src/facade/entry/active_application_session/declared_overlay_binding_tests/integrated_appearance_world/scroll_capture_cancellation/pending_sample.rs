//! Input cannot capture an older pose after accepting an async Scroll sample.
use super::*;
use crate::certification_support::ScriptedSurfaceCompletion;
use crate::facade::entry::active_application_session::scroll_chrome_ingress::UiScrollChromeIngressOutcome;
use crate::facade::entry::active_application_session::scroll_chrome_interaction::UiScrollChromeInteractionDenial;
use crate::runtime::interaction::UiHostInteractionIngressOutcome;

fn pending_sample() -> ScrollWorld {
    let mut scroll = chrome_world();
    assert!(matches!(
        scroll.wheel(ONE_NOTCH, one_notch_up(), 5),
        UiHostScrollObservationOutcome::Applied(_)
    ));
    settle_frame(&mut scroll, 6);
    scroll.world.host.push_in_flight(
        vec![ScriptedSurfaceCompletion::Presented(
            UiMountedSurfacePresentationCompletion::new(
                UiHostSurfacePresentationMode::NativeDisplay,
                UiHostPresentationEpoch::issued_by_host(100),
                UiMountedCompletedEffects::new(vec![UiMountedEffectFamily::NativePaint]),
                UiHostPresentationCostReport::default(),
            ),
        )],
        UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
    );
    super::super::scroll_settle_frame::settle_scripted_frame(&mut scroll, 7);
    assert!(scroll
        .world
        .session
        .mounted
        .motion_sample_presentation_pending());
    scroll
}

#[test]
fn input_drain_settles_newly_accepted_scroll_pixels_before_thumb_capture() {
    let mut scroll = pending_sample();
    let before = scroll.accepted_offset();
    let grabbed = centre(block_chrome(&scroll).1);
    let predecessor = scroll.presentation();
    let presentation = UiHostObservationPresentationBasis::new(
        predecessor.host_surface(),
        predecessor.frame(),
        predecessor.binding(),
        UiHostPresentationEpoch::issued_by_host(100),
    );
    let UiHostProtocolNegotiation::Compatible(protocol) =
        UiHostProtocolContract::current().negotiate()
    else {
        panic!("current protocol")
    };
    let sequence = UiHostObservationSequence::new(1);
    let batch = UiHostObservationBatch::new(UiHostObservationBatchInput {
        protocol,
        host_session: scroll.world.session.host_session.identity().as_u64(),
        presentation,
        sequences: UiHostObservationSequenceRange::new(sequence, sequence),
        loss: UiHostObservationLoss::Complete,
        reports: vec![UiHostObservationReport::new(
            sequence,
            UiHostObservationTimeBasis::HostMonotonicMillis(8),
            UiHostObservationPayload::PointerButton {
                pointer: UiHostPointerIdentity::new(POINTER),
                capture_epoch: UiHostPointerCaptureEpoch::new(CAPTURE_EPOCH),
                button: UiHostPointerButton::Primary,
                transition: UiHostPointerButtonTransition::Pressed,
                position: UiHostSurfacePosition::viewport_logical(
                    (grabbed[0] * 1000.0).round() as i64,
                    (grabbed[1] * 1000.0).round() as i64,
                ),
            },
        )
        .with_pointer_device_kind(UiHostPointerDeviceKind::Mouse)
        .unwrap()],
    })
    .unwrap();
    scroll.world.host.enqueue_observation_for_next_drain(batch);
    let outcomes = scroll
        .world
        .session
        .drain_and_admit_host_observation_batches(Default::default())
        .into_outcomes();
    let [UiHostInteractionIngressOutcome::Applied(receipt)] = outcomes.as_ref() else {
        panic!("applied press: {outcomes:?}")
    };
    assert!(matches!(
        receipt.scroll_chrome_interactions(),
        [UiScrollChromeIngressOutcome::Pressed(
            UiScrollChromePressOutcome::ThumbCaptured(_)
        )]
    ));
    let held = scroll.accepted_offset();
    assert!(held.block_subpixels() > before.block_subpixels());
    assert_eq!(scroll.displayed_offset(), Some(held));
    let (_, thumb) = block_chrome(&scroll);
    let latch = scroll
        .world
        .session
        .interaction
        .scroll_chrome_latch()
        .unwrap();
    assert!((latch.grab_offset_logical_points() - (grabbed[1] - thumb.y())).abs() < 0.001);
    assert_eq!(pending_transitions(&scroll), 0);
    let placed = drag(&mut scroll, [grabbed[0], grabbed[1] + 2.0]);
    assert!((placed.block_subpixels() - held.block_subpixels() - 10_000).abs() <= 1);
    scroll.publish_direct(9);
    assert_eq!(scroll.accepted_offset(), placed);
    assert!((block_chrome(&scroll).1.y() - thumb.y() - 2.0).abs() < 0.001);
    let _ = scroll.world.session.shutdown();
}

#[test]
fn capture_cannot_retire_an_accepted_sample_before_its_pose_is_reconciled() {
    let mut scroll = pending_sample();
    let grabbed = centre(block_chrome(&scroll).1);
    // Exercise the partial handoff directly: the mounted owner accepted pixels,
    // but the session has not yet reconciled its Scroll owner/geometry.
    let settlement = scroll
        .world
        .session
        .mounted
        .complete_motion_sample_presentation(&scroll.world.session.host_session);
    assert!(matches!(
        settlement,
        Some(crate::mounting::UiMountedMotionSampleSettlement::Committed(
            _
        ))
    ));
    let surface = scroll.surface();
    let binding = scroll.presentation().binding();
    let outcome = scroll.world.session.press_scroll_chrome(
        surface,
        grabbed,
        UiHostPointerIdentity::new(POINTER),
        UiHostPointerCaptureEpoch::new(CAPTURE_EPOCH),
        binding,
    );
    assert!(matches!(
        outcome,
        Err(UiScrollChromeInteractionDenial::AcceptedPoseUnsettled)
    ));
    assert!(scroll
        .world
        .session
        .interaction
        .scroll_chrome_latch()
        .is_none());
    assert_eq!(pending_transitions(&scroll), 1);
    assert!(scroll.world.session.mounted.has_active_motion_samples());
    let basis = scroll.presentation();
    assert_eq!(
        scroll.world.session.settle_accepted_scroll_sample(basis),
        UiScrollSettleDisposition::Applied
    );
    assert!(matches!(
        press(&mut scroll, grabbed),
        UiScrollChromePressOutcome::ThumbCaptured(_)
    ));
    let _ = scroll.world.session.shutdown();
}

#[test]
fn accepted_sample_completion_does_not_overwrite_staged_direct_geometry() {
    use crate::runtime::scroll::UiScrollDeltaCause;

    let mut declared = smooth_scroll(true);
    declared.region = declared
        .region
        .clone()
        .with_scroll_chrome(super::super::scroll_chrome_fixture::contract());
    let mut scroll = ScrollWorld::publish_with_nested_content(World::launch_with_scroll(declared));
    assert!(matches!(
        scroll.wheel(ONE_NOTCH, one_notch_up(), 5),
        UiHostScrollObservationOutcome::Applied(_)
    ));
    settle_frame(&mut scroll, 6);
    // The prior frame's accepted sample remains retained while direct input
    // stages a successor. An unrelated input cannot replay that old sample.
    let staged = UiScrollOffset::new(0, 10_000).unwrap();
    scroll
        .world
        .session
        .place_scroll_chrome_offset(
            scroll.owner,
            scroll.incarnation,
            scroll.target(),
            0,
            staged,
            UiScrollDeltaCause::ChromeTrackPage,
        )
        .unwrap();
    let presentation = scroll.presentation();
    scroll.world.host.enqueue_observation_for_next_drain(
        super::super::pointer_geometry::pointer_batch(
            scroll.world.session.host_session.identity().as_u64(),
            presentation,
            1,
            [150_000, 55_000],
        ),
    );
    let outcomes = scroll
        .world
        .session
        .drain_and_admit_host_observation_batches(Default::default())
        .into_outcomes();
    assert!(matches!(
        outcomes.as_ref(),
        [UiHostInteractionIngressOutcome::Applied(_)]
    ));
    assert_eq!(
        scroll
            .world
            .session
            .settle_accepted_scroll_sample(presentation),
        UiScrollSettleDisposition::DeferredPendingGeometry
    );
    scroll.publish_direct(9);
    assert_eq!(scroll.accepted_offset(), staged);
    let nested = scroll.world.instances[2];
    let presented = scroll
        .world
        .host
        .last_node_changes()
        .into_iter()
        .find_map(|change| match change {
            UiMountedPresentationNodeChange::Upsert(state)
                if state.mounted_instance() == nested =>
            {
                Some(state)
            }
            _ => None,
        })
        .expect("direct publication must present the nested scrolled occurrence");
    let UiMountedAllocationProjection::Known { bounds, .. } = presented.allocation() else {
        panic!("nested occurrence has an allocated host rectangle")
    };
    assert_eq!(bounds.y(), 52.0);
    if let UiMountedPresentationNodePaint::Command(command) = presented.paint() {
        assert!(
            scroll
                .world
                .host
                .last_appearance_samples()
                .iter()
                .all(|sample| {
                    sample.command() != command
                        || sample
                            .transform()
                            .is_none_or(|transform| transform.source() == transform.sampled())
                }),
            "a stale sample must not displace the host's direct geometry"
        );
    }
    assert_eq!(scroll.displayed_offset(), Some(staged));
    let _ = scroll.world.session.shutdown();
}
