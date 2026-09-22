//! Facade-level proof that a coarse wheel notch travels the distance its
//! region's author declared, under a wheel that settles nothing.
//!
//! A host reports a coarse wheel as a count of lines, already multiplied by the
//! platform's lines-per-notch. A count is not a distance. How far a line
//! reaches is a product decision the region declares, and the two have to be
//! multiplied somewhere before an offset can move.
//!
//! Under a declared smooth wheel that happens at the far end, where the notch
//! becomes a settle target. A declared immediate wheel has no settle to do it
//! later, so it happens on the way in -- and if it did not happen at all, the
//! line count would be spent as though lines were points. The scales conspire
//! to hide that: the offset model counts a thousand subpixels to the point and
//! the wheel counts a thousand thousandths to the line, so a notch would still
//! move, just one point per line instead of the extent the author declared.
//!
//! The pixel path is the control. A host that measured in pixels already
//! reported a distance, and nothing here may touch it.

use super::scroll_pose_authority::{block, ScrollWorld};
use super::scroll_settle_commit::{scroll_region, LINE_EXTENT_POINTS};
use super::session::{World, WorldScroll};
use crate::runtime::scroll::{UiHostScrollObservationDenial, UiHostScrollObservationOutcome};
use worth_ui_host_contract::{
    UiHostScrollDeltaPrecision, UiHostScrollLineCountBasis,
    UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT,
};

/// The platform's lines-per-notch, as the host reports having applied it. Two,
/// so a notch against the shared ten-point line extent travels twenty points
/// and no number here can be confused with the line count that produced it.
pub(super) const LINES_PER_NOTCH: u16 = 2;
pub(super) const ONE_NOTCH: UiHostScrollDeltaPrecision = UiHostScrollDeltaPrecision::Line {
    platform_lines_per_notch: LINES_PER_NOTCH,
    basis: UiHostScrollLineCountBasis::PlatformReported,
};

/// One notch toward the end of the content, as the host reports it: a count of
/// lines in host sign, with the platform's multiplier already in it.
pub(super) fn one_notch_down() -> i64 {
    -i64::from(LINES_PER_NOTCH) * UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT
}

/// The World with a wheel that moves the offset as it arrives, and a primary
/// region that does or does not declare how far one of its lines reaches.
pub(super) fn immediate_world(line_extent: bool) -> ScrollWorld {
    ScrollWorld::publish_with_nested_content(World::launch_with_scroll(WorldScroll {
        policy: crate::declaration::UiScrollPolicy::nested_region(),
        region: scroll_region(line_extent),
    }))
}

/// One notch moves the platform's lines-per-notch times the declared extent,
/// and lands nowhere near the line count it was reported as.
#[test]
fn one_notch_travels_the_platform_line_count_times_the_declared_extent() {
    let mut scroll = immediate_world(true);
    assert_eq!(
        scroll.accepted_offset(),
        block(0),
        "the reader starts at rest"
    );

    let outcome = scroll.wheel(ONE_NOTCH, one_notch_down(), 5);
    assert!(
        matches!(outcome, UiHostScrollObservationOutcome::Applied(_)),
        "an immediate wheel over a region declaring a line extent applies: {outcome:?}"
    );

    let travelled = i64::from(LINES_PER_NOTCH) * i64::from(LINE_EXTENT_POINTS);
    scroll.publish_direct(6);
    assert_eq!(
        scroll.accepted_offset(),
        block(travelled),
        "two lines of ten points each"
    );
    assert_ne!(
        scroll.accepted_offset(),
        block(i64::from(LINES_PER_NOTCH)),
        "and not the line count spent as though a line were a point"
    );
    assert_eq!(
        scroll.displayed_offset(),
        Some(block(travelled)),
        "the pixels moved the same distance the offset did"
    );
    let _ = scroll.world.session.shutdown();
}

/// The same notch over a region that never said how tall its lines are. There
/// is no distance to travel and none the runtime is entitled to invent, so the
/// notch is refused and the reader stays where they were.
#[test]
fn a_notch_over_a_region_declaring_no_line_extent_moves_nothing() {
    let mut scroll = immediate_world(false);

    let outcome = scroll.wheel(ONE_NOTCH, one_notch_down(), 5);
    assert!(
        matches!(
            outcome,
            UiHostScrollObservationOutcome::Denied(
                UiHostScrollObservationDenial::OwnerDeclaresNoLineExtent
            )
        ),
        "a line count with no extent behind it is refused: {outcome:?}"
    );
    assert_eq!(
        scroll.accepted_offset(),
        block(0),
        "and the reader stays where they were"
    );
    let _ = scroll.world.session.shutdown();
}

