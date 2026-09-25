//! Facade-level proof that a reader who has asked for reduced motion is taken
//! to the place a notch asked for, without being carried there.
//!
//! A scroll settle is not decorative. The reader asked to be somewhere else in
//! the content, and dropping the transition would drop the answer. What
//! reduced motion is about is the carrying, so the settle arrives outright on
//! the first frame instead of walking its declared horizon.
//!
//! Both halves matter and are asserted together. Arriving early is only
//! correct if it arrives at the same offset the unreduced settle would have
//! reached, and only complete if nothing is left behind afterwards: no
//! semantic target, no accepted sample, no Motion track asking for more
//! frames. The same World is run under both postures so the two answers are
//! compared rather than described.

use super::super::super::UiScrollSettleDisposition;
use super::scroll_pose_authority::{block, ScrollWorld};
use super::scroll_settle_commit::{
    one_notch_up, pending_transitions, smooth_world, LINE_EXTENT_POINTS, ONE_NOTCH,
};
use crate::mounting::presentation::motion_sampling::UiPresentationReducedMotionPosture;
use crate::runtime::scroll::UiHostScrollObservationOutcome;

const NOTCH_TICK: u64 = 5;

/// Where one notch of a ten-point line asks the content to end up.
fn notch_target() -> crate::runtime::scroll::UiScrollOffset {
    block(i64::from(LINE_EXTENT_POINTS))
}

/// One Motion frame the way the native shell runs it.
fn frame(scroll: &mut ScrollWorld, tick: u64) -> UiScrollSettleDisposition {
    super::scroll_settle_frame::quiet_frame(scroll, tick)
}

/// A smooth-wheel World under `posture`, holding one published notch with
/// nothing moved yet. A notch is answered by a settle either way: what the
/// posture changes is how many frames that settle spends.
fn world_with_one_notch(posture: UiPresentationReducedMotionPosture) -> ScrollWorld {
    let mut scroll = smooth_world(true);
    scroll
        .world
        .session
        .mounted
        .set_reduced_motion_posture(posture);
    let outcome = scroll.wheel(ONE_NOTCH, one_notch_up(), NOTCH_TICK);
    assert!(
        matches!(outcome, UiHostScrollObservationOutcome::Applied(_)),
        "a smooth notch publishes its settle: {outcome:?}"
    );
    assert_eq!(
        scroll.accepted_offset(),
        block(0),
        "a smooth notch stages a target and moves nothing"
    );
    assert_eq!(pending_transitions(&scroll), 1);
    scroll
}

/// A reader who has asked for reduced motion gets the destination on the first
/// frame, and gets all of it.
#[test]
fn a_reduced_motion_settle_arrives_on_its_first_frame() {
    let mut scroll = world_with_one_notch(UiPresentationReducedMotionPosture::Reduce);

    assert_eq!(
        frame(&mut scroll, NOTCH_TICK + 1),
        UiScrollSettleDisposition::Applied
    );
    assert_eq!(
        scroll.accepted_offset(),
        notch_target(),
        "the notch asked to be a line further down, and that is where the reader is"
    );
    assert_eq!(
        scroll.mounted_offset(),
        Some(notch_target()),
        "the displayed pose is the offset Scroll holds"
    );
    let _ = scroll.world.session.shutdown();
}

/// Arriving outright ends the settle rather than short-circuiting one frame of
/// it. Nothing is left asking for the frames the reader declined.
#[test]
fn a_reduced_motion_settle_leaves_nothing_still_settling() {
    let mut scroll = world_with_one_notch(UiPresentationReducedMotionPosture::Reduce);
    assert_eq!(
        frame(&mut scroll, NOTCH_TICK + 1),
        UiScrollSettleDisposition::Applied
    );

    assert_eq!(
        pending_transitions(&scroll),
        0,
        "the semantic target has been reached, so nothing is aiming at it"
    );
    assert!(
        !scroll.world.session.mounted.has_active_motion_samples(),
        "no Motion track is left walking content that has arrived"
    );
    frame(&mut scroll, NOTCH_TICK + 2);
    assert_eq!(
        scroll.accepted_offset(),
        notch_target(),
        "a later frame has nothing left that could move the reader again"
    );
    let _ = scroll.world.session.shutdown();
}

/// The same World, same notch, same first frame, without the posture. The
/// content is on its way rather than there, which is what makes the reduced
/// answer a different one rather than the declared horizon being trivial.
#[test]
fn the_same_notch_without_the_posture_is_still_traveling_on_that_frame() {
    let mut scroll = world_with_one_notch(UiPresentationReducedMotionPosture::NoPreference);

    assert_eq!(
        frame(&mut scroll, NOTCH_TICK + 1),
        UiScrollSettleDisposition::Applied
    );
    assert_ne!(
        scroll.accepted_offset(),
        notch_target(),
        "an unreduced settle spends its horizon getting there"
    );
    assert_eq!(
        pending_transitions(&scroll),
        1,
        "the target it is heading for is still staged"
    );
    assert!(scroll.world.session.mounted.has_active_motion_samples());
    let _ = scroll.world.session.shutdown();
}
