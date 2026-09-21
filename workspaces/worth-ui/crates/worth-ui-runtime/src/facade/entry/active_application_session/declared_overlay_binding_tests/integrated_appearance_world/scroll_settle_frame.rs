//! Facade-level proof that a published settle is paid one accepted frame at a
//! time, through the same entry points the native shell runs.
//!
//! A smooth notch publishes a Motion transition; nothing else moves the
//! accepted offset. Each frame prepares and presents a Motion tick and then
//! settles the accepted Scroll sample, so the offset Scroll holds is always the
//! pose geometry displays. A tick that sampled only scrolled content owes the
//! host no sample work: the displacement is geometry the settle re-lowers, so
//! the scripted host is never scripted here. A frame refused because the host
//! holds a presentation open is reported as deferred, keeps the sample, and is
//! paid by the next frame. A second notch inside the horizon retargets the
//! settle from the first target rather than doubling it. A track page refused
//! by mounted geometry leaves the settle it would have ended exactly as it
//! was; only a page whose pose landed ends it.
//!
//! The transition samples its rest pose on the first tick after the notch and
//! arrives `SETTLE_TICKS` ticks later, so a settle spans `SETTLE_TICKS + 1`
//! frames counted from the notch.

use super::super::super::UiScrollSettleDisposition;
use super::scroll_pose_authority::{block, ScrollWorld};
use super::scroll_settle_commit::{
    one_notch_up, pending_transitions, smooth_world, LINE_EXTENT_POINTS, ONE_NOTCH, SETTLE_TICKS,
};
use crate::mounting::UiMountedOccurrenceGeometryDenial;
use crate::runtime::scroll::{UiHostScrollObservationOutcome, UiScrollDeltaCause, UiScrollOffset};
use worth_ui_host_contract::UiHostObservationPresentationBasis;

const NOTCH_TICK: u64 = 5;
/// Frames from a notch to the tick its settle arrives on.
const ARRIVAL: u64 = SETTLE_TICKS as u64 + 1;

fn presentation(scroll: &ScrollWorld) -> UiHostObservationPresentationBasis {
    scroll
        .world
        .session
        .mounted
        .current_presentation_for_surface(scroll.surface())
        .expect("the first surface is published")
}

/// One Motion frame the way the native shell runs it: prepare the tick,
/// present it, then settle the accepted Scroll sample.
pub(super) fn settle_frame(scroll: &mut ScrollWorld, tick: u64) -> UiScrollSettleDisposition {
    let basis = presentation(scroll);
    let prepared = scroll
        .world
        .session
        .prepare_motion_tick(tick, basis)
        .expect("an armed settle prepares its tick");
    scroll
        .world
        .session
        .present_prepared_motion_tick(prepared, basis);
    scroll.world.session.settle_accepted_scroll_sample(basis)
}

/// The frame the interaction lane runs when a settle is owed and no Motion
/// tick asked for one.
pub(super) fn owed_frame(scroll: &mut ScrollWorld) -> UiScrollSettleDisposition {
    let basis = presentation(scroll);
    scroll.world.session.settle_accepted_scroll_sample(basis)
}

fn staged_target(scroll: &ScrollWorld) -> Option<UiScrollOffset> {
    scroll
        .world
        .session
        .scroll
        .as_ref()
        .expect("Scroll stays installed")
        .transition_target(scroll.owner, scroll.incarnation)
        .map(|target| target.target_offset())
}

fn notch(scroll: &mut ScrollWorld, tick: u64) {
    let outcome = scroll.wheel(ONE_NOTCH, one_notch_up(), tick);
    assert!(
        matches!(outcome, UiHostScrollObservationOutcome::Applied(_)),
        "a smooth notch publishes its settle: {outcome:?} (stop {:?})",
        scroll.world.session.last_scroll_settle_stop()
    );
}

/// Run frames `first..=last` after `notch_tick`, each of which must apply its
/// accepted sample and display exactly the offset Scroll holds.
fn applied_frames(scroll: &mut ScrollWorld, notch_tick: u64, first: u64, last: u64) {
    for elapsed in first..=last {
        assert_eq!(
            settle_frame(scroll, notch_tick + elapsed),
            UiScrollSettleDisposition::Applied,
            "frame {elapsed} after the notch applies its accepted sample"
        );
        assert_eq!(
            scroll.displayed_offset(),
            Some(scroll.accepted_offset()),
            "frame {elapsed}: the displayed pose is the offset Scroll holds"
        );
    }
}

#[test]
fn each_settle_frame_moves_the_offset_exactly_to_its_accepted_sample() {
    let mut scroll = smooth_world(true);
    notch(&mut scroll, NOTCH_TICK);
    assert_eq!(scroll.accepted_offset(), block(0));

    let mut previous = block(0);
    for elapsed in 1..=ARRIVAL {
        applied_frames(&mut scroll, NOTCH_TICK, elapsed, elapsed);
        let accepted = scroll.accepted_offset();
        assert!(
            accepted.block_subpixels() >= previous.block_subpixels(),
            "frame {elapsed}: the settle never moves back ({previous:?} then {accepted:?})"
        );
        if elapsed == 1 {
            assert_eq!(
                accepted,
                block(0),
                "the frame a notch arrives on samples the rest pose, so it moves \
                 nothing and the settle arrives one frame past its horizon"
            );
        } else {
            assert!(
                accepted.block_subpixels() > previous.block_subpixels(),
                "frame {elapsed}: every frame inside the horizon moves the content"
            );
        }
        previous = accepted;
    }
    assert_eq!(
        scroll.accepted_offset(),
        block(i64::from(LINE_EXTENT_POINTS)),
        "the settle arrives at the target of the notch"
    );
    assert_eq!(
        scroll.world.session.last_scroll_settle_disposition(),
        UiScrollSettleDisposition::Applied
    );
    let _ = scroll.world.session.shutdown();
}

