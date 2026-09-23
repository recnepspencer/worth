//! A settle overtaken by shrinking bounds.
//!
//! Content can get shorter while a settle is still running, and when it does
//! the target the reader aimed at may no longer be a place the owner can be.
//! The write-back is a route, so the reconciled bounds that clamp the accepted
//! offset clamp the pending target in the same call. Clamping one without the
//! other would leave the owner either holding an offset outside its bounds or
//! still travelling toward one.

use super::assert_close;
use super::settle_world::{bounds, points, UiScrollSettleWorld, ONE_NOTCH_POINTS, SETTLE_TICKS};

/// The whole travel is legal under the bounds it was staged against. This is
/// the control the clamped case is read against: without it, a clamp that
/// simply refused every settle would look identical.
#[test]
fn a_settle_inside_its_bounds_arrives_untouched() {
    let mut world = UiScrollSettleWorld::new(1_000.0);
    world.notch(1, 0);
    settle_through_horizon(&mut world);

    let bounds = world.bounds();
    world
        .settle_accepted(bounds)
        .expect("an in-bounds settle routes");
    assert_close(
        world.accepted_offset_points(),
        ONE_NOTCH_POINTS,
        "an in-bounds settle keeps every point it travelled",
    );
}

/// Bounds that shrink under a running settle clamp the accepted offset and the
/// pending target together, to the same place, in the same call.
#[test]
fn shrunken_bounds_clamp_the_accepted_offset_and_the_pending_target_together() {
    let mut world = UiScrollSettleWorld::new(1_000.0);
    let notch = world.notch(1, 0);
    settle_through_horizon(&mut world);
    assert_eq!(notch.target_points(), ONE_NOTCH_POINTS);

    // The content is now only twenty-five points taller than its viewport, so
    // the sixty points this settle aimed at is no longer a place to be.
    let reclamped = bounds(25.0);
    world
        .settle_accepted(reclamped)
        .expect("a clamped settle still routes");

    assert_close(
        world.accepted_offset_points(),
        25.0,
        "the accepted offset is clamped to the reconciled bounds",
    );
    let pending = world
        .pending_target()
        .expect("the settle is still pending until it is retired");
    assert_close(
        points(pending.target_offset().block_subpixels()),
        25.0,
        "the pending target is reclamped to the same bounds in the same call",
    );
    assert_eq!(
        points(pending.target_offset().block_subpixels()),
        world.accepted_offset_points(),
        "a clamped settle leaves nothing still travelling"
    );
}

/// With the target and the offset clamped to the same place, the settle has
/// arrived: it retires rather than lingering as an intention toward an offset
/// the owner already holds.
#[test]
fn a_clamped_settle_has_arrived_and_retires() {
    let mut world = UiScrollSettleWorld::new(1_000.0);
    world.notch(1, 0);
    settle_through_horizon(&mut world);
    world
        .settle_accepted(bounds(25.0))
        .expect("a clamped settle still routes");

    assert_eq!(
        world.scroll_mut().retire_reached_transitions(),
        1,
        "a target the owner has reached is no longer pending"
    );
    assert_eq!(world.pending_target(), None);
}

fn settle_through_horizon(world: &mut UiScrollSettleWorld) {
    for elapsed in 0..=u64::from(SETTLE_TICKS) {
        world.commit(elapsed);
    }
}
