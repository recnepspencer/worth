//! Dismissing a Portal restores focus to the participant it opened over and
//! lands the reveal bringing that participant back into view.
use worth_ui::facade::intent::UiIntentConsequencePublicationOutcome;
use worth_ui::facade::interaction::UiHostInteractionIngressOutcome;
use worth_ui::facade::observation_report::{
    UiHostKey, UiHostKeyTransition, UiHostKeyboardModifiers, UiHostObservationLoss,
    UiHostObservationPayload,
};
use worth_ui::facade::rebind::{UiRebindExecutionPolicy, UiRebindExecutionRequest};
use worth_ui_host_headless::{UiHeadlessRecorderCapacity, WorthUiHeadlessRecorder};
use worth_ui_query_binding::UiLiveCollectionProjectionCloseOutcome;
use worth_ui_runtime::facade::measurement_exchange::UiViewportExtentObservation;
use worth_ui_runtime::facade::mounted::UiMountedFrameRequest;
use worth_ui_test_support::{
    UiPortalDismissalCertificationOutcome, WorthUiActiveSessionCertificationExt,
    WorthUiFocusRuntimeCertificationExt, WorthUiPortalRuntimeCertificationExt,
};

use super::super::world::PayloadWorld;
use super::focus_reveal_frame::publish_focus_reveal_frame;
use super::scrolled_selection_portal::{
    activate_scrolled_row, dispatch_selection_portal, launch_selection_portal, SelectionPortalWorld,
};

#[test]
fn portal_dismissal_restores_focus_and_lands_its_reveal_with_the_carrying_frame() {
    let recorder = WorthUiHeadlessRecorder::with_viewport_extent(
        UiHeadlessRecorderCapacity::new(8, 1, 16_384),
        UiViewportExtentObservation {
            width: 160.0,
            height: 96.0,
        },
    );
    let SelectionPortalWorld {
        mut world,
        mut query,
        live,
        option,
        ..
    } = launch_selection_portal(recorder.clone());
    // Opening the Portal records the focused row as the focus to restore.
    let trigger = focus_first_participant(&mut world);
    let scrolled = activate_scrolled_row(&mut world);
    let consequence = dispatch_selection_portal(&mut world, scrolled.activation, option);
    let _ = recorder.drain_transcripts();
    match world.interaction.session.publish_intent_consequences(
        consequence,
        UiRebindExecutionPolicy::ordinary(),
        UiRebindExecutionRequest::new(315_052),
    ) {
        UiIntentConsequencePublicationOutcome::Published(_) => {}
        UiIntentConsequencePublicationOutcome::Stopped(stop) => {
            panic!("selection Portal publication stopped: {:?}", stop.reason())
        }
        _ => panic!("selection Portal publication did not publish"),
    }
    let opened = publish_focus_reveal_frame(&mut world, &recorder, scrolled.scrolled_offset);
    assert_eq!(
        opened.owner_geometry()[0].block_offset_subpixels(),
        scrolled.expected_reveal_offset
    );

    // Wheel the row out of view again while the Portal stays open.
    assert!(matches!(
        world.interaction.scroll([20, 20], -1_000_000_000),
        UiHostInteractionIngressOutcome::Applied(_)
    ));
    let frame = world
        .interaction
        .session
        .prepare_application_presentation_frame(UiMountedFrameRequest::all_bound_surfaces())
        .expect("the direct wheel frame prepares over the open Portal");
    world.interaction.publish_prepared_successor(frame);
    let before = world
        .interaction
        .session
        .inspect_focus_runtime_for_certification();
    let _ = recorder.drain_transcripts();
    assert_eq!(
        world
            .interaction
            .session
            .publish_escape_portal_dismissal_for_certification(315_060),
        UiPortalDismissalCertificationOutcome::Published
    );
    let restored = world
        .interaction
        .session
        .inspect_focus_runtime_for_certification();
    assert_eq!(restored.current_participant(), Some(trigger));
    assert_eq!(restored.revision(), before.revision() + 1);
    let revealed = publish_focus_reveal_frame(&mut world, &recorder, scrolled.scrolled_offset);
    assert_eq!(
        revealed.owner_geometry()[0].block_offset_subpixels(),
        scrolled.expected_reveal_offset,
        "the restored focus is revealed by the frame carrying its placement"
    );

    match live.close(&mut query) {
        UiLiveCollectionProjectionCloseOutcome::Closed(closed) => assert!(closed.owner_terminal()),
        UiLiveCollectionProjectionCloseOutcome::Stopped(stop) => {
            panic!(
                "selection portal Query owner closes: {:?}",
                stop.query_error()
            )
        }
    }
    let _ = world.interaction.session.shutdown();
}

/// Tab to the first focus participant and return it.
fn focus_first_participant(world: &mut PayloadWorld) -> u64 {
    let presentation = world.interaction.presentation;
    let tab = world.interaction.admit(
        presentation,
        UiHostObservationLoss::Complete,
        vec![UiHostObservationPayload::Keyboard {
            logical_key: UiHostKey::Tab,
            physical_key: Some(UiHostKey::Tab),
            modifiers: UiHostKeyboardModifiers::default(),
            transition: UiHostKeyTransition::Pressed { repeat: false },
        }],
    );
    assert!(matches!(tab, UiHostInteractionIngressOutcome::Applied(_)));
    world
        .interaction
        .session
        .inspect_focus_runtime_for_certification()
        .current_participant()
        .expect("Tab focuses the first participant")
}
