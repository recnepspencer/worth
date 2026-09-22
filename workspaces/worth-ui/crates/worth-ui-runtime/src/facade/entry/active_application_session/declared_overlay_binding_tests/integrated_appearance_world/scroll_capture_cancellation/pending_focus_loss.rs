use super::*;
use crate::certification_support::ScriptedSurfaceCompletion;
use crate::runtime::interaction::UiHostInteractionIngressOutcome;

fn old_epoch_focus_loss_batch(scroll: &ScrollWorld) -> UiHostObservationBatch {
    let UiHostProtocolNegotiation::Compatible(protocol) =
        UiHostProtocolContract::current().negotiate()
    else {
        panic!("current protocol")
    };
    let presentation = scroll.presentation();
    let sequence = UiHostObservationSequence::new(2);
    UiHostObservationBatch::new(UiHostObservationBatchInput {
        protocol,
        host_session: scroll.world.session.host_session.identity().as_u64(),
        presentation,
        sequences: UiHostObservationSequenceRange::new(sequence, sequence),
        loss: UiHostObservationLoss::Complete,
        reports: vec![UiHostObservationReport::new(
            sequence,
            UiHostObservationTimeBasis::HostMonotonicMillis(9),
            UiHostObservationPayload::WindowFocus {
                surface: presentation.host_surface(),
                focused: false,
            },
        )],
    })
    .unwrap()
}

#[test]
fn focus_loss_cancels_pending_capture_before_physical_completion() {
    let mut scroll = pending_sample::pending_sample_with(vec![
        ScriptedSurfaceCompletion::Pending,
        ScriptedSurfaceCompletion::Pending,
        pending_sample::presented_sample(),
    ]);
    let grabbed = centre(block_chrome(&scroll).1);
    scroll
        .world
        .host
        .enqueue_observation_for_next_drain(pending_sample::old_epoch_press_batch(
            &scroll, grabbed,
        ));
    assert!(matches!(
        scroll
            .world
            .session
            .drain_and_admit_host_observation_batches(Default::default())
            .into_outcomes()
            .as_ref(),
        [UiHostInteractionIngressOutcome::Applied(_)]
    ));
    assert!(scroll
        .world
        .session
        .interaction
        .scroll_chrome_pending_capture()
        .is_some());
    scroll
        .world
        .host
        .enqueue_observation_for_next_drain(old_epoch_focus_loss_batch(&scroll));
    assert!(matches!(
        scroll
            .world
            .session
            .drain_and_admit_host_observation_batches(Default::default())
            .into_outcomes()
            .as_ref(),
        [UiHostInteractionIngressOutcome::Applied(_)]
    ));
    assert!(scroll
        .world
        .session
        .interaction
        .scroll_chrome_pending_capture()
        .is_none());
    scroll.world.session.complete_motion_sample_presentation();
    assert!(scroll
        .world
        .session
        .interaction
        .scroll_chrome_latch()
        .is_none());
    let _ = scroll.world.session.shutdown();
}
