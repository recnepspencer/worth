use worth_ui_host_contract::{
    UiMountedAppearanceColor, UiMountedSurfaceAppearanceMechanic, UiMountedSurfaceBorderEdges,
    UiMountedSurfaceBorderSide, UiMountedSurfacePaint,
};

#[path = "surface_pipeline/raster_operation.rs"]
mod raster_operation;

use super::antialiasing::{
    coverage_from_signed_distance, rounded_signed_distance, UiNativeAnalyticCoverage,
};
use super::damage::UiNativeAppearanceDamageRect;
use super::geometry::{
    physical_length, physical_radii, UiNativeAppearanceScale, UiNativeGeometryDenial,
    UiNativePhysicalPixelRect, UiNativePhysicalRect,
};
use crate::native::presentation::{raster::raster_physical_bounds, RasterRect};

/// The shader source is staged for the later publisher; no current wgpu
/// pipeline references it.
pub(crate) const ANALYTIC_ROUNDED_SURFACE_SHADER: &str = r#"
fn coverage_from_signed_distance(distance: f32) -> f32 {
    return clamp(0.5 - distance, 0.0, 1.0);
}
// The host supplies the normalized corner radii and performs the inward ring
// as outer coverage minus the inset contour coverage.
"#;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiNativeSurfacePaintKind {
    Fill,
    Border,
    FillAndBorder,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiNativeSurfaceSample {
    pub(crate) fill_coverage: UiNativeAnalyticCoverage,
    pub(crate) border_coverage: UiNativeAnalyticCoverage,
    pub(crate) fill_color: Option<UiMountedAppearanceColor>,
    pub(crate) border_color: Option<UiMountedAppearanceColor>,
    pub(crate) opacity: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiNativeSurfacePrimitive {
    allocation: UiNativePhysicalRect,
    clip: UiNativePhysicalPixelRect,
    radii: [i64; 4],
    border_width: i64,
    paint_kind: UiNativeSurfacePaintKind,
    fill_color: Option<UiMountedAppearanceColor>,
    border_color: Option<UiMountedAppearanceColor>,
    opacity: u16,
    border_edges: UiMountedSurfaceBorderEdges,
    border_omissions: Box<[UiNativeSurfaceBorderOmission]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct UiNativeSurfaceBorderOmission {
    side: UiMountedSurfaceBorderSide,
    start: i64,
    end: i64,
}

pub(crate) struct UiNativeSurfacePipeline;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UiNativeSurfaceRasterOperation {
    rect: RasterRect,
    storage: Box<[f32]>,
}

impl UiNativeSurfacePipeline {
    pub(crate) fn prepare(
        mechanic: &UiMountedSurfaceAppearanceMechanic,
        scale: UiNativeAppearanceScale,
    ) -> Result<UiNativeSurfacePrimitive, UiNativeGeometryDenial> {
        let allocation = UiNativePhysicalRect::from_allocation(mechanic.bounds(), scale)?;
        let clip = super::geometry::UiNativePhysicalRect::from_clip(mechanic.clip(), scale)?
            .pixel_bounds();
        let radii = physical_radii(mechanic.radii().corners(), scale)?;
        let (paint_kind, fill_color, border_color, border_width) = match mechanic.paint() {
            UiMountedSurfacePaint::Fill(color) => {
                (UiNativeSurfacePaintKind::Fill, Some(*color), None, 0)
            }
            UiMountedSurfacePaint::Border {
                color,
                inward_width,
            } => (
                UiNativeSurfacePaintKind::Border,
                None,
                Some(*color),
                physical_length(*inward_width, scale)?,
            ),
            UiMountedSurfacePaint::FillAndBorder {
                fill,
                border,
                inward_width,
            } => (
                UiNativeSurfacePaintKind::FillAndBorder,
                Some(*fill),
                Some(*border),
                physical_length(*inward_width, scale)?,
            ),
        };
        Ok(UiNativeSurfacePrimitive {
            allocation,
            clip,
            radii,
            border_width,
            paint_kind,
            fill_color,
            border_color,
            opacity: mechanic.opacity().units(),
            border_edges: mechanic.border_edges(),
            border_omissions: mechanic
                .border_omissions()
                .iter()
                .map(|omission| {
                    Ok(UiNativeSurfaceBorderOmission {
                        side: omission.side(),
                        start: scale.scale_logical(i64::from(omission.start()))?,
                        end: scale.scale_logical(i64::from(omission.end()))?,
                    })
                })
                .collect::<Result<Vec<_>, UiNativeGeometryDenial>>()?
                .into_boxed_slice(),
        })
    }

    pub(crate) fn prepare_outline(
        outline: &super::outline_pipeline::UiNativeOutlinePrimitive,
    ) -> UiNativeSurfacePrimitive {
        UiNativeSurfacePrimitive {
            allocation: outline.outer(),
            clip: outline.clip(),
            radii: outline.outer_radii(),
            border_width: outline.width(),
            paint_kind: UiNativeSurfacePaintKind::Border,
            fill_color: None,
            border_color: Some(outline.color()),
            opacity: outline.opacity(),
            border_edges: UiMountedSurfaceBorderEdges::ALL,
            border_omissions: Box::new([]),
        }
    }
}

impl UiNativeSurfacePrimitive {
    pub(crate) fn raster_operation(
        &self,
        extent: [u32; 2],
    ) -> Result<Option<UiNativeSurfaceRasterOperation>, UiNativeGeometryDenial> {
        let bounds = self.visual_bounds();
        let left = bounds
            .left
            .max(self.clip.left)
            .clamp(0, i64::from(extent[0]));
        let top = bounds.top.max(self.clip.top).clamp(0, i64::from(extent[1]));
        let right = bounds
            .right
            .min(self.clip.right)
            .clamp(0, i64::from(extent[0]));
        let bottom = bounds
            .bottom
            .min(self.clip.bottom)
            .clamp(0, i64::from(extent[1]));
        if left >= right || top >= bottom {
            return Ok(None);
        }
        let physical = [
            u32::try_from(left).map_err(|_| UiNativeGeometryDenial::CoordinateOverflow)?,
            u32::try_from(top).map_err(|_| UiNativeGeometryDenial::CoordinateOverflow)?,
            u32::try_from(right).map_err(|_| UiNativeGeometryDenial::CoordinateOverflow)?,
            u32::try_from(bottom).map_err(|_| UiNativeGeometryDenial::CoordinateOverflow)?,
        ];
        let rect = raster_physical_bounds(physical, extent)
            .ok_or(UiNativeGeometryDenial::CoordinateOverflow)?;
        Ok(Some(UiNativeSurfaceRasterOperation {
            rect,
            storage: self.raster_storage(),
        }))
    }

    pub(super) fn raster_storage(&self) -> Box<[f32]> {
        let mut storage = Vec::with_capacity(28 + self.border_omissions.len() * 4);
        storage.extend(self.allocation_edges_pixels());
        storage.extend([
            self.clip.left as f32,
            self.clip.top as f32,
            self.clip.right as f32,
            self.clip.bottom as f32,
        ]);
        storage.extend(self.radii.map(micros_to_pixels));
        storage.extend(color_vector(self.fill_color));
        storage.extend(color_vector(self.border_color));
        storage.extend([
            micros_to_pixels(self.border_width),
            f32::from(self.opacity) / f32::from(u16::MAX),
            match self.paint_kind {
                UiNativeSurfacePaintKind::Fill => 1.0,
                UiNativeSurfacePaintKind::Border => 2.0,
                UiNativeSurfacePaintKind::FillAndBorder => 3.0,
            },
            f32::from(border_edge_bits(self.border_edges)),
        ]);
        storage.extend([self.border_omissions.len() as f32, 0.0, 0.0, 0.0]);
        storage.extend(self.border_omissions.iter().flat_map(|omission| {
            [
                match omission.side {
                    UiMountedSurfaceBorderSide::Top => 0.0,
                    UiMountedSurfaceBorderSide::Right => 1.0,
                    UiMountedSurfaceBorderSide::Bottom => 2.0,
                    UiMountedSurfaceBorderSide::Left => 3.0,
                },
                micros_to_pixels(omission.start),
                micros_to_pixels(omission.end),
                0.0,
            ]
        }));
        storage.into_boxed_slice()
    }

    fn allocation_edges_pixels(&self) -> [f32; 4] {
        [
            micros_to_pixels(self.allocation.left),
            micros_to_pixels(self.allocation.top),
            micros_to_pixels(self.allocation.right),
            micros_to_pixels(self.allocation.bottom),
        ]
    }

    pub(crate) fn allocation(&self) -> UiNativePhysicalRect {
        self.allocation
    }

    pub(crate) fn damage_rect(&self) -> UiNativeAppearanceDamageRect {
        UiNativeAppearanceDamageRect::from_pixel_rect(self.allocation.pixel_bounds())
    }

    pub(crate) fn visual_bounds(&self) -> UiNativePhysicalPixelRect {
        self.allocation.pixel_bounds()
    }

    pub(crate) fn sample(&self, pixel_x: i64, pixel_y: i64) -> UiNativeSurfaceSample {
        let zero = UiNativeAnalyticCoverage::ZERO;
        if !self.clip.contains(pixel_x, pixel_y) {
            return UiNativeSurfaceSample {
                fill_coverage: zero,
                border_coverage: zero,
                fill_color: self.fill_color,
                border_color: self.border_color,
                opacity: self.opacity,
            };
        }
        let outer = coverage_from_signed_distance(rounded_signed_distance(
            pixel_x,
            pixel_y,
            self.allocation,
            self.radii,
        ));
        let inner_rect = self.allocation.inset(self.border_width).ok();
        let inner = if self.border_width == 0 {
            outer
        } else {
            inner_rect.map_or(UiNativeAnalyticCoverage::ZERO, |inner_rect| {
                let inner_radii = self
                    .radii
                    .map(|radius| radius.saturating_sub(self.border_width));
                coverage_from_signed_distance(rounded_signed_distance(
                    pixel_x,
                    pixel_y,
                    inner_rect,
                    inner_radii,
                ))
            })
        };
        let border_coverage = match self.paint_kind {
            UiNativeSurfacePaintKind::Fill => zero,
            UiNativeSurfacePaintKind::Border | UiNativeSurfacePaintKind::FillAndBorder => {
                if self.border_edge_enabled(pixel_x, pixel_y) {
                    outer.subtract(inner)
                } else {
                    zero
                }
            }
        };
        UiNativeSurfaceSample {
            fill_coverage: match self.paint_kind {
                UiNativeSurfacePaintKind::Border => zero,
                UiNativeSurfacePaintKind::Fill | UiNativeSurfacePaintKind::FillAndBorder => outer,
            },
            border_coverage,
            fill_color: self.fill_color,
            border_color: self.border_color,
            opacity: self.opacity,
        }
    }

    pub(crate) fn has_inward_border(&self) -> bool {
        matches!(
            self.paint_kind,
            UiNativeSurfacePaintKind::Border | UiNativeSurfacePaintKind::FillAndBorder
        ) && self.border_width != 0
    }

    fn border_edge_enabled(&self, pixel_x: i64, pixel_y: i64) -> bool {
        let [x, y] = super::geometry::pixel_center(pixel_x, pixel_y)
            .expect("qualified physical pixel coordinates fit the surface sample basis");
        let candidates = [
            (
                y - self.allocation.top,
                UiMountedSurfaceBorderSide::Top,
                x - self.allocation.left,
            ),
            (
                self.allocation.right - x,
                UiMountedSurfaceBorderSide::Right,
                y - self.allocation.top,
            ),
            (
                self.allocation.bottom - y,
                UiMountedSurfaceBorderSide::Bottom,
                x - self.allocation.left,
            ),
            (
                x - self.allocation.left,
                UiMountedSurfaceBorderSide::Left,
                y - self.allocation.top,
            ),
        ];
        let (_, side, offset) = candidates
            .into_iter()
            .min_by_key(|(distance, _, _)| *distance)
            .expect("a surface always has four allocation edges");
        self.side_enabled(side, offset)
    }

    fn side_enabled(&self, side: UiMountedSurfaceBorderSide, offset: i64) -> bool {
        let enabled = match side {
            UiMountedSurfaceBorderSide::Top => self.border_edges.top(),
            UiMountedSurfaceBorderSide::Right => self.border_edges.right(),
            UiMountedSurfaceBorderSide::Bottom => self.border_edges.bottom(),
            UiMountedSurfaceBorderSide::Left => self.border_edges.left(),
        };
        enabled
            && !self.border_omissions.iter().any(|omission| {
                omission.side == side && omission.start <= offset && offset < omission.end
            })
    }
}

fn micros_to_pixels(value: i64) -> f32 {
    value as f32 / super::geometry::PHYSICAL_MICROS_PER_PIXEL as f32
}

fn color_vector(color: Option<UiMountedAppearanceColor>) -> [f32; 4] {
    let Some(color) = color else {
        return [0.0; 4];
    };
    crate::native::presentation::raster::premultiplied_linear_color(color.straight_srgba())
}

const fn border_edge_bits(edges: UiMountedSurfaceBorderEdges) -> u8 {
    (if edges.top() { 1 } else { 0 })
        | (if edges.right() { 2 } else { 0 })
        | (if edges.bottom() { 4 } else { 0 })
        | (if edges.left() { 8 } else { 0 })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shader_declares_pixel_center_signed_distance_coverage() {
        assert!(ANALYTIC_ROUNDED_SURFACE_SHADER.contains("0.5 - distance"));
    }
}
