use super::fixed_application_builder::FixedCertificationApplicationBuilder;
use super::fixed_host::FixedCertificationHostBinding;
use worth_ui::facade::app::WorthUi;
use worth_ui::facade::appearance::{
    FrozenAppearanceThemeCapabilities, UiAppearanceAspect, UiAppearanceAspectContract,
    UiAppearanceCell, UiAppearancePartitionAuthoring, UiAppearanceRole, UiAppearanceRoleIdentity,
    UiDslComponentReference, UiThemeColor, UiThemeDefinition, UiThemeDefinitionIdentity,
    UiThemeSlotCatalog, UiThemeSlotDeclaration, UiThemeSlotDisclosure, UiThemeSlotIdentity,
    UiThemeSlotSuccessorCompatibility, UiThemeValue, UiThemeValueKind,
};
use worth_ui::facade::declaration::{
    ComponentAllocationMeasurementContract, ComponentChildPolicy, ComponentDescriptor, ComponentId,
    ComponentPropSchema, ComponentStateOwnership, ComponentViewportInset, SurfaceDescriptor,
    SurfaceId, SurfaceKind, SurfacePlacementClass, SurfaceStateClass, ThemeTokenAlias,
    ThemeTokenDescriptor, ThemeTokenFamily, ThemeTokenId, ThemeTokenSource, ThemeTokenValue,
};

type WorthUiApplicationBuilder = FixedCertificationApplicationBuilder;

pub(crate) const PLATFORM_PULSE_BACKGROUND_COMPONENT: &str = "platform.pulse.component.seed";
pub(crate) const PLATFORM_PULSE_IDENTITY_TARGET_COMPONENT: &str =
    "platform.pulse.component.identity_target";
pub(crate) const PLATFORM_PULSE_SURFACE: &str = "platform.pulse.surface.main";
pub(crate) const PLATFORM_PULSE_FILL_TOKEN: &str = "theme.platform_pulse.fill";
pub(crate) const PLATFORM_PULSE_IDENTITY_TARGET_FILL_TOKEN: &str =
    "theme.platform_pulse.identity_target_fill";
pub(crate) const PLATFORM_PULSE_BLUE_TOKEN: &str = "theme.platform_pulse.blue";
pub(crate) const PLATFORM_PULSE_GREEN_TOKEN: &str = "theme.platform_pulse.green";
pub(crate) const PLATFORM_PULSE_YELLOW_TOKEN: &str = "theme.platform_pulse.yellow";
pub(crate) const PLATFORM_PULSE_TARGET_HORIZONTAL_INSET: u16 = 48;
pub(crate) const PLATFORM_PULSE_TARGET_VERTICAL_INSET: u16 = 24;

pub(crate) fn platform_pulse_application_builder_with_host<Host>(
    host: Host,
) -> WorthUiApplicationBuilder
where
    Host: FixedCertificationHostBinding,
{
    platform_pulse_application_builder_with_host_and_unrelated_width(host, 0)
}

pub(crate) fn platform_pulse_application_builder_with_host_and_unrelated_width<Host>(
    host: Host,
    unrelated_width: usize,
) -> WorthUiApplicationBuilder
where
    Host: FixedCertificationHostBinding,
{
    let mut builder = WorthUi::app()
        .with_change_profile(worth_ui::facade::rebind::UiChangeProfile::platform_pulse())
        .register_component(
            appearance_component(PLATFORM_PULSE_BACKGROUND_COMPONENT, 0)
                .with_allocation_measurement_contract(
                    ComponentAllocationMeasurementContract::fill_viewport(),
                ),
        )
        .register_component(
            appearance_component(PLATFORM_PULSE_IDENTITY_TARGET_COMPONENT, 1)
                .with_allocation_measurement_contract(
                    ComponentAllocationMeasurementContract::viewport_inset(
                        ComponentViewportInset::symmetric(
                            PLATFORM_PULSE_TARGET_HORIZONTAL_INSET,
                            PLATFORM_PULSE_TARGET_VERTICAL_INSET,
                        ),
                    ),
                ),
        )
        .register_surface(SurfaceDescriptor::new(
            SurfaceId::new(PLATFORM_PULSE_SURFACE).expect("valid platform pulse surface id"),
            SurfaceKind::primary_content(),
            ComponentId::new(PLATFORM_PULSE_BACKGROUND_COMPONENT)
                .expect("valid platform pulse background component id"),
            SurfacePlacementClass::primary_region(),
            SurfaceStateClass::ephemeral(),
        ))
        .register_theme_token(color_token(PLATFORM_PULSE_YELLOW_TOKEN, "#f2cc60"))
        .register_theme_token(color_token(PLATFORM_PULSE_BLUE_TOKEN, "#2f81f7"))
        .register_theme_token(color_token(PLATFORM_PULSE_GREEN_TOKEN, "#3fb950"))
        .register_theme_token(ThemeTokenDescriptor::alias(
            token_id(PLATFORM_PULSE_IDENTITY_TARGET_FILL_TOKEN),
            ThemeTokenFamily::surface(),
            ThemeTokenSource::application(),
            ThemeTokenAlias::to(token_id(PLATFORM_PULSE_YELLOW_TOKEN)),
        ))
        .register_theme_token(ThemeTokenDescriptor::alias(
            token_id(PLATFORM_PULSE_FILL_TOKEN),
            ThemeTokenFamily::surface(),
            ThemeTokenSource::application(),
            ThemeTokenAlias::to(token_id(PLATFORM_PULSE_BLUE_TOKEN)),
        ));
    let background_role = appearance_role(
        "cert.platform_pulse.background",
        PLATFORM_PULSE_BACKGROUND_COMPONENT,
        PLATFORM_PULSE_FILL_TOKEN,
    );
    let target_role = appearance_role(
        "cert.platform_pulse.identity_target",
        PLATFORM_PULSE_IDENTITY_TARGET_COMPONENT,
        PLATFORM_PULSE_IDENTITY_TARGET_FILL_TOKEN,
    );
    builder = builder
        .register_appearance_role(background_role.clone())
        .expect("certification background appearance role is valid")
        .register_appearance_role(target_role.clone())
        .expect("certification target appearance role is valid")
        .register_appearance_theme_bundle(appearance_theme())
        .expect("certification appearance theme is valid")
        .with_rust_authored_input(appearance_input(background_role, target_role));
    for index in 0..unrelated_width {
        builder = builder.register_component(component(&unrelated_component_id(index)));
    }
    FixedCertificationApplicationBuilder::new(builder, host)
}

