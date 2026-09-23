//! Which axis a point belongs to, what a press on it means, and the offset a
//! drag or a track click places.

use super::geometry_oracle::*;
use super::pointer_target::{classify_press, UiScrollChromePress};
use super::*;

fn both_axis_facts() -> UiScrollChromeFacts {
    UiScrollChromeFacts::derive(
        viewport_box(),
        bounds(CONTENT_WIDTH, CONTENT_HEIGHT),
        offset(0.0, 0.0),
        &chrome(UiScrollChromeAxisSupport::Both),
    )
    .expect("both axes overflow")
}

/// The oracle states the milestone's numbers as its own literals. This pins the
/// declared constants to those same numbers, so the two can only agree
/// deliberately, and shows the admissions that refuse an unusable chrome.
#[test]
fn the_declared_metrics_are_the_numbers_the_oracle_states() {
    assert!((f64::from(UI_SCROLL_GUTTER_LOGICAL_POINTS) - GUTTER).abs() < f64::EPSILON);
    assert!((f64::from(UI_SCROLL_THUMB_LOGICAL_POINTS) - THUMB_THICKNESS).abs() < f64::EPSILON);
    assert!(
        (f64::from(UI_SCROLL_THUMB_MINIMUM_LOGICAL_POINTS) - MINIMUM_THUMB_LENGTH).abs()
            < f64::EPSILON
    );

    assert_eq!(
        UiScrollChromeMetrics::admit(
            UI_SCROLL_GUTTER_LOGICAL_POINTS,
            UI_SCROLL_GUTTER_LOGICAL_POINTS + 1.0,
            UI_SCROLL_THUMB_MINIMUM_LOGICAL_POINTS,
        ),
        Err(UiScrollChromeMetricsDenial::ThumbThicknessExceedsGutter)
    );
    assert_eq!(
        UiScrollAdmittedChrome::admit_declared_chrome(
            UiScrollChromeAxisSupport::Both,
            worth_ui_dsl::UiAppearanceRoleIdentity::new("test.one_role").expect("role"),
            worth_ui_dsl::UiAppearanceRoleIdentity::new("test.one_role").expect("role"),
            UiScrollChromeMetrics::declared(),
        ),
        Err(UiScrollChromeAdmissionDenial::RoleIdentitiesCollide)
    );
}

#[test]
fn a_point_across_the_gutter_classifies_to_one_axis_and_the_corner_to_neither() {
    let facts = both_axis_facts();
    let block_track = oracle_track(true, true);
    let inline_track = oracle_track(false, true);

    for across in [0.5, 6.0, 11.5] {
        assert_eq!(
            facts.pointer_axis([(block_track[0] + across) as f32, 100.0]),
            Some(UiScrollChromeAxis::Block),
            "the effective target fills the whole gutter"
        );
        assert_eq!(
            facts.pointer_axis([100.0, (inline_track[1] + across) as f32]),
            Some(UiScrollChromeAxis::Inline)
        );
    }
    assert_eq!(
        facts.pointer_axis([(block_track[0] - 0.5) as f32, 100.0]),
        None
    );
    assert_eq!(
        facts.pointer_axis([
            (block_track[0] + 6.0) as f32,
            (inline_track[1] + 6.0) as f32
        ]),
        None,
        "the shared corner belongs to neither track"
    );
}

/// The thumb is six points wide but the whole twelve-point gutter is its
/// pointer target, so a press beside the visible thumb still grabs it.
#[test]
fn the_effective_pointer_rect_fills_the_gutter_across_the_thumb() {
    let facts = both_axis_facts();
    let block: UiScrollChromeAxisFacts = facts.axis(UiScrollChromeAxis::Block).expect("block");
    let track = oracle_track(true, true);
    let (length, travel) = oracle_clamped_length_and_travel(
        track[3],
        oracle_proportional_thumb_length(track[3], VIEWPORT_HEIGHT, CONTENT_HEIGHT),
    );

    assert!((f64::from(block.extent().length_logical_points()) - length).abs() < 0.01);
    assert!((f64::from(block.extent().travel_logical_points()) - travel).abs() < 0.01);
    assert_eq!(
        block.max_offset_subpixels(),
        ((CONTENT_HEIGHT - VIEWPORT_HEIGHT) * SUBPIXELS_PER_POINT) as i64
    );

    let pointer = block
        .effective_thumb_pointer_rect(UiScrollChromeAxis::Block)
        .expect("a thumb has a pointer target");
    assert_box(pointer, [track[0], track[1], GUTTER, length]);
    assert_eq!(
        effective_thumb_pointer_rect(UiScrollChromeAxis::Block, block.track(), block.thumb()),
        Some(pointer)
    );
    assert!(rect_contains(
        pointer,
        [(track[0] + 0.5) as f32, (track[1] + 1.0) as f32]
    ));
    assert!(!rect_contains(
        pointer,
        [(track[0] - 0.5) as f32, (track[1] + 1.0) as f32]
    ));
}

