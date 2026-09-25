//! Input cannot capture an older pose after accepting an async Scroll sample.
use super::*;
use crate::certification_support::ScriptedPresentationAcknowledgement;
use crate::certification_support::ScriptedSurfaceCompletion;
use crate::facade::entry::active_application_session::scroll_chrome_ingress::UiScrollChromeIngressOutcome;
use crate::facade::entry::active_application_session::scroll_chrome_interaction::UiScrollChromeInteractionDenial;
use crate::runtime::interaction::UiHostInteractionIngressOutcome;

fn pending_sample() -> ScrollWorld {
    pending_sample_with(vec![presented_sample()])
}

pub(super) fn presented_sample() -> ScriptedSurfaceCompletion {
    ScriptedSurfaceCompletion::Presented(ScriptedPresentationAcknowledgement::new(
        UiHostSurfacePresentationMode::NativeDisplay,
        UiHostPresentationEpoch::issued_by_host(100),
        UiMountedCompletedEffects::new(vec![UiMountedEffectFamily::NativePaint]),
        UiHostPresentationCostReport::default(),
    ))
}

pub(super) fn pending_sample_with(completions: Vec<ScriptedSurfaceCompletion>) -> ScrollWorld {
    let mut scroll = chrome_world();
    assert!(matches!(
        scroll.wheel(ONE_NOTCH, one_notch_up(), 5),
        UiHostScrollObservationOutcome::Applied(_)
    ));
    settle_frame(&mut scroll, 6);
    scroll.world.host.push_in_flight(
        completions,
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

pub(super) fn old_epoch_press_batch(
    scroll: &ScrollWorld,
    grabbed: [f32; 2],
) -> UiHostObservationBatch {
    old_epoch_button_batch(scroll, grabbed, 1, UiHostPointerButtonTransition::Pressed)
}

fn old_epoch_button_batch(
    scroll: &ScrollWorld,
    point: [f32; 2],
    sequence_value: u64,
    transition: UiHostPointerButtonTransition,
) -> UiHostObservationBatch {
    let UiHostProtocolNegotiation::Compatible(protocol) =
        UiHostProtocolContract::current().negotiate()
    else {
        panic!("current protocol")
    };
    let sequence = UiHostObservationSequence::new(sequence_value);
    UiHostObservationBatch::new(UiHostObservationBatchInput {
        protocol,
        host_session: scroll.world.session.host_session.identity().as_u64(),
        presentation: scroll.presentation(),
        sequences: UiHostObservationSequenceRange::new(sequence, sequence),
        loss: UiHostObservationLoss::Complete,
        reports: vec![UiHostObservationReport::new(
            sequence,
            UiHostObservationTimeBasis::HostMonotonicMillis(8),
            UiHostObservationPayload::PointerButton {
                pointer: UiHostPointerIdentity::new(POINTER),
                capture_epoch: UiHostPointerCaptureEpoch::new(CAPTURE_EPOCH),
                button: UiHostPointerButton::Primary,
                transition,
                position: UiHostSurfacePosition::viewport_logical(
                    (point[0] * 1000.0).round() as i64,
                    (point[1] * 1000.0).round() as i64,
                ),
            },
        )
        .with_pointer_device_kind(UiHostPointerDeviceKind::Mouse)
        .unwrap()],
    })
    .unwrap()
}

#[test]
fn input_drain_settles_newly_accepted_scroll_pixels_before_thumb_capture() {
    let mut scroll = pending_sample();
    let before = scroll.accepted_offset();
    let grabbed = centre(block_chrome(&scroll).1);
    scroll
        .world
        .host
        .enqueue_observation_for_next_drain(old_epoch_press_batch(&scroll, grabbed));
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
            UiScrollChromePressOutcome::ThumbAwaitingPhysical
        )]
    ));
    let held = scroll.accepted_offset();
    assert!(held.block_subpixels() > before.block_subpixels());
    assert_eq!(scroll.mounted_offset(), Some(held));
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
fn old_epoch_press_waits_for_pending_physical_sample_then_grabs_accepted_thumb() {
    let mut scroll =
        pending_sample_with(vec![ScriptedSurfaceCompletion::Pending, presented_sample()]);
    let original = scroll.presentation();
    let before = scroll.accepted_offset();
    let grabbed = centre(block_chrome(&scroll).1);
    scroll
        .world
        .host
        .enqueue_observation_for_next_drain(old_epoch_press_batch(&scroll, grabbed));
    let outcomes = scroll
        .world
        .session
        .drain_and_admit_host_observation_batches(Default::default())
        .into_outcomes();
    let [UiHostInteractionIngressOutcome::Applied(receipt)] = outcomes.as_ref() else {
        panic!("old-epoch press must be validated before physical completion: {outcomes:?}")
    };
    assert!(matches!(
        receipt.scroll_chrome_interactions(),
        [UiScrollChromeIngressOutcome::Pressed(
            UiScrollChromePressOutcome::ThumbAwaitingPhysical
        )]
    ));
    assert_eq!(scroll.accepted_offset(), before);
    assert!(scroll
        .world
        .session
        .mounted
        .motion_sample_presentation_pending());
    assert!(scroll
        .world
        .session
        .interaction
        .scroll_chrome_latch()
        .is_none());
    assert!(scroll
        .world
        .session
        .interaction
        .scroll_chrome_latch_mut()
        .pending()
        .is_some());
    assert_eq!(pending_transitions(&scroll), 1);

    scroll.world.session.complete_motion_sample_presentation();
    let accepted = scroll.accepted_offset();
    assert!(accepted.block_subpixels() > before.block_subpixels());
    assert_eq!(scroll.mounted_offset(), Some(accepted));
    assert!(scroll
        .world
        .session
        .mounted
        .current_presentation_for_surface(scroll.surface())
        .is_some_and(|current| current.basis().epoch() > original.epoch()));
    let thumb = block_chrome(&scroll).1;
    let latch = scroll
        .world
        .session
        .interaction
        .scroll_chrome_latch()
        .unwrap();
    assert!((latch.grab_offset_logical_points() - (grabbed[1] - thumb.y())).abs() < 0.001);
    assert_eq!(pending_transitions(&scroll), 0);
    let placed = drag(&mut scroll, [grabbed[0], grabbed[1] + 2.0]);
    assert!((placed.block_subpixels() - accepted.block_subpixels() - 10_000).abs() <= 1);
    let _ = scroll.world.session.shutdown();
}

