use super::UiActiveThemeBinding;

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
pub(crate) struct UiPreparedThemeBindingAdmission {
    definition: super::UiThemeDefinitionIdentity,
    definition_revision: u64,
    slot_catalog_revision: u64,
    required_roles: Box<[UiThemeRequiredRoleBasis]>,
    application: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    host_profile: worth_ui_host_contract::UiHostAppearanceProfileContract,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiThemeRequiredRoleBasis {
    identity: worth_ui_dsl::UiAppearanceRoleIdentity,
    revision: worth_ui_dsl::UiAppearanceRoleRevision,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiThemeCapabilityReceiptDenial {
    MissingBundle,
    MissingHostProfile,
    CatalogRevisionMismatch,
    EmptyRequiredRoleSet,
    DuplicateRequiredRole,
    MissingRequiredRole,
    MissingRequiredSlot,
    RequiredSlotKindMismatch,
    MissingDefinitionValue,
    MissingDefinition,
    RequiredRoleRevisionMismatch,
    HostProfileMismatch,
    BindingGenerationExhausted,
    StaleBinding,
    GenerationMismatch,
}

pub(crate) struct UiThemeCapabilityAdmission<'basis> {
    definition: &'basis super::UiThemeDefinition,
    catalog: &'basis crate::capability::UiThemeSlotCatalog,
    registered_roles: &'basis crate::capability::FrozenAppearanceRoleCapabilities,
    host_profile: &'basis worth_ui_host_contract::UiHostAppearanceProfileContract,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiPreparedThemeGenerationRebinding {
    predecessor: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    successor: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    replacements: Box<[UiPreparedThemeBindingReplacement]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiPreparedThemeBindingReplacement {
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    predecessor_generation: u64,
    capability: UiThemeCapabilityReceipt,
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
        Ok(self
            .prepare(required_roles, application)?
            .materialize(surface))
    }

    pub(crate) fn prepare(
        &self,
        required_roles: impl IntoIterator<Item = worth_ui_dsl::UiAppearanceRoleIdentity>,
        application: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    ) -> Result<UiPreparedThemeBindingAdmission, UiThemeCapabilityReceiptDenial> {
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
        Ok(UiPreparedThemeBindingAdmission {
            definition: self.definition.identity().clone(),
            definition_revision: self.definition.revision(),
            slot_catalog_revision: self.catalog.revision(),
            required_roles: roles.into_boxed_slice(),
            application,
            host_profile: self.host_profile.clone(),
        })
    }
}

pub(crate) fn prepare_theme_generation_rebinding<'binding>(
    themes: &crate::capability::FrozenAppearanceThemeCapabilities,
    registered_roles: &crate::capability::FrozenAppearanceRoleCapabilities,
    host_profile: &worth_ui_host_contract::UiHostAppearanceProfileContract,
    predecessor: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    successor: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    required_roles: &[worth_ui_dsl::UiAppearanceRoleIdentity],
    bindings: impl IntoIterator<Item = &'binding UiActiveThemeBinding>,
) -> Result<UiPreparedThemeGenerationRebinding, UiThemeCapabilityReceiptDenial> {
    let mut replacements = Vec::new();
    for binding in bindings {
        if binding.capability().application() != predecessor {
            return Err(UiThemeCapabilityReceiptDenial::GenerationMismatch);
        }
        if binding.capability().host_profile() != host_profile {
            return Err(UiThemeCapabilityReceiptDenial::HostProfileMismatch);
        }
        if binding.binding_generation() == u64::MAX {
            return Err(UiThemeCapabilityReceiptDenial::BindingGenerationExhausted);
        }
        let admission = UiThemeCapabilityAdmission::from_frozen_capabilities(
            themes,
            binding.capability().definition(),
            registered_roles,
            host_profile,
        )?;
        let prepared = admission.prepare(required_roles.iter().cloned(), successor.clone())?;
        replacements.push(UiPreparedThemeBindingReplacement {
            surface: binding.surface(),
            predecessor_generation: binding.binding_generation(),
            capability: prepared.materialize(binding.surface()),
        });
    }
    Ok(UiPreparedThemeGenerationRebinding {
        predecessor: predecessor.clone(),
        successor,
        replacements: replacements.into_boxed_slice(),
    })
}

