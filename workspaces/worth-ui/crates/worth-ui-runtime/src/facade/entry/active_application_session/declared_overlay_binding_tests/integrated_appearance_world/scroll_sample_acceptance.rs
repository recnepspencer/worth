//! Physical acceptance, not agreement between two mutable runtime views,
//! authorizes the displayed Scroll offset.

use super::scroll_pose_authority::ScrollWorld;
use super::scroll_settle_commit::{one_notch_up, smooth_scroll, ONE_NOTCH};
use super::scroll_settle_frame::{settle_frame, settle_scripted_frame};
use super::World;
use crate::runtime::scroll::UiHostScrollObservationOutcome;

#[test]
fn rejected_scroll_sample_preserves_the_previously_displayed_offset() {
    let mut scroll =
        ScrollWorld::publish_with_nested_content(World::launch_with_scroll(smooth_scroll(true)));
    assert!(matches!(
        scroll.wheel(ONE_NOTCH, one_notch_up(), 5),
        UiHostScrollObservationOutcome::Applied(_)
    ));
    // The first tick establishes the rest sample. Its host acceptance is
    // deliberately separate from the rejection of the next moving sample.
    settle_frame(&mut scroll, 6);
    let before = scroll.accepted_offset();
    let before_pose = scroll.displayed_offset();
    let calls_before = scroll.world.host.presentation_calls();
    scroll.world.host.push_rejected();
    settle_scripted_frame(&mut scroll, 7);

    let actual = (
        scroll.world.host.presentation_calls() - calls_before,
        scroll.accepted_offset(),
        scroll.displayed_offset(),
    );
    let _ = scroll.world.session.shutdown();
    assert_eq!(
        actual,
        (1, before, before_pose),
        "a moving sample must reach the host, and refusal must preserve accepted Scroll and pose"
    );
}

#[test]
fn rejected_endpoint_retains_signed_target_for_input_after_the_old_horizon() {
    let mut scroll =
        ScrollWorld::publish_with_nested_content(World::launch_with_scroll(smooth_scroll(true)));
    assert!(matches!(
        scroll.wheel(ONE_NOTCH, 3 * one_notch_up(), 5),
        UiHostScrollObservationOutcome::Applied(_)
    ));
    settle_frame(&mut scroll, 6);
    let before = scroll.accepted_offset();
    let calls = scroll.world.host.presentation_calls();
    scroll.world.host.push_rejected();
    settle_scripted_frame(&mut scroll, 12); // Past the original six-tick horizon.
    assert_eq!(scroll.world.host.presentation_calls(), calls + 1);
    assert_eq!(scroll.accepted_offset(), before);
    assert!(matches!(
        scroll.wheel(ONE_NOTCH, -one_notch_up(), 20),
        UiHostScrollObservationOutcome::Applied(_)
    ));
    let pending = scroll
        .world
        .session
        .scroll
        .as_ref()
        .unwrap()
        .transition_target(scroll.owner, scroll.incarnation)
        .unwrap();
    assert_eq!(pending.target_offset().block_subpixels(), 20_000, "three forward and one reverse notch remain twenty points, regardless of rejected display progress");
    assert_eq!(scroll.accepted_offset(), before);
    for tick in 21..=28 {
        settle_frame(&mut scroll, tick);
    }
    assert_eq!(scroll.accepted_offset().block_subpixels(), 20_000);
    assert_eq!(scroll.displayed_offset(), Some(scroll.accepted_offset()));
    let _ = scroll.world.session.shutdown();
}

#[test]
fn in_flight_scroll_sample_moves_nothing_until_matching_physical_completion() {
    use crate::certification_support::ScriptedSurfaceCompletion;
    use worth_ui_host_contract::*;
    let mut scroll =
        ScrollWorld::publish_with_nested_content(World::launch_with_scroll(smooth_scroll(true)));
    assert!(matches!(
        scroll.wheel(ONE_NOTCH, one_notch_up(), 5),
        UiHostScrollObservationOutcome::Applied(_)
    ));
    settle_frame(&mut scroll, 6);
    let before = scroll.accepted_offset();
    let basis = scroll.presentation();
    scroll.world.host.push_in_flight(
        vec![
            ScriptedSurfaceCompletion::Pending,
            ScriptedSurfaceCompletion::Presented(UiMountedSurfacePresentationCompletion::new(
                UiHostSurfacePresentationMode::NativeDisplay,
                UiHostPresentationEpoch::issued_by_host(100),
                UiMountedCompletedEffects::new(vec![UiMountedEffectFamily::NativePaint]),
                UiHostPresentationCostReport::default(),
            )),
        ],
        UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
    );
    settle_scripted_frame(&mut scroll, 7);
    assert!(scroll
        .world
        .session
        .mounted
        .motion_sample_presentation_pending());
    assert_eq!(scroll.accepted_offset(), before);
    assert_eq!(scroll.displayed_offset(), Some(before));
    scroll.world.session.complete_motion_sample_presentation();
    assert!(scroll
        .world
        .session
        .mounted
        .motion_sample_presentation_pending());
    assert_eq!(scroll.accepted_offset(), before);
    scroll.world.session.complete_motion_sample_presentation();
    scroll.world.session.settle_accepted_scroll_sample(basis);
    assert!(!scroll
        .world
        .session
        .mounted
        .motion_sample_presentation_pending());
    assert!(scroll.accepted_offset().block_subpixels() > before.block_subpixels());
    assert_eq!(scroll.displayed_offset(), Some(scroll.accepted_offset()));
    let _ = scroll.world.session.shutdown();
}
