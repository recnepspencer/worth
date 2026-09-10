use crate::native::UiNativePresentationAccess;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct UiNativeRasterBasis {
    extent: [u32; 2],
    scale_factor: f32,
}

impl UiNativeRasterBasis {
    pub(super) fn from_presentation_access(access: &UiNativePresentationAccess) -> Self {
        Self {
            extent: access.extent(),
            scale_factor: access.scale_factor() as f32,
        }
    }

    pub(super) const fn new(extent: [u32; 2], scale_factor: f32) -> Self {
        Self {
            extent,
            scale_factor,
        }
    }

    pub(super) const fn extent(self) -> [u32; 2] {
        self.extent
    }

    pub(super) const fn scale_factor(self) -> f32 {
        self.scale_factor
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RasterRect {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
    pub(super) physical_width: u32,
    pub(super) physical_height: u32,
    physical_left: u32,
    physical_top: u32,
}

impl RasterRect {
    pub(super) fn intersection(self, other: Self, extent: [u32; 2]) -> Option<Self> {
        raster_physical_bounds(
            [
                self.physical_left.max(other.physical_left),
                self.physical_top.max(other.physical_top),
                (self.physical_left + self.physical_width)
                    .min(other.physical_left + other.physical_width),
                (self.physical_top + self.physical_height)
                    .min(other.physical_top + other.physical_height),
            ],
            extent,
        )
    }

    pub(super) const fn physical_bounds(self) -> [f32; 4] {
        [
            self.physical_left as f32,
            self.physical_top as f32,
            self.physical_width as f32,
            self.physical_height as f32,
        ]
    }
}

pub(super) fn raster_physical_bounds(bounds: [u32; 4], extent: [u32; 2]) -> Option<RasterRect> {
    let [left, top, right, bottom] = bounds;
    (left < right && top < bottom && right <= extent[0] && bottom <= extent[1]).then(|| {
        RasterRect {
            left: left as f32 * 2.0 / extent[0] as f32 - 1.0,
            top: 1.0 - top as f32 * 2.0 / extent[1] as f32,
            right: right as f32 * 2.0 / extent[0] as f32 - 1.0,
            bottom: 1.0 - bottom as f32 * 2.0 / extent[1] as f32,
            physical_width: right - left,
            physical_height: bottom - top,
            physical_left: left,
            physical_top: top,
        }
    })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct RasterVertex {
    pub(super) position: [f32; 2],
    pub(super) color: [f32; 4],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct GlyphVertex {
    pub(super) position: [f32; 2],
    pub(super) texture_uv: [f32; 2],
    pub(super) color: [f32; 4],
}

pub(super) fn raster_rect(
    mechanic: worth_ui_host_contract::UiMountedFilledRectMechanic,
    graphics: &UiNativePresentationAccess,
) -> Result<RasterRect, ()> {
    raster_rect_for_basis(
        mechanic,
        UiNativeRasterBasis::from_presentation_access(graphics),
    )
}

pub(super) fn raster_rect_for_basis(
    mechanic: worth_ui_host_contract::UiMountedFilledRectMechanic,
    basis: UiNativeRasterBasis,
) -> Result<RasterRect, ()> {
    let bounds = mechanic.bounds();
    let clip = mechanic.clip_bounds();
    raster_from_basis(
        [bounds.x(), bounds.y(), bounds.width(), bounds.height()],
        [clip.x(), clip.y(), clip.width(), clip.height()],
        basis.extent(),
        basis.scale_factor(),
    )
}

pub(super) fn raster_portal_overlay(
    mechanic: worth_ui_host_contract::UiMountedPortalOverlayMechanic,
    graphics: &UiNativePresentationAccess,
) -> Result<RasterRect, ()> {
    raster_portal_overlay_for_basis(
        mechanic,
        UiNativeRasterBasis::from_presentation_access(graphics),
    )
}

pub(super) fn raster_portal_overlay_for_basis(
    mechanic: worth_ui_host_contract::UiMountedPortalOverlayMechanic,
    basis: UiNativeRasterBasis,
) -> Result<RasterRect, ()> {
    let bounds = mechanic.bounds();
    let clip = mechanic.clip_bounds();
    raster_from_basis(
        [bounds.x(), bounds.y(), bounds.width(), bounds.height()],
        [clip.x(), clip.y(), clip.width(), clip.height()],
        basis.extent(),
        basis.scale_factor(),
    )
}

pub(super) fn raster_damage_for_basis(
    bounds: worth_ui_host_contract::UiMountedCanonicalBox,
    basis: UiNativeRasterBasis,
) -> Result<Option<RasterRect>, ()> {
    raster_from_basis_optional(
        [bounds.x(), bounds.y(), bounds.width(), bounds.height()],
        [bounds.x(), bounds.y(), bounds.width(), bounds.height()],
        basis.extent,
        basis.scale_factor,
    )
}

fn raster_from_basis(
    bounds: [f32; 4],
    clip: [f32; 4],
    extent: [u32; 2],
    scale: f32,
) -> Result<RasterRect, ()> {
    raster_from_basis_optional(bounds, clip, extent, scale)?.ok_or(())
}

fn raster_from_basis_optional(
    bounds: [f32; 4],
    clip: [f32; 4],
    extent: [u32; 2],
    scale: f32,
) -> Result<Option<RasterRect>, ()> {
    if !scale.is_finite()
        || scale <= 0.0
        || bounds
            .iter()
            .chain(clip.iter())
            .any(|value| !value.is_finite())
        || extent.contains(&0)
    {
        return Err(());
    }
    let viewport = [extent[0] as f32 / scale, extent[1] as f32 / scale];
    let left = bounds[0].max(clip[0]).max(0.0);
    let top = bounds[1].max(clip[1]).max(0.0);
    let right = (bounds[0] + bounds[2])
        .min(clip[0] + clip[2])
        .min(viewport[0]);
    let bottom = (bounds[1] + bounds[3])
        .min(clip[1] + clip[3])
        .min(viewport[1]);
    if right <= left || bottom <= top {
        return Ok(None);
    }
    let (physical_left, physical_right) = snap_axis(left, right, scale, extent[0])?;
    let (physical_top, physical_bottom) = snap_axis(top, bottom, scale, extent[1])?;
    Ok(Some(RasterRect {
        left: physical_left as f32 * 2.0 / extent[0] as f32 - 1.0,
        top: 1.0 - physical_top as f32 * 2.0 / extent[1] as f32,
        right: physical_right as f32 * 2.0 / extent[0] as f32 - 1.0,
        bottom: 1.0 - physical_bottom as f32 * 2.0 / extent[1] as f32,
        physical_width: physical_right - physical_left,
        physical_height: physical_bottom - physical_top,
        physical_left,
        physical_top,
    }))
}

fn snap_axis(
    logical_min: f32,
    logical_max: f32,
    scale: f32,
    physical_limit: u32,
) -> Result<(u32, u32), ()> {
    let minimum = (logical_min * scale)
        .floor()
        .clamp(0.0, physical_limit as f32) as u32;
    let maximum = (logical_max * scale)
        .ceil()
        .clamp(0.0, physical_limit as f32) as u32;
    (maximum > minimum).then_some((minimum, maximum)).ok_or(())
}

pub(super) fn rectangle_vertices(rect: RasterRect, rgba: [u8; 4]) -> [RasterVertex; 6] {
    let color = premultiplied_linear_color(rgba);
    [
        vertex(rect.left, rect.bottom, color),
        vertex(rect.right, rect.bottom, color),
        vertex(rect.left, rect.top, color),
        vertex(rect.left, rect.top, color),
        vertex(rect.right, rect.bottom, color),
        vertex(rect.right, rect.top, color),
    ]
}

pub(super) fn premultiplied_linear_color(rgba: [u8; 4]) -> [f32; 4] {
    let alpha = f32::from(rgba[3]) / 255.0;
    [
        linear_channel(rgba[0]) * alpha,
        linear_channel(rgba[1]) * alpha,
        linear_channel(rgba[2]) * alpha,
        alpha,
    ]
}

fn vertex(x: f32, y: f32, color: [f32; 4]) -> RasterVertex {
    RasterVertex {
        position: [x, y],
        color,
    }
}

fn linear_channel(channel: u8) -> f32 {
    let encoded = f32::from(channel) / 255.0;
    if encoded <= 0.040_45 {
        encoded / 12.92
    } else {
        ((encoded + 0.055) / 1.055).powf(2.4)
    }
}

#[cfg(test)]
mod tests {
    use super::{raster_from_basis, rectangle_vertices};

    #[test]
    fn geometry_and_color_are_derived_from_the_admitted_command() {
        let rect = raster_from_basis(
            [10.0, 20.0, 1.0, 1.0],
            [10.25, 20.25, 0.5, 0.5],
            [200, 100],
            2.0,
        )
        .unwrap();
        assert_eq!(rect.physical_width, 2);
        assert_eq!(rect.physical_height, 2);
        for (observed, expected) in [
            (rect.left, -0.8),
            (rect.right, -0.78),
            (rect.top, 0.2),
            (rect.bottom, 0.16),
        ] {
            assert!((observed - expected).abs() < 0.000_001);
        }
        let red = rectangle_vertices(rect, [255, 0, 0, 128]);
        let blue = rectangle_vertices(rect, [0, 0, 255, 128]);
        let shifted = rectangle_vertices(
            raster_from_basis([1.0, 2.0, 3.0, 4.0], [1.0, 2.0, 3.0, 4.0], [20, 20], 1.0).unwrap(),
            [255, 0, 0, 128],
        );
        assert_ne!(red, blue);
        assert_ne!(red, shifted);
        assert_eq!(red[0].position, [rect.left, rect.bottom]);
        assert_eq!(red[2].position, [rect.left, rect.top]);
        assert_eq!(red[5].position, [rect.right, rect.top]);
        assert!(red.iter().all(|vertex| vertex.color[0] > 0.0));
        assert!(red.iter().all(|vertex| vertex.color[2] == 0.0));
        assert!(blue.iter().all(|vertex| vertex.color[0] == 0.0));
        assert!(blue.iter().all(|vertex| vertex.color[2] > 0.0));
    }

    #[test]
    fn physical_edges_floor_minima_and_ceil_maxima_after_clipping() {
        let clipped_negative = raster_from_basis(
            [-0.25, -0.25, 0.5, 0.5],
            [-1.0, -1.0, 2.0, 2.0],
            [20, 20],
            2.0,
        )
        .unwrap();
        assert_eq!(clipped_negative.physical_width, 1);
        assert_eq!(clipped_negative.physical_height, 1);
        let fractional = raster_from_basis(
            [1.2, 2.4, 1.1, 1.1],
            [1.25, 2.45, 0.75, 0.75],
            [30, 30],
            1.5,
        )
        .unwrap();
        assert_eq!(fractional.physical_width, 2);
        assert_eq!(fractional.physical_height, 2);
    }

    #[test]
    fn empty_clipped_geometry_is_denied_before_effects() {
        assert!(raster_from_basis(
            [10.0, 10.0, 4.0, 4.0],
            [20.0, 20.0, 1.0, 1.0],
            [100, 100],
            1.0,
        )
        .is_err());
    }
}
