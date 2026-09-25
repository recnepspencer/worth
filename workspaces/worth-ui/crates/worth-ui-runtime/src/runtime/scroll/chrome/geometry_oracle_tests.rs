//! Track and thumb rectangles against the independent oracle.

use super::geometry_oracle::*;
use super::*;

#[test]
fn both_axes_place_tracks_in_reserved_gutters_that_surrender_the_shared_corner() {
    let facts = UiScrollChromeFacts::derive(
        viewport_box(),
        bounds(CONTENT_WIDTH, CONTENT_HEIGHT),
        offset(0.0, 0.0),
        &chrome(UiScrollChromeAxisSupport::Both),
    )
    .expect("both axes overflow");

    assert_box(
        facts
            .axis(UiScrollChromeAxis::Block)
            .expect("block chrome")
            .track(),
        oracle_track(true, true),
    );
    assert_box(
        facts
            .axis(UiScrollChromeAxis::Inline)
            .expect("inline chrome")
            .track(),
        oracle_track(false, true),
    );
}

/// A single presenting axis keeps its whole gutter: there is no other track to
/// reserve a corner for.
#[test]
fn a_single_presenting_axis_keeps_its_whole_gutter() {
    let facts = UiScrollChromeFacts::derive(
        viewport_box(),
        bounds(VIEWPORT_WIDTH, CONTENT_HEIGHT),
        offset(0.0, 0.0),
        &chrome(UiScrollChromeAxisSupport::Both),
    )
    .expect("block overflows");

    assert_box(
        facts
            .axis(UiScrollChromeAxis::Block)
            .expect("block chrome")
            .track(),
        oracle_track(true, false),
    );
}

#[test]
fn block_thumb_matches_the_proportional_oracle_at_origin_half_and_maximum() {
    let track = oracle_track(true, true);
    let (length, travel) = oracle_clamped_length_and_travel(
        track[3],
        oracle_proportional_thumb_length(track[3], VIEWPORT_HEIGHT, CONTENT_HEIGHT),
    );
    let max_offset_points = CONTENT_HEIGHT - VIEWPORT_HEIGHT;

    for offset_points in [0.0, max_offset_points / 2.0, max_offset_points] {
        let facts = UiScrollChromeFacts::derive(
            viewport_box(),
            bounds(CONTENT_WIDTH, CONTENT_HEIGHT),
            offset(0.0, offset_points),
            &chrome(UiScrollChromeAxisSupport::Both),
        )
        .expect("both axes overflow");
        let start = oracle_thumb_start(travel, offset_points, max_offset_points);
        assert_box(
            facts
                .axis(UiScrollChromeAxis::Block)
                .expect("block chrome")
                .thumb(),
            [
                track[0] + (GUTTER - THUMB_THICKNESS) / 2.0,
                track[1] + start,
                THUMB_THICKNESS,
                length,
            ],
        );
    }
}

#[test]
fn inline_thumb_matches_the_proportional_oracle_at_origin_half_and_maximum() {
    let track = oracle_track(false, true);
    let (length, travel) = oracle_clamped_length_and_travel(
        track[2],
        oracle_proportional_thumb_length(track[2], VIEWPORT_WIDTH, CONTENT_WIDTH),
    );
    let max_offset_points = CONTENT_WIDTH - VIEWPORT_WIDTH;

    for offset_points in [0.0, max_offset_points / 2.0, max_offset_points] {
        let facts = UiScrollChromeFacts::derive(
            viewport_box(),
            bounds(CONTENT_WIDTH, CONTENT_HEIGHT),
            offset(offset_points, 0.0),
            &chrome(UiScrollChromeAxisSupport::Both),
        )
        .expect("both axes overflow");
        let start = oracle_thumb_start(travel, offset_points, max_offset_points);
        assert_box(
            facts
                .axis(UiScrollChromeAxis::Inline)
                .expect("inline chrome")
                .thumb(),
            [
                track[0] + start,
                track[1] + (GUTTER - THUMB_THICKNESS) / 2.0,
                length,
                THUMB_THICKNESS,
            ],
        );
    }
}

