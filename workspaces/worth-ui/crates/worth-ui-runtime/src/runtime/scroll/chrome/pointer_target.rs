//! Where a pointer meets the chrome: which axis a point belongs to, the
//! effective grab rectangle, and the offsets a press produces.
//!
//! The drawn thumb is only 6 points across, but the effective pointer target
//! fills the whole reserved gutter, so a press anywhere across the gutter and
//! alongside the thumb is a thumb press. A press elsewhere in the track is a
//! track click, which pages toward the pointer by one viewport minus one line.

/// A point's position along one axis, resolved against that axis's track.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiScrollChromePress {
    Thumb,
    TrackBeforeThumb,
    TrackAfterThumb,
}

/// The rectangle a pointer must hit to grab the thumb: the thumb's extent
/// along the axis, widened across the axis to fill the reserved gutter.
pub(crate) fn effective_thumb_pointer_rect(
    axis: super::UiScrollChromeAxis,
    track: worth_ui_host_contract::UiMountedCanonicalBox,
    thumb: worth_ui_host_contract::UiMountedCanonicalBox,
) -> Option<worth_ui_host_contract::UiMountedCanonicalBox> {
    let input = match axis {
        super::UiScrollChromeAxis::Block => worth_ui_host_contract::UiMountedCanonicalBoxInput {
            x: track.x(),
            y: thumb.y(),
            width: track.width(),
            height: thumb.height(),
            coordinate_space: track.coordinate_space(),
        },
        super::UiScrollChromeAxis::Inline => worth_ui_host_contract::UiMountedCanonicalBoxInput {
            x: thumb.x(),
            y: track.y(),
            width: thumb.width(),
            height: track.height(),
            coordinate_space: track.coordinate_space(),
        },
    };
    worth_ui_host_contract::UiMountedCanonicalBox::canonicalize(input).ok()
}

/// A point is inside a chrome rectangle on its leading edges and outside on its
/// trailing edges, so two adjoining rectangles never both claim it.
pub(crate) fn rect_contains(
    rect: worth_ui_host_contract::UiMountedCanonicalBox,
    point: crate::mounting::presentation::UiPlatformPoint,
) -> bool {
    let (x, y) = (point.x(), point.y());
    x >= rect.x() && x < rect.x() + rect.width() && y >= rect.y() && y < rect.y() + rect.height()
}

/// The point's coordinate along one axis.
pub(crate) fn point_along(
    axis: super::UiScrollChromeAxis,
    point: crate::mounting::presentation::UiPlatformPoint,
) -> f32 {
    match axis {
        super::UiScrollChromeAxis::Inline => point.x(),
        super::UiScrollChromeAxis::Block => point.y(),
    }
}

pub(crate) fn classify_press(
    axis: super::UiScrollChromeAxis,
    thumb: worth_ui_host_contract::UiMountedCanonicalBox,
    point: crate::mounting::presentation::UiPlatformPoint,
) -> UiScrollChromePress {
    let along = point_along(axis, point);
    let start = super::track_origin_logical_points(axis, thumb);
    let length = super::track_length_logical_points(axis, thumb);
    if along < start {
        UiScrollChromePress::TrackBeforeThumb
    } else if along < start + length {
        UiScrollChromePress::Thumb
    } else {
        UiScrollChromePress::TrackAfterThumb
    }
}

/// The pointer-to-thumb grab offset a press establishes, in logical points
/// along the axis. A direct drag preserves it for the gesture's lifetime.
pub(crate) fn grab_offset_logical_points(
    axis: super::UiScrollChromeAxis,
    thumb: worth_ui_host_contract::UiMountedCanonicalBox,
    point: crate::mounting::presentation::UiPlatformPoint,
) -> f32 {
    point_along(axis, point) - super::track_origin_logical_points(axis, thumb)
}

/// The offset a dragged thumb places, preserving the grab offset established at
/// the press. Direct and unsmoothed: the caller applies it as the successor
/// offset, not as a transition target.
pub(crate) fn offset_for_thumb_position(
    axis: super::UiScrollChromeAxis,
    track: worth_ui_host_contract::UiMountedCanonicalBox,
    extent: super::UiScrollThumbExtent,
    point: crate::mounting::presentation::UiPlatformPoint,
    grab_offset_logical_points: f32,
    max_offset_subpixels: i64,
) -> i64 {
    let start = point_along(axis, point)
        - grab_offset_logical_points
        - super::track_origin_logical_points(axis, track);
    extent.offset_subpixels_for_start(start, max_offset_subpixels)
}

/// One page step in subpixels: a viewport minus a line, never negative and
/// never larger than the viewport itself.
pub(crate) fn page_step_subpixels(
    viewport_extent_logical_points: f32,
    line_extent_logical_points: u16,
) -> i64 {
    let viewport = f64::from(viewport_extent_logical_points)
        * worth_ui_host_contract::UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as f64;
    let line = f64::from(line_extent_logical_points)
        * worth_ui_host_contract::UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as f64;
    (viewport - line).max(0.0).round() as i64
}

/// The offset a track click produces: one page toward the pointer, clamped into
/// bounds. Returns `None` when the click landed on the thumb itself.
pub(crate) fn offset_for_track_click(
    axis: super::UiScrollChromeAxis,
    thumb: worth_ui_host_contract::UiMountedCanonicalBox,
    point: crate::mounting::presentation::UiPlatformPoint,
    current_offset_subpixels: i64,
    page_step_subpixels: i64,
    max_offset_subpixels: i64,
) -> Option<i64> {
    let stepped = match classify_press(axis, thumb, point) {
        UiScrollChromePress::Thumb => return None,
        UiScrollChromePress::TrackBeforeThumb => {
            current_offset_subpixels.saturating_sub(page_step_subpixels)
        }
        UiScrollChromePress::TrackAfterThumb => {
            current_offset_subpixels.saturating_add(page_step_subpixels)
        }
    };
    Some(stepped.clamp(0, max_offset_subpixels.max(0)))
}
