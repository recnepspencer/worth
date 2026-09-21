//! Facade-level proof that a reader keeps their place through a cold surface
//! reconstruction.
//!
//! Reconstruction throws away every qualified text layout and rebuilds the
//! surface against a new binding. Nothing the reader did is supposed to be
//! part of what is thrown away, and where they had scrolled to is something
//! they did. The offset is semantic state Scroll holds, but the pose that
//! displays it is mounted geometry that reconstruction rebuilds, so the two
//! have to be asked separately: an offset that survived while the pose came
//! back at rest would put the state and the pixels in different frames.
//!
//! The owner is resolved again afterwards rather than remembered. A rebound
//! surface mints a new incarnation, and a test that kept the old one would
//! read an offset nobody is looking at.

use super::scroll_pose_authority::{block, ScrollWorld};
use crate::runtime::scroll::{
    UiHostScrollObservationOutcome, UiScrollOffset, UiScrollOwnerIdentity,
};
use worth_ui_host_contract::{
    UiHostScrollDeltaPrecision, UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT,
};

/// Far enough down the region's thirty points of travel to be unmistakable,
/// and short of the bound so a clamp could not produce it by accident.
const TRAVEL_POINTS: i64 = 10;
const WHEEL_TICK: u64 = 5;

fn pixels(points: i64) -> i64 {
    -points * UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT
}

/// The offset the first component's region holds right now, resolved through
/// whatever owner and incarnation it currently has.
fn current_offset(scroll: &ScrollWorld) -> UiScrollOffset {
    let target = scroll.target();
    let owner = scroll
        .world
        .session
        .scroll
        .as_ref()
        .expect("Scroll stays installed")
        .ownership_chain(target)
        .expect("the first component resolves its Scroll chain")
        .owners()[0];
    assert!(
        matches!(owner, UiScrollOwnerIdentity::Region { .. }),
        "the innermost owner is still the declared region: {owner:?}"
    );
    let incarnation = scroll
        .world
        .session
        .scroll_region_incarnation(target, 0)
        .expect("the region occurrence has a current allocation");
    scroll
        .world
        .session
        .scroll
        .as_ref()
        .expect("Scroll stays installed")
        .offset(owner, incarnation)
        .expect("the region keeps an offset through its incarnation")
}

/// A published World scrolled a known distance down its region.
fn scrolled_world() -> ScrollWorld {
    let mut scroll = ScrollWorld::launch_published();
    assert!(matches!(
        scroll.wheel(
            UiHostScrollDeltaPrecision::Pixel,
            pixels(TRAVEL_POINTS),
            WHEEL_TICK,
        ),
        UiHostScrollObservationOutcome::Applied(_)
    ));
    assert_eq!(scroll.accepted_offset(), block(TRAVEL_POINTS));
    scroll
}

/// The offset the reader left the content at is the offset the rebuilt surface
/// comes back holding.
#[test]
fn a_cold_reconstruction_gives_the_reader_back_the_place_they_were_at() {
    let mut scroll = scrolled_world();

    super::reconstruction::reconstruct_surface(&mut scroll.world);

    assert_eq!(
        current_offset(&scroll),
        block(TRAVEL_POINTS),
        "rebuilding the surface rebuilds what it displays, not where the reader was"
    );
    let _ = scroll.world.session.shutdown();
}

/// And the pose that displays it comes back with it, so the rebuilt pixels
/// describe the same frame the offset does.
#[test]
fn the_rebuilt_surface_displays_the_offset_it_came_back_holding() {
    let mut scroll = scrolled_world();

    super::reconstruction::reconstruct_surface(&mut scroll.world);

    assert_eq!(
        scroll.displayed_offset(),
        Some(block(TRAVEL_POINTS)),
        "the rebuilt pose is built from the offset rather than from rest"
    );
    let _ = scroll.world.session.shutdown();
}
