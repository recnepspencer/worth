//! Facade-level proof that replacing a scrolled region's layout does not move
//! the content the reader is looking at.
//!
//! An offset is a distance from rest, and a layout replacement is what moves
//! rest. Rows inserted above the reader push everything below them down inside
//! the content, so an offset carried across unchanged would carry the reader
//! up by the height of the insertion. The region instead remembers one of its
//! content occurrences and how far into the content it sits, and moves the
//! offset by whatever distance that occurrence moved.
//!
//! The occurrence it remembers in this World is the first component's child,
//! which is the only content that component's region carries when the World
//! launches. The third component joins that content afterwards, which is what
//! lets these scenarios take the remembered occurrence out of the region while
//! leaving it carrying content that moved.
//!
//! Both halves of the rule are here. When the remembered occurrence survives
//! the replacement, the offset follows it. When it does not, there is nothing
//! left to measure against, so the existing offset is clamped rather than
//! rebased off whichever sibling happened to remain.

use super::geometry::scrollable::{
    install_scrollable_primary_with_inserted_content,
    install_scrollable_primary_without_the_anchored_content, INSERTED_POINTS,
};
use super::scroll_pose_authority::{block, ScrollWorld};
use super::World;
use crate::runtime::scroll::UiHostScrollObservationOutcome;
use worth_ui_host_contract::{
    UiHostScrollDeltaPrecision, UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT,
};

/// How far the reader has scrolled before the layout is replaced under them.
const TRAVEL_POINTS: i64 = 10;

/// How far the content moves, as an offset distance. The installers state the
/// same move in layout points.
fn inserted() -> i64 {
    INSERTED_POINTS as i64
}

/// The nested World with the reader ten points into the content.
fn scrolled_world() -> ScrollWorld {
    // The anchor belongs to what the reader actually saw. Publish the original
    // one-child layout before admitting a sibling into the region; an earlier
    // unpresented layout is no longer allowed to seed accepted Scroll records.
    let mut scroll = ScrollWorld::publish(World::launch());
    super::geometry::scrollable::install_scrollable_primary_with_nested_content(
        &mut scroll.world.session,
        scroll.world.surfaces,
        scroll.world.instances,
    );
    let frame = scroll.world.prepare_surface(scroll.surface());
    scroll.world.publish(frame, 2, false);
    let outcome = scroll.wheel(
        UiHostScrollDeltaPrecision::Pixel,
        -TRAVEL_POINTS * UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT,
        5,
    );
    assert!(
        matches!(outcome, UiHostScrollObservationOutcome::Applied(_)),
        "an immediate wheel over the region applies: {outcome:?}"
    );
    scroll.publish_direct(5);
    assert_eq!(scroll.accepted_offset(), block(TRAVEL_POINTS));
    scroll
}

/// Rows are inserted at the top of the region's content, so the occurrence the
/// region is holding now begins `INSERTED_POINTS` further into it. The offset
/// moves the same distance and the reader keeps looking at the same part of
/// the content.
#[test]
fn content_inserted_above_the_anchor_moves_the_offset_with_it() {
    let mut scroll = scrolled_world();
    install_scrollable_primary_with_inserted_content(
        &mut scroll.world.session,
        scroll.world.surfaces,
        scroll.world.instances,
    );
    let frame = scroll.world.prepare_surface(scroll.surface());
    scroll.world.publish(frame, 6, false);

    assert_eq!(
        scroll.accepted_offset(),
        block(TRAVEL_POINTS + inserted()),
        "the offset moved exactly as far as the anchored content did"
    );
    assert_eq!(
        scroll.mounted_offset(),
        Some(block(TRAVEL_POINTS + inserted())),
        "the displayed pose is the offset Scroll holds, so the pixels moved too"
    );
    let _ = scroll.world.session.shutdown();
}

/// The occurrence the region was holding is no longer inside it, and the
/// content that remains has moved. There is no surviving basis to measure the
/// move against, so the offset is clamped where it was rather than dragged by
/// a sibling the reader was never anchored to.
#[test]
fn losing_the_anchor_clamps_the_offset_instead_of_following_a_sibling() {
    let mut scroll = scrolled_world();
    install_scrollable_primary_without_the_anchored_content(
        &mut scroll.world.session,
        scroll.world.surfaces,
        scroll.world.instances,
    );
    let frame = scroll.world.prepare_surface(scroll.surface());
    scroll.world.publish(frame, 6, false);

    assert_eq!(
        scroll.accepted_offset(),
        block(TRAVEL_POINTS),
        "an offset with nothing left to anchor to stays where it was"
    );
    assert_eq!(
        scroll.mounted_offset(),
        Some(block(TRAVEL_POINTS)),
        "and the pixels stay with it"
    );
    let _ = scroll.world.session.shutdown();
}
