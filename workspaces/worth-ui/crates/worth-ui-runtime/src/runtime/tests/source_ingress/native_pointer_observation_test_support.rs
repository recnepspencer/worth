use super::source_ingress_boundary_test_support::{
    lower_file_submission, source_backed_package_component, source_backed_package_region,
    source_backed_package_sizing,
};

const COMPONENT: &str = "workspace.component.active_session_current";
const CANDIDATE: &str = "workspace.component.active_session_candidate";
const HOVER_CONSUMER: &str = "workspace.pointer.hover.consumer";
const TOKEN: &str = "theme.pointer.hover";

pub(crate) fn source_backed_hover_consumer_app_with_host(
    host: crate::certification_support::ScriptedPresentationHost,
) -> crate::facade::WorthUiApp {
    host.set_capabilities(worth_ui_host_native::appearance_capability_report());
    let role = hover_background_role();
    let snapshot = hover_component_builder(&role)
        .freeze()
        .map(crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host)
        .expect("hover consumer capability snapshot should prepare");
    hover_component_builder(&role)
        .with_candidate_submission(hover_candidate_submission(snapshot.capabilities(), &role))
        .freeze()
        .map(|application| {
            crate::facade::entry::WorthUiCertificationApplicationTransition::activate_test_host(
                application,
                host,
            )
        })
        .expect("hover consumer source application should prepare")
}

fn hover_component_builder(
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
) -> crate::facade::entry::WorthUiApplicationBuilder {
    let (_, _, world_profile) =
        crate::evidence::measurement::projection::fact_test_support::display_field_projection_context(
            "native-pointer-observation",
        );
    crate::facade::WorthUi::app()
        .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
        .with_graph_world_profile(world_profile)
        .register_component(interactive_component(COMPONENT))
        .register_component(interactive_component(CANDIDATE))
        .register_appearance_role(role.clone())
        .unwrap()
        .register_appearance_theme_bundle(hover_theme_bundle())
        .unwrap()
        .register_theme_token(crate::capability::ThemeTokenDescriptor::define(
            crate::capability::ThemeTokenId::new(TOKEN).unwrap(),
            crate::capability::ThemeTokenFamily::surface(),
            crate::capability::ThemeTokenSource::application(),
            crate::capability::ThemeTokenValue::color(
                crate::capability::ThemeColorValue::hex("#224466").unwrap(),
            ),
        ))
        .register_mosaic_region_kind(source_backed_package_region())
        .register_mosaic_sizing_contract(source_backed_package_sizing())
}

fn hover_theme_bundle() -> crate::capability::FrozenAppearanceThemeCapabilities {
    use crate::capability::{
        FrozenAppearanceThemeCapabilities, ThemeTokenFamily, ThemeTokenId, ThemeTokenSource,
        UiThemeDefinition, UiThemeDefinitionIdentity, UiThemeSlotCatalog, UiThemeSlotDeclaration,
        UiThemeSlotDisclosure, UiThemeSlotSuccessorCompatibility,
    };
    let token = ThemeTokenId::new(TOKEN).unwrap();
    let catalog = UiThemeSlotCatalog::admit(
        1,
        [UiThemeSlotDeclaration::new(
            token.clone(),
            ThemeTokenFamily::surface(),
            worth_ui_dsl::UiThemeValueKind::Color,
            ThemeTokenSource::application(),
            UiThemeSlotDisclosure::Public,
            UiThemeSlotSuccessorCompatibility::ExactMeaning,
            None,
        )],
    )
    .unwrap();
    let identity = UiThemeDefinitionIdentity::new("theme.pointer-observation").unwrap();
    let definition = UiThemeDefinition::admit(
        identity.clone(),
        1,
        &catalog,
        [(
            token,
            worth_ui_dsl::UiThemeValue::Color(worth_ui_dsl::UiThemeColor::from_channels([
                34, 68, 102, 255,
            ])),
        )],
    )
    .unwrap();
    FrozenAppearanceThemeCapabilities::admit(catalog, identity, vec![definition]).unwrap()
}