#[test]
fn a_settle_frame_refused_mid_presentation_is_owed_and_paid_by_the_next() {
    let mut scroll = smooth_world(true);
    notch(&mut scroll, NOTCH_TICK);
    applied_frames(&mut scroll, NOTCH_TICK, 1, 2);
    let before = scroll.accepted_offset();
    assert!(
        before.block_subpixels() > 0,
        "two frames into the settle the content has moved"
    );

    // The host holds the next presentation open. The owed frame cannot apply
    // the accepted sample; it says so, keeps the sample, and moves nothing.
    let pending = scroll.hold_presentation_open(NOTCH_TICK + 3);
    assert_eq!(
        owed_frame(&mut scroll),
        UiScrollSettleDisposition::DeferredPresentationInFlight
    );
    assert!(scroll.world.session.awaits_scroll_settle_retry());
    assert_eq!(scroll.accepted_offset(), before);
    assert_eq!(scroll.displayed_offset(), Some(before));

    // Once the host completes, the same accepted sample is applied: the
    // deferral cost the settle a frame, not its offset.
    scroll.complete(pending, NOTCH_TICK + 4);
    assert_eq!(owed_frame(&mut scroll), UiScrollSettleDisposition::Applied);
    assert!(!scroll.world.session.awaits_scroll_settle_retry());
    assert_eq!(scroll.accepted_offset(), before);
    assert_eq!(scroll.displayed_offset(), Some(before));

    // And the settle still arrives on its own clock.
    applied_frames(&mut scroll, NOTCH_TICK, 5, ARRIVAL);
    assert_eq!(
        scroll.accepted_offset(),
        block(i64::from(LINE_EXTENT_POINTS))
    );
    let _ = scroll.world.session.shutdown();
}

#[test]
fn a_second_notch_inside_the_horizon_retargets_from_the_first_target() {
    let mut scroll = smooth_world(true);
    notch(&mut scroll, NOTCH_TICK);
    applied_frames(&mut scroll, NOTCH_TICK, 1, 1);
    assert_eq!(
        staged_target(&scroll),
        Some(block(i64::from(LINE_EXTENT_POINTS)))
    );

    let second = NOTCH_TICK + 2;
    notch(&mut scroll, second);
    assert_eq!(
        staged_target(&scroll),
        Some(block(2 * i64::from(LINE_EXTENT_POINTS))),
        "the second notch aims one notch past the first target"
    );
    assert_eq!(
        pending_transitions(&scroll),
        1,
        "one settle carries both notches"
    );

    applied_frames(&mut scroll, second, 1, ARRIVAL);
    assert_eq!(
        scroll.accepted_offset(),
        block(2 * i64::from(LINE_EXTENT_POINTS)),
        "the retargeted settle arrives at the accumulated target"
    );
    let _ = scroll.world.session.shutdown();
}

#[test]
fn a_track_page_refused_mid_presentation_leaves_the_settle_alive() {
    use super::super::super::scroll_chrome_interaction::UiScrollChromeInteractionDenial;

    let mut scroll = smooth_world(true);
    notch(&mut scroll, NOTCH_TICK);
    applied_frames(&mut scroll, NOTCH_TICK, 1, 2);
    let before = scroll.accepted_offset();
    let (owner, incarnation, target) = (scroll.owner, scroll.incarnation, scroll.target());

    // The host holds a presentation open, so the page cannot land its pose.
    // Nothing it would have ended may end: the pending target, the retained
    // sample and the Motion track all stay, and the offset is untouched.
    let pending = scroll.hold_presentation_open(NOTCH_TICK + 3);
    assert_eq!(
        scroll.world.session.place_scroll_chrome_offset(
            owner,
            incarnation,
            target,
            0,
            block(3),
            UiScrollDeltaCause::ChromeTrackPage,
        ),
        Err(UiScrollChromeInteractionDenial::Geometry(
            UiMountedOccurrenceGeometryDenial::PresentationInFlight
        ))
    );
    assert_eq!(scroll.accepted_offset(), before);
    assert_eq!(scroll.displayed_offset(), Some(before));
    assert_eq!(pending_transitions(&scroll), 1);
    assert_eq!(
        staged_target(&scroll),
        Some(block(i64::from(LINE_EXTENT_POINTS)))
    );
    assert!(scroll.world.session.mounted.has_active_motion_samples());

    // Once the host completes, the settle is still walking: the next frame
    // moves the content past where the refused page found it.
    scroll.complete(pending, NOTCH_TICK + 4);
    applied_frames(&mut scroll, NOTCH_TICK, 5, 5);
    assert!(scroll.accepted_offset().block_subpixels() > before.block_subpixels());

    // A page whose pose lands ends the settle: the target is gone, the offset
    // is the placed one, and no accepted sample stands behind the pose.
    let placed = scroll
        .world
        .session
        .place_scroll_chrome_offset(
            owner,
            incarnation,
            target,
            0,
            block(3),
            UiScrollDeltaCause::ChromeTrackPage,
        )
        .expect("a page with no attempt in flight applies");
    assert_eq!(placed.transitions()[0].current(), block(3));
    assert_eq!(scroll.accepted_offset(), block(3));
    assert_eq!(scroll.displayed_offset(), Some(block(3)));
    assert_eq!(pending_transitions(&scroll), 0);
    assert_eq!(staged_target(&scroll), None);
    assert!(!scroll.world.session.mounted.has_active_motion_samples());
    assert_eq!(owed_frame(&mut scroll), UiScrollSettleDisposition::Idle);
    assert_eq!(scroll.accepted_offset(), block(3));
    let _ = scroll.world.session.shutdown();
}
