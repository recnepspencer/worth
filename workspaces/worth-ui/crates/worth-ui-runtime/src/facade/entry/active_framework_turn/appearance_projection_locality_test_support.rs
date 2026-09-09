use super::{CANDIDATE_TOKEN, UNSTYLED_COMPONENT};
use crate::runtime::tests::appearance_component_session_test_support as support;

pub(super) fn admit_theme(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    token: crate::capability::ThemeTokenId,
    revision: u64,
    color: &str,
) {
    let value = crate::capability::ThemeTokenValue::color(
        crate::capability::ThemeColorValue::hex(color).unwrap(),
    );
    let change =
        crate::facade::entry::UiNativeThemeTokenValueChange::successor(token, revision, value)
            .unwrap();
    session.admit_application_theme_values(&[change]).unwrap();
}

pub(super) fn admit_owner_snapshot(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    role_a: &worth_ui_dsl::UiAppearanceRoleDeclaration,
    role_b: &worth_ui_dsl::UiAppearanceRoleDeclaration,
    source_name: &str,
) {
    let candidate = both_roles_submission(session, role_a, role_b, source_name);
    let observations = {
        let mut turn = session.begin_observation_turn().unwrap();
        turn.admit_source(candidate).unwrap();
        turn.seal().unwrap()
    };
    session.classify_observations(observations).unwrap();
    assert!(session.has_appearance_owner_snapshot_for_test());
}

fn both_roles_submission(
    session: &crate::facade::WorthUiActiveApplicationSession,
    role_a: &worth_ui_dsl::UiAppearanceRoleDeclaration,
    role_b: &worth_ui_dsl::UiAppearanceRoleDeclaration,
    source_name: &str,
) -> crate::runtime::WorthUiWatchedCandidateSubmission {
    let attachment_a = worth_ui_dsl::UiAppearanceRoleAttachmentDeclaration::new(
        role_a.role().clone(),
        role_a.revision(),
    );
    let attachment_b = worth_ui_dsl::UiAppearanceRoleAttachmentDeclaration::new(
        role_b.role().clone(),
        role_b.revision(),
    );
    let module = worth_ui_dsl::WorthUiRustAuthoredArtifactInputModule::new("appearance/consumer")
        .with_appearance_role(role_a.clone())
        .with_appearance_role(role_b.clone());
    let module = module
        .with_component_appearance_role(support::APPEARANCE_NODE_A, attachment_a)
        .unwrap();
    let module = module
        .with_component_appearance_role(support::APPEARANCE_NODE_B, attachment_b)
        .unwrap()
        .with_semantic_declaration(unstyled_semantic());
    let input = worth_ui_dsl::WorthUiRustAuthoredArtifactInput::from_modules([module]);
    crate::runtime::tests::source_ingress_boundary_test_support::lower_rust_submission(
        crate::runtime::WorthUiSourceProvider::rust_authored(source_name)
            .with_rust_authored_input(input),
        [crate::runtime::WorthUiWatcherEvent::provider_revision(
            source_name,
        )],
        session.capabilities(),
    )
}

pub(super) fn graph_node_for(
    session: &crate::facade::WorthUiActiveApplicationSession,
    name: &str,
) -> crate::graph::UiGraphNodeIdentity {
    let graph = session.graph();
    let found = graph.node_identities().find(|identity| {
        graph.lookup().graph_node(*identity).is_some_and(|node| {
            let value = node.value();
            let declaration_identity = value.declaration_identity();
            let authored_name = declaration_identity.authored_semantic_name();
            authored_name == name || authored_name.strip_prefix("component:") == Some(name)
        })
    });
    found.unwrap_or_else(|| {
        let available = graph
            .node_identities()
            .filter_map(|identity| {
                graph.lookup().graph_node(identity).map(|node| {
                    node.value()
                        .declaration_identity()
                        .authored_semantic_name()
                        .to_owned()
                })
            })
            .collect::<Vec<_>>();
        panic!("missing locality graph node {name}; available: {available:?}")
    })
}

