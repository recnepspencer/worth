//! Ordinary text raster planning consumes retained runs and presentation paint once.
use super::{UiNativeRetainedDrawList, UiNativeRetainedDrawListDenial as Denial};
use crate::native::presentation::{
    raster::UiNativeRasterBasis,
    text::{plan_raw_glyph_commands, sampled_glyph, UiNativeGlyphCommand},
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
        let mut glyphs = plan_raw_glyph_commands(self.glyph_runs(identity), atlas)
            .map_err(|_| Denial::CommandMismatch)?;
        // Match original immutable image evidence before any sampled geometry.
        self.apply_text_paint(mechanic, &mut glyphs)?;
        let sample = self.sample_override(identity);
        glyphs
            .into_vec()
            .into_iter()
            .map(|glyph| sampled_glyph(glyph, sample, basis).map_err(|_| Denial::CommandMismatch))
            .collect::<Result<Vec<_>, _>>()
            .map(|glyphs| glyphs.into_iter().flatten().collect())
    }
}
