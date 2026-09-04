use super::WorthUiActiveApplicationSession;

impl WorthUiActiveApplicationSession {
    pub(crate) fn admit_application_theme_values(
        &mut self,
        changes: &[super::super::UiNativeThemeTokenValueChange],
    ) -> Result<(), ()> {
        if changes.is_empty() {
            return Ok(());
        }
        let update = if self
            .application
            .prepared_authority()
            .consumed_fact_index()
            .has_appearance_consumers()
        {
            let themes = self.capabilities().appearance_themes().ok_or(())?;
            self.presentation.prepare_theme_values_for_appearance(
                changes,
                themes,
                &self.active_generation_identity(),
            )?
        } else {
            self.presentation.prepare_theme_values(changes)?
        };
        let mut invalidation: Option<crate::runtime::appearance::UiAppearanceInvalidationBatch> =
            None;
        for token in update.changed_tokens() {
            let batch = self
                .application
                .appearance_theme_invalidation_batch_for_slot(token)
                .map_err(|_| ())?;
            if let Some(current) = invalidation.as_mut() {
                current.merge(batch);
            } else {
                invalidation = Some(batch);
            }
        }
        self.presentation.commit_theme_values(update, invalidation)
    }

    pub(crate) fn complete_application_theme_values_source(
        &self,
    ) -> crate::mounting::UiMountedThemeValueSource {
        self.presentation.theme_values_source()
    }
}
