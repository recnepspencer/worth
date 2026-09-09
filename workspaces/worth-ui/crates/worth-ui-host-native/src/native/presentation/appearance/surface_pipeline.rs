use worth_ui_host_contract::{
    UiMountedAppearanceColor, UiMountedSurfaceAppearanceMechanic, UiMountedSurfaceBorderEdges,
    UiMountedSurfaceBorderSide, UiMountedSurfacePaint,
};

use super::antialiasing::{
    coverage_from_signed_distance, rounded_signed_distance, UiNativeAnalyticCoverage,
};
use super::damage::UiNativeAppearanceDamageRect;
use super::geometry::{
    physical_length, physical_radii, UiNativeAppearanceScale, UiNativeGeometryDenial,
    UiNativePhysicalPixelRect, UiNativePhysicalRect,
};

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
}

impl UiNativeSurfacePrimitive {
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
        (y < self.allocation.top + self.border_width
            && self.side_enabled(UiMountedSurfaceBorderSide::Top, x - self.allocation.left))
            || (x >= self.allocation.right - self.border_width
                && self.side_enabled(UiMountedSurfaceBorderSide::Right, y - self.allocation.top))
            || (y >= self.allocation.bottom - self.border_width
                && self.side_enabled(UiMountedSurfaceBorderSide::Bottom, x - self.allocation.left))
            || (x < self.allocation.left + self.border_width
                && self.side_enabled(UiMountedSurfaceBorderSide::Left, y - self.allocation.top))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shader_declares_pixel_center_signed_distance_coverage() {
        assert!(ANALYTIC_ROUNDED_SURFACE_SHADER.contains("0.5 - distance"));
    }
}
