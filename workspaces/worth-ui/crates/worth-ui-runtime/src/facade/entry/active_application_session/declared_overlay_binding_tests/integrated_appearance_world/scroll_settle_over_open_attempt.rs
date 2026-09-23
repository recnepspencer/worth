//! A notch that arrives while the host is still holding a frame open.
//!
//! Publishing a settle submits into the frame already on screen and mints no
//! new one, so an attempt in flight is not a reason to refuse the reader: the
//! notch is answered, and nothing is recorded as having stopped it. What an
//! open attempt does change is when that settle can be paid. The first owed
//! frame is deferred and the settle arrives on its own clock afterwards.
//!
//! Both halves are asserted here, because a notch that published and was then
//! never paid would look, to a reader, exactly like one that was refused.

use super::super::super::UiScrollSettleDisposition;
use super::scroll_pose_authority::{block, ScrollWorld};
use super::scroll_settle_commit::{
    one_notch_up, pending_transitions, smooth_world, LINE_EXTENT_POINTS, ONE_NOTCH, SETTLE_TICKS,
};
use crate::runtime::scroll::UiHostScrollObservationOutcome;
use worth_ui_host_contract::UiHostObservationPresentationBasis;

const ATTEMPT_TICK: u64 = 2;
const NOTCH_TICK: u64 = 5;
/// Frames from a notch to the tick its settle arrives on. The first Motion
/// frame samples the rest pose, so the horizon costs one frame more than it
/// declares.
const ARRIVAL: u64 = SETTLE_TICKS as u64 + 1;

fn presentation(scroll: &ScrollWorld) -> UiHostObservationPresentationBasis {
    scroll
        .world
        .session
        .mounted
        .current_presentation_for_surface(scroll.surface())
        .expect("the first surface is published")
}

/// One Motion frame the way the native shell runs it.
fn settle_frame(scroll: &mut ScrollWorld, tick: u64) -> UiScrollSettleDisposition {
    let basis = presentation(scroll);
    let prepared = scroll
        .world
        .session
        .prepare_motion_tick(tick, basis)
        .expect("an armed settle prepares its tick");
    if !prepared.receipt().samples().is_empty() {
        scroll.world.host.push_native_display_presented();
    }
    scroll
        .world
        .session
        .present_prepared_motion_tick(prepared, basis);
    scroll.world.session.settle_accepted_scroll_sample(basis)
}

/// The reader turned the wheel while the host had not finished the frame it
/// was given. The settle goes into that frame, and the session records no
/// stop, because nothing stopped.
#[test]
fn a_notch_over_an_open_attempt_publishes_into_the_frame_already_on_screen() {
    let mut scroll = smooth_world(true);
    let pending = scroll.hold_presentation_open(ATTEMPT_TICK);

    let outcome = scroll.wheel(ONE_NOTCH, one_notch_up(), NOTCH_TICK);
    assert!(
        matches!(outcome, UiHostScrollObservationOutcome::Applied(_)),
        "an open attempt is not a refusal: {outcome:?} (stop {:?})",
        scroll.world.session.last_scroll_settle_stop()
    );
    assert_eq!(
        scroll.world.session.last_scroll_settle_stop(),
        None,
        "a notch that published leaves nothing recorded as having stopped it"
    );
    assert_eq!(pending_transitions(&scroll), 1);
    assert_eq!(
        scroll.accepted_offset(),
        block(0),
        "the notch stages a target and moves no offset"
    );

    // The frame the host was holding completes on its own terms: a notch that
    // published into it changed nothing it was carrying.
    scroll.complete(pending, NOTCH_TICK + 1);
    let _ = scroll.world.session.shutdown();
}

/// The settle that published over the open attempt is a real one: once the
/// host finishes its frame, it is paid out to the offset the notch asked for.
#[test]
fn the_settle_it_published_arrives_once_the_host_finishes_its_frame() {
    let mut scroll = smooth_world(true);
    let pending = scroll.hold_presentation_open(ATTEMPT_TICK);
    let outcome = scroll.wheel(ONE_NOTCH, one_notch_up(), NOTCH_TICK);
    assert!(matches!(
        outcome,
        UiHostScrollObservationOutcome::Applied(_)
    ));

    scroll.complete(pending, NOTCH_TICK + 1);
    for elapsed in 1..=ARRIVAL {
        assert_eq!(
            settle_frame(&mut scroll, NOTCH_TICK + elapsed),
            UiScrollSettleDisposition::Applied,
            "frame {elapsed} after the notch applies its accepted sample"
        );
    }

    assert_eq!(
        scroll.accepted_offset(),
        block(i64::from(LINE_EXTENT_POINTS)),
        "the settle arrives at the target the notch asked for"
    );
    assert_eq!(pending_transitions(&scroll), 0);
    let _ = scroll.world.session.shutdown();
}
