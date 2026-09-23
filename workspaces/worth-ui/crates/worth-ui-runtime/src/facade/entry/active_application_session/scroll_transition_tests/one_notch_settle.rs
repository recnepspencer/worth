//! One notch, settled.
//!
//! A single coarse wheel notch against a twenty-point line extent is sixty
//! points of travel, and a declared smooth wheel spends the whole 120-tick
//! horizon getting there. The accepted sample never overshoots, never reverses
//! and never jumps: it leaves rest at rest, moves monotonically, and arrives
//! exactly on the declared endpoint at the deadline.

use super::settle_world::{UiScrollSettleWorld, ONE_NOTCH_POINTS, SETTLE_TICKS};
use super::{assert_close, hermite_position, TOLERANCE};

/// Three lines against a twenty-point line is sixty points, and the horizon the
/// notch establishes ends exactly `SETTLE_TICKS` after the input that opened
/// it. Both numbers are the declaration, so both are asserted against the
/// arithmetic rather than against the code that produced them.
#[test]
fn one_notch_targets_sixty_points_over_the_declared_horizon() {
    let mut world = UiScrollSettleWorld::new(1_000.0);
    let notch = world.notch(1, 10);

    assert_eq!(ONE_NOTCH_POINTS, 60.0);
    assert_eq!(notch.target_points(), 60.0);
    assert_eq!(notch.target.horizon().latest_input_tick(), 10);
    assert_eq!(
        notch.target.settle_deadline_tick(),
        10 + u64::from(SETTLE_TICKS)
    );
    assert_eq!(
        notch.fact,
        crate::runtime::motion::UiMotionProducedFactKind::Started,
        "the first notch of an idle owner starts a track rather than retargeting one"
    );
    assert_eq!(
        notch.retarget, None,
        "there is no incumbent track for a first notch to interrupt"
    );
    assert_eq!(
        world.accepted_offset_points(),
        0.0,
        "the observation stages a target and moves no accepted offset"
    );
}

/// The accepted sample walks the whole sixty points over the horizon and lands
/// on the endpoint exactly. The curve is checked against an independently
/// written cubic Hermite from rest to rest, so the sampler is never its own
/// oracle, and the sampled positions are checked for monotonicity so a curve
/// that arrived by overshooting and returning could not pass.
#[test]
fn the_accepted_sample_settles_monotonically_onto_sixty_points() {
    let mut world = UiScrollSettleWorld::new(1_000.0);
    world.notch(1, 0);

    let start = world.commit(0);
    assert_close(start, 0.0, "the settle opens at the accepted offset");

    let mut previous = start;
    let mut moved = 0.0_f64;
    for elapsed in 1..=u64::from(SETTLE_TICKS) {
        let sampled = world.commit(elapsed);
        assert_close(
            sampled,
            hermite_position(
                0.0,
                0.0,
                -ONE_NOTCH_POINTS,
                elapsed as f64,
                f64::from(SETTLE_TICKS),
            ),
            "the settle follows the declared rest-to-rest curve",
        );
        assert!(
            sampled <= previous + TOLERANCE,
            "tick {elapsed} reversed: {sampled} after {previous}"
        );
        assert!(
            sampled >= -ONE_NOTCH_POINTS - TOLERANCE,
            "tick {elapsed} overshot the endpoint: {sampled}"
        );
        moved += (sampled - previous).abs();
        previous = sampled;
    }

    assert_close(
        previous,
        -ONE_NOTCH_POINTS,
        "the settle ends exactly on the declared endpoint",
    );
    assert_close(
        moved,
        ONE_NOTCH_POINTS,
        "a monotone settle travels exactly the distance between its endpoints",
    );
    assert!(
        !world.sampling_active(),
        "the track retires once the horizon is spent"
    );
}

/// The settle is velocity-continuous at both ends: it departs from rest and
/// arrives at rest. A curve that jumped into motion or stopped dead would show
/// a first or last step comparable to its mid-horizon step.
#[test]
fn the_settle_departs_from_rest_and_arrives_at_rest() {
    let mut world = UiScrollSettleWorld::new(1_000.0);
    world.notch(1, 0);
    world.commit(0);

    let mut steps = Vec::new();
    let mut previous = 0.0_f64;
    for elapsed in 1..=u64::from(SETTLE_TICKS) {
        let sampled = world.commit(elapsed);
        steps.push((sampled - previous).abs());
        previous = sampled;
    }

    let middle = steps[usize::try_from(SETTLE_TICKS).expect("the horizon fits a usize") / 2];
    assert!(
        middle > 0.5,
        "the settle is genuinely moving mid-horizon, step {middle}"
    );
    assert!(
        steps[0] < middle / 10.0,
        "the settle departs from rest: first step {}, mid step {middle}",
        steps[0]
    );
    let last = *steps.last().expect("the horizon has ticks");
    assert!(
        last < middle / 10.0,
        "the settle arrives at rest: last step {last}, mid step {middle}"
    );
}

/// After the accepted sample has arrived, the write-back brings the owner's
/// semantic offset to exactly the accepted displayed offset, and the settle
/// stops being a pending intention.
#[test]
fn the_write_back_leaves_the_owner_at_the_accepted_displayed_offset() {
    let mut world = UiScrollSettleWorld::new(1_000.0);
    world.notch(1, 0);
    for elapsed in 0..=u64::from(SETTLE_TICKS) {
        world.commit(elapsed);
    }

    let displacement = world
        .accepted_displacement_points()
        .expect("a presented settle reports its accepted position");
    assert_close(
        displacement,
        -ONE_NOTCH_POINTS,
        "the accepted position is the arrived endpoint, one notch from rest",
    );

    let bounds = world.bounds();
    let receipt = world
        .settle_accepted(bounds)
        .expect("the accepted offset is routable under the owner's own bounds");
    assert_eq!(
        receipt.cause(),
        crate::runtime::scroll::UiScrollDeltaCause::AcceptedSampleSettlement,
        "the write-back is a settlement, never a host delta"
    );
    assert_close(
        world.accepted_offset_points(),
        ONE_NOTCH_POINTS,
        "the owner's offset equals the accepted displayed offset",
    );

    assert_eq!(
        world.scroll_mut().retire_reached_transitions(),
        1,
        "an arrived settle stops being a pending intention"
    );
    assert_eq!(world.pending_target(), None);
}
