//! A nested region's settle while the region carrying it scrolls.
//!
//! The second component owns a region and is laid out as the first
//! component's content, and the third component is the second's region's
//! content. The first component's offset carries the inner region and its
//! row across the surface while the inner offset stays where its settle puts
//! it: a frame that publishes the outer pose lays the inner content out
//! somewhere new without moving the inner offset. The paint/hit oracle holds
//! at every frame, the row is drawn where hit testing reads it, and each
//! region comes to rest at its own target.

use super::geometry::scrollable::install_scrollable_primary_with_inner_region;
use super::scroll_pose_authority::{block, ScrollWorld};
use super::scroll_settle_commit::{
    one_notch_up, pending_transitions, smooth_scroll, LINE_EXTENT_POINTS, ONE_NOTCH, SETTLE_TICKS,
};
use super::scroll_settle_frame::{notch, quiet_frame};
use super::scroll_settle_hit_lead::{assert_paint_and_hit_agree, nested_hit_y};
use super::session::World;
use crate::runtime::scroll::{
    UiHostScrollObservationOutcome, UiScrollDeltaCause, UiScrollOffset, UiScrollOwnerIdentity,
    UiScrollOwnerIncarnation, UiScrollPresentationDeviceScale,
};
use worth_ui_host_contract::*;

/// A point over the inner region beside and below its row, so a wheel there
/// addresses the inner region rather than the row's or the first
/// component's.
const INNER_WHEEL_POSITION: [i64; 2] = [190_000, 77_000];

/// The World with the second component's region nested in the first
/// component's, one frame published, and the Scroll owner the inner region
/// resolves to.
fn nested_world() -> (ScrollWorld, UiScrollOwnerIdentity, UiScrollOwnerIncarnation) {
    let mut world = World::launch_with_scroll(smooth_scroll(true));
    install_scrollable_primary_with_inner_region(
        &mut world.session,
        world.surfaces,
        world.instances,
    );
    let scroll = ScrollWorld::publish_installed(world);
    let (owner, incarnation) = inner_owner(&scroll);
    assert!(
        owner != scroll.owner,
        "the inner region is its own owner, nested in the first component's"
    );
    (scroll, owner, incarnation)
}

/// The Scroll owner the inner region resolves to.
fn inner_owner(scroll: &ScrollWorld) -> (UiScrollOwnerIdentity, UiScrollOwnerIncarnation) {
    let inner = scroll.world.instances[1];
    let owner = scroll
        .world
        .session
        .scroll
        .as_ref()
        .expect("the World installs Scroll from policy")
        .ownership_chain(inner)
        .expect("the inner region's owner resolves its Scroll chain")
        .owners()[0];
    let incarnation = scroll
        .world
        .session
        .mounted
        .scroll_region_incarnation(inner, 0)
        .expect("the inner region has a current allocation");
    (owner, incarnation)
}

/// One smooth notch over the inner region.
fn notch_inner(scroll: &mut ScrollWorld, tick: u64) {
    let target = UiHostScrollDeltaTargetAffinity::exact_coordinate(
        scroll.presentation(),
        UiHostSurfacePosition::viewport_logical(INNER_WHEEL_POSITION[0], INNER_WHEEL_POSITION[1]),
    );
    let outcome = scroll.targeted_wheel(
        UiHostScrollDeltaPhase::Updated,
        target,
        ONE_NOTCH,
        one_notch_up(),
        tick,
    );
    assert!(
        matches!(outcome, UiHostScrollObservationOutcome::Applied(_)),
        "a smooth notch over the inner region publishes its settle: {outcome:?}"
    );
}

fn inner_offset(
    scroll: &ScrollWorld,
    owner: UiScrollOwnerIdentity,
    incarnation: UiScrollOwnerIncarnation,
) -> UiScrollOffset {
    scroll
        .world
        .session
        .scroll
        .as_ref()
        .expect("Scroll stays installed")
        .offset(owner, incarnation)
        .expect("the inner region keeps its offset")
}

/// The host draws the nested row where each region's offset, rounded to the
/// device grid on its own, puts it. Hit testing reads the row where the exact
/// offsets put it, so the two stand apart by exactly the fraction of a device
/// pixel each region's offset carries: up to a whole pixel across the two
/// regions, never anything else. It reads the accepted offsets, so it holds
/// on frames that applied their sample, not beside a Motion tick in flight.
fn assert_drawn_by_each_region(scroll: &ScrollWorld, label: &str) {
    let (owner, incarnation) = inner_owner(scroll);
    let scale =
        UiScrollPresentationDeviceScale::admit(UiScrollPresentationDeviceScale::UNSCALED_MILLI)
            .expect("the World binds its surface unscaled");
    let subpixels = UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as f64;
    let residue =
        |offset: UiScrollOffset| scale.grid_residue(offset.block_subpixels() as f64 / subpixels);
    let hit = nested_hit_y(scroll).expect("hit testing reads the nested row");
    let drawn_at = f64::from(hit)
        + residue(scroll.accepted_offset())
        + residue(inner_offset(scroll, owner, incarnation));
    let nested = scroll.world.instances[2];
    let drawn = scroll
        .world
        .host
        .accepted_text_drawn_bounds(scroll.surface())
        .into_iter()
        .filter(|(identity, _)| identity.mounted_instance() == nested)
        .map(|(_, drawn)| drawn)
        .collect::<Vec<_>>();
    assert!(!drawn.is_empty(), "{label}: the host draws the nested row");
    for drawn in drawn {
        assert!(
            (f64::from(drawn.y()) - drawn_at).abs() < 1e-3,
            "{label}: the host draws the nested row at {}, each region's rounding puts it at \
             {drawn_at} (hit testing reads it at {hit})",
            drawn.y()
        );
    }
}

