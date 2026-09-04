#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiAppearanceThemeBindingDenial {
    MissingBundle,
    MissingActiveBinding,
    SurfaceMismatch,
    GenerationMismatch,
    RoleNotAdmitted,
    StaleTypedValues,
    Resolution(crate::runtime::appearance::UiThemeResolutionDenial),
}

impl super::UiApplicationPresentationState {
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

    pub(crate) fn appearance_theme_resolution_view(
        &self,
        capabilities: &crate::capability::CapabilitySnapshot,
        role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    ) -> Result<crate::runtime::appearance::UiThemeResolutionView, UiAppearanceThemeBindingDenial>
    {
        let themes = capabilities
            .appearance_themes()
            .ok_or(UiAppearanceThemeBindingDenial::MissingBundle)?;
        let binding = self
            .active_appearance_theme_binding(surface)
            .ok_or(UiAppearanceThemeBindingDenial::MissingActiveBinding)?;
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
        if let Some(values) = self.appearance_theme_values.get(&surface) {
            if values.capability() != binding.capability() {
                return Err(UiAppearanceThemeBindingDenial::StaleTypedValues);
            }
            return Ok(view.with_typed_values(values.values()));
        }
        Ok(view)
    }
}
