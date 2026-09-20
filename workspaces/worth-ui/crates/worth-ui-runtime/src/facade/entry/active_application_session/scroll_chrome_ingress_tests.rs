//! What pointer ingress hands to chrome, and what a captured axis does to a
//! wheel that arrives during the drag.
//!
//! The decision is `scroll_chrome_pointer_intent`, a function of the report and
//! the latch, so it can be read here directly against reports built from the
//! host contract's own payloads. Driving it through a live session would need a
//! declared mosaic region that owns a scroll viewport, which no fixture in this
//! crate declares; the placement arithmetic those presses and drags go on to
//! perform is proven against an independent oracle in the chrome geometry
//! tests.

use super::scroll_chrome_ingress::{
    chrome_point, pointer_report_position, scroll_chrome_pointer_intent, suppress_captured_axes,
    UiScrollChromePointerIntent,
};
use worth_ui_host_contract::{
    UiHostObservationPayload, UiHostPointerButton, UiHostPointerButtonTransition,
    UiHostPointerCaptureEpoch, UiHostPointerIdentity, UiHostPressedPointerButtons,
    UiHostSurfacePosition, UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT,
};

fn pointer(value: u64) -> UiHostPointerIdentity {
    UiHostPointerIdentity::new(value)
}

fn epoch(value: u64) -> UiHostPointerCaptureEpoch {
    UiHostPointerCaptureEpoch::new(value)
}

fn at(x_points: f64, y_points: f64) -> UiHostSurfacePosition {
    let scale = UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as f64;
    UiHostSurfacePosition::viewport_logical((x_points * scale) as i64, (y_points * scale) as i64)
}

fn button(
    pointer_value: u64,
    epoch_value: u64,
    button: UiHostPointerButton,
    transition: UiHostPointerButtonTransition,
    position: UiHostSurfacePosition,
) -> UiHostObservationPayload {
    UiHostObservationPayload::PointerButton {
        pointer: pointer(pointer_value),
        capture_epoch: epoch(epoch_value),
        button,
        transition,
        position,
    }
}

fn motion(
    pointer_value: u64,
    epoch_value: u64,
    position: UiHostSurfacePosition,
) -> UiHostObservationPayload {
    UiHostObservationPayload::PointerMotion {
        pointer: pointer(pointer_value),
        capture_epoch: epoch(epoch_value),
        pressed_buttons: UiHostPressedPointerButtons::from_buttons([UiHostPointerButton::Primary]),
        position,
    }
}

/// A primary press with no drag running asks chrome first. Chrome may still
/// decline it -- that is the point at which an ordinary press stays ordinary --
/// but the question is asked of every press.
#[test]
fn a_primary_press_asks_chrome_before_the_node_tree() {
    assert_eq!(
        scroll_chrome_pointer_intent(
            &button(
                1,
                7,
                UiHostPointerButton::Primary,
                UiHostPointerButtonTransition::Pressed,
                at(760.0, 40.0),
            ),
            None,
        ),
        UiScrollChromePointerIntent::Press {
            pointer: pointer(1),
            capture_epoch: epoch(7),
            position: at(760.0, 40.0),
        }
    );
}

/// Without a latch, moves and releases are ordinary routing's: nothing in this
/// lane touches a pointer that is not dragging a thumb.
#[test]
fn ordinary_pointer_traffic_is_untouched_when_no_drag_is_latched() {
    assert_eq!(
        scroll_chrome_pointer_intent(&motion(1, 7, at(100.0, 100.0)), None),
        UiScrollChromePointerIntent::Untouched
    );
    assert_eq!(
        scroll_chrome_pointer_intent(
            &button(
                1,
                7,
                UiHostPointerButton::Primary,
                UiHostPointerButtonTransition::Released,
                at(100.0, 100.0),
            ),
            None,
        ),
        UiScrollChromePointerIntent::Untouched
    );
    assert_eq!(
        scroll_chrome_pointer_intent(
            &button(
                1,
                7,
                UiHostPointerButton::Secondary,
                UiHostPointerButtonTransition::Pressed,
                at(760.0, 40.0),
            ),
            None,
        ),
        UiScrollChromePointerIntent::Untouched,
        "only the primary button presses a scrollbar"
    );
}