fn appearance_component(id: &str, order: u32) -> ComponentDescriptor {
    component(id)
        .with_surface_paint_order(order)
        .with_appearance_aspect_contract(
            UiAppearanceAspectContract::component([UiAppearanceAspect::Background], []).unwrap(),
        )
        .unwrap()
}

fn appearance_role(
    identity: &str,
    component: &str,
    slot: &str,
) -> worth_ui::facade::appearance::UiAppearanceRoleDeclaration {
    UiAppearanceRole::authoring(UiAppearanceRoleIdentity::new(identity).unwrap())
        .applies_to_component(UiDslComponentReference::new(component).unwrap())
        .cover(
            UiAppearanceAspect::Background,
            UiAppearancePartitionAuthoring::new([]).with_cell(
                UiAppearanceCell::when([]).uses_slot(
                    UiThemeSlotIdentity::new(slot).unwrap(),
                    UiThemeValueKind::Color,
                ),
            ),
        )
        .unwrap()
        .build()
        .unwrap()
}

fn appearance_theme() -> FrozenAppearanceThemeCapabilities {
    let slots = [
        (PLATFORM_PULSE_FILL_TOKEN, [47, 129, 247, 255]),
        (
            PLATFORM_PULSE_IDENTITY_TARGET_FILL_TOKEN,
            [242, 204, 96, 255],
        ),
    ];
    let catalog = UiThemeSlotCatalog::admit(
        1,
        slots.map(|(identity, _)| {
            UiThemeSlotDeclaration::new(
                token_id(identity),
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
    let identity = UiThemeDefinitionIdentity::new("cert.platform_pulse.default").unwrap();
    let definition = UiThemeDefinition::admit(
        identity.clone(),
        1,
        &catalog,
        slots.map(|(slot, channels)| {
            (
                token_id(slot),
                UiThemeValue::Color(UiThemeColor::from_channels(channels)),
            )
        }),
    )
    .unwrap();
    FrozenAppearanceThemeCapabilities::admit(catalog, identity, vec![definition]).unwrap()
}

fn appearance_input(
    background: worth_ui::facade::appearance::UiAppearanceRoleDeclaration,
    target: worth_ui::facade::appearance::UiAppearanceRoleDeclaration,
) -> worth_ui::facade::declaration::WorthUiRustAuthoredArtifactInput {
    let background_attachment =
        worth_ui::facade::appearance::UiAppearanceRoleAttachmentDeclaration::new(
            background.role().clone(),
            background.revision(),
        );
    let target_attachment =
        worth_ui::facade::appearance::UiAppearanceRoleAttachmentDeclaration::new(
            target.role().clone(),
            target.revision(),
        );
    let module = worth_ui::facade::declaration::WorthUiRustAuthoredArtifactInputModule::new(
        "cert/platform-pulse-appearance.wui",
    )
    .with_appearance_role(background)
    .with_appearance_role(target)
    .with_component_authored_identity(PLATFORM_PULSE_BACKGROUND_COMPONENT, "background")
    .with_component_authored_identity(PLATFORM_PULSE_IDENTITY_TARGET_COMPONENT, "target")
    .with_component_appearance_role(PLATFORM_PULSE_BACKGROUND_COMPONENT, background_attachment)
    .unwrap()
    .with_component_appearance_role(PLATFORM_PULSE_IDENTITY_TARGET_COMPONENT, target_attachment)
    .unwrap();
    worth_ui::facade::declaration::WorthUiRustAuthoredArtifactInput::from_modules([module])
}

fn component(id: &str) -> ComponentDescriptor {
    ComponentDescriptor::new(
        ComponentId::new(id).expect("valid platform pulse component id"),
        ComponentPropSchema::named(format!("{id}.props")),
        ComponentChildPolicy::no_children(),
        ComponentStateOwnership::runtime_owned(),
    )
}

fn color_token(id: &str, color: &str) -> ThemeTokenDescriptor {
    ThemeTokenDescriptor::define(
        token_id(id),
        ThemeTokenFamily::surface(),
        ThemeTokenSource::application(),
        ThemeTokenValue::color(
            UiThemeColor::parse(color).expect("valid platform pulse theme color"),
        ),
    )
}

fn token_id(id: &str) -> ThemeTokenId {
    ThemeTokenId::new(id).expect("valid platform pulse theme token id")
}

pub(crate) fn unrelated_component_id(index: usize) -> String {
    format!("platform.pulse.component.unrelated_{index:04}")
}
