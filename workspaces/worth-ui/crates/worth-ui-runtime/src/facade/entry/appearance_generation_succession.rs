use super::WorthUiActiveApplicationSession;

impl WorthUiActiveApplicationSession {
    pub(super) fn prepare_appearance_generation_succession(
        &self,
        succession: &crate::facade::prepared_application_authority::
            WorthUiPreparedApplicationGenerationSuccession,
    ) -> Result<
        crate::runtime::presentation_state::UiPreparedAppearanceGenerationSuccession,
        crate::runtime::presentation_state::UiAppearanceGenerationSuccessionDenial,
    > {
        let predecessor = crate::runtime::WorthUiActiveApplicationGenerationIdentity::current(
            self.identity,
            self.application.generation_identity(),
        );
        let successor = crate::runtime::WorthUiActiveApplicationGenerationIdentity::current(
            self.identity,
            succession.successor(),
        );
        self.presentation.prepare_appearance_generation_succession(
            succession,
            &predecessor,
            &successor,
            self.appearance_theme_admission.as_ref(),
        )
    }

    pub(super) fn commit_appearance_generation_succession(
        &mut self,
        prepared: crate::runtime::presentation_state::UiPreparedAppearanceGenerationSuccession,
    ) {
        let admission = prepared.admission().cloned();
        self.presentation
            .commit_appearance_generation_succession(prepared);
        self.appearance_theme_admission = admission;
        self.appearance_owner_snapshot = None;
        self.appearance_inspection.reset_for_new_generation();
    }

    pub(super) fn prepare_appearance_replacement_succession(
        &self,
        successor: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
        successor_admission: Option<crate::runtime::appearance::UiPreparedThemeBindingAdmission>,
        rebinding: Option<&crate::runtime::appearance::UiPreparedThemeGenerationRebinding>,
        themes: Option<&crate::capability::FrozenAppearanceThemeCapabilities>,
    ) -> Result<
        crate::runtime::presentation_state::UiPreparedAppearanceGenerationSuccession,
        crate::runtime::presentation_state::UiAppearanceGenerationSuccessionDenial,
    > {
        let predecessor = self.active_generation_identity();
        self.presentation.prepare_appearance_replacement_succession(
            &predecessor,
            &successor,
            self.appearance_theme_admission.as_ref(),
            successor_admission,
            rebinding,
            themes,
        )
    }
}
