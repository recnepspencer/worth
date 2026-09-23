use super::*;
use crate::facade::entry::active_application_session::scroll_chrome_ingress::UiScrollChromeIngressOutcome;
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

fn button(
    scroll: &mut ScrollWorld,
    number: u64,
    point: [f32; 2],
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
                position: UiHostSurfacePosition::viewport_logical(
                    (point[0] * 1000.0).round() as i64,
                    (point[1] * 1000.0).round() as i64,
                ),
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
