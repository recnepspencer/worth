use super::WorthUiActiveApplicationSession;

#[derive(Debug)]
pub enum UiAppearanceGenerationSuccessionDenial {
    Theme(crate::runtime::presentation_state::UiAppearanceGenerationSuccessionDenial),
    Inspection(crate::runtime::appearance::UiAppearanceInspectionGenerationSuccessionDenial),
}

pub struct UiPreparedAppearanceGenerationSuccession {
    theme: crate::runtime::presentation_state::UiPreparedAppearanceGenerationSuccession,
    inspection: crate::runtime::appearance::UiPreparedAppearanceInspectionGenerationSuccession,
}

impl WorthUiActiveApplicationSession {
    pub(super) fn prepare_appearance_generation_succession(
        &self,
        succession: &crate::facade::prepared_application_authority::
            WorthUiPreparedApplicationGenerationSuccession,
    ) -> Result<UiPreparedAppearanceGenerationSuccession, UiAppearanceGenerationSuccessionDenial>
    {
        let predecessor = crate::runtime::WorthUiActiveApplicationGenerationIdentity::current(
            self.identity,
            self.application.generation_identity(),
        );
        let successor = crate::runtime::WorthUiActiveApplicationGenerationIdentity::current(
            self.identity,
            succession.successor(),
        );
        let theme = self
            .presentation
            .prepare_appearance_generation_succession(
                succession,
                &predecessor,
                &successor,
                self.appearance_theme_admission.as_ref(),
            )
            .map_err(UiAppearanceGenerationSuccessionDenial::Theme)?;
        let inspection = self
            .appearance_inspection
            .prepare_generation_succession(&predecessor, &successor)
            .map_err(UiAppearanceGenerationSuccessionDenial::Inspection)?;
        Ok(UiPreparedAppearanceGenerationSuccession { theme, inspection })
    }

    pub(super) fn commit_appearance_generation_succession(
        &mut self,
        prepared: UiPreparedAppearanceGenerationSuccession,
    ) {
        let UiPreparedAppearanceGenerationSuccession { theme, inspection } = prepared;
        let admission = theme.admission().cloned();
        self.presentation
            .commit_appearance_generation_succession(theme);
        self.appearance_theme_admission = admission;
        self.appearance_owner_snapshot = None;
        self.appearance_inspection
            .commit_generation_succession(inspection);
    }

    pub(super) fn prepare_appearance_replacement_succession(
        &self,
        successor: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
        successor_admission: Option<crate::runtime::appearance::UiPreparedThemeBindingAdmission>,
        rebinding: Option<&crate::runtime::appearance::UiPreparedThemeGenerationRebinding>,
        themes: Option<&crate::capability::FrozenAppearanceThemeCapabilities>,
    ) -> Result<UiPreparedAppearanceGenerationSuccession, UiAppearanceGenerationSuccessionDenial>
    {
        let predecessor = self.active_generation_identity();
        let theme = self
            .presentation
            .prepare_appearance_replacement_succession(
                &predecessor,
                &successor,
                self.appearance_theme_admission.as_ref(),
                successor_admission,
                rebinding,
                themes,
            )
            .map_err(UiAppearanceGenerationSuccessionDenial::Theme)?;
        let inspection = self
            .appearance_inspection
            .prepare_generation_succession(&predecessor, &successor)
            .map_err(UiAppearanceGenerationSuccessionDenial::Inspection)?;
        Ok(UiPreparedAppearanceGenerationSuccession { theme, inspection })
    }
}