fn interactive_component(identity: &str) -> crate::capability::ComponentDescriptor {
    let allocation = crate::capability::ComponentAllocationMeasurementContract::fill_viewport();
    source_backed_package_component(identity)
        .with_allocation_measurement_contract(allocation)
        .with_hit_test(
            crate::capability::ComponentHitTestContract::allocation_bounds(
                crate::capability::ComponentHitTestOrder::front_to_back(0),
                allocation,
            ),
        )
        .with_appearance_aspect_contract(
            worth_ui_dsl::UiAppearanceAspectContract::component(
                [worth_ui_dsl::UiAppearanceAspect::Background],
                [],
            )
            .unwrap(),
        )
        .unwrap()
}

fn hover_candidate_submission(
    capabilities: &crate::capability::CapabilitySnapshot,
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
) -> crate::runtime::WorthUiWatchedCandidateSubmission {
    let attachment = worth_ui_dsl::UiAppearanceRoleAttachmentDeclaration::new(
        role.role().clone(),
        role.revision(),
    );
    let declaration = worth_ui_dsl::WorthUiSemanticArtifactDeclaration::new(
        worth_ui_dsl::UiDslSemanticKey::new(HOVER_CONSUMER),
        worth_ui_dsl::UiDslSemanticFamily::Control,
    )
    .with_structural_token(worth_ui_dsl::UiDslStructuralToken::new(
        "control:hover-consumer",
    ))
    .with_component_reference(worth_ui_dsl::UiDslComponentReference::new(COMPONENT).unwrap())
    .unwrap()
    .with_appearance_role_attachment(attachment)
    .unwrap();
    let input = worth_ui_dsl::WorthUiRustAuthoredArtifactInput::from_modules([
        worth_ui_dsl::WorthUiRustAuthoredArtifactInputModule::new("app/main.wui")
            .with_component_body_atoms(
                COMPONENT,
                vec![
                    ident("region"),
                    ident("workspace.region.primary"),
                    worth_ui_dsl::WorthUiArtifactInputBodyAtom::LeftBrace,
                    ident("sizing"),
                    ident("workspace.sizing.mosaic_support"),
                    worth_ui_dsl::WorthUiArtifactInputBodyAtom::Semicolon,
                    worth_ui_dsl::WorthUiArtifactInputBodyAtom::RightBrace,
                ],
            )
            .with_semantic_declaration(declaration),
    ]);
    let source_name = "native-pointer-observation-current";
    let provider = crate::runtime::WorthUiSourceProvider::rust_authored(source_name)
        .with_rust_authored_input(input);
    lower_file_submission(
        provider,
        [crate::runtime::WorthUiWatcherEvent::provider_revision(
            source_name,
        )],
        capabilities,
    )
}

fn ident(text: &str) -> worth_ui_dsl::WorthUiArtifactInputBodyAtom {
    worth_ui_dsl::WorthUiArtifactInputBodyAtom::Identifier(text.to_owned())
}

fn hover_background_role() -> worth_ui_dsl::UiAppearanceRoleDeclaration {
    let contract = worth_ui_dsl::UiAppearanceAspectContract::component(
        [worth_ui_dsl::UiAppearanceAspect::Background],
        [],
    )
    .unwrap();
    let partition = worth_ui_dsl::UiAppearancePartitionAuthoring::new([
        worth_ui_dsl::UiAppearanceAxisDomain::complete(worth_ui_dsl::UiAppearanceStateAxis::Hover),
    ])
    .with_cell(
        worth_ui_dsl::UiAppearanceCell::when([worth_ui_dsl::UiAppearanceAxisPredicate::any(
            worth_ui_dsl::UiAppearanceStateAxis::Hover,
        )])
        .uses_slot(
            worth_ui_dsl::UiThemeSlotIdentity::new(TOKEN).unwrap(),
            worth_ui_dsl::UiThemeValueKind::Color,
        ),
    )
    .compile(worth_ui_dsl::UiAppearanceAspect::Background)
    .unwrap();
    worth_ui_dsl::UiAppearanceRoleDeclaration::admit(
        worth_ui_dsl::UiAppearanceRoleIdentity::new("test.hover-background").unwrap(),
        worth_ui_dsl::UiAppearanceRoleRevision::new(1).unwrap(),
        worth_ui_dsl::UiAppearanceRoleApplicability::AnyComponent,
        &contract,
        [(worth_ui_dsl::UiAppearanceAspect::Background, partition)],
    )
    .unwrap()
}
