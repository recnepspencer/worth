use worth_ui_host_contract::{
    UiMountedAppearanceColor, UiMountedSurfaceAppearanceMechanic, UiMountedSurfacePaint,
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
        let inner = if self.border_width == 0 {
            UiNativeAnalyticCoverage::ZERO
        } else {
            let inner_rect = self
                .allocation
                .inset(self.border_width)
                .expect("the sealed border-width contract leaves a positive inner contour");
            let inner_radii = self
                .radii
                .map(|radius| radius.saturating_sub(self.border_width));
            coverage_from_signed_distance(rounded_signed_distance(
                pixel_x,
                pixel_y,
                inner_rect,
                inner_radii,
            ))
        };
        let border_coverage = match self.paint_kind {
            UiNativeSurfacePaintKind::Fill => zero,
            UiNativeSurfacePaintKind::Border | UiNativeSurfacePaintKind::FillAndBorder => {
                outer.subtract(inner)
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shader_declares_pixel_center_signed_distance_coverage() {
        assert!(ANALYTIC_ROUNDED_SURFACE_SHADER.contains("0.5 - distance"));
    }
}
