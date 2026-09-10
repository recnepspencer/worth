//! Ordinary text raster planning consumes retained runs and presentation paint once.
use super::{UiNativeRetainedDrawList, UiNativeRetainedDrawListDenial as Denial};
use crate::native::presentation::{
    raster::UiNativeRasterBasis,
    text::{plan_glyph_commands, UiNativeGlyphCommand},
};
use crate::native::text_atlas::UiNativeTextAtlas;
use worth_ui_host_contract::{UiMountedPaintCommand, UiMountedPaintCommandIdentity};

impl UiNativeRetainedDrawList {
    pub(in crate::native::presentation) fn plan_text_commands(
        &self,
        identity: UiMountedPaintCommandIdentity,
        atlas: &UiNativeTextAtlas,
        basis: UiNativeRasterBasis,
    ) -> Result<Box<[UiNativeGlyphCommand]>, Denial> {
        let Some(UiMountedPaintCommand::SemanticText { mechanic, .. }) = self.command(identity)
        else {
            return Err(Denial::CommandMismatch);
        };
        let mut glyphs = plan_glyph_commands(self.glyph_runs(identity), atlas, basis.extent())
            .map_err(|_| Denial::CommandMismatch)?;
        // Match original immutable image evidence before any sampled geometry.
        self.apply_text_paint(mechanic, &mut glyphs)?;
        let sample = self.sample_override(identity);
        if let Some(sample) = sample {
            for glyph in glyphs.iter_mut() {
                glyph.opacity = sample.opacity().factor();
            }
        }
        if let Some(transform) = sample.and_then(|sample| sample.transform()) {
            for glyph in glyphs.iter_mut() {
                glyph.target = crate::native::presentation::sample::transform_physical_box(
                    glyph.target,
                    transform,
                    basis,
                )
                .map_err(|_| Denial::CommandMismatch)?;
            }
        }
        Ok(glyphs)
    }
}