/// The control. A host that reported pixels reported a distance already, and
/// the region's line extent has nothing to say about it -- the same region that
/// scales a notch by ten leaves a pixel delta exactly as it arrived.
#[test]
fn a_pixel_delta_is_the_distance_the_host_reported() {
    const TRAVEL_POINTS: i64 = 12;
    let mut scroll = immediate_world(true);

    let outcome = scroll.wheel(
        UiHostScrollDeltaPrecision::Pixel,
        -TRAVEL_POINTS * UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT,
        5,
    );
    assert!(
        matches!(outcome, UiHostScrollObservationOutcome::Applied(_)),
        "a pixel wheel applies: {outcome:?}"
    );
    scroll.publish_direct(6);
    assert_eq!(
        scroll.accepted_offset(),
        block(TRAVEL_POINTS),
        "the pixels the host measured, untouched by the declared line extent"
    );
    let _ = scroll.world.session.shutdown();
}

/// A notch no owner in the chain can take is not refused for a declaration it
/// never needed.
///
/// The reader is already at the top and pushes further up. No owner has room,
/// so no owner will move, so there is no owner whose extent would measure the
/// notch -- and a region declaring none is beside the point. Refusing here
/// would make an extent the price of asking for travel that does not exist.
#[test]
fn a_notch_no_owner_can_take_is_not_refused_for_an_extent_it_never_needed() {
    let mut scroll = immediate_world(false);
    assert_eq!(
        scroll.accepted_offset(),
        block(0),
        "the reader starts at the top, so there is nowhere further up to go"
    );

    let outcome = scroll.wheel(ONE_NOTCH, -one_notch_down(), 5);
    assert!(
        matches!(outcome, UiHostScrollObservationOutcome::Applied(_)),
        "the notch is answered rather than refused: {outcome:?}"
    );
    assert_eq!(
        scroll.accepted_offset(),
        block(0),
        "and it moved nothing, because nothing had anywhere to move"
    );
    let _ = scroll.world.session.shutdown();
}

/// The notch is measured against the owner the gesture latched to, and in this
/// world that is also the owner under the pointer, because the chain is one
/// owner deep.
///
/// Which owner answers only separates the two readings on a chain deeper than
/// its innermost owner, and no authored source can build one. A graph node is
/// either the root page or a direct child of it -- those are the only two
/// parent claims the topology admits -- so an ownership chain reaches at most
/// the component and the root page above it. The root page is not authored: it
/// is the runtime bootstrap artifact, which carries no region and so owns no
/// scroll. Every chain an application can build is therefore exactly one owner
/// deep, and the difference between the latched owner and the innermost owner
/// has nowhere to show itself.
///
/// So this is not evidence for the rule. It is evidence for why the rule
/// cannot be put to the test here, and it is what fails on the day that stops
/// being true: a chain resolved two owners deep means the distinction has
/// become observable and is owed a scenario that discriminates.
#[test]
fn the_chain_a_notch_travels_is_one_owner_deep() {
    let mut scroll = immediate_world(true);
    let target = scroll.world.instances[0];

    // The resolved chain, not the owners a route reached: a route stops at the
    // first owner that leaves nothing over, so a notch this region can take
    // whole would report one owner visited however deep the chain beneath it
    // ran. What is being pinned here is the depth itself.
    let owners = scroll
        .world
        .session
        .scroll
        .as_ref()
        .expect("the World installs Scroll from policy")
        .ownership_chain(target)
        .expect("the scrolled component resolves its Scroll chain")
        .owners()
        .len();
    assert_eq!(
        owners, 1,
        "the latched owner and the owner under the pointer are one owner, so no          scenario in this world can tell the two readings apart"
    );

    let outcome = scroll.wheel(ONE_NOTCH, one_notch_down(), 5);
    assert!(
        matches!(outcome, UiHostScrollObservationOutcome::Applied(_)),
        "and the notch that travels it is the production one: {outcome:?}"
    );
    let _ = scroll.world.session.shutdown();
}
