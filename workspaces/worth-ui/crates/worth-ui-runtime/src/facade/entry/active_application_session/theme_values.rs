use super::WorthUiActiveApplicationSession;

impl WorthUiActiveApplicationSession {
    pub(crate) fn admit_application_theme_values(
        &mut self,
        changes: &[super::super::UiNativeThemeTokenValueChange],
    ) -> Result<(), ()> {
        let update = self.presentation.prepare_theme_values(changes)?;
        let mut canonical_selection =
            crate::runtime::appearance::UiAppearanceConsumerSelection::empty();
        for token in update.changed_tokens() {
            canonical_selection.merge(
                self.application
                    .appearance_slot_consumers(token)
                    .map_err(|_| ())?,
            );
        }
        self.presentation
            .commit_theme_values(update, canonical_selection)
    }

    pub(crate) fn complete_application_theme_values_source(
        &self,
    ) -> crate::mounting::UiMountedThemeValueSource {
        self.presentation.theme_values_source()
    }
}
