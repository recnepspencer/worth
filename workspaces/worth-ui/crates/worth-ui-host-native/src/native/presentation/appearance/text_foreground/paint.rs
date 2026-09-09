//! Match retained physical paint to the current ordinary text command.
use super::{UiNativeFinalizedTextForeground, UiNativeTextForegroundFinalizationDenial as Denial};
use crate::native::presentation::text::UiNativeGlyphCommand;
use worth_ui_host_contract::{UiMountedPaintCommandIdentity, UiMountedSemanticTextMechanic};

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
        let identity = UiMountedPaintCommandIdentity::semantic_text(command);
        let mut expected = self
            .glyphs
            .iter()
            .filter(|glyph| glyph.run.mechanic() == identity);
        for actual in glyphs
            .iter_mut()
            .filter(|glyph| glyph.run.paint_span() == self.mechanic.paint_span())
        {
            let witness = expected.next().ok_or(Denial::CandidateAttribution)?;
            if actual.run != witness.run
                || actual.target != witness.target
                || actual.atlas_kind != witness.atlas_kind
                || actual.atlas_page != witness.atlas_page
                || actual.texture_uv != witness.texture_uv
            {
                return Err(Denial::CandidateAttribution);
            }
            actual.foreground = witness.foreground;
            actual.opacity = witness.opacity;
        }
        if expected.next().is_some() {
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
        && left.allocation_basis() == right.allocation_basis()
        && left.bounds() == right.bounds()
        && left.clip_bounds() == right.clip_bounds()
        && left.origin_x() == right.origin_x()
        && left.origin_y() == right.origin_y()
        && left.text() == right.text()
        && left.qualified_layout_identity() == right.qualified_layout_identity()
        && left.qualified_layout_request() == right.qualified_layout_request()
        && left.qualified_layout_profile() == right.qualified_layout_profile()
        && left.qualified_layout_fonts() == right.qualified_layout_fonts()
        && left.qualified_layout_scale() == right.qualified_layout_scale()
        && left.qualified_layout_width() == right.qualified_layout_width()
        && left.slot() == right.slot()
        && left.collection_row() == right.collection_row()
        && left.foregrounds() == right.foregrounds()
        && left.profile() == right.profile()
        && left.layer_semantic_order() == right.layer_semantic_order()
        && left.capability_generation() == right.capability_generation()
        && left.capability_profile_digest() == right.capability_profile_digest()
}
