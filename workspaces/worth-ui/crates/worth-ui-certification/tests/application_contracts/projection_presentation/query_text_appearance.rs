use worth_ui::facade::appearance::{
    FrozenAppearanceThemeCapabilities, UiAppearanceAspect, UiAppearanceCell,
    UiAppearancePartitionAuthoring, UiAppearanceRole, UiAppearanceRoleAttachmentDeclaration,
    UiAppearanceRoleDeclaration, UiAppearanceRoleIdentity, UiThemeColor, UiThemeDefinition,
    UiThemeDefinitionIdentity, UiThemeSlotCatalog, UiThemeSlotDeclaration, UiThemeSlotDisclosure,
    UiThemeSlotIdentity, UiThemeSlotSuccessorCompatibility, UiThemeValue, UiThemeValueKind,
};
use worth_ui::facade::declaration::{ThemeTokenFamily, ThemeTokenId, ThemeTokenSource};
use worth_ui_dsl::WorthUiRustAuthoredArtifactInputModule;

pub(crate) const GREEN_RGBA: [u8; 4] = [32, 192, 96, 255];

const WHITE: &str = "mixed.query.text.white";
const GREEN: &str = "mixed.query.text.green";

pub(crate) fn attach(
    module: WorthUiRustAuthoredArtifactInputModule,
    component: &str,
    green: bool,
) -> WorthUiRustAuthoredArtifactInputModule {
    let selected = role(if green { GREEN } else { WHITE });
    module
        .with_appearance_role(role(WHITE))
        .with_appearance_role(role(GREEN))
        .with_component_appearance_role(
            component,
            UiAppearanceRoleAttachmentDeclaration::new(
                selected.role().clone(),
                selected.revision(),
            ),
        )
        .unwrap()
}

pub(crate) fn white_role() -> UiAppearanceRoleDeclaration {
    role(WHITE)
}

pub(crate) fn green_role() -> UiAppearanceRoleDeclaration {
    role(GREEN)
}

fn role(identity: &str) -> UiAppearanceRoleDeclaration {
    UiAppearanceRole::authoring(UiAppearanceRoleIdentity::new(identity).unwrap())
        .cover(
            UiAppearanceAspect::Foreground,
            UiAppearancePartitionAuthoring::new([]).with_cell(
                UiAppearanceCell::when([]).uses_slot(
                    UiThemeSlotIdentity::new(identity).unwrap(),
                    UiThemeValueKind::Color,
                ),
            ),
        )
        .unwrap()
        .build()
        .unwrap()
}

pub(crate) fn theme() -> FrozenAppearanceThemeCapabilities {
    let slots = [(WHITE, [255, 255, 255, 255]), (GREEN, GREEN_RGBA)];
    let catalog = UiThemeSlotCatalog::admit(
        1,
        slots.map(|(identity, _)| {
            UiThemeSlotDeclaration::new(
                ThemeTokenId::new(identity).unwrap(),
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
    let identity = UiThemeDefinitionIdentity::new("theme.mixed.query").unwrap();
    let definition = UiThemeDefinition::admit(
        identity.clone(),
        1,
        &catalog,
        slots.map(|(slot, rgba)| {
            (
                ThemeTokenId::new(slot).unwrap(),
                UiThemeValue::Color(UiThemeColor::from_channels(rgba)),
            )
        }),
    )
    .unwrap();
    FrozenAppearanceThemeCapabilities::admit(catalog, identity, vec![definition]).unwrap()
}