/// Run quiet frames `ticks`, holding the oracles at each.
fn frames(scroll: &mut ScrollWorld, ticks: std::ops::RangeInclusive<u64>, label: &str) {
    for tick in ticks {
        quiet_frame(scroll, tick);
        assert_paint_and_hit_agree(scroll, &format!("{label}: frame {tick}"));
        assert_drawn_by_each_region(scroll, &format!("{label}: frame {tick}"));
    }
}

/// The inner settle keeps its target while a page carries its region: the
/// page publishes the outer pose mid-settle, and the row is drawn where it
/// is hit at every frame on the way to rest.
#[test]
fn a_nested_settle_lands_while_a_page_carries_its_region() {
    let (mut scroll, inner, incarnation) = nested_world();
    let line = block(i64::from(LINE_EXTENT_POINTS));
    notch_inner(&mut scroll, 5);
    frames(&mut scroll, 6..=7, "the inner region settling");
    let (owner, primary, target) = (scroll.owner, scroll.incarnation, scroll.target());
    scroll
        .world
        .session
        .place_scroll_chrome_offset(
            owner,
            primary,
            target,
            0,
            block(12),
            UiScrollDeltaCause::ChromeTrackPage,
        )
        .expect("a page with no attempt in flight applies");
    scroll.publish_direct(8);
    assert_paint_and_hit_agree(&scroll, "the page published mid-settle");
    assert_drawn_by_each_region(&scroll, "the page published mid-settle");
    frames(&mut scroll, 9..=20, "the page published");
    assert_eq!(pending_transitions(&scroll), 0, "nothing strands");
    assert!(!scroll.world.session.awaits_scroll_settle_retry());
    assert_eq!(scroll.accepted_offset(), block(12), "the page holds");
    assert_eq!(inner_offset(&scroll, inner, incarnation), line);
    let _ = scroll.world.session.shutdown();
}

/// A settle of the region carrying the inner one moves the inner content box
/// every frame without moving the inner offset, which the inner settle has
/// already brought to rest. Both settling at once is
/// `nested_settles_in_flight_together_each_land_at_their_own_target`.
#[test]
fn an_outer_settle_carries_a_nested_region_at_its_own_offset() {
    let (mut scroll, inner, incarnation) = nested_world();
    let line = block(i64::from(LINE_EXTENT_POINTS));
    notch_inner(&mut scroll, 5);
    frames(&mut scroll, 6..=12, "the inner region settling");
    assert_eq!(inner_offset(&scroll, inner, incarnation), line);
    // The latch the inner notch took has lapsed: the next notch is the
    // first component's.
    notch(&mut scroll, 14);
    frames(&mut scroll, 15..=24, "the first component settling");
    assert_eq!(pending_transitions(&scroll), 0, "nothing strands");
    assert_eq!(scroll.accepted_offset(), line);
    assert_eq!(scroll.mounted_offset(), Some(line));
    assert_eq!(inner_offset(&scroll, inner, incarnation), line);
    let _ = scroll.world.session.shutdown();
}

/// A settle of the region carrying the inner one while the inner region's
/// own settle is still in flight: both advance on the same frames, the row
/// is hit where it is drawn at every one, and each comes to rest at its own
/// target.
#[test]
fn nested_settles_in_flight_together_each_land_at_their_own_target() {
    let (mut scroll, inner, incarnation) = nested_world();
    let line = block(i64::from(LINE_EXTENT_POINTS));
    notch_inner(&mut scroll, 5);
    // No frame has sampled the inner settle, so it has not started when the
    // latch the inner notch took lapses and the next notch is the first
    // component's.
    notch(&mut scroll, 5 + u64::from(SETTLE_TICKS) + 1);
    assert_eq!(
        pending_transitions(&scroll),
        2,
        "both settles are in flight"
    );
    frames(&mut scroll, 12..=14, "both regions settling");
    for (region, offset) in [
        ("the first component", scroll.accepted_offset()),
        (
            "the inner region",
            inner_offset(&scroll, inner, incarnation),
        ),
    ] {
        assert!(
            (1..line.block_subpixels()).contains(&offset.block_subpixels()),
            "{region} is mid-settle: {offset:?}"
        );
    }
    assert_eq!(pending_transitions(&scroll), 2, "neither settle has ended");
    frames(&mut scroll, 15..=24, "both regions coming to rest");
    assert_eq!(pending_transitions(&scroll), 0, "nothing strands");
    assert!(!scroll.world.session.awaits_scroll_settle_retry());
    assert_eq!(scroll.accepted_offset(), line);
    assert_eq!(scroll.mounted_offset(), Some(line));
    assert_eq!(inner_offset(&scroll, inner, incarnation), line);
    let _ = scroll.world.session.shutdown();
}
