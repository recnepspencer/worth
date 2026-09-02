#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiThemeCapabilityReceipt {
    definition: super::UiThemeDefinitionIdentity,
    definition_revision: u64,
    slot_catalog_revision: u64,
    required_roles: Box<[UiThemeRequiredRoleBasis]>,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    application: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    host_profile: worth_ui_host_contract::UiHostAppearanceProfileContract,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiThemeRequiredRoleBasis {
    identity: worth_ui_dsl::UiAppearanceRoleIdentity,
    revision: worth_ui_dsl::UiAppearanceRoleRevision,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiThemeCapabilityReceiptDenial {
    CatalogRevisionMismatch,
    EmptyRequiredRoleSet,
    DuplicateRequiredRole,
    MissingRequiredRole,
    MissingRequiredSlot,
    RequiredSlotKindMismatch,
    MissingDefinitionValue,
    MissingDefinition,
}

pub(crate) struct UiThemeCapabilityAdmission<'basis> {
    definition: &'basis super::UiThemeDefinition,
    catalog: &'basis crate::capability::UiThemeSlotCatalog,
    registered_roles: &'basis crate::capability::FrozenAppearanceRoleCapabilities,
    host_profile: &'basis worth_ui_host_contract::UiHostAppearanceProfileContract,
}

impl<'basis> UiThemeCapabilityAdmission<'basis> {
    pub(crate) fn from_frozen_capabilities(
        themes: &'basis crate::capability::FrozenAppearanceThemeCapabilities,
        definition: &super::UiThemeDefinitionIdentity,
        registered_roles: &'basis crate::capability::FrozenAppearanceRoleCapabilities,
        host_profile: &'basis worth_ui_host_contract::UiHostAppearanceProfileContract,
    ) -> Result<Self, UiThemeCapabilityReceiptDenial> {
        let definition = themes
            .get(definition)
            .ok_or(UiThemeCapabilityReceiptDenial::MissingDefinition)?;
        Ok(Self {
            definition,
            catalog: themes.catalog(),
            registered_roles,
            host_profile,
        })
    }

    pub(crate) fn issue(
        self,
        required_roles: impl IntoIterator<Item = worth_ui_dsl::UiAppearanceRoleIdentity>,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        application: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    ) -> Result<UiThemeCapabilityReceipt, UiThemeCapabilityReceiptDenial> {
        if self.definition.catalog_revision() != self.catalog.revision() {
            return Err(UiThemeCapabilityReceiptDenial::CatalogRevisionMismatch);
        }
        let mut identities = required_roles.into_iter().collect::<Vec<_>>();
        identities.sort();
        if identities.is_empty() {
            return Err(UiThemeCapabilityReceiptDenial::EmptyRequiredRoleSet);
        }
        if identities.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(UiThemeCapabilityReceiptDenial::DuplicateRequiredRole);
        }
        let roles = identities
            .into_iter()
            .map(|identity| {
                let role = self
                    .registered_roles
                    .get(&identity)
                    .ok_or(UiThemeCapabilityReceiptDenial::MissingRequiredRole)?;
                for slot_use in role.slot_uses() {
                    let slot = crate::capability::ThemeTokenId::new(slot_use.slot().as_str())
                        .map_err(|_| UiThemeCapabilityReceiptDenial::MissingRequiredSlot)?;
                    let declaration = self
                        .catalog
                        .get(&slot)
                        .ok_or(UiThemeCapabilityReceiptDenial::MissingRequiredSlot)?;
                    if declaration.kind() != slot_use.expected_kind() {
                        return Err(UiThemeCapabilityReceiptDenial::RequiredSlotKindMismatch);
                    }
                    let target = self
                        .catalog
                        .resolved_target(&slot)
                        .ok_or(UiThemeCapabilityReceiptDenial::MissingRequiredSlot)?;
                    if self.definition.value(target).is_none() {
                        return Err(UiThemeCapabilityReceiptDenial::MissingDefinitionValue);
                    }
                }
                Ok(UiThemeRequiredRoleBasis {
                    identity,
                    revision: role.revision(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(UiThemeCapabilityReceipt {
            definition: self.definition.identity().clone(),
            definition_revision: self.definition.revision(),
            slot_catalog_revision: self.catalog.revision(),
            required_roles: roles.into_boxed_slice(),
            surface,
            application,
            host_profile: self.host_profile.clone(),
        })
    }
}

impl UiThemeCapabilityReceipt {
    pub(crate) fn definition(&self) -> &super::UiThemeDefinitionIdentity {
        &self.definition
    }
    pub(crate) const fn definition_revision(&self) -> u64 {
        self.definition_revision
    }
    pub(crate) const fn slot_catalog_revision(&self) -> u64 {
        self.slot_catalog_revision
    }
    pub(crate) fn required_roles(&self) -> &[UiThemeRequiredRoleBasis] {
        &self.required_roles
    }
    pub(crate) const fn surface(&self) -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
        self.surface
    }
    pub(crate) const fn application(
        &self,
    ) -> &crate::runtime::WorthUiActiveApplicationGenerationIdentity {
        &self.application
    }
    pub(crate) const fn host_profile(
        &self,
    ) -> &worth_ui_host_contract::UiHostAppearanceProfileContract {
        &self.host_profile
    }

    #[cfg(test)]
    pub(crate) fn for_test(
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        application: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    ) -> Self {
        Self {
            definition: super::UiThemeDefinitionIdentity::new("appearance-state-test")
                .expect("test theme definition identity"),
            definition_revision: 1,
            slot_catalog_revision: 1,
            required_roles: Box::new([]),
            surface,
            application,
            host_profile: worth_ui_host_contract::UiHostAppearanceProfileContract::admit(
                "appearance-state-test-host",
                1,
                worth_ui_host_contract::UiHostAppearanceMechanicFamily::ALL,
                Some(worth_ui_host_contract::UiHostPrimaryPointerKind::Mouse),
            )
            .expect("test host appearance profile"),
        }
    }
}

impl UiThemeRequiredRoleBasis {
    pub(crate) const fn identity(&self) -> &worth_ui_dsl::UiAppearanceRoleIdentity {
        &self.identity
    }
    pub(crate) const fn revision(&self) -> worth_ui_dsl::UiAppearanceRoleRevision {
        self.revision
    }
}
