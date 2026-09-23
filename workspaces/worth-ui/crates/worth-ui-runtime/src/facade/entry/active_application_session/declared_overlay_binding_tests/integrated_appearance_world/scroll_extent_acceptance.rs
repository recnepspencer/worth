//! Extent succession crosses actual host refusal and acceptance before changing
//! accepted Scroll state or the Motion endpoint carrying its pending target.

use super::geometry::scrollable::install_scrollable_primary_with_shorter_content;
use super::scroll_pose_authority::{block, ScrollWorld};
use super::scroll_settle_commit::{one_notch_up, smooth_scroll, ONE_NOTCH, SETTLE_TICKS};
use super::scroll_settle_frame::settle_frame;
use super::World;
use crate::mounting::UiMountedFrameOutcome;
use crate::runtime::scroll::UiHostScrollObservationOutcome;
use worth_ui_host_contract::UiPresentationDeadline;

fn moving_world(last_tick: u64) -> ScrollWorld {
    let mut scroll =
        ScrollWorld::publish_with_nested_content(World::launch_with_scroll(smooth_scroll(true)));
    let reserved = scroll
        .world
        .session
        .mounted
        .retention_snapshot()
        .current
        .retained_structural_bytes;
    assert!(matches!(
        scroll.wheel(ONE_NOTCH, one_notch_up(), 5),
        UiHostScrollObservationOutcome::Applied(_)
    ));
    for tick in 6..=last_tick {
        settle_frame(&mut scroll, tick);
    }
    assert_eq!(
        scroll
            .world
            .session
            .mounted
            .retention_snapshot()
            .current
            .retained_structural_bytes,
        reserved,
        "sample ticks consume the index and slots reserved at mounted admission"
    );
    scroll
}

fn motion_target(scroll: &ScrollWorld) -> crate::runtime::motion::UiMotionTargetIdentity {
    super::super::super::scroll_direct_control::scroll_content_motion_target(
        scroll.owner,
        scroll.target(),
    )
}

#[test]
fn rejected_partial_shrink_preserves_offset_target_and_motion_then_retry_retargets() {
    let mut scroll = moving_world(7);
    let before = scroll.accepted_offset();
    assert!(before.block_subpixels() > 0 && before.block_subpixels() < 5_000);
    let target = motion_target(&scroll);
    let track = scroll
        .world
        .session
        .motion
        .as_ref()
        .unwrap()
        .committed_track(target)
        .unwrap();
    let pending = scroll
        .world
        .session
        .scroll
        .as_ref()
        .unwrap()
        .transition_target(scroll.owner, scroll.incarnation)
        .unwrap();
    install_scrollable_primary_with_shorter_content(
        &mut scroll.world.session,
        scroll.world.surfaces,
        scroll.world.instances,
    );
    assert_eq!(
        scroll.accepted_offset(),
        before,
        "layout preparation cannot move accepted Scroll"
    );
    let frame = scroll.world.prepare_surface(scroll.surface());
    scroll.world.host.push_rejected();
    let outcome = scroll
        .world
        .session
        .present_prepared_mounted_frame_internal(
            frame,
            UiPresentationDeadline::at_tick(u64::MAX),
            8,
        );
    assert!(matches!(
        outcome,
        UiMountedFrameOutcome::RejectedBeforeEffects(_)
    ));
    assert_eq!(scroll.accepted_offset(), before);
    assert_eq!(
        scroll
            .world
            .session
            .motion
            .as_ref()
            .unwrap()
            .committed_track(target),
        Some(track)
    );
    assert_eq!(
        scroll
            .world
            .session
            .scroll
            .as_ref()
            .unwrap()
            .transition_target(scroll.owner, scroll.incarnation),
        Some(pending)
    );

    let frame = scroll.world.prepare_surface(scroll.surface());
    scroll.world.publish(frame, 8, false);
    let successor = scroll
        .world
        .session
        .motion
        .as_ref()
        .unwrap()
        .committed_track(target)
        .unwrap();
    assert_eq!(
        successor.identity(),
        track.identity(),
        "extent reconciliation preserves track lifecycle"
    );
    assert_ne!(successor.successor_geometry(), track.successor_geometry());
    assert_eq!(
        scroll
            .world
            .session
            .scroll
            .as_ref()
            .unwrap()
            .transition_target(scroll.owner, scroll.incarnation)
            .unwrap()
            .target_offset(),
        block(5)
    );
    for tick in 9..=5 + u64::from(SETTLE_TICKS) + 1 {
        settle_frame(&mut scroll, tick);
        assert!(scroll.accepted_offset().block_subpixels() <= 5_000);
        assert_eq!(scroll.displayed_offset(), Some(scroll.accepted_offset()));
    }
    assert_eq!(
        scroll.accepted_offset(),
        block(5),
        "extent retarget retains the original settle horizon"
    );
    let _ = scroll.world.session.shutdown();
}

#[test]
fn accepted_partial_shrink_clamps_past_edge_sample_and_ends_the_track() {
    let mut scroll = moving_world(10);
    assert!(scroll.accepted_offset().block_subpixels() > 5_000);
    install_scrollable_primary_with_shorter_content(
        &mut scroll.world.session,
        scroll.world.surfaces,
        scroll.world.instances,
    );
    assert!(
        scroll.accepted_offset().block_subpixels() > 5_000,
        "prepared extent cannot replace accepted state"
    );
    let frame = scroll.world.prepare_surface(scroll.surface());
    scroll.world.publish(frame, 11, false);
    assert_eq!(scroll.accepted_offset(), block(5));
    assert_eq!(scroll.displayed_offset(), Some(block(5)));
    assert!(scroll
        .world
        .session
        .motion
        .as_ref()
        .unwrap()
        .committed_track(motion_target(&scroll))
        .is_none());
    let _ = scroll.world.session.shutdown();
}

#[test]
fn delayed_extent_acceptance_does_not_restart_the_input_horizon() {
    let mut scroll = moving_world(7);
    assert!(scroll.accepted_offset().block_subpixels() < 5_000);
    install_scrollable_primary_with_shorter_content(
        &mut scroll.world.session,
        scroll.world.surfaces,
        scroll.world.instances,
    );
    let frame = scroll.world.prepare_surface(scroll.surface());
    scroll.world.publish(frame, 100, false);
    settle_frame(&mut scroll, 101);
    assert_eq!(
        scroll.accepted_offset(),
        block(5),
        "an extent accepted after the input horizon settles on its next accepted sample"
    );
    assert_eq!(scroll.displayed_offset(), Some(block(5)));
    let _ = scroll.world.session.shutdown();
}
