//! Facade-level proof that a modal takes a scroll gesture away from the
//! content it closed over.
//!
//! A latch exists so that the rest of one physical gesture stays with the
//! owner it started on, and it answers before the pointer is consulted. That
//! shortcut is also where modal shielding would be lost: shielding is applied
//! where a coordinate becomes a target, and a latched event never asks that
//! question. So a reader who flicks a list and is handed a dialog mid-flick
//! would keep scrolling the list behind it.
//!
//! Both halves are asserted against the same World, because the interesting
//! claim is not that a republished frame stops a gesture -- it does not -- but
//! that an accepted modal does. The two scenarios open the same overlay
//! machinery at the same point in the same gesture and differ only in whether
//! the overlay shields.

use super::scroll_pose_authority::{block, ScrollWorld};
use crate::runtime::scroll::{UiHostScrollObservationDenial, UiHostScrollObservationOutcome};
use worth_ui_host_contract::{
    UiHostScrollDeltaPhase, UiHostScrollDeltaPrecision, UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT,
};

/// The overlay declaration the World publishes with modal input shielding.
const SHIELDING_OVERLAY: &str = "overlay.child";
/// The overlay declaration the World publishes shielding only its own bounds,
/// so everything behind it keeps its input.
const UNSHIELDING_OVERLAY: &str = "overlay.menu";

const GESTURE_START_TICK: u64 = 5;
const OVERLAY_TICK: u64 = 10;
const CONTINUATION_TICK: u64 = 15;

/// Travel far enough to be unmistakable, reported the way a host reports it:
/// content pushed toward the top of the viewport is a negative block delta and
/// a positive offset.
const TRAVEL_POINTS: i64 = 10;

fn pixels(points: i64) -> i64 {
    -points * UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT
}

/// A World whose first component has taken a scroll gesture and moved once, so
/// a latch is held and the offset it moved is on the record.
fn world_mid_gesture() -> ScrollWorld {
    let mut scroll = ScrollWorld::launch_published();
    let target = scroll.pointer_target();
    assert!(
        matches!(
            scroll.targeted_wheel(
                UiHostScrollDeltaPhase::Started,
                target,
                UiHostScrollDeltaPrecision::Pixel,
                pixels(TRAVEL_POINTS),
                GESTURE_START_TICK,
            ),
            UiHostScrollObservationOutcome::Applied(_)
        ),
        "the first event of the gesture routes from the pointer and takes the latch"
    );
    scroll.publish_direct(GESTURE_START_TICK);
    assert_eq!(scroll.accepted_offset(), block(TRAVEL_POINTS));
    scroll
}

/// The rest of the gesture, reported the way a host reports it once it can no
/// longer say which owner the pointer is over. Only a latch can answer this,
/// which is what makes it the question worth asking of a shielded frame.
fn continue_the_gesture(scroll: &mut ScrollWorld) -> UiHostScrollObservationOutcome {
    let target = scroll.surface_only_target();
    scroll.targeted_wheel(
        UiHostScrollDeltaPhase::Updated,
        target,
        UiHostScrollDeltaPrecision::Pixel,
        pixels(TRAVEL_POINTS),
        CONTINUATION_TICK,
    )
}

/// An accepted modal ends the gesture's reach into the content behind it. The
/// denial is typed and the offset is exactly where the modal found it.
#[test]
fn an_accepted_modal_stops_the_gesture_it_closed_over() {
    let mut scroll = world_mid_gesture();
    scroll.world.open(0, SHIELDING_OVERLAY, None, OVERLAY_TICK);

    let outcome = continue_the_gesture(&mut scroll);
    assert!(
        matches!(
            outcome,
            UiHostScrollObservationOutcome::Denied(
                UiHostScrollObservationDenial::LatchedOwnerNotAdmitted(_)
            )
        ),
        "the latch names content this frame no longer admits: {outcome:?}"
    );
    assert_eq!(
        scroll.accepted_offset(),
        block(TRAVEL_POINTS),
        "the reader stopped scrolling the moment the dialog covered the list"
    );
    let _ = scroll.world.session.shutdown();
}

/// The same overlay machinery, the same republished frame, the same point in
/// the same gesture -- without shielding. The gesture carries on, so what
/// stopped it above was the modality rather than the republication.
#[test]
fn an_overlay_that_shields_only_itself_leaves_the_gesture_alone() {
    let mut scroll = world_mid_gesture();
    scroll
        .world
        .open(0, UNSHIELDING_OVERLAY, None, OVERLAY_TICK);

    let outcome = continue_the_gesture(&mut scroll);
    assert!(
        matches!(outcome, UiHostScrollObservationOutcome::Applied(_)),
        "an overlay that admits the input behind it leaves the latch answerable: {outcome:?}"
    );
    let frame = scroll
        .world
        .prepare_surface_with_current_portals(scroll.surface());
    scroll.world.publish(frame, CONTINUATION_TICK, true);
    assert_eq!(
        scroll.accepted_offset(),
        block(TRAVEL_POINTS * 2),
        "the second event of the gesture moved the same owner again"
    );
    let _ = scroll.world.session.shutdown();
}
