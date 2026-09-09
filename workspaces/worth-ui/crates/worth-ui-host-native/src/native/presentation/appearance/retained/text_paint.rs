//! Adopted paint lookup stays with the existing appearance retention owner.
use super::{
    UiNativeAppearanceCommand, UiNativeAppearanceCommandIdentity, UiNativeAppearanceRetained,
};
use crate::native::presentation::appearance::text_foreground::UiNativeTextForegroundFinalizationDenial as Denial;
use crate::native::presentation::text::UiNativeGlyphCommand;
use crate::native::text_atlas::UiNativeTextAtlas;
use worth_ui_host_contract::UiMountedSemanticTextMechanic;

impl UiNativeAppearanceRetained {
    pub(crate) fn apply_text_paint(
        &self,
        command: &UiMountedSemanticTextMechanic,
        glyphs: &mut [UiNativeGlyphCommand],
        atlas: &UiNativeTextAtlas,
    ) -> Result<(), Denial> {
        for span in command.foregrounds() {
            let identity = UiNativeAppearanceCommandIdentity::TextForeground {
                target: command.mounted_instance(),
                command: worth_ui_host_contract::UiMountedPaintCommandIdentity::semantic_text(
                    command,
                )
                .semantic_text_identity_parts()
                .expect("semantic text command has text identity parts"),
                span_digest: span.identity().digest(),
            };
            let Some(key) = self.identities.get(&identity) else {
                continue;
            };
            let Some(UiNativeAppearanceCommand::TextForeground(foreground)) =
                self.commands.get(key)
            else {
                return Err(Denial::CandidateAttribution);
            };
            foreground.validate_images(atlas)?;
            foreground.apply_paint(command, glyphs)?;
        }
        Ok(())
    }
}
