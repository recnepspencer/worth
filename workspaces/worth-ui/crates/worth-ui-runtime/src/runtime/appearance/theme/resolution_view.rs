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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct UiThemeResolutionWorkEvidence {
    theme_slots_compared: u32,
}

impl UiThemeResolutionWorkEvidence {
    pub(crate) const fn theme_slots_compared(self) -> u32 {
        self.theme_slots_compared
    }

    fn record_catalog_entry(&mut self) {
        self.theme_slots_compared = self
            .theme_slots_compared
            .checked_add(1)
            .expect("theme catalog traversal count fits its bounded catalog");
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiThemeResolutionFailure {
    denial: UiThemeResolutionDenial,
    work: UiThemeResolutionWorkEvidence,
}

impl UiThemeResolutionFailure {
    fn new(denial: UiThemeResolutionDenial, work: UiThemeResolutionWorkEvidence) -> Self {
        Self { denial, work }
    }

    pub(crate) const fn denial(self) -> UiThemeResolutionDenial {
        self.denial
    }

    pub(crate) const fn work(self) -> UiThemeResolutionWorkEvidence {
        self.work
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiResolvedThemeSlot {
    requested: UiThemeSlotIdentity,
    terminal: UiThemeSlotIdentity,
    value: UiThemeValue,
    aliases_compared: u8,
    work: UiThemeResolutionWorkEvidence,
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
    ) -> Result<UiResolvedThemeSlot, UiThemeResolutionFailure> {
        let mut work = UiThemeResolutionWorkEvidence::default();
        let requested_id =
            crate::capability::ThemeTokenId::new(requested.as_str()).map_err(|_| {
                UiThemeResolutionFailure::new(UiThemeResolutionDenial::InvalidSlotIdentity, work)
            })?;
        let Some(mut declaration) = self.catalog.get(&requested_id) else {
            return Err(UiThemeResolutionFailure::new(
                UiThemeResolutionDenial::MissingSlot,
                work,
            ));
        };
        work.record_catalog_entry();
        if declaration.kind() != expected_kind {
            return Err(UiThemeResolutionFailure::new(
                UiThemeResolutionDenial::ValueKindMismatch,
                work,
            ));
        }
        let mut current_id = requested_id.clone();
        let mut aliases_compared = 0_u8;
        loop {
            let Some(target) = declaration.alias_target().cloned() else {
                break;
            };
            aliases_compared = aliases_compared.checked_add(1).ok_or_else(|| {
                UiThemeResolutionFailure::new(UiThemeResolutionDenial::MissingAliasTarget, work)
            })?;
            current_id = target;
            let Some(next) = self.catalog.get(&current_id) else {
                return Err(UiThemeResolutionFailure::new(
                    UiThemeResolutionDenial::MissingAliasTarget,
                    work,
                ));
            };
            work.record_catalog_entry();
            declaration = next;
        }
        let terminal = UiThemeSlotIdentity::new(current_id.as_str()).ok_or_else(|| {
            UiThemeResolutionFailure::new(UiThemeResolutionDenial::InvalidSlotIdentity, work)
        })?;
        let value = self
            .typed_values
            .as_ref()
            .and_then(|values| values.get(&current_id).copied())
            .or_else(|| self.definition.value(&current_id))
            .ok_or_else(|| {
                UiThemeResolutionFailure::new(UiThemeResolutionDenial::MissingValue, work)
            })?;
        if value.kind() != expected_kind {
            return Err(UiThemeResolutionFailure::new(
                UiThemeResolutionDenial::ValueKindMismatch,
                work,
            ));
        }
        Ok(UiResolvedThemeSlot {
            requested: requested.clone(),
            terminal,
            value,
            aliases_compared,
            work,
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

    pub(crate) const fn work(&self) -> UiThemeResolutionWorkEvidence {
        self.work
    }
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
