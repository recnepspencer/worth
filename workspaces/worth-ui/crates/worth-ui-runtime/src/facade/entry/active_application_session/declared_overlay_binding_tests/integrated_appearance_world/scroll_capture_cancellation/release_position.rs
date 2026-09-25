use super::pending_sample::{
    old_epoch_button_batch, old_epoch_press_batch, pending_sample_with, presented_sample, viewport,
};
use super::*;
use crate::certification_support::ScriptedSurfaceCompletion;
use crate::facade::entry::active_application_session::scroll_chrome_ingress::UiScrollChromeIngressOutcome;
use crate::facade::entry::active_application_session::scroll_chrome_interaction::UiScrollChromeInteractionDenial;
use crate::runtime::interaction::UiHostInteractionIngressOutcome;

#[test]
fn thumb_release_without_final_motion_stages_its_position_until_host_acceptance() {
    let mut scroll = chrome_world();
    let (track, thumb) = block_chrome(&scroll);
    assert_eq!(track.height() - thumb.height(), 6.0);
    let grabbed = centre(thumb);
    let pressed = button(
        &mut scroll,
        1,
        grabbed,
        UiHostPointerButtonTransition::Pressed,
    );
    assert!(matches!(
        pressed,
        UiScrollChromeIngressOutcome::Pressed(UiScrollChromePressOutcome::ThumbCaptured(_))
    ));
    // No PointerMotion is delivered. The event-time release names two of the
    // six thumb-travel points, independently ten of thirty content points.
    let released = button(
        &mut scroll,
        2,
        [grabbed[0], grabbed[1] + 2.0],
        UiHostPointerButtonTransition::Released,
    );
    assert!(matches!(
        released,
        UiScrollChromeIngressOutcome::Released(_)
    ));
    assert!(scroll
        .world
        .session
        .interaction
        .scroll_chrome_latch()
        .is_none());
    assert_eq!(scroll.accepted_offset(), block(0));
    let frame = scroll.world.prepare_surface(scroll.surface());
    let calls = scroll.world.host.presentation_calls();
    scroll.world.host.push_rejected();
    let outcome = scroll
        .world
        .session
        .present_prepared_mounted_frame_internal(
            frame,
            UiPresentationDeadline::at_tick(u64::MAX),
            3,
        );
    assert!(matches!(
        outcome,
        crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(_)
    ));
    assert_eq!(scroll.world.host.presentation_calls(), calls + 1);
    assert_eq!(scroll.accepted_offset(), block(0));
    let refused = block_chrome(&scroll).1;
    // Retained chrome spells the same logical client geometry as Viewport;
    // initial layout spells HostSurface. Compare the actual rectangle.
    assert_eq!(
        [refused.x(), refused.y(), refused.width(), refused.height()],
        [thumb.x(), thumb.y(), thumb.width(), thumb.height()],
    );
    scroll.publish_direct(4);
    assert_eq!(scroll.accepted_offset(), block(10));
    assert_eq!(block_chrome(&scroll).1.y() - thumb.y(), 2.0);
    assert!(scroll
        .world
        .session
        .interaction
        .scroll_chrome_latch()
        .is_none());
    let _ = scroll.world.session.shutdown();
}

/// A release on a basis chrome cannot read places nothing, and it still ends
/// the drag: the wheel and new presses are not left behind a stale capture.
#[test]
fn a_release_on_a_basis_chrome_cannot_read_still_ends_the_drag() {
    let mut scroll = chrome_world();
    let grabbed = centre(block_chrome(&scroll).1);
    assert!(matches!(
        button(
            &mut scroll,
            1,
            grabbed,
            UiHostPointerButtonTransition::Pressed
        ),
        UiScrollChromeIngressOutcome::Pressed(UiScrollChromePressOutcome::ThumbCaptured(_))
    ));
    let window = UiHostSurfacePosition::new(
        UiHostSurfacePositionBasis::new(
            UiHostSurfaceCoordinateSpace::Window,
            UiHostSurfaceCoordinateUnit::LogicalPoint,
        ),
        (grabbed[0] * 1000.0).round() as i64,
        ((grabbed[1] + 2.0) * 1000.0).round() as i64,
    );
    let released = button_at(
        &mut scroll,
        2,
        window,
        UiHostPointerButtonTransition::Released,
    );
    assert_eq!(
        released,
        UiScrollChromeIngressOutcome::Denied(
            UiScrollChromeInteractionDenial::PositionBasisRefused(window.basis())
        )
    );
    assert!(scroll
        .world
        .session
        .interaction
        .scroll_chrome_latch()
        .is_none());
    assert_eq!(scroll.accepted_offset(), block(0));
    let _ = scroll.world.session.shutdown();
}

