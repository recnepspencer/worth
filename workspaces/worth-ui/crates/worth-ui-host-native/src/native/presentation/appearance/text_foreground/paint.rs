//! Match retained physical paint to the current ordinary text command.
use super::{UiNativeFinalizedTextForeground, UiNativeTextForegroundFinalizationDenial as Denial};
use crate::native::presentation::text::UiNativeGlyphCommand;
use worth_ui_host_contract::{
    UiGlyphRasterSource, UiMountedPaintCommandIdentity, UiMountedRgba8,
    UiMountedSemanticTextMechanic,
};

impl UiNativeFinalizedTextForeground {
    pub(crate) fn qualified_candidates(&self) -> &[UiMountedSemanticTextMechanic] {
        &self.candidates
    }

    pub(crate) fn candidate_commands(
        &self,
    ) -> impl Iterator<Item = UiMountedPaintCommandIdentity> + '_ {
        self.candidates
            .iter()
            .map(UiMountedPaintCommandIdentity::semantic_text)
    }

    pub(crate) fn validate_command(
        &self,
        command: &UiMountedSemanticTextMechanic,
    ) -> Result<(), Denial> {
        let identity = UiMountedPaintCommandIdentity::semantic_text(command);
        let mut candidates = self.candidates.iter().filter(|candidate| {
            UiMountedPaintCommandIdentity::semantic_text(candidate) == identity
        });
        let candidate = candidates.next().ok_or(Denial::CandidateAttribution)?;
        if candidates.next().is_some() || !same_text_projection(candidate, command) {
            return Err(Denial::CandidateAttribution);
        }
        Ok(())
    }

    pub(crate) fn apply_paint(
        &self,
        command: &UiMountedSemanticTextMechanic,
        glyphs: &mut [UiNativeGlyphCommand],
    ) -> Result<(), Denial> {
        self.validate_command(command)?;
        let expected = self
            .glyphs
            .iter()
            .filter(|glyph| glyph.run.paint_span() == self.mechanic.paint_span())
            .count();
        let [red, green, blue, alpha] = self.mechanic.foreground().straight_srgba();
        let foreground = UiMountedRgba8::new(red, green, blue, alpha);
        let opacity = f32::from(self.mechanic.opacity().units()) / f32::from(u16::MAX);
        let mut applied = 0;
        for actual in glyphs
            .iter_mut()
            .filter(|glyph| glyph.run.paint_span() == self.mechanic.paint_span())
        {
            if matches!(
                actual.run.raster_key().source(),
                UiGlyphRasterSource::ColorOutline | UiGlyphRasterSource::ColorBitmap
            ) {
                return Err(Denial::IntrinsicColor);
            }
            actual.foreground = foreground;
            actual.opacity = opacity;
            applied += 1;
        }
        if applied != expected {
            return Err(Denial::CandidateAttribution);
        }
        Ok(())
    }
}

fn same_text_projection(
    left: &UiMountedSemanticTextMechanic,
    right: &UiMountedSemanticTextMechanic,
) -> bool {
    // Frame/receipt advancement and performed-work accounting do not change the
    // retained image. Staging validates their presentation affinity separately.
    left.schema() == right.schema()
        && left.surface() == right.surface()
        && left.binding() == right.binding()
        && left.mounted_instance() == right.mounted_instance()
        && left.bounds() == right.bounds()
        && left.clip_bounds() == right.clip_bounds()
        && left.origin_x() == right.origin_x()
        && left.origin_y() == right.origin_y()
        && left.text() == right.text()
        && left.qualified_layout_width() == right.qualified_layout_width()
        && left.slot() == right.slot()
        && left.collection_row() == right.collection_row()
        && left.foregrounds() == right.foregrounds()
        && left.profile() == right.profile()
        && left.layer_semantic_order() == right.layer_semantic_order()
}
