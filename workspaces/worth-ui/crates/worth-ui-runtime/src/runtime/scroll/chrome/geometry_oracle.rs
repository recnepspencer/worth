//! An independent rectangle oracle for scroll chrome, plus the fixtures the
//! chrome tests derive against.
//!
//! The oracle restates the milestone's declared numbers as plain rectangle
//! arithmetic — literal 12-point gutter, 6-point thumb, 24-point minimum,
//! explicit corner reservation, and a separately stated minimum-length clamp
//! with travel compression. It never calls the production derivation, so a
//! production rewrite that agrees with itself still has to agree with this.

pub(super) const GUTTER: f64 = 12.0;
pub(super) const THUMB_THICKNESS: f64 = 6.0;
pub(super) const MINIMUM_THUMB_LENGTH: f64 = 24.0;
pub(super) const SUBPIXELS_PER_POINT: f64 = 1_000.0;

pub(super) const VIEWPORT_WIDTH: f64 = 400.0;
pub(super) const VIEWPORT_HEIGHT: f64 = 269.0;
pub(super) const CONTENT_WIDTH: f64 = 900.0;
pub(super) const CONTENT_HEIGHT: f64 = 672.0;

/// The track strip for one axis, with the shared corner removed when the other
/// axis also presents chrome.
pub(super) fn oracle_track(vertical: bool, other_axis_present: bool) -> [f64; 4] {
    let surrendered = if other_axis_present { GUTTER } else { 0.0 };
    if vertical {
        [
            VIEWPORT_WIDTH - GUTTER,
            0.0,
            GUTTER,
            VIEWPORT_HEIGHT - surrendered,
        ]
    } else {
        [
            0.0,
            VIEWPORT_HEIGHT - GUTTER,
            VIEWPORT_WIDTH - surrendered,
            GUTTER,
        ]
    }
}

/// The proportional thumb length, before any clamp.
pub(super) fn oracle_proportional_thumb_length(
    track_length: f64,
    viewport: f64,
    content: f64,
) -> f64 {
    track_length * viewport / content
}

/// Stated separately from the proportion: the length a short thumb is clamped
/// up to, and the travel that clamp compresses.
pub(super) fn oracle_clamped_length_and_travel(track_length: f64, proportional: f64) -> (f64, f64) {
    let length = proportional.max(MINIMUM_THUMB_LENGTH).min(track_length);
    (length, track_length - length)
}

/// The thumb start for an accepted displayed offset.
pub(super) fn oracle_thumb_start(travel: f64, offset_points: f64, max_offset_points: f64) -> f64 {
    travel * offset_points / max_offset_points
}

pub(super) fn viewport_box() -> worth_ui_host_contract::UiMountedCanonicalBox {
    canonical(0.0, 0.0, VIEWPORT_WIDTH as f32, VIEWPORT_HEIGHT as f32)
}

pub(super) fn canonical(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> worth_ui_host_contract::UiMountedCanonicalBox {
    worth_ui_host_contract::UiMountedCanonicalBox::canonicalize(
        worth_ui_host_contract::UiMountedCanonicalBoxInput {
            x,
            y,
            width,
            height,
            coordinate_space: worth_ui_host_contract::UiMountedCoordinateSpace::Viewport,
        },
    )
    .expect("canonical test box")
}

pub(super) fn chrome(axes: super::UiScrollChromeAxisSupport) -> super::UiScrollAdmittedChrome {
    super::UiScrollAdmittedChrome::admit_declared_chrome(
        axes,
        worth_ui_dsl::UiAppearanceRoleIdentity::new("test.scroll_track").expect("track role"),
        worth_ui_dsl::UiAppearanceRoleIdentity::new("test.scroll_thumb").expect("thumb role"),
        super::UiScrollChromeMetrics::declared(),
    )
    .expect("distinct roles are admitted")
}

pub(super) fn bounds(
    content_width: f64,
    content_height: f64,
) -> crate::runtime::scroll::UiScrollBounds {
    crate::runtime::scroll::UiScrollBounds::new(
        ((content_width - VIEWPORT_WIDTH) * SUBPIXELS_PER_POINT) as i64,
        ((content_height - VIEWPORT_HEIGHT) * SUBPIXELS_PER_POINT) as i64,
    )
    .expect("non-negative overflow")
}

pub(super) fn offset(
    inline_points: f64,
    block_points: f64,
) -> crate::runtime::scroll::UiScrollOffset {
    crate::runtime::scroll::UiScrollOffset::new(
        (inline_points * SUBPIXELS_PER_POINT) as i64,
        (block_points * SUBPIXELS_PER_POINT) as i64,
    )
    .expect("non-negative offset")
}

#[track_caller]
pub(super) fn assert_box(
    actual: worth_ui_host_contract::UiMountedCanonicalBox,
    expected: [f64; 4],
) {
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
