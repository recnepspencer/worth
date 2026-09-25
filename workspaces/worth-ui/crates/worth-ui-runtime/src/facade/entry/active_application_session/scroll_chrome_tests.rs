//! Scroll chrome at Pulse's declared RecentActivity extents, against an
//! independent oracle.
//!
//! Every expected rectangle here is written as plain arithmetic over the
//! milestone's declared numbers — a 12-point gutter, a 6-point thumb centred in
//! it, a 24-point minimum length, and a shared corner surrendered by both
//! tracks. Nothing in this file calls the production derivation to decide what
//! the production derivation should say.
//!
//! The extents are the ones the milestone names for RecentActivity: a 768 by
//! 269 viewport over 1560 by 672 of content, which overflows on both axes and
//! therefore reserves a corner.

use crate::mounting::presentation::platform_point_for_test as point;
use crate::runtime::pointer_affordance::{
    resolve_scroll_chrome_pointer, UiScrollChromeRegionTarget,
};
use crate::runtime::scroll::chrome::{
    UiScrollChromeAxis, UiScrollChromeAxisSupport, UiScrollChromeFacts, UiScrollChromePart,
};

const GUTTER: f64 = 12.0;
const THUMB_THICKNESS: f64 = 6.0;
const MINIMUM_THUMB_LENGTH: f64 = 24.0;
const SUBPIXELS_PER_POINT: f64 = 1_000.0;

const VIEWPORT_WIDTH: f64 = 768.0;
const VIEWPORT_HEIGHT: f64 = 269.0;
const CONTENT_WIDTH: f64 = 1560.0;
const CONTENT_HEIGHT: f64 = 672.0;

/// The gutter each axis keeps, with the shared corner removed: both axes
/// overflow at these extents, so each track surrenders one gutter at its
/// trailing end.
const fn oracle_track(axis: UiScrollChromeAxis) -> [f64; 4] {
    match axis {
        UiScrollChromeAxis::Block => [
            VIEWPORT_WIDTH - GUTTER,
            0.0,
            GUTTER,
            VIEWPORT_HEIGHT - GUTTER,
        ],
        UiScrollChromeAxis::Inline => [
            0.0,
            VIEWPORT_HEIGHT - GUTTER,
            VIEWPORT_WIDTH - GUTTER,
            GUTTER,
        ],
    }
}

/// The thumb length and the travel the minimum-length clamp leaves, stated
/// separately from the proportion so the clamp is visible.
fn oracle_length_and_travel(axis: UiScrollChromeAxis) -> (f64, f64) {
    let track = oracle_track(axis);
    let (track_length, viewport, content) = match axis {
        UiScrollChromeAxis::Block => (track[3], VIEWPORT_HEIGHT, CONTENT_HEIGHT),
        UiScrollChromeAxis::Inline => (track[2], VIEWPORT_WIDTH, CONTENT_WIDTH),
    };
    let proportional = track_length * viewport / content;
    let length = proportional.max(MINIMUM_THUMB_LENGTH).min(track_length);
    (length, track_length - length)
}

/// The thumb rectangle at one displayed offset, centred across the gutter.
fn oracle_thumb(axis: UiScrollChromeAxis, offset_points: f64) -> [f64; 4] {
    let track = oracle_track(axis);
    let (length, travel) = oracle_length_and_travel(axis);
    let inset = (GUTTER - THUMB_THICKNESS) / 2.0;
    let start = travel * offset_points / oracle_max_offset_points(axis);
    match axis {
        UiScrollChromeAxis::Block => [track[0] + inset, track[1] + start, THUMB_THICKNESS, length],
        UiScrollChromeAxis::Inline => [track[0] + start, track[1] + inset, length, THUMB_THICKNESS],
    }
}

const fn oracle_max_offset_points(axis: UiScrollChromeAxis) -> f64 {
    match axis {
        UiScrollChromeAxis::Block => CONTENT_HEIGHT - VIEWPORT_HEIGHT,
        UiScrollChromeAxis::Inline => CONTENT_WIDTH - VIEWPORT_WIDTH,
    }
}

