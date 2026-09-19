use worth_ui::facade::appearance::*;
use worth_ui::facade::declaration::{ThemeTokenFamily, ThemeTokenId, ThemeTokenSource};

pub(super) const BASE_THEME: &str = "theme.appearance.scale.base";
pub(super) const CHANGED_THEME: &str = "theme.appearance.scale.changed";
pub(super) const CHANGED_USED: [usize; 5] = [0, 1, 2, 3, 4];
const EQUAL_USED: [usize; 3] = [5, 6, 7];
const CHANGED_UNUSED: [usize; 4] = [508, 509, 510, 511];
const AXES: [UiAppearanceStateAxis; 6] = [
    UiAppearanceStateAxis::Operability,
    UiAppearanceStateAxis::Focus,
    UiAppearanceStateAxis::Validation,
    UiAppearanceStateAxis::Selection,
    UiAppearanceStateAxis::Hover,
    UiAppearanceStateAxis::Pressed,
];

pub(super) fn axis_count() -> usize {
    (0..super::ROLE_COUNT)
        .flat_map(|index| {
            role(index).partitions()[0]
                .1
                .axes()
                .iter()
                .map(|version| version.axis())
                .collect::<Vec<_>>()
        })
        .collect::<std::collections::BTreeSet<_>>()
        .len()
}

pub(super) fn role(index: usize) -> UiAppearanceRoleDeclaration {
    let domains = (index >= super::ROLE_COUNT - AXES.len())
        .then(|| UiAppearanceAxisDomain::complete(AXES[index % AXES.len()]));
    UiAppearanceRole::authoring(
        UiAppearanceRoleIdentity::new(format!("appearance.scale.role_{index:03}")).unwrap(),
    )
    .cover(
        UiAppearanceAspect::Background,
        UiAppearancePartitionAuthoring::new(domains).with_cell(
            UiAppearanceCell::when([]).uses_slot(
                UiThemeSlotIdentity::new(slot(index)).unwrap(),
                UiThemeValueKind::Color,
            ),
        ),
    )
    .unwrap()
    .build()
    .unwrap()
}

pub(super) fn theme_bundle() -> FrozenAppearanceThemeCapabilities {
    let catalog = UiThemeSlotCatalog::admit(
        1,
        (0..super::SLOT_COUNT).map(|index| {
            UiThemeSlotDeclaration::new(
                ThemeTokenId::new(slot(index)).unwrap(),
                ThemeTokenFamily::surface(),
                UiThemeValueKind::Color,
                ThemeTokenSource::application(),
                UiThemeSlotDisclosure::Public,
                UiThemeSlotSuccessorCompatibility::ExactMeaning,
                None,
            )
        }),
    )
    .unwrap();
    let definitions = [(BASE_THEME, false), (CHANGED_THEME, true)].map(|(identity, changed)| {
        UiThemeDefinition::admit(
            UiThemeDefinitionIdentity::new(identity).unwrap(),
            1,
            &catalog,
            (0..super::SLOT_COUNT).map(|index| {
                (
                    ThemeTokenId::new(slot(index)).unwrap(),
                    value(index, changed),
                )
            }),
        )
        .unwrap()
    });
    assert_eq!(
        CHANGED_USED.len() + EQUAL_USED.len() + CHANGED_UNUSED.len(),
        12
    );
    FrozenAppearanceThemeCapabilities::admit(
        catalog,
        UiThemeDefinitionIdentity::new(BASE_THEME).unwrap(),
        definitions.to_vec(),
    )
    .unwrap()
}

pub(super) fn rgba(index: usize, changed: bool) -> [u8; 4] {
    if changed && (CHANGED_USED.contains(&index) || CHANGED_UNUSED.contains(&index)) {
        [192, (index % 128) as u8, 64, 255]
    } else {
        [32, (index % 128) as u8, 160, 255]
    }
}

fn value(index: usize, changed: bool) -> UiThemeValue {
    UiThemeValue::Color(UiThemeColor::from_channels(rgba(index, changed)))
}

fn slot(index: usize) -> String {
    format!("theme.appearance.scale.slot_{index:03}")
}
