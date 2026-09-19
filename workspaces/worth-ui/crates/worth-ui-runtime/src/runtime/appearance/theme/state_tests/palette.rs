use super::*;
pub(super) fn capability(
    name: &str,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    application: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
) -> UiThemeCapabilityReceipt {
    let slot = crate::capability::UiThemeSlotDeclaration::new(
        crate::capability::ThemeTokenId::new("surface.base").unwrap(),
        crate::capability::ThemeTokenFamily::surface(),
        worth_ui_dsl::UiThemeValueKind::Color,
        crate::capability::ThemeTokenSource::application(),
        crate::capability::UiThemeSlotDisclosure::Public,
        crate::capability::UiThemeSlotSuccessorCompatibility::ExactMeaning,
        None,
    );
    let catalog = crate::capability::UiThemeSlotCatalog::admit(1, [slot]).unwrap();
    let definition_identity = crate::capability::UiThemeDefinitionIdentity::new(name).unwrap();
    let definition = crate::capability::UiThemeDefinition::admit(
        definition_identity.clone(),
        1,
        &catalog,
        [(
            crate::capability::ThemeTokenId::new("surface.base").unwrap(),
            worth_ui_dsl::UiThemeValue::Color(worth_ui_dsl::UiThemeColor::from_channels([
                1, 2, 3, 255,
            ])),
        )],
    )
    .unwrap();
    let contract = worth_ui_dsl::UiAppearanceAspectContract::component(
        [worth_ui_dsl::UiAppearanceAspect::Background],
        [],
    )
    .unwrap();
    let role_identity = worth_ui_dsl::UiAppearanceRoleIdentity::new("theme.test-role").unwrap();
    let role = worth_ui_dsl::UiAppearanceRoleDeclaration::admit(
        role_identity.clone(),
        worth_ui_dsl::UiAppearanceRoleRevision::new(1).unwrap(),
        worth_ui_dsl::UiAppearanceRoleApplicability::AnyComponent,
        &contract,
        [(
            worth_ui_dsl::UiAppearanceAspect::Background,
            worth_ui_dsl::UiAppearancePartitionAuthoring::new([])
                .with_cell(worth_ui_dsl::UiAppearanceCell::when([]).uses_slot(
                    worth_ui_dsl::UiThemeSlotIdentity::new("surface.base").unwrap(),
                    worth_ui_dsl::UiThemeValueKind::Color,
                ))
                .compile(worth_ui_dsl::UiAppearanceAspect::Background)
                .unwrap(),
        )],
    )
    .unwrap();
    let bundle = crate::capability::FrozenAppearanceThemeCapabilities::admit(
        catalog,
        definition_identity.clone(),
        vec![definition],
    )
    .unwrap();
    let registered = crate::facade::entry::CapabilityRegistrationBuilder::new()
        .register_appearance_role(role)
        .unwrap()
        .register_appearance_theme_bundle(bundle)
        .unwrap()
        .freeze_with_registration_report()
        .into_accepted_snapshot();
    assert_eq!(
        registered
            .freeze_report()
            .registry_family_width(crate::capability::RegistryFamily::AppearanceTheme),
        Some(1)
    );
    assert!(registered
        .freeze_report()
        .has_complete_registry_family_inventory());
    let host_profile = worth_ui_host_contract::UiHostAppearanceProfileContract::admit(
        "test-host",
        1,
        [
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::SurfaceFill,
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::SurfaceBorder,
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::CornerRadii,
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::Outline,
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::TextRangeForeground,
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::PortalSurface,
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::Backdrop,
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::OverlayOrder,
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::PointerAffordance,
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::Damage,
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::Clip,
        ],
        Some(worth_ui_host_contract::UiHostPrimaryPointerKind::Mouse),
        worth_ui_host_contract::UiHostAppearanceGeometryQualification::admit([
            worth_ui_host_contract::UiHostAppearanceScaleGeometryQualification::new(
                1_000,
                1,
                worth_ui_host_contract::UiAppearanceLogicalLength::new(1_000).unwrap(),
                worth_ui_host_contract::UiHostAppearanceGeometryQualificationBasis::AnalyticSignedDistancePixelCenter,
            ),
        ])
        .expect("the test geometry qualification must admit"),
    )
    .unwrap();
    UiThemeCapabilityAdmission::from_frozen_capabilities(
        registered.appearance_themes().unwrap(),
        &definition_identity,
        registered.appearance_roles(),
        &host_profile,
    )
    .unwrap()
    .issue([role_identity], surface, application)
    .unwrap()
}
