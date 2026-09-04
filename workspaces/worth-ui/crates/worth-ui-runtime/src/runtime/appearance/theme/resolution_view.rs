use std::collections::BTreeMap;
use std::sync::Arc;

use worth_ui_dsl::{UiThemeSlotIdentity, UiThemeValue, UiThemeValueKind};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiThemeResolutionView {
    definition: crate::capability::UiThemeDefinition,
    catalog: crate::capability::UiThemeSlotCatalog,
    capability: super::UiThemeCapabilityReceipt,
    typed_values: Option<Arc<BTreeMap<crate::capability::ThemeTokenId, UiThemeValue>>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiThemeResolutionDenial {
    MissingDefinition,
    DefinitionRevisionMismatch,
    CatalogRevisionMismatch,
    InvalidSlotIdentity,
    MissingSlot,
    MissingAliasTarget,
    MissingValue,
    ValueKindMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiResolvedThemeSlot {
    requested: UiThemeSlotIdentity,
    terminal: UiThemeSlotIdentity,
    value: UiThemeValue,
    aliases_compared: u8,
}

impl UiThemeResolutionView {
    pub(crate) fn from_capability(
        capability: &super::UiThemeCapabilityReceipt,
        themes: &crate::capability::FrozenAppearanceThemeCapabilities,
    ) -> Result<Self, UiThemeResolutionDenial> {
        let definition = themes
            .get(capability.definition())
            .ok_or(UiThemeResolutionDenial::MissingDefinition)?;
        if definition.revision() != capability.definition_revision() {
            return Err(UiThemeResolutionDenial::DefinitionRevisionMismatch);
        }
        if themes.catalog().revision() != capability.slot_catalog_revision()
            || definition.catalog_revision() != capability.slot_catalog_revision()
        {
            return Err(UiThemeResolutionDenial::CatalogRevisionMismatch);
        }
        Ok(Self {
            definition: definition.clone(),
            catalog: themes.catalog().clone(),
            capability: capability.clone(),
            typed_values: None,
        })
    }

    pub(crate) fn with_typed_values(
        mut self,
        values: Arc<BTreeMap<crate::capability::ThemeTokenId, UiThemeValue>>,
    ) -> Self {
        self.typed_values = Some(values);
        self
    }

    pub(crate) fn resolve(
        &self,
        requested: &UiThemeSlotIdentity,
        expected_kind: UiThemeValueKind,
    ) -> Result<UiResolvedThemeSlot, UiThemeResolutionDenial> {
        let requested_id = crate::capability::ThemeTokenId::new(requested.as_str())
            .map_err(|_| UiThemeResolutionDenial::InvalidSlotIdentity)?;
        let declaration = self
            .catalog
            .get(&requested_id)
            .ok_or(UiThemeResolutionDenial::MissingSlot)?;
        if declaration.kind() != expected_kind {
            return Err(UiThemeResolutionDenial::ValueKindMismatch);
        }
        let terminal_id = self
            .catalog
            .resolved_target(&requested_id)
            .ok_or(UiThemeResolutionDenial::MissingAliasTarget)?;
        let terminal = UiThemeSlotIdentity::new(terminal_id.as_str())
            .ok_or(UiThemeResolutionDenial::InvalidSlotIdentity)?;
        let value = self
            .typed_values
            .as_ref()
            .and_then(|values| values.get(terminal_id).copied())
            .or_else(|| self.definition.value(terminal_id))
            .ok_or(UiThemeResolutionDenial::MissingValue)?;
        if value.kind() != expected_kind {
            return Err(UiThemeResolutionDenial::ValueKindMismatch);
        }
        let aliases_compared = alias_depth(&self.catalog, &requested_id)?;
        Ok(UiResolvedThemeSlot {
            requested: requested.clone(),
            terminal,
            value,
            aliases_compared,
        })
    }

    pub(crate) fn definition_identity(&self) -> &str {
        self.definition.identity().as_str()
    }

    pub(crate) const fn definition_revision(&self) -> u64 {
        self.definition.revision()
    }

    pub(crate) const fn catalog_revision(&self) -> u64 {
        self.catalog.revision()
    }

    pub(crate) const fn surface(&self) -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
        self.capability.surface()
    }

    pub(crate) const fn application(
        &self,
    ) -> &crate::runtime::WorthUiActiveApplicationGenerationIdentity {
        self.capability.application()
    }

    pub(crate) fn supports(
        &self,
        mechanic: worth_ui_host_contract::UiHostAppearanceMechanicFamily,
    ) -> bool {
        self.capability
            .host_profile()
            .mechanics()
            .contains(&mechanic)
    }

    pub(crate) fn admits_role(&self, role: &worth_ui_dsl::UiAppearanceRoleDeclaration) -> bool {
        self.capability.required_roles().iter().any(|required| {
            required.identity() == role.role() && required.revision() == role.revision()
        })
    }

    pub(crate) fn host_profile(&self) -> &worth_ui_host_contract::UiHostAppearanceProfileContract {
        self.capability.host_profile()
    }

    pub(crate) fn semantic_digest(&self) -> u64 {
        let mut digest = 0xcbf2_9ce4_8422_2325_u64;
        digest = fold_text(digest, self.definition_identity());
        digest = fold(digest, self.definition_revision());
        digest = fold(digest, self.catalog_revision());
        digest = fold(digest, self.surface().diagnostic_value());
        digest = fold(
            digest,
            self.capability.application().session_identity().as_u64(),
        );
        digest = fold(
            digest,
            self.capability
                .application()
                .prepared_generation()
                .semantic_package_identity()
                .narrowing_fingerprint(),
        );
        digest = fold_text(digest, self.host_profile().identity());
        digest = fold(digest, u64::from(self.host_profile().version()));
        for mechanic in self.host_profile().mechanics() {
            digest = fold(digest, *mechanic as u64 + 1);
        }
        digest = fold(
            digest,
            self.host_profile()
                .primary_pointer()
                .map_or(0, |pointer| pointer as u64 + 1),
        );
        for role in self.capability.required_roles() {
            digest = fold_text(digest, role.identity().as_str());
            digest = fold(digest, role.revision().value());
        }
        digest
    }
}

impl UiResolvedThemeSlot {
    pub(crate) fn requested(&self) -> &UiThemeSlotIdentity {
        &self.requested
    }
    pub(crate) fn terminal(&self) -> &UiThemeSlotIdentity {
        &self.terminal
    }
    pub(crate) const fn value(&self) -> UiThemeValue {
        self.value
    }
    pub(crate) const fn aliases_compared(&self) -> u8 {
        self.aliases_compared
    }
}

fn alias_depth(
    catalog: &crate::capability::UiThemeSlotCatalog,
    requested: &crate::capability::ThemeTokenId,
) -> Result<u8, UiThemeResolutionDenial> {
    let mut current = requested;
    let mut depth = 0_u8;
    while let Some(target) = catalog
        .get(current)
        .ok_or(UiThemeResolutionDenial::MissingSlot)?
        .alias_target()
    {
        depth = depth
            .checked_add(1)
            .ok_or(UiThemeResolutionDenial::MissingAliasTarget)?;
        current = target;
    }
    Ok(depth)
}

fn fold(digest: u64, value: u64) -> u64 {
    digest.wrapping_mul(0x0000_0100_0000_01b3) ^ value
}

fn fold_text(mut digest: u64, value: &str) -> u64 {
    digest = fold(digest, value.len() as u64);
    for byte in value.as_bytes() {
        digest = fold(digest, u64::from(*byte));
    }
    digest
}
