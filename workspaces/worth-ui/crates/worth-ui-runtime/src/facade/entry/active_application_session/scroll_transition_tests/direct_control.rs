//! A thumb grab mid-settle takes direct control: the settle that was walking
//! the region stops everywhere it was known, and nothing later resurrects it.
//!
//! Three things know about a live settle -- the Scroll target it is settling
//! toward, the sampler track producing accepted samples, and the Motion track
//! that owns the interpolation -- and a drag that retired only some of them
//! would be pulled back by the rest on the next tick. The second scenario is
//! the hazard a presentation in flight adds: the sampler's prepared successor
//! was cloned before the grab, so the retirement has to survive the commit.

use super::settle_world::{UiScrollSettleWorld, ONE_NOTCH_POINTS};

/// After a few ticks of settling, taking direct control leaves no pending
/// target, no accepted translation and no active track, and the next tick is a
/// quiet frame.
#[test]
fn direct_control_retires_the_target_the_sample_and_the_track() {
    let mut world = UiScrollSettleWorld::new(1_000.0);
    let notch = world.notch(1, 10);
    assert_eq!(notch.target_points(), ONE_NOTCH_POINTS);
    for tick in 11..=14 {
        world.commit(tick);
    }
    assert!(world.pending_target().is_some(), "the settle is mid-flight");
    assert!(world.accepted_translation().is_some());
    assert!(world.sampling_active());
    assert_eq!(world.scroll().pending_transition_count(), 1);

    assert_eq!(
        world.take_direct_control(),
        [true, true, true],
        "the target, the sampler track and the Motion track were all there to retire"
    );

    assert_eq!(world.pending_target(), None);
    assert_eq!(world.scroll().pending_transition_count(), 0);
    assert_eq!(
        world.accepted_translation(),
        None,
        "a retired settle leaves no accepted translation for a write-back to read"
    );
    assert!(!world.sampling_active());
    world.tick(15);
    assert!(!world.sampling_active(), "nothing resumes on the next tick");
    assert_eq!(
        world.take_direct_control(),
        [false, false, false],
        "a second grab finds nothing left to retire"
    );
}

/// A grab that lands between a tick's prepare and its commit still wins: the
/// committed successor carries neither the track nor any sample or terminal
/// for it, and the sampler is idle afterwards.
#[test]
fn a_track_retired_while_a_presentation_is_in_flight_stays_retired() {
    let mut world = UiScrollSettleWorld::new(1_000.0);
    world.notch(1, 10);
    world.commit(11);
    let prepared = world.prepare(12);

    assert_eq!(world.take_direct_control(), [true, true, true]);

    let receipt = world.commit_prepared(prepared);
    assert!(
        receipt.samples().is_empty(),
        "the in-flight tick publishes no sample for a track the pointer took"
    );
    assert!(
        receipt.terminals().is_empty(),
        "nor does it report the track arriving anywhere"
    );
    assert!(!world.sampling_active());
    assert_eq!(world.accepted_translation(), None);
    world.tick(13);
    assert!(!world.sampling_active());
}
