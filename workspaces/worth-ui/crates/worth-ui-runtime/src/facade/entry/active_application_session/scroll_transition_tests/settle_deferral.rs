//! Frames keep coming until the settle is paid, and a settle the frame could
//! not pay is still owed afterwards.
//!
//! Two properties hold this together. The sampler stays armed for every tick of
//! the horizon, so the shell keeps scheduling frames without any further input
//! -- that is what `native_motion_sampling_active` reads. And the accepted
//! sample outlives the tick that produced it, so a frame that refused the pose
//! because a presentation was in flight has not lost anything: the next frame
//! reads the same accepted translation and applies it.
//!
//! The refusal itself belongs to mounted geometry: `apply_scroll_geometries`
//! denies `PresentationInFlight` while a presentation attempt is live, and
//! `scroll_accepted_sample_settlement` turns that denial into the typed
//! `UiScrollSettleDisposition::DeferredPresentationInFlight` rather than
//! dropping the sample.

use super::assert_close;
use super::settle_world::{UiScrollSettleWorld, ONE_NOTCH_POINTS, SETTLE_TICKS};

/// A settle re-arms itself. Every tick from the notch to the deadline leaves
/// the sampler active, so the shell has a reason to schedule the next frame
/// without the reader touching the wheel again; the tick that arrives is the
/// first one that does not.
#[test]
fn a_settle_keeps_the_sampler_armed_until_it_arrives() {
    let mut world = UiScrollSettleWorld::new(1_000.0);
    world.notch(1, 0);
    assert!(
        world.sampling_active(),
        "a committed settle arms sampling before its first tick"
    );

    world.commit(0);
    for elapsed in 1..u64::from(SETTLE_TICKS) {
        world.commit(elapsed);
        assert!(
            world.sampling_active(),
            "tick {elapsed} of {SETTLE_TICKS} must keep the next frame scheduled"
        );
    }

    world.commit(u64::from(SETTLE_TICKS));
    assert!(
        !world.sampling_active(),
        "the arriving tick is the one that stops asking for frames"
    );
}

/// A retarget re-arms sampling that had nearly gone quiet. Without this a notch
/// arriving in the last ticks of a settle could be committed into a track the
/// shell had already stopped scheduling frames for.
#[test]
fn a_notch_late_in_a_settle_re_arms_sampling() {
    let mut world = UiScrollSettleWorld::new(1_000.0);
    world.notch(1, 0);
    world.commit(0);
    for elapsed in 1..u64::from(SETTLE_TICKS) {
        world.commit(elapsed);
    }

    let late = u64::from(SETTLE_TICKS) - 1;
    world.notch(1, late);
    assert!(
        world.sampling_active(),
        "the retarget keeps frames coming past the original deadline"
    );
    for elapsed in 1..u64::from(SETTLE_TICKS) {
        world.commit(late + elapsed);
        assert!(
            world.sampling_active(),
            "the extended horizon stays armed at tick {elapsed} past the notch"
        );
    }
}

/// The accepted sample is not consumed by the frame that reads it. A frame that
/// refused the pose leaves the same translation readable on the next tick, so
/// the deferral costs the settle nothing but a frame.
#[test]
fn the_accepted_sample_outlives_the_frame_that_could_not_apply_it() {
    let mut world = UiScrollSettleWorld::new(1_000.0);
    world.notch(1, 0);
    for elapsed in 0..u64::from(SETTLE_TICKS) {
        world.commit(elapsed);
    }

    let deferred = world
        .accepted_translation()
        .expect("a presented settle reports its accepted translation");
    let reread = world
        .accepted_translation()
        .expect("reading the accepted translation does not consume it");
    assert_eq!(deferred, reread);

    // The next tick arrives and the settle completes. The translation the
    // deferred frame would have applied is still there to be applied, now at
    // the arrived endpoint rather than the one it was refused at.
    world.commit(u64::from(SETTLE_TICKS));
    let after = world
        .accepted_translation()
        .expect("an arrived settle still reports where it arrived");
    assert_close(
        world
            .accepted_displacement_points()
            .expect("an arrived settle still reports where it arrived"),
        -ONE_NOTCH_POINTS,
        "the retry applies the arrived offset rather than the one it was refused at",
    );

    // A frame later still, with the track quiet, the accepted sample remains
    // the truth about where the content is. A retry that ran here would find
    // the settle waiting, not lost.
    world.tick(u64::from(SETTLE_TICKS) + 1);
    assert_eq!(world.accepted_translation(), Some(after));

    let bounds = world.bounds();
    world
        .settle_accepted(bounds)
        .expect("a deferred settle is still routable when its frame finally comes");
    assert_close(
        world.accepted_offset_points(),
        ONE_NOTCH_POINTS,
        "the deferred settle is paid in full, not partly",
    );
}
