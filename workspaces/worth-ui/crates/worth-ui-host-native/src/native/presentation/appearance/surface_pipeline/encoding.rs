//! The unit conversions the surface pipeline encodes samples with: physical
//! micros to pixels, declared colors to premultiplied linear vectors, and the
//! border-edge set to the bit mask the sample carries.

use worth_ui_host_contract::{UiMountedAppearanceColor, UiMountedSurfaceBorderEdges};

pub(super) fn micros_to_pixels(value: i64) -> f32 {
    value as f32 / super::super::geometry::PHYSICAL_MICROS_PER_PIXEL as f32
}

pub(super) fn color_vector(color: Option<UiMountedAppearanceColor>) -> [f32; 4] {
    let Some(color) = color else {
        return [0.0; 4];
    };
    crate::native::presentation::raster::premultiplied_linear_color(color.straight_srgba())
}

pub(super) const fn border_edge_bits(edges: UiMountedSurfaceBorderEdges) -> u8 {
    (if edges.top() { 1 } else { 0 })
        | (if edges.right() { 2 } else { 0 })
        | (if edges.bottom() { 4 } else { 0 })
        | (if edges.left() { 8 } else { 0 })
}
