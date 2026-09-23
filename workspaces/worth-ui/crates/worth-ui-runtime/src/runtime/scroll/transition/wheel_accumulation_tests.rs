//! Coarse-wheel evidence becoming scroll points, stated as plain integers.

use super::*;

const LINES_PER_NOTCH: u16 = 3;
const LINE_EXTENT_POINTS: u16 = 20;
const SETTLE_TICKS: u32 = 8;

/// The host encodes lines as thousandths so a high-resolution wheel keeps its
/// fraction, and a notch is whole lines in that encoding.
#[test]
fn a_notch_is_its_lines_in_the_hosts_thousandths_of_a_line_encoding() {
    assert_eq!(UI_SCROLL_WHEEL_LINE_MILLI_PER_LINE, 1_000);
    let lines = UiScrollWheelLineDelta::from_notches(0, 1, LINES_PER_NOTCH);
    assert_eq!(lines.inline_milli_lines(), 0);
    assert_eq!(
        lines.block_milli_lines(),
        i64::from(LINES_PER_NOTCH) * UI_SCROLL_WHEEL_LINE_MILLI_PER_LINE
    );
}

/// A high-resolution wheel reporting a fraction of a line keeps that fraction:
/// a thousandth of a line against a twenty-point extent is twenty subpixels.
#[test]
fn a_fractional_line_survives_the_product_into_subpixels() {
    let input = UiScrollWheelInput::admit(
        UiScrollWheelLineDelta::new(0, 1),
        worth_ui_host_contract::UiHostScrollDeltaPhase::Updated,
        4,
        LINE_EXTENT_POINTS,
        SETTLE_TICKS,
    )
    .expect("admitted");
    assert_eq!(
        input
            .points_subpixels()
            .expect("in range")
            .block_subpixels(),
        i64::from(LINE_EXTENT_POINTS)
    );
}

#[test]
fn an_unusable_line_extent_or_absent_settle_horizon_is_refused_at_admission() {
    assert_eq!(
        UiScrollWheelInput::admit(
            UiScrollWheelLineDelta::from_notches(0, 1, LINES_PER_NOTCH),
            worth_ui_host_contract::UiHostScrollDeltaPhase::Updated,
            1,
            0,
            SETTLE_TICKS,
        ),
        Err(UiScrollTransitionDenial::LineExtentUnusable)
    );
    assert_eq!(
        UiScrollWheelInput::admit(
            UiScrollWheelLineDelta::from_notches(0, 1, LINES_PER_NOTCH),
            worth_ui_host_contract::UiHostScrollDeltaPhase::Updated,
            1,
            LINE_EXTENT_POINTS,
            0,
        ),
        Err(UiScrollTransitionDenial::SettleHorizonUnavailable)
    );
}
