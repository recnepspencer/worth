//! Geometry that carries its truth status.
//!
//! A rectangle, offset, or translation is one of three truths:
//!
//! - **Published**: what the runtime committed for a frame -- layout,
//!   allocation, and the targets owners declare.
//! - **Accepted**: a sample admitted for presentation, bound to the host basis
//!   it was sampled against.
//! - **Displayed**: what an admitted host acknowledgement proved on screen,
//!   bound to that acknowledgement's displayed basis.
//!
//! The three differ in truth status, so they are distinct types rather than a
//! tag on one shared array (law 10). Arithmetic is defined only within one
//! status. Crossing statuses is a named conversion that consumes the evidence
//! relating them: presentation sampling admits a Motion sample from published
//! to accepted, and an admitted witness's displayed basis carries accepted to
//! displayed.

#[path = "truth_geometry/accepted.rs"]
mod accepted;
#[path = "truth_geometry/displayed.rs"]
mod displayed;
#[path = "truth_geometry/logical_rect.rs"]
mod logical_rect;
#[path = "truth_geometry/platform_point.rs"]
mod platform_point;
#[path = "truth_geometry/pose_shift.rs"]
mod pose_shift;
#[path = "truth_geometry/published.rs"]
mod published;
#[path = "truth_geometry/rect_map.rs"]
mod rect_map;
#[path = "truth_geometry/scroll_offset.rs"]
mod scroll_offset;
#[path = "truth_geometry/transform_carry.rs"]
mod transform_carry;

pub(crate) use accepted::{UiAcceptedRect, UiRebaseDenial};
#[cfg(test)]
pub(crate) use displayed::displayed_rect_for_test;
pub(crate) use displayed::UiDisplayedRect;
pub(crate) use logical_rect::UiTruthGeometryDenial;
#[cfg(test)]
pub(crate) use platform_point::platform_point_for_test;
pub(crate) use platform_point::UiPlatformPoint;
pub(crate) use pose_shift::UiScrollPoseShift;
pub(crate) use published::UiPublishedRect;
pub(crate) use rect_map::{UiPublishedMap, UiPublishedToAcceptedMap};
#[cfg(test)]
pub(crate) use scroll_offset::displayed_scroll_offset_for_test;
pub(crate) use scroll_offset::{UiDisplayedScrollOffset, UiScrollStandingDenial};
pub(in crate::mounting::presentation) use transform_carry::{carried_box, carried_transform};
