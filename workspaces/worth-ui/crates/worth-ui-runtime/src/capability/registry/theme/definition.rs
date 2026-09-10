use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiThemeDefinition {
    identity: super::UiThemeDefinitionIdentity,
    revision: u64,
    catalog_basis: super::UiThemeSlotCatalog,
    values: BTreeMap<crate::capability::ThemeTokenId, worth_ui_dsl::UiThemeValue>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiThemeDefinitionDenial {
    ZeroRevision,
    MissingSlot(crate::capability::ThemeTokenId),
    UnknownSlot(crate::capability::ThemeTokenId),
    DuplicateSlot(crate::capability::ThemeTokenId),
    AliasOverride(crate::capability::ThemeTokenId),
    ValueKindMismatch(crate::capability::ThemeTokenId),
    ForeignSuccessorIdentity,
    ConflictingRevision,
    NonMonotonicRevision,
    RevisionExhausted,
}

impl UiThemeDefinition {
    pub fn admit(
        identity: super::UiThemeDefinitionIdentity,
        revision: u64,
        catalog: &super::UiThemeSlotCatalog,
        values: impl IntoIterator<Item = (crate::capability::ThemeTokenId, worth_ui_dsl::UiThemeValue)>,
    ) -> Result<Self, UiThemeDefinitionDenial> {
        if revision == 0 {
            return Err(UiThemeDefinitionDenial::ZeroRevision);
        }
        let mut admitted = BTreeMap::new();
        for (slot, value) in values {
            let declaration = catalog
                .get(&slot)
                .ok_or_else(|| UiThemeDefinitionDenial::UnknownSlot(slot.clone()))?;
            if declaration.alias_target().is_some() {
                return Err(UiThemeDefinitionDenial::AliasOverride(slot));
            }
            if declaration.kind() != value.kind() {
                return Err(UiThemeDefinitionDenial::ValueKindMismatch(slot));
            }
            if admitted.insert(slot.clone(), value).is_some() {
                return Err(UiThemeDefinitionDenial::DuplicateSlot(slot));
            }
        }
        for slot in catalog.slots() {
            if slot.alias_target().is_none() && !admitted.contains_key(slot.identity()) {
                return Err(UiThemeDefinitionDenial::MissingSlot(
                    slot.identity().clone(),
                ));
            }
        }
        Ok(Self {
            identity,
            revision,
            catalog_basis: catalog.clone(),
            values: admitted,
        })
    }

    pub fn identity(&self) -> &super::UiThemeDefinitionIdentity {
        &self.identity
    }
    pub const fn revision(&self) -> u64 {
        self.revision
    }
    pub const fn catalog_revision(&self) -> u64 {
        self.catalog_basis.revision()
    }
    pub const fn catalog_basis(&self) -> &super::UiThemeSlotCatalog {
        &self.catalog_basis
    }
    pub fn value(
        &self,
        slot: &crate::capability::ThemeTokenId,
    ) -> Option<worth_ui_dsl::UiThemeValue> {
        self.values.get(slot).copied()
    }
    pub fn values(
        &self,
    ) -> impl ExactSizeIterator<
        Item = (
            &crate::capability::ThemeTokenId,
            &worth_ui_dsl::UiThemeValue,
        ),
    > {
        self.values.iter()
    }

    pub fn admit_successor(&self, successor: Self) -> Result<Self, UiThemeDefinitionDenial> {
        if successor.identity != self.identity {
            return Err(UiThemeDefinitionDenial::ForeignSuccessorIdentity);
        }
        if successor.revision == self.revision {
            return if successor == *self {
                Err(UiThemeDefinitionDenial::NonMonotonicRevision)
            } else {
                Err(UiThemeDefinitionDenial::ConflictingRevision)
            };
        }
        let expected = self
            .revision
            .checked_add(1)
            .ok_or(UiThemeDefinitionDenial::RevisionExhausted)?;
        if successor.revision != expected {
            return Err(UiThemeDefinitionDenial::NonMonotonicRevision);
        }
        Ok(successor)
    }
}

#[cfg(test)]
mod tests {
    use super::super::{
        UiThemeDefinitionIdentity, UiThemeSlotDeclaration, UiThemeSlotDisclosure,
        UiThemeSlotSuccessorCompatibility,
    };
    use super::*;

    fn slot(identity: &str, alias: Option<&str>) -> UiThemeSlotDeclaration {
        UiThemeSlotDeclaration::new(
            crate::capability::ThemeTokenId::new(identity).unwrap(),
            crate::capability::ThemeTokenFamily::surface(),
            worth_ui_dsl::UiThemeValueKind::Color,
            crate::capability::ThemeTokenSource::application(),
            UiThemeSlotDisclosure::Public,
            UiThemeSlotSuccessorCompatibility::ExactMeaning,
            alias.map(|target| crate::capability::ThemeTokenId::new(target).unwrap()),
        )
    }

    fn definition(revision: u64, color: [u8; 4]) -> UiThemeDefinition {
        let catalog = super::super::UiThemeSlotCatalog::admit(
            3,
            [
                slot("surface.base", None),
                slot("surface.alias", Some("surface.base")),
            ],
        )
        .unwrap();
        UiThemeDefinition::admit(
            UiThemeDefinitionIdentity::new("theme.default").unwrap(),
            revision,
            &catalog,
            [(
                crate::capability::ThemeTokenId::new("surface.base").unwrap(),
                worth_ui_dsl::UiThemeValue::Color(worth_ui_dsl::UiThemeColor::from_channels(color)),
            )],
        )
        .unwrap()
    }

    #[test]
    fn definitions_are_complete_typed_and_cannot_override_aliases() {
        let definition = definition(1, [1, 2, 3, 255]);
        assert!(definition
            .value(&crate::capability::ThemeTokenId::new("surface.base").unwrap())
            .is_some());
        let catalog = super::super::UiThemeSlotCatalog::admit(
            3,
            [
                slot("surface.base", None),
                slot("surface.alias", Some("surface.base")),
            ],
        )
        .unwrap();
        assert!(matches!(
            UiThemeDefinition::admit(
                UiThemeDefinitionIdentity::new("theme.default").unwrap(),
                1,
                &catalog,
                [(
                    crate::capability::ThemeTokenId::new("surface.alias").unwrap(),
                    worth_ui_dsl::UiThemeValue::Color(worth_ui_dsl::UiThemeColor::from_channels([
                        9, 9, 9, 255,
                    ])),
                )]
            ),
            Err(UiThemeDefinitionDenial::AliasOverride(_))
        ));
    }

    #[test]
    fn successor_revision_is_monotonic_and_conflict_sensitive() {
        let current = definition(4, [1, 2, 3, 255]);
        assert!(current
            .clone()
            .admit_successor(definition(5, [3, 2, 1, 255]))
            .is_ok());
        assert_eq!(
            current
                .clone()
                .admit_successor(definition(4, [3, 2, 1, 255])),
            Err(UiThemeDefinitionDenial::ConflictingRevision)
        );
        assert_eq!(
            current.admit_successor(definition(6, [3, 2, 1, 255])),
            Err(UiThemeDefinitionDenial::NonMonotonicRevision)
        );
    }
}
