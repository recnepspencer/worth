//! A second notch arriving mid-settle.
//!
//! The reader who keeps turning the wheel is not starting a second animation;
//! they are aiming the one already running. So the second notch has to reach
//! the live track -- accumulating onto its target, extending its horizon from
//! the latest input, and departing at the rate the content was already moving.
//! A notch that queued behind the incumbent instead would show up here twice:
//! as a `Started` fact rather than a `Retargeted` one, and as a sample that
//! snapped back to the original endpoint before setting out again.

use super::settle_world::{UiScrollSettleWorld, ONE_NOTCH_POINTS, SETTLE_TICKS};
use super::{assert_close, hermite_position, hermite_rate};

const INTERRUPTION_TICK: u64 = 40;

/// The second notch accumulates onto the pending target instead of replacing
/// it, and the horizon is measured from the latest input, so a reader who keeps
/// turning never has the settle cut short under them.
#[test]
fn a_second_notch_accumulates_the_target_and_extends_the_horizon() {
    let mut world = UiScrollSettleWorld::new(1_000.0);
    let first = world.notch(1, 0);
    world.commit(0);
    for elapsed in 1..=INTERRUPTION_TICK {
        world.commit(elapsed);
    }
    let second = world.notch(1, INTERRUPTION_TICK);

    assert_eq!(first.target_points(), ONE_NOTCH_POINTS);
    assert_eq!(second.target_points(), 2.0 * ONE_NOTCH_POINTS);
    assert_eq!(
        second.target.horizon().latest_input_tick(),
        INTERRUPTION_TICK,
        "the horizon is anchored to the latest input"
    );
    assert_eq!(
        second.target.settle_deadline_tick(),
        INTERRUPTION_TICK + u64::from(SETTLE_TICKS),
        "the deadline is a full horizon after the latest input, not after the first"
    );
    assert_eq!(
        world.accepted_offset_points(),
        0.0,
        "neither notch moved the accepted offset; the samples do that"
    );
}

/// The second notch reaches the incumbent track. Motion reports the commit as a
/// retarget of the live track rather than the start of a new one, and resolves
/// the interruption to a velocity-matched install from the current presentation
/// sample.
#[test]
fn the_second_notch_retargets_the_live_track_rather_than_queueing() {
    let mut world = UiScrollSettleWorld::new(1_000.0);
    world.notch(1, 0);
    world.commit(0);
    for elapsed in 1..=INTERRUPTION_TICK {
        world.commit(elapsed);
    }
    let second = world.notch(1, INTERRUPTION_TICK);

    assert_eq!(
        second.fact,
        crate::runtime::motion::UiMotionProducedFactKind::Retargeted(
            crate::runtime::motion::UiMotionRetargetDisposition::Install {
                predecessor:
                    crate::runtime::motion::UiMotionRetargetPredecessor::CurrentPresentationSample,
            }
        ),
        "a notch against a live settle retargets it, it does not start a second one"
    );
    assert_eq!(
        second.retarget,
        Some(
            crate::runtime::motion::UiMotionRetargetDisposition::Install {
                predecessor:
                    crate::runtime::motion::UiMotionRetargetPredecessor::CurrentPresentationSample,
            }
        ),
        "a settle declares that an interruption continues from the current sample"
    );
}

/// The retarget leaves no seam. The successor opens exactly where the
/// interrupted sample was, and every sample after it lies on the cubic Hermite
/// that starts at that position *and* at the rate the outgoing curve was
/// carrying -- a curve written here independently of the sampler. Its clock
/// runs from the interrupted sample, which is already on screen, so the first
/// frame after the notch moves on instead of showing that sample again.
#[test]
fn the_retarget_snaps_neither_position_nor_velocity() {
    let mut world = UiScrollSettleWorld::new(1_000.0);
    world.notch(1, 0);
    world.commit(0);
    let mut interrupted = 0.0;
    for elapsed in 1..=INTERRUPTION_TICK {
        interrupted = world.commit(elapsed);
    }

    let horizon = f64::from(SETTLE_TICKS);
    assert_close(
        interrupted,
        hermite_position(
            0.0,
            0.0,
            -ONE_NOTCH_POINTS,
            INTERRUPTION_TICK as f64,
            horizon,
        ),
        "the interrupted sample sits on the outgoing curve",
    );
    let departure_rate = hermite_rate(
        0.0,
        0.0,
        -ONE_NOTCH_POINTS,
        INTERRUPTION_TICK as f64,
        horizon,
    );
    assert!(
        departure_rate < -0.1,
        "the outgoing settle is genuinely moving at the interruption, rate {departure_rate}"
    );

    let second = world.notch(1, INTERRUPTION_TICK);
    assert_close(
        second.installed_y,
        interrupted,
        "the successor opens at the accepted sample, with no positional snap",
    );

    for elapsed in [1_u64, 2, 20, 60, 119, u64::from(SETTLE_TICKS)] {
        let sampled = world.commit(INTERRUPTION_TICK + elapsed);
        assert_close(
            sampled,
            hermite_position(
                interrupted,
                departure_rate,
                -2.0 * ONE_NOTCH_POINTS,
                elapsed as f64,
                horizon,
            ),
            "the successor follows the velocity-matched curve from the interrupted rate",
        );
    }
}

/// The retargeted settle arrives at the accumulated target, and the write-back
/// leaves the owner holding exactly that offset. Two notches are one hundred
/// and twenty points, not sixty and not a hundred and eighty.
#[test]
fn the_retargeted_settle_arrives_at_the_accumulated_target() {
    let mut world = UiScrollSettleWorld::new(1_000.0);
    world.notch(1, 0);
    world.commit(0);
    for elapsed in 1..=INTERRUPTION_TICK {
        world.commit(elapsed);
    }
    world.notch(1, INTERRUPTION_TICK);
    for elapsed in 1..=u64::from(SETTLE_TICKS) {
        world.commit(INTERRUPTION_TICK + elapsed);
    }

    assert_close(
        world
            .accepted_displacement_points()
            .expect("a presented settle reports its accepted position"),
        -2.0 * ONE_NOTCH_POINTS,
        "the retargeted settle arrives at the accumulated endpoint",
    );
    let bounds = world.bounds();
    world
        .settle_accepted(bounds)
        .expect("the arrived offset is routable under the owner's own bounds");
    assert_close(
        world.accepted_offset_points(),
        2.0 * ONE_NOTCH_POINTS,
        "the owner holds exactly what two notches asked for",
    );
}