/// A press on the track is classified by which side of the thumb it fell, so a
/// page step knows which way to move without re-deriving the geometry.
#[test]
fn a_track_press_is_classified_against_the_thumb_it_missed() {
    let facts = UiScrollChromeFacts::derive(
        viewport_box(),
        bounds(CONTENT_WIDTH, CONTENT_HEIGHT),
        offset(0.0, (CONTENT_HEIGHT - VIEWPORT_HEIGHT) / 2.0),
        &chrome(UiScrollChromeAxisSupport::Both),
    )
    .expect("both axes overflow");
    let thumb = facts
        .axis(UiScrollChromeAxis::Block)
        .expect("block")
        .thumb();
    let start = f64::from(thumb.y());
    let end = start + f64::from(thumb.height());

    assert_eq!(
        classify_press(
            UiScrollChromeAxis::Block,
            thumb,
            [394.0, (start - 5.0) as f32]
        ),
        UiScrollChromePress::TrackBeforeThumb
    );
    assert_eq!(
        classify_press(
            UiScrollChromeAxis::Block,
            thumb,
            [394.0, (start + 1.0) as f32]
        ),
        UiScrollChromePress::Thumb
    );
    assert_eq!(
        classify_press(
            UiScrollChromeAxis::Block,
            thumb,
            [394.0, (end + 5.0) as f32]
        ),
        UiScrollChromePress::TrackAfterThumb
    );
}

/// A press anywhere on the thumb establishes a grab offset the drag keeps, so
/// the content under the grabbed point does not jump on capture.
#[test]
fn direct_drag_preserves_the_grab_offset_established_at_the_press() {
    let track = oracle_track(true, true);
    let (length, travel) = oracle_clamped_length_and_travel(
        track[3],
        oracle_proportional_thumb_length(track[3], VIEWPORT_HEIGHT, CONTENT_HEIGHT),
    );
    let max_offset_points = CONTENT_HEIGHT - VIEWPORT_HEIGHT;
    let pressed_at_points = max_offset_points / 2.0;
    let thumb_start = oracle_thumb_start(travel, pressed_at_points, max_offset_points);

    let facts = UiScrollChromeFacts::derive(
        viewport_box(),
        bounds(CONTENT_WIDTH, CONTENT_HEIGHT),
        offset(0.0, pressed_at_points),
        &chrome(UiScrollChromeAxisSupport::Both),
    )
    .expect("both axes overflow");
    let thumb = facts
        .axis(UiScrollChromeAxis::Block)
        .expect("block chrome")
        .thumb();

    // Grab three quarters of the way down the thumb, off the thumb's centre.
    let press = [394.0_f32, (thumb_start + length * 0.75) as f32];
    let grab = grab_offset_logical_points(UiScrollChromeAxis::Block, thumb, press);
    assert!((f64::from(grab) - length * 0.75).abs() < 0.01);

    // Releasing without moving must leave the offset exactly where it was.
    assert_eq!(
        facts
            .offset_for_thumb_position(
                UiScrollChromeAxis::Block,
                press,
                grab,
                offset(0.0, pressed_at_points)
            )
            .expect("block chrome")
            .block_subpixels(),
        (pressed_at_points * SUBPIXELS_PER_POINT).round() as i64
    );

    // Dragging 40 points down moves the thumb start 40 points down the
    // compressed travel, and the offset by the same fraction of the bound.
    let dragged = [394.0_f32, press[1] + 40.0];
    let expected_start = thumb_start + 40.0;
    let expected_offset = max_offset_points * expected_start / travel;
    let placed = facts
        .offset_for_thumb_position(
            UiScrollChromeAxis::Block,
            dragged,
            grab,
            offset(0.0, pressed_at_points),
        )
        .expect("block chrome");
    assert!(
        (placed.block_subpixels() as f64 - expected_offset * SUBPIXELS_PER_POINT).abs() < 2.0,
        "{} is not {}",
        placed.block_subpixels(),
        expected_offset * SUBPIXELS_PER_POINT
    );
    assert_eq!(placed.inline_subpixels(), 0, "the other axis must not move");
}

/// A track click pages toward the pointer by one viewport minus one line.
#[test]
fn a_track_click_pages_one_viewport_minus_one_line_toward_the_pointer() {
    let facts = both_axis_facts();
    let line_extent_points = 20_u16;
    let expected = ((VIEWPORT_HEIGHT - f64::from(line_extent_points)) * SUBPIXELS_PER_POINT) as i64;
    assert_eq!(
        page_step_subpixels(VIEWPORT_HEIGHT as f32, line_extent_points),
        expected
    );

    assert_eq!(
        facts
            .offset_for_track_click(
                UiScrollChromeAxis::Block,
                [394.0, 250.0],
                offset(0.0, 0.0),
                line_extent_points
            )
            .expect("a click below the thumb pages forward")
            .block_subpixels(),
        expected
    );
    assert_eq!(
        facts.offset_for_track_click(
            UiScrollChromeAxis::Block,
            [394.0, 4.0],
            offset(0.0, 0.0),
            line_extent_points
        ),
        None,
        "a click on the thumb is a grab, not a page"
    );
}