/// While a drag is latched, that pointer's moves and its release belong to the
/// thumb, and nothing else does: another pointer's move, and a move under a
/// capture the host has since replaced, are not this drag's.
#[test]
fn a_latched_drag_claims_its_own_pointers_moves_and_release() {
    let latched = Some((pointer(1), epoch(7)));
    assert_eq!(
        scroll_chrome_pointer_intent(&motion(1, 7, at(760.0, 90.0)), latched),
        UiScrollChromePointerIntent::Drag {
            pointer: pointer(1),
            capture_epoch: epoch(7),
            position: at(760.0, 90.0),
        }
    );
    assert_eq!(
        scroll_chrome_pointer_intent(
            &button(
                1,
                7,
                UiHostPointerButton::Primary,
                UiHostPointerButtonTransition::Released,
                at(760.0, 90.0),
            ),
            latched,
        ),
        UiScrollChromePointerIntent::Release {
            pointer: pointer(1),
            capture_epoch: epoch(7),
        }
    );
    assert_eq!(
        scroll_chrome_pointer_intent(&motion(2, 7, at(760.0, 90.0)), latched),
        UiScrollChromePointerIntent::Untouched,
        "a second pointer is not dragging this thumb"
    );
    assert_eq!(
        scroll_chrome_pointer_intent(&motion(1, 8, at(760.0, 90.0)), latched),
        UiScrollChromePointerIntent::Untouched,
        "a re-captured pointer's moves predate no latch this lane holds"
    );
}

/// One pointer drags one thumb. A second press arriving mid-drag is left to
/// ordinary routing rather than silently stealing the capture.
#[test]
fn a_second_press_during_a_drag_is_not_chromes_to_take() {
    assert_eq!(
        scroll_chrome_pointer_intent(
            &button(
                2,
                7,
                UiHostPointerButton::Primary,
                UiHostPointerButtonTransition::Pressed,
                at(760.0, 40.0),
            ),
            Some((pointer(1), epoch(7))),
        ),
        UiScrollChromePointerIntent::Untouched
    );
}

/// The point a report names is read in the same logical points the chrome
/// rectangles are derived in, so a press at 760.5 points is a press at 760.5
/// points and not at its subpixel count.
#[test]
fn a_reports_position_is_read_in_logical_points() {
    assert_eq!(chrome_point(at(760.5, 41.25)), [760.5, 41.25]);
}

/// A wheel on the axis a thumb drag holds is ignored while the drag runs: the
/// drag is placing that offset directly, and a wheel moving the same offset
/// underneath it would fight the pointer.
#[test]
fn a_captured_axis_silences_that_axis_of_the_wheel_and_no_other() {
    assert_eq!(
        suppress_captured_axes([3_000, 4_000], [false, true]),
        [3_000, 0],
        "the dragged axis is silenced and the other axis still scrolls"
    );
    assert_eq!(
        suppress_captured_axes([3_000, 4_000], [true, false]),
        [0, 4_000]
    );
    assert_eq!(
        suppress_captured_axes([3_000, 4_000], [true, true]),
        [0, 0],
        "a drag on both axes leaves the wheel nothing to move"
    );
    assert_eq!(
        suppress_captured_axes([3_000, 4_000], [false, false]),
        [3_000, 4_000],
        "with no drag running the wheel is untouched"
    );
}

/// Hover is re-resolved from wherever the pointer last was, so every report that
/// carries a position -- a move, a press, a release, whichever button and
/// whether or not a drag is latched -- supplies one, and nothing else does.
#[test]
fn every_positioned_pointer_report_supplies_the_hover_position() {
    let position = at(750.0, 40.0);
    for payload in [
        motion(1, 1, position),
        button(
            1,
            1,
            UiHostPointerButton::Primary,
            UiHostPointerButtonTransition::Pressed,
            position,
        ),
        button(
            1,
            1,
            UiHostPointerButton::Primary,
            UiHostPointerButtonTransition::Released,
            position,
        ),
        button(
            2,
            3,
            UiHostPointerButton::Secondary,
            UiHostPointerButtonTransition::Pressed,
            position,
        ),
    ] {
        assert_eq!(
            pointer_report_position(&payload),
            Some(position),
            "{payload:?} carries the position the pointer is at"
        );
    }
    assert_eq!(
        pointer_report_position(&UiHostObservationPayload::Tick { tick: 7 }),
        None,
        "a report with no pointer on it names no hover position"
    );
}
