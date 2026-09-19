use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiThemeSlotDisclosure {
    Public,
    InspectionOnly,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiThemeSlotSuccessorCompatibility {
    ExactMeaning,
    KindPreserving,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiThemeSlotDeclaration {
    identity: crate::capability::ThemeTokenId,
    family: crate::capability::ThemeTokenFamily,
    kind: worth_ui_dsl::UiThemeValueKind,
    source_owner: crate::capability::ThemeTokenSource,
    disclosure: UiThemeSlotDisclosure,
    successor_compatibility: UiThemeSlotSuccessorCompatibility,
    alias_target: Option<crate::capability::ThemeTokenId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiThemeSlotCatalog {
    revision: u64,
    slots: std::sync::Arc<BTreeMap<crate::capability::ThemeTokenId, UiThemeSlotDeclaration>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiThemeSlotCatalogDenial {
    Empty,
    ZeroRevision,
    UnknownFamily(crate::capability::ThemeTokenId),
    UnsupportedSource(crate::capability::ThemeTokenId),
    CapacityExceeded,
    Duplicate(crate::capability::ThemeTokenId),
    MissingAliasTarget(crate::capability::ThemeTokenId),
    AliasKindMismatch(crate::capability::ThemeTokenId),
    AliasCycle(crate::capability::ThemeTokenId),
    AliasDepthExceeded(crate::capability::ThemeTokenId),
}

impl UiThemeSlotDeclaration {
    pub fn new(
        identity: crate::capability::ThemeTokenId,
        family: crate::capability::ThemeTokenFamily,
        kind: worth_ui_dsl::UiThemeValueKind,
        source_owner: crate::capability::ThemeTokenSource,
        disclosure: UiThemeSlotDisclosure,
        successor_compatibility: UiThemeSlotSuccessorCompatibility,
        alias_target: Option<crate::capability::ThemeTokenId>,
    ) -> Self {
        Self {
            identity,
            family,
            kind,
            source_owner,
            disclosure,
            successor_compatibility,
            alias_target,
        }
    }

    pub fn identity(&self) -> &crate::capability::ThemeTokenId {
        &self.identity
    }
    pub const fn kind(&self) -> worth_ui_dsl::UiThemeValueKind {
        self.kind
    }
    pub fn family(&self) -> &crate::capability::ThemeTokenFamily {
        &self.family
    }
    pub const fn source_owner(&self) -> &crate::capability::ThemeTokenSource {
        &self.source_owner
    }
    pub const fn disclosure(&self) -> UiThemeSlotDisclosure {
        self.disclosure
    }
    pub const fn successor_compatibility(&self) -> UiThemeSlotSuccessorCompatibility {
        self.successor_compatibility
    }
    pub fn alias_target(&self) -> Option<&crate::capability::ThemeTokenId> {
        self.alias_target.as_ref()
    }
}

impl UiThemeSlotCatalog {
    pub const CAPACITY: usize = 4_096;
    pub const MAX_ALIAS_DEPTH: usize = 16;

    pub fn admit(
        revision: u64,
        declarations: impl IntoIterator<Item = UiThemeSlotDeclaration>,
    ) -> Result<Self, UiThemeSlotCatalogDenial> {
        if revision == 0 {
            return Err(UiThemeSlotCatalogDenial::ZeroRevision);
        }
        let mut slots = BTreeMap::new();
        for declaration in declarations {
            let identity = declaration.identity.clone();
            if !declaration.family.is_known() {
                return Err(UiThemeSlotCatalogDenial::UnknownFamily(identity));
            }
            if declaration.source_owner.is_plugin_contribution() {
                return Err(UiThemeSlotCatalogDenial::UnsupportedSource(identity));
            }
            if slots.insert(identity.clone(), declaration).is_some() {
                return Err(UiThemeSlotCatalogDenial::Duplicate(identity));
            }
            if slots.len() > Self::CAPACITY {
                return Err(UiThemeSlotCatalogDenial::CapacityExceeded);
            }
        }
        if slots.is_empty() {
            return Err(UiThemeSlotCatalogDenial::Empty);
        }
        for declaration in slots.values() {
            if let Some(target) = declaration.alias_target() {
                let target_slot = slots
                    .get(target)
                    .ok_or_else(|| UiThemeSlotCatalogDenial::MissingAliasTarget(target.clone()))?;
                if target_slot.kind != declaration.kind {
                    return Err(UiThemeSlotCatalogDenial::AliasKindMismatch(
                        declaration.identity.clone(),
                    ));
                }
                ensure_acyclic(&slots, &declaration.identity)?;
            }
        }
        Ok(Self {
            revision,
            slots: std::sync::Arc::new(slots),
        })
    }

    pub const fn revision(&self) -> u64 {
        self.revision
    }
    pub fn slots(&self) -> impl ExactSizeIterator<Item = &UiThemeSlotDeclaration> {
        self.slots.values()
    }

    pub fn get(
        &self,
        identity: &crate::capability::ThemeTokenId,
    ) -> Option<&UiThemeSlotDeclaration> {
        self.slots.get(identity)
    }

    pub fn resolved_target(
        &self,
        identity: &crate::capability::ThemeTokenId,
    ) -> Option<&crate::capability::ThemeTokenId> {
        let mut current = self.slots.get_key_value(identity)?.0;
        for _ in 0..=Self::MAX_ALIAS_DEPTH {
            let declaration = self.slots.get(current)?;
            match declaration.alias_target() {
                Some(target) => current = self.slots.get_key_value(target)?.0,
                None => return Some(current),
            }
        }
        None
    }
}

fn ensure_acyclic(
    slots: &BTreeMap<crate::capability::ThemeTokenId, UiThemeSlotDeclaration>,
    start: &crate::capability::ThemeTokenId,
) -> Result<(), UiThemeSlotCatalogDenial> {
    let mut seen = BTreeSet::new();
    let mut cursor = start;
    let mut depth = 0_usize;
    while let Some(next) = slots
        .get(cursor)
        .and_then(UiThemeSlotDeclaration::alias_target)
    {
        if !seen.insert(cursor.clone()) || next == start {
            return Err(UiThemeSlotCatalogDenial::AliasCycle(start.clone()));
        }
        depth += 1;
        if depth > UiThemeSlotCatalog::MAX_ALIAS_DEPTH {
            return Err(UiThemeSlotCatalogDenial::AliasDepthExceeded(start.clone()));
        }
        cursor = next;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slot(
        identity: &str,
        kind: worth_ui_dsl::UiThemeValueKind,
        alias_target: Option<&str>,
    ) -> UiThemeSlotDeclaration {
        UiThemeSlotDeclaration::new(
            crate::capability::ThemeTokenId::new(identity).unwrap(),
            crate::capability::ThemeTokenFamily::surface(),
            kind,
            crate::capability::ThemeTokenSource::application(),
            UiThemeSlotDisclosure::Public,
            UiThemeSlotSuccessorCompatibility::KindPreserving,
            alias_target.map(|target| crate::capability::ThemeTokenId::new(target).unwrap()),
        )
    }

    #[test]
    fn catalog_preserves_typed_source_and_disclosure_contracts() {
        let catalog = UiThemeSlotCatalog::admit(
            7,
            [slot(
                "surface.base",
                worth_ui_dsl::UiThemeValueKind::Color,
                None,
            )],
        )
        .unwrap();
        let declaration = catalog.slots().next().unwrap();
        assert_eq!(catalog.revision(), 7);
        assert_eq!(
            declaration.family(),
            &crate::capability::ThemeTokenFamily::surface()
        );
        assert_eq!(
            declaration.source_owner(),
            &crate::capability::ThemeTokenSource::application()
        );
        assert_eq!(declaration.disclosure(), UiThemeSlotDisclosure::Public);
        assert_eq!(
            declaration.successor_compatibility(),
            UiThemeSlotSuccessorCompatibility::KindPreserving
        );
    }

    #[test]
    fn unknown_families_and_plugin_sources_are_denied_before_freeze() {
        let declaration = |family, source| {
            UiThemeSlotDeclaration::new(
                crate::capability::ThemeTokenId::new("surface.base").unwrap(),
                family,
                worth_ui_dsl::UiThemeValueKind::Color,
                source,
                UiThemeSlotDisclosure::Public,
                UiThemeSlotSuccessorCompatibility::ExactMeaning,
                None,
            )
        };
        assert!(matches!(
            UiThemeSlotCatalog::admit(
                1,
                [declaration(
                    crate::capability::ThemeTokenFamily::unknown_for_diagnostics("foreign"),
                    crate::capability::ThemeTokenSource::application(),
                )],
            ),
            Err(UiThemeSlotCatalogDenial::UnknownFamily(_))
        ));
        for source in [
            crate::capability::ThemeTokenSource::plugin_custom(),
            crate::capability::ThemeTokenSource::plugin_alias(),
            crate::capability::ThemeTokenSource::plugin_platform_override_for_diagnostics(),
        ] {
            assert!(matches!(
                UiThemeSlotCatalog::admit(
                    1,
                    [declaration(
                        crate::capability::ThemeTokenFamily::surface(),
                        source
                    )],
                ),
                Err(UiThemeSlotCatalogDenial::UnsupportedSource(_))
            ));
        }
    }

    #[test]
    fn aliases_are_kind_preserving_and_acyclic() {
        let color = worth_ui_dsl::UiThemeValueKind::Color;
        let opacity = worth_ui_dsl::UiThemeValueKind::Opacity;
        assert!(matches!(
            UiThemeSlotCatalog::admit(1, [slot("a", color, Some("b")), slot("b", opacity, None)]),
            Err(UiThemeSlotCatalogDenial::AliasKindMismatch(_))
        ));
        assert!(matches!(
            UiThemeSlotCatalog::admit(
                1,
                [slot("a", color, Some("b")), slot("b", color, Some("a"))]
            ),
            Err(UiThemeSlotCatalogDenial::AliasCycle(_))
        ));
    }

    #[test]
    fn slot_catalog_capacity_is_exact() {
        let admitted = (0..UiThemeSlotCatalog::CAPACITY).map(|index| {
            slot(
                &format!("slot.s{index}"),
                worth_ui_dsl::UiThemeValueKind::Color,
                None,
            )
        });
        assert!(UiThemeSlotCatalog::admit(1, admitted).is_ok());
        let denied = (0..=UiThemeSlotCatalog::CAPACITY).map(|index| {
            slot(
                &format!("slot.s{index}"),
                worth_ui_dsl::UiThemeValueKind::Color,
                None,
            )
        });
        assert_eq!(
            UiThemeSlotCatalog::admit(1, denied),
            Err(UiThemeSlotCatalogDenial::CapacityExceeded)
        );
    }

    #[test]
    fn aliases_admit_sixteen_hops_and_deny_seventeen() {
        let chain = |hops: usize| {
            (0..=hops).map(move |index| {
                let target = (index < hops).then(|| format!("slot.s{}", index + 1));
                slot(
                    &format!("slot.s{index}"),
                    worth_ui_dsl::UiThemeValueKind::Color,
                    target.as_deref(),
                )
            })
        };
        assert!(UiThemeSlotCatalog::admit(1, chain(16)).is_ok());
        assert!(matches!(
            UiThemeSlotCatalog::admit(1, chain(17)),
            Err(UiThemeSlotCatalogDenial::AliasDepthExceeded(_))
        ));
    }
}
