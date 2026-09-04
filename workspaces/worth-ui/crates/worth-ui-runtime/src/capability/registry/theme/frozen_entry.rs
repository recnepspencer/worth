#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FrozenAppearanceThemeCapabilities {
    catalog: super::UiThemeSlotCatalog,
    initial_definition: super::UiThemeDefinitionIdentity,
    definitions: Box<[super::UiThemeDefinition]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum FrozenAppearanceThemeCapabilitiesDenial {
    EmptyDefinitions,
    DuplicateDefinition,
    CatalogBasisMismatch,
    BundleAlreadyInstalled,
    DefinitionCapacityExceeded,
    MissingInitialDefinition,
}

impl FrozenAppearanceThemeCapabilities {
    pub(crate) const DEFINITION_CAPACITY: usize = 32;
    pub(crate) fn admit(
        catalog: super::UiThemeSlotCatalog,
        initial_definition: super::UiThemeDefinitionIdentity,
        mut definitions: Vec<super::UiThemeDefinition>,
    ) -> Result<Self, FrozenAppearanceThemeCapabilitiesDenial> {
        if definitions.is_empty() {
            return Err(FrozenAppearanceThemeCapabilitiesDenial::EmptyDefinitions);
        }
        if definitions.len() > Self::DEFINITION_CAPACITY {
            return Err(FrozenAppearanceThemeCapabilitiesDenial::DefinitionCapacityExceeded);
        }
        if definitions
            .iter()
            .any(|definition| definition.catalog_basis() != &catalog)
        {
            return Err(FrozenAppearanceThemeCapabilitiesDenial::CatalogBasisMismatch);
        }
        definitions.sort_by(|left, right| left.identity().cmp(right.identity()));
        if definitions
            .windows(2)
            .any(|pair| pair[0].identity() == pair[1].identity())
        {
            return Err(FrozenAppearanceThemeCapabilitiesDenial::DuplicateDefinition);
        }
        if definitions
            .binary_search_by(|definition| definition.identity().cmp(&initial_definition))
            .is_err()
        {
            return Err(FrozenAppearanceThemeCapabilitiesDenial::MissingInitialDefinition);
        }
        Ok(Self {
            catalog,
            initial_definition,
            definitions: definitions.into_boxed_slice(),
        })
    }

    pub(crate) const fn catalog(&self) -> &super::UiThemeSlotCatalog {
        &self.catalog
    }
    pub(crate) fn definitions(&self) -> &[super::UiThemeDefinition] {
        &self.definitions
    }
    pub(crate) fn initial_definition_identity(&self) -> &super::UiThemeDefinitionIdentity {
        &self.initial_definition
    }
    pub(crate) fn get(
        &self,
        identity: &super::UiThemeDefinitionIdentity,
    ) -> Option<&super::UiThemeDefinition> {
        self.definitions
            .binary_search_by(|definition| definition.identity().cmp(identity))
            .ok()
            .map(|index| &self.definitions[index])
    }
    pub(crate) fn digest_basis(&self) -> u64 {
        let catalog = self
            .catalog
            .slots()
            .fold(self.catalog.revision(), |mut digest, slot| {
                for byte in slot.identity().as_str().as_bytes() {
                    digest = fold(digest, u64::from(*byte));
                }
                for byte in slot.family().digest_basis().as_bytes() {
                    digest = fold(digest, u64::from(*byte));
                }
                digest = fold(digest, slot.kind() as u64 + 1);
                for byte in slot.source_owner().digest_basis().as_bytes() {
                    digest = fold(digest, u64::from(*byte));
                }
                digest = fold(digest, slot.disclosure() as u64 + 1);
                digest = fold(digest, slot.successor_compatibility() as u64 + 1);
                if let Some(alias) = slot.alias_target() {
                    for byte in alias.as_str().as_bytes() {
                        digest = fold(digest, u64::from(*byte));
                    }
                }
                digest
            });
        let catalog = self
            .initial_definition
            .as_str()
            .bytes()
            .fold(catalog, |digest, byte| fold(digest, u64::from(byte)));
        self.definitions
            .iter()
            .fold(catalog, |mut digest, definition| {
                for byte in definition.identity().as_str().as_bytes() {
                    digest = fold(digest, u64::from(*byte));
                }
                digest ^= definition.revision().rotate_left(17);
                for (slot, value) in definition.values() {
                    for byte in slot.as_str().as_bytes() {
                        digest = fold(digest, u64::from(*byte));
                    }
                    digest = fold_theme_value(digest, *value);
                }
                digest
            })
    }
}

fn fold(mut digest: u64, value: u64) -> u64 {
    digest ^= value;
    digest.wrapping_mul(0x0000_0100_0000_01b3)
}

