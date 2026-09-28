//! The part of a text foreground's paint a target of a given extent shows.
use super::{UiNativeFinalizedTextForeground, UiNativeTextForegroundFinalizationDenial as Denial};
use crate::native::presentation::appearance::damage::{
    UiNativeAppearanceDamage, UiNativeAppearanceDamageRect,
};
use crate::native::presentation::raster::UiNativeRasterBasis;
use crate::native::presentation::text::{sampled_glyph, UiNativeGlyphCommand};
use worth_ui_host_contract::UiMountedSurfaceBindingRequirement;

impl UiNativeFinalizedTextForeground {
    /// This paint's coverage in a successor target of `extent`.
    pub(crate) fn coverage_at(
        &self,
        extent: [u32; 2],
    ) -> Result<Box<[UiNativeAppearanceDamageRect]>, Denial> {
        visible_coverage(&self.glyphs, self.binding, extent)
    }

    pub(crate) fn rebase_coverage(&mut self, coverage: Box<[UiNativeAppearanceDamageRect]>) {
        self.coverage = coverage;
    }
}

/// Paint applies to every admitted image, including offscreen rows a Scroll
/// sample can reveal. Only coverage the target shows contributes damage.
pub(super) fn visible_coverage(
    glyphs: &[UiNativeGlyphCommand],
    binding: UiMountedSurfaceBindingRequirement,
    extent: [u32; 2],
) -> Result<Box<[UiNativeAppearanceDamageRect]>, Denial> {
    let basis = UiNativeRasterBasis::new(extent, binding.device_scale_milli() as f32 / 1_000.0);
    let mut coverage = UiNativeAppearanceDamage::new(usize::from(
        crate::native_profile::APPEARANCE_PROFILE.damage_regions,
    ));
    for glyph in glyphs {
        let Some(visible) = sampled_glyph(*glyph, None, basis).map_err(|_| Denial::Geometry)?
        else {
            continue;
        };
        let [x, y, width, height] = visible.target;
        let edges = [
            x.floor(),
            y.floor(),
            (x + width).ceil(),
            (y + height).ceil(),
        ];
        if edges
            .iter()
            .any(|edge| !edge.is_finite() || *edge < 0.0 || f64::from(*edge) > i64::MAX as f64)
        {
            return Err(Denial::Geometry);
        }
        coverage
            .add(UiNativeAppearanceDamageRect {
                left: edges[0] as i64,
                top: edges[1] as i64,
                right: edges[2] as i64,
                bottom: edges[3] as i64,
            })
            .map_err(|_| Denial::CoverageCapacity)?;
    }
    Ok(coverage.take())
}