impl UiPreparedThemeBindingAdmission {
    pub(crate) fn materialize(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> UiThemeCapabilityReceipt {
        UiThemeCapabilityReceipt {
            definition: self.definition.clone(),
            definition_revision: self.definition_revision,
            slot_catalog_revision: self.slot_catalog_revision,
            required_roles: self.required_roles.clone(),
            surface,
            application: self.application.clone(),
            host_profile: self.host_profile.clone(),
        }
    }

    pub(crate) fn application(
        &self,
    ) -> &crate::runtime::WorthUiActiveApplicationGenerationIdentity {
        &self.application
    }

    pub(crate) fn for_successor_application(
        &self,
        application: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    ) -> Self {
        Self {
            definition: self.definition.clone(),
            definition_revision: self.definition_revision,
            slot_catalog_revision: self.slot_catalog_revision,
            required_roles: self.required_roles.clone(),
            application,
            host_profile: self.host_profile.clone(),
        }
    }
}

impl UiThemeCapabilityReceipt {
    pub(crate) fn for_successor_application(
        &self,
        application: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    ) -> Self {
        Self {
            definition: self.definition.clone(),
            definition_revision: self.definition_revision,
            slot_catalog_revision: self.slot_catalog_revision,
            required_roles: self.required_roles.clone(),
            surface: self.surface,
            application,
            host_profile: self.host_profile.clone(),
        }
    }

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
                [
                    worth_ui_host_contract::UiHostAppearanceMechanicFamily::SurfaceFill,
                    worth_ui_host_contract::UiHostAppearanceMechanicFamily::SurfaceBorder,
                    worth_ui_host_contract::UiHostAppearanceMechanicFamily::CornerRadii,
                    worth_ui_host_contract::UiHostAppearanceMechanicFamily::Outline,
                    worth_ui_host_contract::UiHostAppearanceMechanicFamily::TextRangeForeground,
                    worth_ui_host_contract::UiHostAppearanceMechanicFamily::PortalSurface,
                    worth_ui_host_contract::UiHostAppearanceMechanicFamily::Backdrop,
                    worth_ui_host_contract::UiHostAppearanceMechanicFamily::OverlayOrder,
                    worth_ui_host_contract::UiHostAppearanceMechanicFamily::PointerAffordance,
                    worth_ui_host_contract::UiHostAppearanceMechanicFamily::Damage,
                    worth_ui_host_contract::UiHostAppearanceMechanicFamily::Clip,
                ],
                Some(worth_ui_host_contract::UiHostPrimaryPointerKind::Mouse),
                worth_ui_host_contract::UiHostAppearanceGeometryQualification::admit([
                    worth_ui_host_contract::UiHostAppearanceScaleGeometryQualification::new(
                        1_000,
                        1,
                        worth_ui_host_contract::UiAppearanceLogicalLength::new(1_000).unwrap(),
                        worth_ui_host_contract::UiHostAppearanceGeometryQualificationBasis::AnalyticSignedDistancePixelCenter,
                    ),
                ])
                .expect("the test geometry qualification must admit"),
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

impl UiPreparedThemeGenerationRebinding {
    pub(crate) fn predecessor(
        &self,
    ) -> &crate::runtime::WorthUiActiveApplicationGenerationIdentity {
        &self.predecessor
    }

    pub(crate) fn successor(&self) -> &crate::runtime::WorthUiActiveApplicationGenerationIdentity {
        &self.successor
    }

    pub(crate) fn replacements(&self) -> &[UiPreparedThemeBindingReplacement] {
        &self.replacements
    }
}

impl UiPreparedThemeBindingReplacement {
    pub(crate) fn surface(&self) -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
        self.surface
    }

    pub(crate) fn predecessor_generation(&self) -> u64 {
        self.predecessor_generation
    }

    pub(crate) fn capability(&self) -> &UiThemeCapabilityReceipt {
        &self.capability
    }
}
