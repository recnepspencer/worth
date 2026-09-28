//! A host delta keeps the unit its precision names, in offset direction.
use super::*;
use crate::units::host_count_of;
use worth_ui_host_contract::{UiHostScrollDeltaPrecision, UiHostScrollLineCountBasis};

const LINES: UiHostScrollDeltaPrecision = UiHostScrollDeltaPrecision::Line {
    platform_lines_per_notch: 3,
    basis: UiHostScrollLineCountBasis::PlatformReported,
};

#[test]
fn a_pixel_delta_is_a_distance_turned_to_offset_direction() {
    assert_eq!(
        UiScrollHostTravel::from_host(UiHostScrollDeltaPrecision::Pixel, [250, -12_000]),
        Some(UiScrollHostTravel::Distance(
            super::super::UiScrollDelta::new(-250, 12_000)
        ))
    );
}

/// Lines and pages are counts; neither is ever handed on as a distance.
#[test]
fn line_and_page_deltas_stay_counts() {
    assert_eq!(
        UiScrollHostTravel::from_host(LINES, [0, -host_count_of(3)]),
        Some(UiScrollHostTravel::Lines(
            super::super::transition::UiScrollWheelLineDelta::new(0, 3_000)
        ))
    );
    assert_eq!(
        UiScrollHostTravel::from_host(UiHostScrollDeltaPrecision::Page, [0, -host_count_of(1)]),
        Some(UiScrollHostTravel::Pages {
            inline_milli_pages: 0,
            block_milli_pages: 1_000,
        })
    );
}

#[test]
fn a_delta_with_no_offset_direction_counterpart_is_refused() {
    assert_eq!(
        UiScrollHostTravel::from_host(UiHostScrollDeltaPrecision::Pixel, [i64::MIN, 0]),
        None
    );
}

#[test]
fn every_unit_heads_the_way_its_sign_says() {
    let down = UiScrollHeading::new(0, 1);
    for precision in [
        UiHostScrollDeltaPrecision::Pixel,
        LINES,
        UiHostScrollDeltaPrecision::Page,
    ] {
        let travel = UiScrollHostTravel::from_host(precision, [0, -host_count_of(1)]).unwrap();
        assert_eq!(travel.heading(), down, "{precision:?}");
    }
}

/// One page notch travels one page step, not the thousand subpixels its
/// encoding happens to count.
#[test]
fn a_page_travels_its_page_step() {
    let step = 176_000;
    assert_eq!(
        page_travel([0, 1_000], [640_000, step]),
        Some(super::super::UiScrollDelta::new(0, step))
    );
    assert_eq!(
        page_travel([-500, 2_000], [640_000, step]),
        Some(super::super::UiScrollDelta::new(-320_000, 2 * step))
    );
    assert_eq!(page_travel([0, i64::MAX], [0, step]), None);
}
