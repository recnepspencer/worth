use worth_ui_host_contract::{UiMountedAppearanceColor, UiMountedOutlineAppearanceMechanic};

use super::antialiasing::{
    coverage_from_signed_distance, rounded_signed_distance, UiNativeAnalyticCoverage,
};
use super::damage::UiNativeAppearanceDamageRect;
use super::geometry::{
    physical_length, physical_radii, UiNativeAppearanceScale, UiNativeGeometryDenial,
    UiNativePhysicalPixelRect, UiNativePhysicalRect,
};

/// Staged WGSL posture for the outside ring. It is not referenced by the
/// current `UiNativeRasterOperation` publisher.
pub(crate) const ANALYTIC_OUTLINE_SHADER: &str = r#"
fn ring_coverage(outer_distance: f32, inner_distance: f32) -> f32 {
    return max(coverage_from_signed_distance(outer_distance)
        - coverage_from_signed_distance(inner_distance), 0.0);
}
"#;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiNativeOutlineSample {
    pub(crate) coverage: UiNativeAnalyticCoverage,
    pub(crate) color: UiMountedAppearanceColor,
    pub(crate) opacity: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiNativeOutlinePrimitive {
    allocation: UiNativePhysicalRect,
    visual_bounds: UiNativePhysicalPixelRect,
    clip: UiNativePhysicalPixelRect,
    outer: UiNativePhysicalRect,
    inner: UiNativePhysicalRect,
    outer_radii: [i64; 4],
    inner_radii: [i64; 4],
    color: UiMountedAppearanceColor,
    opacity: u16,
}

pub(crate) struct UiNativeOutlinePipeline;

impl UiNativeOutlinePipeline {
    pub(crate) fn prepare(
        mechanic: &UiMountedOutlineAppearanceMechanic,
        scale: UiNativeAppearanceScale,
    ) -> Result<UiNativeOutlinePrimitive, UiNativeGeometryDenial> {
        let allocation = UiNativePhysicalRect::from_allocation(mechanic.allocation(), scale)?;
        let visual_bounds =
            UiNativePhysicalRect::from_visual_bounds(mechanic.visual_bounds(), scale)?
                .pixel_bounds();
        let clip = UiNativePhysicalRect::from_clip(mechanic.clip(), scale)?.pixel_bounds();
        let offset = physical_length(mechanic.offset(), scale)?;
        let width = physical_length(mechanic.width(), scale)?;
        let inner = allocation.expand(offset)?;
        let outer = inner.expand(width)?;
        let inner_radii = physical_radii(mechanic.radii().corners(), scale)?;
        let mut outer_radii = [0; 4];
        for (output, radius) in outer_radii.iter_mut().zip(inner_radii) {
            *output = radius
                .checked_add(width)
                .ok_or(UiNativeGeometryDenial::CoordinateOverflow)?;
        }
        Ok(UiNativeOutlinePrimitive {
            allocation,
            visual_bounds,
            clip,
            outer,
            inner,
            outer_radii,
            inner_radii,
            color: mechanic.color(),
            opacity: mechanic.opacity().units(),
        })
    }
}

impl UiNativeOutlinePrimitive {
    pub(super) const fn outer(&self) -> UiNativePhysicalRect {
        self.outer
    }

    pub(super) const fn clip(&self) -> UiNativePhysicalPixelRect {
        self.clip
    }

    pub(super) const fn outer_radii(&self) -> [i64; 4] {
        self.outer_radii
    }

    pub(super) const fn width(&self) -> i64 {
        self.inner.left - self.outer.left
    }

    pub(super) const fn color(&self) -> UiMountedAppearanceColor {
        self.color
    }

    pub(super) const fn opacity(&self) -> u16 {
        self.opacity
    }

    pub(crate) fn allocation(&self) -> UiNativePhysicalRect {
        self.allocation
    }

    pub(crate) fn visual_bounds(&self) -> UiNativePhysicalPixelRect {
        self.visual_bounds
    }

    pub(crate) fn damage_rect(&self) -> UiNativeAppearanceDamageRect {
        UiNativeAppearanceDamageRect::from_pixel_rect(self.visual_bounds)
    }

    pub(crate) fn sample(&self, pixel_x: i64, pixel_y: i64) -> UiNativeOutlineSample {
        if !self.clip.contains(pixel_x, pixel_y) {
            return UiNativeOutlineSample {
                coverage: UiNativeAnalyticCoverage::ZERO,
                color: self.color,
                opacity: self.opacity,
            };
        }
        let outer = coverage_from_signed_distance(rounded_signed_distance(
            pixel_x,
            pixel_y,
            self.outer,
            self.outer_radii,
        ));
        let inner = coverage_from_signed_distance(rounded_signed_distance(
            pixel_x,
            pixel_y,
            self.inner,
            self.inner_radii,
        ));
        UiNativeOutlineSample {
            coverage: outer.subtract(inner),
            color: self.color,
            opacity: self.opacity,
        }
    }

    pub(crate) fn extends_beyond_allocation(&self) -> bool {
        self.visual_bounds.left < self.allocation.pixel_bounds().left
            || self.visual_bounds.top < self.allocation.pixel_bounds().top
            || self.visual_bounds.right > self.allocation.pixel_bounds().right
            || self.visual_bounds.bottom > self.allocation.pixel_bounds().bottom
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shader_keeps_outline_as_outer_minus_inner_coverage() {
        assert!(ANALYTIC_OUTLINE_SHADER.contains("outer_distance"));
        assert!(ANALYTIC_OUTLINE_SHADER.contains("inner_distance"));
    }
}
