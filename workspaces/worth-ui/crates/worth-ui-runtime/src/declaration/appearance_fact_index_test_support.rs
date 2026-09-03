use crate::runtime::tests::appearance_component_session_test_support::{
    appearance_theme_token, static_paint_component, validation_background_role,
};

pub(crate) fn unattached_static_paint_app(
    fixture_name: &str,
    component_identity: &str,
    token_identity: &str,
) -> crate::facade::WorthUiApp {
    let token = crate::capability::ThemeTokenId::new(token_identity).unwrap();
    let role = validation_background_role(token_identity);
    let (_, _, world_profile) =
        crate::evidence::measurement::projection::fact_test_support::display_field_projection_context(
            "appearance-fact-index",
        );
    crate::facade::WorthUi::app()
        .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
        .with_graph_world_profile(world_profile)
        .register_component(static_paint_component(component_identity, token.clone()))
        .register_appearance_role(role)
        .unwrap()
        .register_theme_token(appearance_theme_token(token))
        .with_rust_authored_declaration_fixture(unattached_fixture(
            fixture_name,
            component_identity,
        ))
        .freeze()
        .map(crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host)
        .expect("unattached appearance fixture should prepare")
}

pub(crate) fn role_only_fact_index_app(
    fixture_name: &str,
    token_identity: &str,
    attached_component: &str,
    peer_component: &str,
) -> crate::facade::WorthUiApp {
    let role = validation_background_role(token_identity);
    let token = crate::capability::ThemeTokenId::new(token_identity).unwrap();
    crate::facade::WorthUi::app()
        .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
        .register_component(role_only_component(attached_component))
        .register_component(role_only_component(peer_component))
        .register_appearance_role(role.clone())
        .unwrap()
        .register_theme_token(appearance_theme_token(token))
        .with_rust_authored_declaration_fixture(
            crate::facade::WorthUiRustAuthoredDeclarationFixture::named(fixture_name)
                .with_semantic_artifact_spec(role_only_spec(attached_component, Some(&role), 0))
                .with_semantic_artifact_spec(role_only_spec(peer_component, None, 1)),
        )
        .freeze()
        .map(crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host)
        .expect("role-only fact-index fixture should prepare")
}

pub(crate) fn aliased_role_only_fact_index_app(
    fixture_name: &str,
    alias_identity: &str,
    terminal_identity: &str,
    alias_authored_identity: &str,
    terminal_authored_identity: &str,
    attached_component: &str,
    peer_component: &str,
) -> crate::facade::WorthUiApp {
    let role = validation_background_role(alias_identity);
    let alias = crate::capability::ThemeTokenId::new(alias_identity).unwrap();
    let terminal = crate::capability::ThemeTokenId::new(terminal_identity).unwrap();
    let module = worth_ui_dsl::WorthUiRustAuthoredArtifactInputModule::new(format!(
        "app/{fixture_name}.wui"
    ))
    .with_appearance_role(role.clone())
    .with_token_authored_identity(
        terminal_identity,
        terminal_authored_identity,
        terminal_identity,
    )
    .with_token_authored_identity(alias_identity, alias_authored_identity, terminal_identity)
    .with_semantic_declaration(role_only_declaration(attached_component, Some(&role)))
    .with_semantic_declaration(role_only_declaration(peer_component, None));
    crate::facade::WorthUi::app()
        .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
        .register_component(role_only_component(attached_component))
        .register_component(role_only_component(peer_component))
        .register_appearance_role(role)
        .unwrap()
        .register_theme_token(appearance_theme_token(terminal.clone()))
        .register_theme_token(crate::capability::ThemeTokenDescriptor::alias(
            alias,
            crate::capability::ThemeTokenFamily::surface(),
            crate::capability::ThemeTokenSource::application(),
            crate::capability::ThemeTokenAlias::to(terminal),
        ))
        .with_rust_authored_input(
            worth_ui_dsl::WorthUiRustAuthoredArtifactInput::from_modules([module]),
        )
        .freeze()
        .map(crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host)
        .expect("aliased role-only fact-index fixture should prepare")
}

fn role_only_declaration(
    identity: &str,
    role: Option<&worth_ui_dsl::UiAppearanceRoleDeclaration>,
) -> worth_ui_dsl::WorthUiSemanticArtifactDeclaration {
    let declaration = worth_ui_dsl::WorthUiSemanticArtifactDeclaration::new(
        worth_ui_dsl::UiDslSemanticKey::new(identity),
        worth_ui_dsl::UiDslSemanticFamily::Control,
    )
    .with_structural_token(worth_ui_dsl::UiDslStructuralToken::new(
        "control:role-only-fact-index",
    ))
    .with_component_reference(worth_ui_dsl::UiDslComponentReference::new(identity).unwrap())
    .unwrap();
    match role {
        Some(role) => declaration
            .with_appearance_role_attachment(
                worth_ui_dsl::UiAppearanceRoleAttachmentDeclaration::new(
                    role.role().clone(),
                    role.revision(),
                ),
            )
            .unwrap(),
        None => declaration,
    }
}

fn unattached_fixture(
    fixture_name: &str,
    component_identity: &str,
) -> crate::facade::WorthUiRustAuthoredDeclarationFixture {
    crate::facade::WorthUiRustAuthoredDeclarationFixture::named(fixture_name)
        .with_semantic_artifact_spec(
            worth_ui_dsl::UiDslSemanticArtifactSpec::new(
                worth_ui_dsl::UiDslSemanticKey::new(component_identity),
                worth_ui_dsl::UiDslSemanticFamily::Control,
                worth_ui_dsl::UiDslSourceProvenance::file_authored("app/unattached.wui", 0),
            )
            .with_structural_token(worth_ui_dsl::UiDslStructuralToken::new(
                "control:unattached-static-paint",
            ))
            .with_component_reference(
                worth_ui_dsl::UiDslComponentReference::new(component_identity).unwrap(),
            )
            .unwrap(),
        )
}

fn role_only_component(identity: &str) -> crate::capability::ComponentDescriptor {
    crate::runtime::tests::source_ingress_boundary_test_support::source_backed_package_component(
        identity,
    )
    .with_appearance_aspect_contract(
        worth_ui_dsl::UiAppearanceAspectContract::component(
            [worth_ui_dsl::UiAppearanceAspect::Background],
            [],
        )
        .unwrap(),
    )
    .expect("role-only component contract should be admitted")
}

fn role_only_spec(
    identity: &str,
    role: Option<&worth_ui_dsl::UiAppearanceRoleDeclaration>,
    declaration_index: usize,
) -> worth_ui_dsl::UiDslSemanticArtifactSpec {
    let spec = worth_ui_dsl::UiDslSemanticArtifactSpec::new(
        worth_ui_dsl::UiDslSemanticKey::new(identity),
        worth_ui_dsl::UiDslSemanticFamily::Control,
        worth_ui_dsl::UiDslSourceProvenance::file_authored(
            "app/role-only-fact-index.wui",
            declaration_index,
        ),
    )
    .with_structural_token(worth_ui_dsl::UiDslStructuralToken::new(
        "control:role-only-fact-index",
    ))
    .with_component_reference(worth_ui_dsl::UiDslComponentReference::new(identity).unwrap())
    .unwrap();
    role.map_or(spec.clone(), |role| {
        spec.with_appearance_role_attachment(
            worth_ui_dsl::UiAppearanceRoleAttachmentDeclaration::new(
                role.role().clone(),
                role.revision(),
            ),
        )
        .unwrap()
    })
}