/// A pending release on a basis chrome cannot read still ends the capture. It
/// is released where the pointer was last admitted, so completion drags
/// nothing.
#[test]
fn pending_release_on_a_basis_chrome_cannot_read_still_ends_the_capture() {
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
    let _ = scroll
        .world
        .session
        .drain_and_admit_host_observation_batches(Default::default());
    let moved = viewport([grabbed[0], grabbed[1] + 2.0]);
    let window = UiHostSurfacePosition::new(
        UiHostSurfacePositionBasis::new(
            UiHostSurfaceCoordinateSpace::Window,
            UiHostSurfaceCoordinateUnit::LogicalPoint,
        ),
        moved.x_subpixels(),
        moved.y_subpixels(),
    );
    scroll
        .world
        .host
        .enqueue_observation_for_next_drain(old_epoch_button_batch(
            &scroll,
            window,
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
    assert_eq!(
        receipt.scroll_chrome_interactions(),
        [UiScrollChromeIngressOutcome::Denied(
            UiScrollChromeInteractionDenial::PositionBasisRefused(window.basis())
        )]
    );
    scroll.world.session.complete_motion_sample_presentation();
    let latch = scroll.world.session.interaction.scroll_chrome_latch_mut();
    assert!(latch.pending().is_none());
    assert!(scroll
        .world
        .session
        .interaction
        .scroll_chrome_latch()
        .is_none());
    assert_eq!(pending_transitions(&scroll), 0);
    let accepted = scroll.accepted_offset();
    scroll.publish_direct(9);
    assert_eq!(scroll.accepted_offset(), accepted);
    let _ = scroll.world.session.shutdown();
}

fn button(
    scroll: &mut ScrollWorld,
    number: u64,
    point: [f32; 2],
    transition: UiHostPointerButtonTransition,
) -> UiScrollChromeIngressOutcome {
    let position = UiHostSurfacePosition::viewport_logical(
        (point[0] * 1000.0).round() as i64,
        (point[1] * 1000.0).round() as i64,
    );
    button_at(scroll, number, position, transition)
}

fn button_at(
    scroll: &mut ScrollWorld,
    number: u64,
    position: UiHostSurfacePosition,
    transition: UiHostPointerButtonTransition,
) -> UiScrollChromeIngressOutcome {
    let UiHostProtocolNegotiation::Compatible(protocol) =
        UiHostProtocolContract::current().negotiate()
    else {
        panic!("current protocol")
    };
    let sequence = UiHostObservationSequence::new(number);
    let batch = UiHostObservationBatch::new(UiHostObservationBatchInput {
        protocol,
        host_session: scroll.world.session.host_session.identity().as_u64(),
        presentation: scroll.presentation(),
        sequences: UiHostObservationSequenceRange::new(sequence, sequence),
        loss: UiHostObservationLoss::Complete,
        reports: vec![UiHostObservationReport::new(
            sequence,
            UiHostObservationTimeBasis::HostMonotonicMillis(number),
            UiHostObservationPayload::PointerButton {
                pointer: UiHostPointerIdentity::new(POINTER),
                capture_epoch: UiHostPointerCaptureEpoch::new(CAPTURE_EPOCH),
                button: UiHostPointerButton::Primary,
                transition,
                position,
            },
        )
        .with_pointer_device_kind(UiHostPointerDeviceKind::Mouse)
        .unwrap()],
    })
    .unwrap();
    let ingress = scroll.world.session.admit_host_interaction_batch(batch);
    let UiHostInteractionIngressOutcome::Applied(receipt) = ingress else {
        panic!("pointer must reach chrome: {ingress:?}");
    };
    let [outcome] = receipt.scroll_chrome_interactions() else {
        panic!("exactly one chrome outcome: {receipt:?}");
    };
    outcome.clone()
}