pub(super) fn locality_fixture(
    role_a: &worth_ui_dsl::UiAppearanceRoleDeclaration,
    role_b: &worth_ui_dsl::UiAppearanceRoleDeclaration,
) -> crate::facade::WorthUiRustAuthoredDeclarationFixture {
    let attachment_b = worth_ui_dsl::UiAppearanceRoleAttachmentDeclaration::new(
        role_b.role().clone(),
        role_b.revision(),
    );
    support::appearance_fixture(role_a)
        .with_appearance_role("appearance/consumer", role_b.clone())
        .with_component_appearance_role(
            "appearance/consumer",
            support::APPEARANCE_NODE_B,
            attachment_b,
        )
        .with_semantic_artifact_spec(
            worth_ui_dsl::UiDslSemanticArtifactSpec::new(
                worth_ui_dsl::UiDslSemanticKey::new(UNSTYLED_COMPONENT),
                worth_ui_dsl::UiDslSemanticFamily::Control,
                worth_ui_dsl::UiDslSourceProvenance::rust_authored("appearance/consumer", 0),
            )
            .with_structural_token(worth_ui_dsl::UiDslStructuralToken::new(
                "control:appearance-locality-unstyled",
            ))
            .with_component_reference(
                worth_ui_dsl::UiDslComponentReference::new(UNSTYLED_COMPONENT).unwrap(),
            )
            .unwrap(),
        )
}

fn unstyled_semantic() -> worth_ui_dsl::WorthUiSemanticArtifactDeclaration {
    worth_ui_dsl::WorthUiSemanticArtifactDeclaration::new(
        worth_ui_dsl::UiDslSemanticKey::new(UNSTYLED_COMPONENT),
        worth_ui_dsl::UiDslSemanticFamily::Control,
    )
    .with_structural_token(worth_ui_dsl::UiDslStructuralToken::new(
        "control:appearance-locality-unstyled",
    ))
    .with_component_reference(
        worth_ui_dsl::UiDslComponentReference::new(UNSTYLED_COMPONENT).unwrap(),
    )
    .unwrap()
}

pub(super) fn background_role(
    identity: &str,
    token: &str,
) -> worth_ui_dsl::UiAppearanceRoleDeclaration {
    let aspect = worth_ui_dsl::UiAppearanceAspect::Background;
    let contract = worth_ui_dsl::UiAppearanceAspectContract::component([aspect], []).unwrap();
    let partition = worth_ui_dsl::UiAppearancePartitionAuthoring::new([])
        .with_cell(worth_ui_dsl::UiAppearanceCell::when([]).uses_slot(
            worth_ui_dsl::UiThemeSlotIdentity::new(token).unwrap(),
            worth_ui_dsl::UiThemeValueKind::Color,
        ))
        .compile(aspect)
        .unwrap();
    worth_ui_dsl::UiAppearanceRoleDeclaration::admit(
        worth_ui_dsl::UiAppearanceRoleIdentity::new(identity).unwrap(),
        worth_ui_dsl::UiAppearanceRoleRevision::new(1).unwrap(),
        worth_ui_dsl::UiAppearanceRoleApplicability::AnyComponent,
        &contract,
        [(aspect, partition)],
    )
    .unwrap()
}

pub(super) fn theme_bundle() -> crate::capability::FrozenAppearanceThemeCapabilities {
    let token_a = crate::capability::ThemeTokenId::new(support::APPEARANCE_TOKEN).unwrap();
    let token_b = crate::capability::ThemeTokenId::new(CANDIDATE_TOKEN).unwrap();
    let slot = |token| {
        crate::capability::UiThemeSlotDeclaration::new(
            token,
            crate::capability::ThemeTokenFamily::surface(),
            worth_ui_dsl::UiThemeValueKind::Color,
            crate::capability::ThemeTokenSource::application(),
            crate::capability::UiThemeSlotDisclosure::Public,
            crate::capability::UiThemeSlotSuccessorCompatibility::ExactMeaning,
            None,
        )
    };
    let catalog = crate::capability::UiThemeSlotCatalog::admit(
        1,
        [slot(token_a.clone()), slot(token_b.clone())],
    )
    .unwrap();
    let definition = crate::capability::UiThemeDefinition::admit(
        crate::capability::UiThemeDefinitionIdentity::new("theme.appearance.locality").unwrap(),
        1,
        &catalog,
        [
            (
                token_a,
                worth_ui_dsl::UiThemeValue::Color(worth_ui_dsl::UiThemeColor::from_channels([
                    17, 34, 51, 255,
                ])),
            ),
            (
                token_b,
                worth_ui_dsl::UiThemeValue::Color(worth_ui_dsl::UiThemeColor::from_channels([
                    68, 85, 102, 255,
                ])),
            ),
        ],
    )
    .unwrap();
    crate::capability::FrozenAppearanceThemeCapabilities::admit(
        catalog,
        crate::capability::UiThemeDefinitionIdentity::new("theme.appearance.locality").unwrap(),
        vec![definition],
    )
    .unwrap()
}
