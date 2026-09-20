//! A second notch after the first has settled.
//!
//! Once a settle has arrived and been written back, the owner holds sixty
//! points and the mounted geometry shows that offset as a pose on the owner's
//! descendants. The owner box itself has not moved: it is the content box at
//! rest, and the next notch composes with it as such. A write-back that added
//! the applied offset back onto that box would credit the pose twice, and the
//! second settle would arrive at one hundred eighty points instead of one
//! hundred twenty. These scenarios pin the at-rest reading of the owner box
//! through a full notch, settle, notch, settle sequence.

use super::assert_close;
use super::settle_world::{UiScrollSettleWorld, ONE_NOTCH_POINTS, SETTLE_TICKS};

const SECOND_NOTCH_TICK: u64 = 1_000;

/// Runs one notch to arrival and writes it back, leaving the owner at rest on
/// sixty points with no pending intention.
fn settle_first_notch(world: &mut UiScrollSettleWorld) {
    world.notch(1, 0);
    for elapsed in 0..=u64::from(SETTLE_TICKS) {
        world.commit(elapsed);
    }
    let bounds = world.bounds();
    world
        .settle_accepted(bounds)
        .expect("the first notch settles inside the owner's bounds");
    assert_eq!(world.scroll_mut().retire_reached_transitions(), 1);
    assert_close(
        world.accepted_offset_points(),
        ONE_NOTCH_POINTS,
        "the first notch leaves the owner holding sixty points",
    );
}

/// The second notch opens where the first one arrived and aims one notch
/// further, so the settle it starts is a fresh track from sixty to one hundred
/// twenty, not a retarget and not a restart from zero.
#[test]
fn a_notch_after_settlement_opens_at_the_settled_offset() {
    let mut world = UiScrollSettleWorld::new(1_000.0);
    settle_first_notch(&mut world);

    let second = world.notch(1, SECOND_NOTCH_TICK);

    assert_eq!(second.target_points(), 2.0 * ONE_NOTCH_POINTS);
    assert_eq!(
        second.fact,
        crate::runtime::motion::UiMotionProducedFactKind::Started,
        "a notch against a settled owner starts a new track"
    );
    assert_eq!(second.retarget, None);
    assert_close(
        second.installed_y,
        -ONE_NOTCH_POINTS,
        "the second settle opens on the pose the first one left applied",
    );
}

/// The second settle arrives at the accumulated target and the write-back
/// reads it against the owner box at rest, so the owner ends on exactly one
/// hundred twenty points.
#[test]
fn the_second_write_back_reads_the_owner_box_at_rest() {
    let mut world = UiScrollSettleWorld::new(1_000.0);
    settle_first_notch(&mut world);
    world.notch(1, SECOND_NOTCH_TICK);
    for elapsed in 0..=u64::from(SETTLE_TICKS) {
        world.commit(SECOND_NOTCH_TICK + elapsed);
    }

    assert_close(
        world
            .accepted_displacement_points()
            .expect("a presented settle reports its accepted position"),
        -2.0 * ONE_NOTCH_POINTS,
        "the accepted position is the whole distance from rest",
    );

    let bounds = world.bounds();
    world
        .settle_accepted(bounds)
        .expect("one hundred twenty points is inside the owner's bounds");
    assert_close(
        world.accepted_offset_points(),
        2.0 * ONE_NOTCH_POINTS,
        "two settled notches are one hundred twenty points, not one hundred eighty",
    );
    assert_eq!(world.scroll_mut().retire_reached_transitions(), 1);
    assert_eq!(world.pending_target(), None);
    assert!(!world.sampling_active());
}

/// A write-back that composed the applied offset back onto the owner box would
/// ask for one hundred eighty points under bounds that admit one hundred
/// twenty, and the route would clamp at the bound with sixty points left
/// unrouted. Declaring bounds that end exactly on the second target makes a
/// doubled reading visible as a remainder rather than a silently clamped
/// success.
#[test]
fn the_second_settle_is_accepted_under_bounds_that_reject_a_doubled_reading() {
    let mut world = UiScrollSettleWorld::new(2.0 * ONE_NOTCH_POINTS);
    settle_first_notch(&mut world);
    world.notch(1, SECOND_NOTCH_TICK);
    for elapsed in 0..=u64::from(SETTLE_TICKS) {
        world.commit(SECOND_NOTCH_TICK + elapsed);
    }

    let bounds = world.bounds();
    let receipt = world
        .settle_accepted(bounds)
        .expect("the accepted offset lands exactly on the block bound");
    assert_eq!(
        receipt.cause(),
        crate::runtime::scroll::UiScrollDeltaCause::AcceptedSampleSettlement
    );
    assert_eq!(
        receipt.remainder().block_subpixels(),
        0,
        "the whole settle is consumed: a doubled reading would leave sixty points unrouted"
    );
    assert_close(
        world.accepted_offset_points(),
        2.0 * ONE_NOTCH_POINTS,
        "the settle lands on the bound without being clamped down to it",
    );
}
