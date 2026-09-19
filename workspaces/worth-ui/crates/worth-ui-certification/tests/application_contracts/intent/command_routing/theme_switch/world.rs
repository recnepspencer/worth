use super::*;
use worth_ui::facade::{app::WorthUi, declaration::*, intent::*};
use worth_ui_certification::scenario::application_authority_closure::FixedCertificationApplicationBuilder;
use worth_ui_dsl::{
    WorthUiIntentInteractionFamily, WorthUiIntentInteractionRoute,
    WorthUiRustAuthoredArtifactInput, WorthUiRustAuthoredArtifactInputModule,
};

pub(super) fn application(
    host: worth_ui_runtime::certification_support::ScriptedPresentationHost,
) -> worth_ui::facade::app::WorthUiApp {
    let slot = "theme.command.background";
    let role =
        UiAppearanceRole::authoring(UiAppearanceRoleIdentity::new("command.surface").unwrap())
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
            .unwrap();
    let allocation = ComponentAllocationMeasurementContract::fill_viewport();
    let component = ComponentDescriptor::new(
        ComponentId::new("command.control").unwrap(),
        ComponentPropSchema::named("command.control.props"),
        ComponentChildPolicy::no_children(),
        ComponentStateOwnership::runtime_owned(),
    )
    .with_allocation_measurement_contract(allocation)
    .with_surface_paint_order(1)
    .with_hit_test(ComponentHitTestContract::allocation_bounds(
        ComponentHitTestOrder::front_to_back(0),
        allocation,
    ))
    .with_appearance_aspect_contract(role.aspect_contract().clone())
    .unwrap();
    let fact = crate::filesystem_mounted_world::intent_world_operability_fact();
    let declaration = UiIntentDeclaration::<AdvanceStatus>::activate("command.action")
        .unwrap()
        .operability_from(
            UiIntentOperabilityContract::new(
                "command.operability",
                UiIntentMutabilitySource::application_fact(&fact),
                UiIntentReadinessSource::application_fact(&fact),
                UiIntentPolicySource::application_fact(&fact),
            )
            .unwrap(),
        )
        .confirmation(UiIntentConfirmationContract::not_required("command.confirmation").unwrap())
        .concurrency(UiIntentConcurrencyScope::TargetRouteSingleFlight)
        .consequences(UiIntentConsequenceContract::none());
    let module = WorthUiRustAuthoredArtifactInputModule::new("app/main.wui")
        .with_control_routes(
            "command.control",
            [WorthUiIntentInteractionRoute::product(
                WorthUiIntentInteractionFamily::Activate,
                "command.action",
            )],
        )
        .with_surface("command.surface")
        .with_appearance_role(role.clone())
        .with_component_appearance_role(
            "command.control",
            UiAppearanceRoleAttachmentDeclaration::new(role.role().clone(), role.revision()),
        )
        .unwrap()
        .with_intent_declaration(declaration.into_dsl_spec());
    let catalog = UiThemeSlotCatalog::admit(
        1,
        [UiThemeSlotDeclaration::new(
            ThemeTokenId::new(slot).unwrap(),
            ThemeTokenFamily::surface(),
            UiThemeValueKind::Color,
            ThemeTokenSource::application(),
            UiThemeSlotDisclosure::Public,
            UiThemeSlotSuccessorCompatibility::ExactMeaning,
            None,
        )],
    )
    .unwrap();
    let definitions = [
        ("theme.command.blue", [24, 48, 160, 255]),
        ("theme.command.green", [16, 144, 48, 255]),
    ]
    .map(|(identity, rgba)| {
        UiThemeDefinition::admit(
            UiThemeDefinitionIdentity::new(identity).unwrap(),
            1,
            &catalog,
            [(
                ThemeTokenId::new(slot).unwrap(),
                UiThemeValue::Color(UiThemeColor::from_channels(rgba)),
            )],
        )
        .unwrap()
    });
    let bundle = FrozenAppearanceThemeCapabilities::admit(
        catalog,
        UiThemeDefinitionIdentity::new("theme.command.blue").unwrap(),
        definitions.to_vec(),
    )
    .unwrap();
    FixedCertificationApplicationBuilder::new(
        WorthUi::app()
            .with_change_profile(worth_ui::facade::rebind::UiChangeProfile::platform_pulse()),
        host,
    )
    .register_component(component)
    .register_surface(SurfaceDescriptor::new(
        SurfaceId::new("command.surface").unwrap(),
        SurfaceKind::primary_content(),
        ComponentId::new("command.control").unwrap(),
        SurfacePlacementClass::primary_region(),
        SurfaceStateClass::ephemeral(),
    ))
    .register_appearance_role(role)
    .unwrap()
    .register_appearance_theme_bundle(bundle)
    .unwrap()
    .with_command_routing_policy_defaults(
        worth_ui::facade::service::UiCommandRoutingPolicy::desktop(),
    )
    .register_intent_boolean_fact(fact, true)
    .unwrap()
    .register_runtime_service_intent_definition(
        UiIntentDefinition::<AdvanceStatus>::runtime_service(
            UiIntentRuntimeServiceDestination::InvokeCommand,
        ),
    )
    .unwrap()
    .register_command(command_descriptor())
    .with_rust_authored_input(WorthUiRustAuthoredArtifactInput::from_modules([module]))
    .freeze()
    .unwrap()
}
