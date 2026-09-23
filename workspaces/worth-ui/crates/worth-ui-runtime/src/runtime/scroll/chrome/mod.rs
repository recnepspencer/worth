//! Derived scroll chrome: track and thumb geometry, pointer targets and the
//! per-owner fact bundle handed to projection. Nothing here is authoritative;
//! every value is reproducible from the viewport box, the Scroll bounds, the
//! accepted displayed offset and the admitted chrome.

mod appearance_state;
mod contract;
mod derived_facts;
mod metrics;
mod part;
mod pointer_target;
mod thumb_geometry;
mod track_geometry;

pub(crate) use appearance_state::{UiScrollChromeAppearanceState, UiScrollChromeDragPosture};
pub(crate) use contract::{
    UiScrollAdmittedChrome, UiScrollChromeAdmissionDenial, UiScrollChromeAxisSupport,
};
pub(crate) use derived_facts::{UiScrollChromeAxisFacts, UiScrollChromeFacts};
pub(crate) use metrics::UiScrollChromeMetrics;
#[cfg(test)]
pub(crate) use metrics::{
    UiScrollChromeMetricsDenial, UI_SCROLL_GUTTER_LOGICAL_POINTS, UI_SCROLL_THUMB_LOGICAL_POINTS,
    UI_SCROLL_THUMB_MINIMUM_LOGICAL_POINTS,
};
pub(crate) use part::UiScrollChromePart;
pub(crate) use pointer_target::{
    effective_thumb_pointer_rect, grab_offset_logical_points, offset_for_thumb_position,
    offset_for_track_click, page_step_subpixels, rect_contains,
};
pub(crate) use thumb_geometry::{thumb_extent, thumb_rect, UiScrollThumbExtent};
pub(crate) use track_geometry::{
    corner_rect, track_length_logical_points, track_origin_logical_points, track_rect,
    viewport_extent_logical_points, UiScrollChromeAxis, UiScrollChromeCorner,
};

#[cfg(test)]
mod geometry_oracle;
#[cfg(test)]
mod geometry_oracle_tests;
#[cfg(test)]
mod pointer_target_tests;
