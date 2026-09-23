//! The scrollbar track rectangle for one axis, taken from the stationary
//! viewport box and the admitted gutter.
//!
//! The gutter is a stable reservation along the inside edge of the viewport:
//! the block (vertical) track occupies the trailing inline edge, the inline
//! (horizontal) track occupies the trailing block edge. When both axes present
//! chrome, each track gives up one gutter at its trailing end so the corner
//! belongs to neither track and a pointer there is never ambiguous.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiScrollChromeAxis {
    Inline,
    Block,
}

/// Whether the other axis also presents chrome, and therefore whether this
/// track must surrender the shared corner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiScrollChromeCorner {
    Unshared,
    SharedWithOtherAxis,
}

pub(crate) fn track_rect(
    axis: UiScrollChromeAxis,
    viewport: worth_ui_host_contract::UiMountedCanonicalBox,
    metrics: super::UiScrollChromeMetrics,
    corner: UiScrollChromeCorner,
) -> Option<worth_ui_host_contract::UiMountedCanonicalBox> {
    let gutter = metrics.gutter_logical_points();
    let surrendered = match corner {
        UiScrollChromeCorner::Unshared => 0.0,
        UiScrollChromeCorner::SharedWithOtherAxis => gutter,
    };
    let input = match axis {
        UiScrollChromeAxis::Block => worth_ui_host_contract::UiMountedCanonicalBoxInput {
            x: viewport.x() + viewport.width() - gutter,
            y: viewport.y(),
            width: gutter,
            height: viewport.height() - surrendered,
            coordinate_space: viewport.coordinate_space(),
        },
        UiScrollChromeAxis::Inline => worth_ui_host_contract::UiMountedCanonicalBoxInput {
            x: viewport.x(),
            y: viewport.y() + viewport.height() - gutter,
            width: viewport.width() - surrendered,
            height: gutter,
            coordinate_space: viewport.coordinate_space(),
        },
    };
    if input.width <= 0.0 || input.height <= 0.0 {
        return None;
    }
    worth_ui_host_contract::UiMountedCanonicalBox::canonicalize(input).ok()
}

/// The square both tracks surrendered when they share a corner.
///
/// It is derived from the two tracks rather than from the viewport, so the one
/// place the gutter width is decided stays [`track_rect`]: the block track's
/// across-axis strip crossed with the inline track's across-axis strip is
/// exactly the square neither track kept. A pointer there belongs to no axis,
/// which is what stops a corner press from guessing between two scrollbars.
pub(crate) fn corner_rect(
    inline_track: worth_ui_host_contract::UiMountedCanonicalBox,
    block_track: worth_ui_host_contract::UiMountedCanonicalBox,
) -> Option<worth_ui_host_contract::UiMountedCanonicalBox> {
    worth_ui_host_contract::UiMountedCanonicalBox::canonicalize(
        worth_ui_host_contract::UiMountedCanonicalBoxInput {
            x: block_track.x(),
            y: inline_track.y(),
            width: block_track.width(),
            height: inline_track.height(),
            coordinate_space: block_track.coordinate_space(),
        },
    )
    .ok()
}

/// The track's extent along its own scrolling axis.
pub(crate) fn track_length_logical_points(
    axis: UiScrollChromeAxis,
    track: worth_ui_host_contract::UiMountedCanonicalBox,
) -> f32 {
    match axis {
        UiScrollChromeAxis::Inline => track.width(),
        UiScrollChromeAxis::Block => track.height(),
    }
}

/// The track's origin along its own scrolling axis.
pub(crate) fn track_origin_logical_points(
    axis: UiScrollChromeAxis,
    track: worth_ui_host_contract::UiMountedCanonicalBox,
) -> f32 {
    match axis {
        UiScrollChromeAxis::Inline => track.x(),
        UiScrollChromeAxis::Block => track.y(),
    }
}

/// The viewport's extent along one scrolling axis, which is the numerator of
/// the thumb's viewport/content proportion.
pub(crate) fn viewport_extent_logical_points(
    axis: UiScrollChromeAxis,
    viewport: worth_ui_host_contract::UiMountedCanonicalBox,
) -> f32 {
    match axis {
        UiScrollChromeAxis::Inline => viewport.width(),
        UiScrollChromeAxis::Block => viewport.height(),
    }
}
