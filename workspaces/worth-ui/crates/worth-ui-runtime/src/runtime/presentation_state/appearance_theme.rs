#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiAppearanceThemeBindingDenial {
    MissingBundle,
    MissingDefinition,
    HostProfileUnavailable,
    Capability(crate::runtime::appearance::UiThemeCapabilityReceiptDenial),
    Initial(crate::runtime::appearance::UiThemeInitialBindingDenial),
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
        &mut self,
        capabilities: &crate::capability::CapabilitySnapshot,
        role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    ) -> Result<crate::runtime::appearance::UiThemeResolutionView, UiAppearanceThemeBindingDenial>
    {
        let themes = capabilities
            .appearance_themes()
            .ok_or(UiAppearanceThemeBindingDenial::MissingBundle)?;
        if let Some(binding) = self
            .appearance_theme_state
            .as_ref()
            .and_then(|state| state.active_binding(surface))
            .cloned()
        {
            if binding.capability().application() == generation {
                return crate::runtime::appearance::UiThemeResolutionView::from_capability(
                    &binding.capability(),
                    themes,
                )
                .map(|view| view.with_value_overrides(std::sync::Arc::clone(&self.token_values)))
                .map_err(UiAppearanceThemeBindingDenial::Resolution);
            }
        }
        let definition = themes
            .definitions()
            .first()
            .ok_or(UiAppearanceThemeBindingDenial::MissingDefinition)?;
        let host_profile = worth_ui_host_contract::UiHostAppearanceProfileContract::admit(
            "worth-ui-runtime-appearance-observation",
            1,
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::ALL,
            None,
        )
        .map_err(|_| UiAppearanceThemeBindingDenial::HostProfileUnavailable)?;
        let capability =
            crate::runtime::appearance::UiThemeCapabilityAdmission::from_frozen_capabilities(
                themes,
                definition.identity(),
                capabilities.appearance_roles(),
                &host_profile,
            )
            .map_err(UiAppearanceThemeBindingDenial::Capability)?
            .issue([role.role().clone()], surface, generation.clone())
            .map_err(UiAppearanceThemeBindingDenial::Capability)?;
        if self
            .appearance_theme_state
            .as_ref()
            .and_then(|state| state.active_binding(surface))
            .is_some()
        {
            self.appearance_theme_state
                .as_mut()
                .expect("an active appearance theme binding exists")
                .install_for_generation(capability);
        } else {
            self.install_initial_appearance_theme_binding(capability)
                .map_err(UiAppearanceThemeBindingDenial::Initial)?;
        }
        let binding = self
            .appearance_theme_state
            .as_ref()
            .and_then(|state| state.active_binding(surface))
            .expect("initial appearance theme binding is installed")
            .clone();
        crate::runtime::appearance::UiThemeResolutionView::from_capability(
            &binding.capability(),
            themes,
        )
        .map(|view| view.with_value_overrides(std::sync::Arc::clone(&self.token_values)))
        .map_err(UiAppearanceThemeBindingDenial::Resolution)
    }
}