fn fold_theme_value(digest: u64, value: worth_ui_dsl::UiThemeValue) -> u64 {
    use worth_ui_dsl::UiThemeValue;
    match value {
        UiThemeValue::Color(color) => color
            .channels()
            .into_iter()
            .fold(digest, |d, v| fold(d, u64::from(v))),
        UiThemeValue::Opacity(opacity) => fold(digest, u64::from(opacity.units())),
        UiThemeValue::LogicalLength(length) => fold(digest, length.subpixels() as u64),
        UiThemeValue::CornerRadii(radii) => radii
            .corners()
            .into_iter()
            .fold(digest, |d, v| fold(d, v.subpixels() as u64)),
        UiThemeValue::SolidStroke(stroke) => {
            let color = stroke
                .color()
                .channels()
                .into_iter()
                .fold(digest, |d, v| fold(d, u64::from(v)));
            fold(color, stroke.width().subpixels() as u64)
        }
        UiThemeValue::SolidOutline(outline) => {
            let stroke = outline.stroke();
            let color = stroke
                .color()
                .channels()
                .into_iter()
                .fold(digest, |d, v| fold(d, u64::from(v)));
            fold(
                fold(color, stroke.width().subpixels() as u64),
                outline.offset().subpixels() as u64,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::registry::theme::{
        UiThemeDefinitionIdentity, UiThemeSlotDeclaration, UiThemeSlotDisclosure,
        UiThemeSlotSuccessorCompatibility,
    };

    #[test]
    fn bundle_requires_the_exact_catalog_basis_not_only_its_revision() {
        let first = catalog("surface.first");
        let second = catalog("surface.second");
        let definition = definition("theme.first", &first, [1, 2, 3, 255]);
        assert_eq!(
            FrozenAppearanceThemeCapabilities::admit(
                second,
                UiThemeDefinitionIdentity::new("theme.first").unwrap(),
                vec![definition],
            ),
            Err(FrozenAppearanceThemeCapabilitiesDenial::CatalogBasisMismatch)
        );
    }

    #[test]
    fn registry_rejects_a_second_whole_bundle_with_an_exact_denial() {
        let catalog = catalog("surface.base");
        let first = FrozenAppearanceThemeCapabilities::admit(
            catalog.clone(),
            UiThemeDefinitionIdentity::new("theme.first").unwrap(),
            vec![definition("theme.first", &catalog, [1, 2, 3, 255])],
        )
        .unwrap();
        let second = FrozenAppearanceThemeCapabilities::admit(
            catalog.clone(),
            UiThemeDefinitionIdentity::new("theme.second").unwrap(),
            vec![definition("theme.second", &catalog, [3, 2, 1, 255])],
        )
        .unwrap();
        let mut registry = super::super::ThemeRegistry::default();

        registry.install(first).unwrap();
        assert_eq!(
            registry.install(second),
            Err(FrozenAppearanceThemeCapabilitiesDenial::BundleAlreadyInstalled)
        );
    }

    #[test]
    fn definition_capacity_and_digest_mutation_are_exact() {
        let catalog = catalog("surface.base");
        let definitions = (0..=FrozenAppearanceThemeCapabilities::DEFINITION_CAPACITY)
            .map(|index| definition(&format!("theme.{index}"), &catalog, [1, 2, 3, 255]))
            .collect();
        assert_eq!(
            FrozenAppearanceThemeCapabilities::admit(
                catalog.clone(),
                UiThemeDefinitionIdentity::new("theme.0").unwrap(),
                definitions,
            ),
            Err(FrozenAppearanceThemeCapabilitiesDenial::DefinitionCapacityExceeded)
        );

        let first = FrozenAppearanceThemeCapabilities::admit(
            catalog.clone(),
            UiThemeDefinitionIdentity::new("theme.first").unwrap(),
            vec![definition("theme.first", &catalog, [1, 2, 3, 255])],
        )
        .unwrap();
        let changed = FrozenAppearanceThemeCapabilities::admit(
            catalog.clone(),
            UiThemeDefinitionIdentity::new("theme.first").unwrap(),
            vec![definition("theme.first", &catalog, [3, 2, 1, 255])],
        )
        .unwrap();
        assert_ne!(first.digest_basis(), changed.digest_basis());
    }

    fn catalog(identity: &str) -> super::super::UiThemeSlotCatalog {
        super::super::UiThemeSlotCatalog::admit(
            1,
            [UiThemeSlotDeclaration::new(
                crate::capability::ThemeTokenId::new(identity).unwrap(),
                crate::capability::ThemeTokenFamily::surface(),
                worth_ui_dsl::UiThemeValueKind::Color,
                crate::capability::ThemeTokenSource::application(),
                UiThemeSlotDisclosure::Public,
                UiThemeSlotSuccessorCompatibility::ExactMeaning,
                None,
            )],
        )
        .unwrap()
    }

    fn definition(
        identity: &str,
        catalog: &super::super::UiThemeSlotCatalog,
        color: [u8; 4],
    ) -> super::super::UiThemeDefinition {
        let slot = catalog.slots().next().unwrap().identity().clone();
        super::super::UiThemeDefinition::admit(
            UiThemeDefinitionIdentity::new(identity).unwrap(),
            1,
            catalog,
            [(
                slot,
                worth_ui_dsl::UiThemeValue::Color(worth_ui_dsl::UiThemeColor::from_channels(color)),
            )],
        )
        .unwrap()
    }
}