fn canonical(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> worth_ui_host_contract::UiMountedCanonicalBox {
    worth_ui_host_contract::UiMountedCanonicalBox::canonicalize(
        worth_ui_host_contract::UiMountedCanonicalBoxInput {
            x: x as f32,
            y: y as f32,
            width: width as f32,
            height: height as f32,
            coordinate_space: worth_ui_host_contract::UiMountedCoordinateSpace::Viewport,
        },
    )
    .expect("canonical test box")
}

fn viewport() -> worth_ui_host_contract::UiMountedCanonicalBox {
    canonical(0.0, 0.0, VIEWPORT_WIDTH, VIEWPORT_HEIGHT)
}

fn admitted_chrome() -> crate::runtime::scroll::chrome::UiScrollAdmittedChrome {
    crate::runtime::scroll::chrome::UiScrollAdmittedChrome::admit_declared_chrome(
        UiScrollChromeAxisSupport::Both,
        worth_ui_dsl::UiAppearanceRoleIdentity::new("platform.pulse.appearance.scroll_track")
            .expect("track role"),
        worth_ui_dsl::UiAppearanceRoleIdentity::new("platform.pulse.appearance.scroll_thumb")
            .expect("thumb role"),
        crate::runtime::scroll::chrome::UiScrollChromeMetrics::declared(),
    )
    .expect("the declared Pulse roles differ")
}

fn bounds(content_width: f64, content_height: f64) -> crate::runtime::scroll::UiScrollBounds {
    crate::runtime::scroll::UiScrollBounds::new(
        ((content_width - VIEWPORT_WIDTH) * SUBPIXELS_PER_POINT) as i64,
        ((content_height - VIEWPORT_HEIGHT) * SUBPIXELS_PER_POINT) as i64,
    )
    .expect("non-negative overflow")
}

fn offset(inline_points: f64, block_points: f64) -> crate::runtime::scroll::UiScrollOffset {
    crate::runtime::scroll::UiScrollOffset::new(
        (inline_points * SUBPIXELS_PER_POINT) as i64,
        (block_points * SUBPIXELS_PER_POINT) as i64,
    )
    .expect("non-negative offset")
}

fn facts_at(inline_points: f64, block_points: f64) -> UiScrollChromeFacts {
    UiScrollChromeFacts::derive(
        viewport(),
        bounds(CONTENT_WIDTH, CONTENT_HEIGHT),
        offset(inline_points, block_points),
        &admitted_chrome(),
    )
    .expect("both RecentActivity axes overflow")
}

#[track_caller]
fn assert_box(actual: worth_ui_host_contract::UiMountedCanonicalBox, expected: [f64; 4]) {
    for (index, (value, expected)) in [actual.x(), actual.y(), actual.width(), actual.height()]
        .into_iter()
        .zip(expected)
        .enumerate()
    {
        assert!(
            (f64::from(value) - expected).abs() < 0.01,
            "component {index}: {value} is not {expected}"
        );
    }
}

/// The block thumb starts at the top of its track at rest and ends flush with
/// the bottom of it at the bound: a scrollbar that stopped short would say the
/// content had more left when it did not.
#[test]
fn the_block_thumb_spans_its_track_between_offset_zero_and_the_bound() {
    let axis = UiScrollChromeAxis::Block;
    assert_box(
        facts_at(0.0, 0.0).axis(axis).expect("block chrome").thumb(),
        oracle_thumb(axis, 0.0),
    );
    let max = oracle_max_offset_points(axis);
    assert_box(
        facts_at(0.0, max).axis(axis).expect("block chrome").thumb(),
        oracle_thumb(axis, max),
    );
    let (length, travel) = oracle_length_and_travel(axis);
    assert!(
        (length + travel - oracle_track(axis)[3]).abs() < 0.01,
        "the thumb and its travel must fill the track exactly"
    );
}

/// The inline axis answers the same way over its own extents, and its track is
/// the one that surrendered the corner along the bottom edge.
#[test]
fn the_inline_thumb_spans_its_track_between_offset_zero_and_the_bound() {
    let axis = UiScrollChromeAxis::Inline;
    assert_box(
        facts_at(0.0, 0.0)
            .axis(axis)
            .expect("inline chrome")
            .track(),
        oracle_track(axis),
    );
    assert_box(
        facts_at(0.0, 0.0)
            .axis(axis)
            .expect("inline chrome")
            .thumb(),
        oracle_thumb(axis, 0.0),
    );
    let max = oracle_max_offset_points(axis);
    assert_box(
        facts_at(max, 0.0)
            .axis(axis)
            .expect("inline chrome")
            .thumb(),
        oracle_thumb(axis, max),
    );
}

/// A direct drag of N points along the track moves the offset by N times the
/// content-per-track-point the travel compressed it to, and it keeps the grab
/// offset so the thumb never jumps its centre to the pointer.
#[test]
fn a_drag_moves_the_offset_by_the_compressed_travel_ratio() {
    let axis = UiScrollChromeAxis::Block;
    let facts = facts_at(0.0, 0.0);
    let (_, travel) = oracle_length_and_travel(axis);
    let max_points = oracle_max_offset_points(axis);
    let grab = 7.5;
    let dragged_points = 20.0;
    let placed = facts
        .offset_for_thumb_position(
            axis,
            point(762.0, (grab + dragged_points) as f32),
            grab as f32,
            offset(0.0, 0.0),
        )
        .expect("the block axis presents chrome");
    let expected = (max_points * dragged_points / travel * SUBPIXELS_PER_POINT).round() as i64;
    assert!(
        (placed.block_subpixels() - expected).abs() <= 1,
        "a {dragged_points} point drag placed {} rather than {expected}",
        placed.block_subpixels()
    );
    assert_eq!(placed.inline_subpixels(), 0, "a block drag is block-only");
}

/// A press in the track past the thumb pages forward by one viewport minus one
/// line, and a press before it pages back the same distance.
#[test]
fn a_track_press_pages_one_viewport_minus_one_line_toward_the_pointer() {
    let axis = UiScrollChromeAxis::Block;
    let line: u16 = 24;
    let page = ((VIEWPORT_HEIGHT - f64::from(line)) * SUBPIXELS_PER_POINT).round() as i64;
    let (length, _) = oracle_length_and_travel(axis);
    let start_offset = 200.0;
    let facts = facts_at(0.0, start_offset);
    let thumb_start = oracle_thumb(axis, start_offset)[1];

    let forward = facts
        .offset_for_track_click(
            axis,
            point(762.0, (thumb_start + length + 4.0) as f32),
            offset(0.0, start_offset),
            line,
        )
        .expect("a press past the thumb pages");
    assert_eq!(
        forward.block_subpixels(),
        ((start_offset * SUBPIXELS_PER_POINT) as i64 + page)
            .min((oracle_max_offset_points(axis) * SUBPIXELS_PER_POINT) as i64)
    );

    let back = facts
        .offset_for_track_click(
            axis,
            point(762.0, (thumb_start - 4.0) as f32),
            offset(0.0, start_offset),
            line,
        )
        .expect("a press before the thumb pages");
    assert_eq!(
        back.block_subpixels(),
        ((start_offset * SUBPIXELS_PER_POINT) as i64 - page).max(0)
    );
}

/// A press on the thumb is a grab, not a page, so paging answers nothing for it.
#[test]
fn a_press_on_the_thumb_itself_pages_nothing() {
    let facts = facts_at(0.0, 0.0);
    assert_eq!(
        facts.offset_for_track_click(
            UiScrollChromeAxis::Block,
            point(762.0, 10.0),
            offset(0.0, 0.0),
            24
        ),
        None
    );
}

/// The gutter is the owning region's, so a pointer anywhere in it — including
/// the corner neither track kept — names that region and nothing behind it.
#[test]
fn every_point_in_the_reserved_gutter_names_the_owning_region() {
    let facts = facts_at(0.0, 0.0);
    let owner = crate::runtime::scroll::UiScrollOwnerIdentity::declared_region(
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().expect("surface"),
        crate::graph::UiGraphNodeIdentity::new(3_161),
        1,
        4,
    );
    let targets = [UiScrollChromeRegionTarget {
        owner,
        facts: &facts,
    }];

    let on_track =
        resolve_scroll_chrome_pointer(point(762.0, 200.0), &targets).expect("in the gutter");
    assert_eq!(on_track.owner(), owner);
    assert_eq!(
        on_track.part().map(|part| part.axis()),
        Some(UiScrollChromeAxis::Block)
    );

    // The corner routes a wheel to the same region while reaching no axis, so a
    // press there moves neither scrollbar.
    let corner =
        resolve_scroll_chrome_pointer(point(762.0, 262.0), &targets).expect("on the corner");
    assert_eq!(corner.owner(), owner);
    assert_eq!(corner.part(), None);

    assert_eq!(
        resolve_scroll_chrome_pointer(point(100.0, 100.0), &targets),
        None
    );
}

/// The drawn thumb is six points across but the grab target fills the gutter,
/// so a press beside the bar grabs it instead of paging past it.
#[test]
fn a_press_beside_the_drawn_thumb_still_grabs_it() {
    let facts = facts_at(0.0, 0.0);
    let owner = crate::runtime::scroll::UiScrollOwnerIdentity::declared_region(
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().expect("surface"),
        crate::graph::UiGraphNodeIdentity::new(3_162),
        1,
        4,
    );
    let targets = [UiScrollChromeRegionTarget {
        owner,
        facts: &facts,
    }];
    // x = 757 is inside the 12-point gutter but outside the 6-point bar.
    let answer = resolve_scroll_chrome_pointer(point(757.0, 10.0), &targets)
        .and_then(|answer| answer.part())
        .expect("beside the thumb is still the thumb");
    assert_eq!(answer.part(), UiScrollChromePart::Thumb);
}

/// A region whose content fits presents no chrome at all: no track to click and
/// no thumb to drag, because there is no travel for either to report.
#[test]
fn a_region_without_travel_presents_no_chrome() {
    assert_eq!(
        UiScrollChromeFacts::derive(
            viewport(),
            bounds(VIEWPORT_WIDTH, VIEWPORT_HEIGHT),
            offset(0.0, 0.0),
            &admitted_chrome(),
        ),
        None
    );
    // One axis fitting leaves the other axis's chrome, and takes the corner
    // reservation with it: a single scrollbar keeps its whole gutter.
    let single = UiScrollChromeFacts::derive(
        viewport(),
        bounds(VIEWPORT_WIDTH, CONTENT_HEIGHT),
        offset(0.0, 0.0),
        &admitted_chrome(),
    )
    .expect("the block axis still overflows");
    assert_eq!(single.axis(UiScrollChromeAxis::Inline), None);
    assert_eq!(single.corner(), None);
    assert_box(
        single
            .axis(UiScrollChromeAxis::Block)
            .expect("block")
            .track(),
        [VIEWPORT_WIDTH - GUTTER, 0.0, GUTTER, VIEWPORT_HEIGHT],
    );
}
