use worth_ui::facade::appearance::{
    FrozenAppearanceThemeCapabilities, UiAppearanceAspect, UiAppearanceCell,
    UiAppearancePartitionAuthoring, UiAppearanceRole, UiAppearanceRoleAttachmentDeclaration,
    UiAppearanceRoleDeclaration, UiAppearanceRoleIdentity, UiThemeColor, UiThemeDefinition,
    UiThemeDefinitionIdentity, UiThemeSlotCatalog, UiThemeSlotDeclaration, UiThemeSlotDisclosure,
    UiThemeSlotIdentity, UiThemeSlotSuccessorCompatibility, UiThemeValue, UiThemeValueKind,
};
use worth_ui::facade::declaration::{ThemeTokenFamily, ThemeTokenId, ThemeTokenSource};
use worth_ui_dsl::WorthUiRustAuthoredArtifactInputModule;

const BLUE_ROLE: &str = "host.platform.maximum.surface.blue";
const YELLOW_ROLE: &str = "host.platform.maximum.surface.yellow";
const BLUE_SLOT: &str = "theme.host_platform.surface.blue";
const YELLOW_SLOT: &str = "theme.host_platform.surface.yellow";

pub(in crate::host_platform) fn register(
    builder: worth_ui_certification::scenario::application_authority_closure::FixedCertificationApplicationBuilder,
) -> worth_ui_certification::scenario::application_authority_closure::FixedCertificationApplicationBuilder{
    let [blue, yellow] = roles();
    builder
        .register_appearance_role(blue)
        .expect("blue maximum-overlap appearance role registers")
        .register_appearance_role(yellow)
        .expect("yellow maximum-overlap appearance role registers")
        .register_appearance_theme_bundle(theme_bundle())
        .expect("maximum-overlap appearance theme registers")
}

pub(in crate::host_platform) fn declare_roles(
    module: WorthUiRustAuthoredArtifactInputModule,
) -> WorthUiRustAuthoredArtifactInputModule {
    let [blue, yellow] = roles();
    module
        .with_appearance_role(blue)
        .with_appearance_role(yellow)
}

pub(in crate::host_platform) fn attach(
    module: WorthUiRustAuthoredArtifactInputModule,
    component: &str,
    index: usize,
) -> WorthUiRustAuthoredArtifactInputModule {
    let role = role(index);
    module
        .with_component_appearance_role(
            component,
            UiAppearanceRoleAttachmentDeclaration::new(role.role().clone(), role.revision()),
        )
        .expect("maximum-overlap component appearance attaches")
}

pub(super) fn role(index: usize) -> UiAppearanceRoleDeclaration {
    if index == 1 {
        surface_role(YELLOW_ROLE, YELLOW_SLOT)
    } else {
        surface_role(BLUE_ROLE, BLUE_SLOT)
    }
}

fn roles() -> [UiAppearanceRoleDeclaration; 2] {
    [
        surface_role(BLUE_ROLE, BLUE_SLOT),
        surface_role(YELLOW_ROLE, YELLOW_SLOT),
    ]
}

fn surface_role(identity: &str, slot: &str) -> UiAppearanceRoleDeclaration {
    let slot = UiThemeSlotIdentity::new(slot).expect("host-platform slot identity");
    UiAppearanceRole::authoring(
        UiAppearanceRoleIdentity::new(identity).expect("host-platform role identity"),
    )
    .cover(
        UiAppearanceAspect::Background,
        UiAppearancePartitionAuthoring::new([]).with_cell(
            UiAppearanceCell::when([]).uses_slot(slot, UiAppearanceAspect::Background.value_kind()),
        ),
    )
    .expect("host-platform appearance partition is complete")
    .build()
    .expect("host-platform appearance role is valid")
}

fn theme_bundle() -> FrozenAppearanceThemeCapabilities {
    let slots = [
        (BLUE_SLOT, [47, 129, 247, 255]),
        (YELLOW_SLOT, [242, 204, 96, 255]),
    ];
    let catalog = UiThemeSlotCatalog::admit(
        1,
        slots.map(|(identity, _)| {
            UiThemeSlotDeclaration::new(
                ThemeTokenId::new(identity).expect("host-platform theme slot identity"),
                ThemeTokenFamily::surface(),
                UiThemeValueKind::Color,
                ThemeTokenSource::application(),
                UiThemeSlotDisclosure::Public,
                UiThemeSlotSuccessorCompatibility::ExactMeaning,
                None,
            )
        }),
    )
    .expect("host-platform theme catalog is valid");
    let identity = UiThemeDefinitionIdentity::new("theme.host_platform.maximum")
        .expect("host-platform theme identity");
    let definition = UiThemeDefinition::admit(
        identity.clone(),
        1,
        &catalog,
        slots.map(|(slot, rgba)| {
            (
                ThemeTokenId::new(slot).expect("host-platform theme slot identity"),
                UiThemeValue::Color(UiThemeColor::from_channels(rgba)),
            )
        }),
    )
    .expect("host-platform theme definition is complete");
    FrozenAppearanceThemeCapabilities::admit(catalog, identity, vec![definition])
        .expect("host-platform appearance theme is valid")
}