/// Far content makes the proportional thumb shorter than the declared minimum.
/// The clamp lengthens it and the travel compresses by exactly the same amount,
/// so the thumb still starts at the track origin and still ends flush with the
/// track end.
#[test]
fn minimum_length_clamp_compresses_travel_and_still_reaches_both_ends() {
    let tall_content = 20_000.0;
    let track = oracle_track(true, false);
    let proportional = oracle_proportional_thumb_length(track[3], VIEWPORT_HEIGHT, tall_content);
    assert!(proportional < MINIMUM_THUMB_LENGTH, "the clamp must engage");
    let (length, travel) = oracle_clamped_length_and_travel(track[3], proportional);
    assert!((length - MINIMUM_THUMB_LENGTH).abs() < 0.01);
    assert!((length + travel - track[3]).abs() < 0.01);

    let max_offset_points = tall_content - VIEWPORT_HEIGHT;
    let at = |offset_points: f64| {
        UiScrollChromeFacts::derive(
            viewport_box(),
            bounds(VIEWPORT_WIDTH, tall_content),
            offset(0.0, offset_points),
            &chrome(UiScrollChromeAxisSupport::Block),
        )
        .expect("block overflow")
        .axis(UiScrollChromeAxis::Block)
        .expect("block chrome")
        .thumb()
    };

    assert_box(at(0.0), [track[0] + 3.0, track[1], THUMB_THICKNESS, length]);
    assert_box(
        at(max_offset_points / 2.0),
        [
            track[0] + 3.0,
            track[1] + travel / 2.0,
            THUMB_THICKNESS,
            length,
        ],
    );
    let end = at(max_offset_points);
    assert_box(
        end,
        [track[0] + 3.0, track[1] + travel, THUMB_THICKNESS, length],
    );
    assert!(
        (f64::from(end.y() + end.height()) - (track[1] + track[3])).abs() < 0.01,
        "the clamped thumb must still end flush with its track"
    );
}

#[test]
fn a_zero_overflow_axis_has_no_thumb_and_no_drag_target() {
    let facts = UiScrollChromeFacts::derive(
        viewport_box(),
        bounds(VIEWPORT_WIDTH, CONTENT_HEIGHT),
        offset(0.0, 0.0),
        &chrome(UiScrollChromeAxisSupport::Both),
    )
    .expect("block still overflows");

    assert!(facts.axis(UiScrollChromeAxis::Inline).is_none());
    assert!(facts.axis(UiScrollChromeAxis::Block).is_some());
    assert!(facts
        .offset_for_thumb_position(
            UiScrollChromeAxis::Inline,
            crate::mounting::presentation::platform_point_for_test(200.0, 262.0),
            0.0,
            offset(0.0, 0.0)
        )
        .is_none());

    assert!(
        UiScrollChromeFacts::derive(
            viewport_box(),
            bounds(VIEWPORT_WIDTH, VIEWPORT_HEIGHT),
            offset(0.0, 0.0),
            &chrome(UiScrollChromeAxisSupport::Both),
        )
        .is_none(),
        "an owner whose content fits presents no chrome at all"
    );
}

/// An axis the declared chrome does not support presents nothing, however much
/// its content overflows.
#[test]
fn an_unsupported_axis_presents_no_chrome_however_far_it_overflows() {
    let facts = UiScrollChromeFacts::derive(
        viewport_box(),
        bounds(CONTENT_WIDTH, CONTENT_HEIGHT),
        offset(0.0, 0.0),
        &chrome(UiScrollChromeAxisSupport::Block),
    )
    .expect("block overflows");

    assert!(facts.axis(UiScrollChromeAxis::Inline).is_none());
    assert!(facts.axis(UiScrollChromeAxis::Block).is_some());
}