#[test]
fn release_before_physical_completion_places_final_pointer_position_once() {
    let mut scroll = pending_sample_with(vec![
        ScriptedSurfaceCompletion::Pending,
        ScriptedSurfaceCompletion::Pending,
        presented_sample(),
    ]);
    let grabbed = centre(block_chrome(&scroll).1);
    scroll
        .world
        .host
        .enqueue_observation_for_next_drain(old_epoch_press_batch(&scroll, grabbed));
    let press = scroll
        .world
        .session
        .drain_and_admit_host_observation_batches(Default::default())
        .into_outcomes();
    assert!(matches!(
        press.as_ref(),
        [UiHostInteractionIngressOutcome::Applied(_)]
    ));
    let release_point = [grabbed[0], grabbed[1] + 2.0];
    scroll
        .world
        .host
        .enqueue_observation_for_next_drain(old_epoch_button_batch(
            &scroll,
            release_point,
            2,
            UiHostPointerButtonTransition::Released,
        ));
    let release = scroll
        .world
        .session
        .drain_and_admit_host_observation_batches(Default::default())
        .into_outcomes();
    let [UiHostInteractionIngressOutcome::Applied(receipt)] = release.as_ref() else {
        panic!("validated release: {release:?}")
    };
    assert!(matches!(
        receipt.scroll_chrome_interactions(),
        [UiScrollChromeIngressOutcome::PendingReleased]
    ));
    assert!(scroll
        .world
        .session
        .mounted
        .motion_sample_presentation_pending());
    assert!(matches!(
        scroll.wheel(ONE_NOTCH, one_notch_up(), 9),
        UiHostScrollObservationOutcome::Denied(
            crate::runtime::scroll::UiHostScrollObservationDenial::PendingChromeRelease
        )
    ));
    scroll.world.session.complete_motion_sample_presentation();
    assert!(scroll
        .world
        .session
        .interaction
        .scroll_chrome_latch()
        .is_none());
    assert!(scroll
        .world
        .session
        .interaction
        .scroll_chrome_latch_mut()
        .pending()
        .is_none());
    assert_eq!(pending_transitions(&scroll), 0);
    let accepted = scroll.accepted_offset();
    scroll.publish_direct(9);
    assert_eq!(
        scroll.accepted_offset().block_subpixels() - accepted.block_subpixels(),
        10_000
    );
    let _ = scroll.world.session.shutdown();
}

#[test]
fn capture_cannot_retire_an_accepted_sample_before_its_pose_is_reconciled() {
    let mut scroll = pending_sample();
    let grabbed = centre(block_chrome(&scroll).1);
    // Exercise the partial handoff directly: the mounted owner accepted pixels,
    // but the session has not yet reconciled its Scroll owner/geometry.
    let Some(crate::mounting::UiMountedMotionSampleSettlement::Committed(sampling)) = scroll
        .world
        .session
        .mounted
        .complete_motion_sample_presentation(&scroll.world.session.host_session)
    else {
        panic!("the in-flight sample commits")
    };
    let presented = sampling
        .presented_surface()
        .expect("a committed sample names the surface its witness proved");
    let surface = scroll.surface();
    let presentation = scroll.presentation();
    let outcome = scroll.world.session.press_scroll_chrome(
        surface,
        grabbed,
        UiHostPointerIdentity::new(POINTER),
        UiHostPointerCaptureEpoch::new(CAPTURE_EPOCH),
        presentation,
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
    assert_eq!(
        scroll
            .world
            .session
            .settle_accepted_scroll_sample(presented),
        UiScrollSettleDisposition::Applied
    );
    assert!(matches!(
        press(&mut scroll, grabbed),
        UiScrollChromePressOutcome::ThumbCaptured(_)
    ));
    let _ = scroll.world.session.shutdown();
}
