#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiAppearanceThemeBindingDenial {
    MissingBundle,
    SurfaceMismatch,
    GenerationMismatch,
    RoleNotAdmitted,
    Resolution(crate::runtime::appearance::UiThemeResolutionDenial),
}

impl super::UiApplicationPresentationState {
    pub(crate) fn prepare_appearance_theme_switch(
        &mut self,
        request: crate::runtime::appearance::UiThemeSwitchRequest,
    ) -> Result<
        crate::runtime::appearance::UiPreparedThemeSwitch,
        crate::runtime::appearance::UiThemeSwitchDenial,
    > {
        self.appearance_theme_state
            .as_mut()
            .ok_or(crate::runtime::appearance::UiThemeSwitchDenial::MissingActiveBinding)?
            .prepare_theme_switch(request)
    }

    pub(crate) fn materialize_initial_appearance_theme_binding(
        &mut self,
        capability: crate::runtime::appearance::UiThemeCapabilityReceipt,
    ) -> Result<(), crate::runtime::appearance::UiThemeInitialBindingDenial> {
        let result = self
            .appearance_theme_state
            .get_or_insert_with(crate::runtime::appearance::UiAppearanceThemeState::default)
            .install_initial(capability);
        result
    }

    pub(crate) fn commit_published_appearance_theme_switch(
        &mut self,
        prepared: crate::runtime::appearance::UiPreparedThemeSwitch,
    ) -> Result<(), crate::runtime::appearance::UiThemeSwitchDenial> {
        self.appearance_theme_state
            .as_mut()
            .ok_or(crate::runtime::appearance::UiThemeSwitchDenial::UnknownPreparedSwitch)?
            .commit_published_switch(prepared)
    }

    pub(crate) fn settle_unchanged_appearance_theme_switch(
        &mut self,
        request: crate::runtime::appearance::UiThemeSwitchRequest,
    ) -> Result<(), crate::runtime::appearance::UiThemeSwitchDenial> {
        self.appearance_theme_state
            .as_mut()
            .ok_or(crate::runtime::appearance::UiThemeSwitchDenial::UnknownPreparedSwitch)?
            .settle_unchanged_switch(request)
    }

    #[cfg(test)]
    pub(crate) fn replace_appearance_theme_binding_for_test(
        &mut self,
        capability: crate::runtime::appearance::UiThemeCapabilityReceipt,
    ) {
        self.appearance_theme_state
            .as_mut()
            .expect("appearance theme state should be installed")
            .replace_for_test(capability);
    }

    #[cfg(test)]
    pub(crate) fn remove_appearance_theme_binding_for_test(
        &mut self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> bool {
        self.appearance_theme_state
            .as_mut()
            .is_some_and(|state| state.remove_for_test(surface))
    }

    pub(crate) fn active_appearance_theme_binding(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Option<&crate::runtime::appearance::UiActiveThemeBinding> {
        self.appearance_theme_state
            .as_ref()
            .and_then(|state| state.active_binding(surface))
    }

    pub(crate) fn appearance_theme_state(
        &self,
    ) -> Option<&crate::runtime::appearance::UiAppearanceThemeState> {
        self.appearance_theme_state.as_ref()
    }

    pub(crate) fn resolve_appearance_theme_binding(
        capabilities: &crate::capability::CapabilitySnapshot,
        role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
        binding: &crate::runtime::appearance::UiActiveThemeBinding,
    ) -> Result<crate::runtime::appearance::UiThemeResolutionView, UiAppearanceThemeBindingDenial>
    {
        let themes = capabilities
            .appearance_themes()
            .ok_or(UiAppearanceThemeBindingDenial::MissingBundle)?;
        if binding.surface() != surface {
            return Err(UiAppearanceThemeBindingDenial::SurfaceMismatch);
        }
        if binding.capability().application() != generation {
            return Err(UiAppearanceThemeBindingDenial::GenerationMismatch);
        }
        let view = crate::runtime::appearance::UiThemeResolutionView::from_capability(
            binding.capability(),
            themes,
        )
        .map_err(UiAppearanceThemeBindingDenial::Resolution)?;
        if !view.admits_role(role) {
            return Err(UiAppearanceThemeBindingDenial::RoleNotAdmitted);
        }
        Ok(view)
    }
}
