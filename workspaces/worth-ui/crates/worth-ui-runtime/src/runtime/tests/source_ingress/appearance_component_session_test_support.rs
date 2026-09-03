use crate::runtime::tests::source_ingress_boundary_test_support::{
    lower_rust_submission, source_backed_package_component, source_backed_package_region,
    source_backed_package_sizing,
};

const ACTIVE_COMPONENT: &str = "workspace.component.active_session_current";
const CANDIDATE_COMPONENT: &str = "workspace.component.active_session_candidate";
pub(crate) const APPEARANCE_TOKEN: &str = "theme.appearance_consumer";
pub(crate) const LEGACY_STATIC_PAINT_TOKEN: &str = "theme.legacy_static_paint";

pub(crate) const APPEARANCE_NODE_A: &str = ACTIVE_COMPONENT;
pub(crate) const APPEARANCE_NODE_B: &str = CANDIDATE_COMPONENT;

pub(crate) fn source_backed_static_paint_consumer_session(
) -> crate::facade::WorthUiActiveApplicationSession {
    let role = validation_background_role(APPEARANCE_TOKEN);
    appearance_component_builder(&role)
        .with_rust_authored_declaration_fixture(appearance_fixture(&role))
        .freeze()
        .map(crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host)
        .expect("appearance consumer source application should prepare")
        .launch()
        .expect("appearance consumer source application should launch")
}

pub(crate) fn source_backed_static_paint_role_capable_session(
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
) -> crate::facade::WorthUiActiveApplicationSession {
    appearance_component_builder(role)
        .with_rust_authored_declaration_fixture(appearance_fixture_without_attachment())
        .freeze()
        .map(crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host)
        .expect("appearance-capable source application should prepare")
        .launch()
        .expect("appearance-capable source application should launch")
}

pub(crate) fn source_backed_two_node_appearance_session(
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
) -> crate::facade::WorthUiActiveApplicationSession {
    let snapshot = appearance_component_builder(role)
        .freeze()
        .map(crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host)
        .expect("two-node appearance capability snapshot should prepare");
    appearance_component_builder(role)
        .with_candidate_submission(two_node_appearance_submission(
            "two-node-appearance-current",
            role,
            APPEARANCE_NODE_A,
            snapshot.capabilities(),
        ))
        .freeze()
        .map(crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host)
        .expect("two-node appearance source application should prepare")
        .launch()
        .expect("two-node appearance source application should launch")
}

pub(crate) fn two_node_appearance_candidate_submission(
    session: &crate::facade::WorthUiActiveApplicationSession,
    source_name: &str,
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
    attached_node: &str,
) -> crate::runtime::WorthUiWatchedCandidateSubmission {
    two_node_appearance_submission(source_name, role, attached_node, session.capabilities())
}

fn two_node_appearance_submission(
    source_name: &str,
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
    attached_node: &str,
    capabilities: &crate::capability::CapabilitySnapshot,
) -> crate::runtime::WorthUiWatchedCandidateSubmission {
    let module = worth_ui_dsl::WorthUiRustAuthoredArtifactInputModule::new("appearance/consumer")
        .with_semantic_declaration(appearance_semantic_declaration(
            ACTIVE_COMPONENT,
            (attached_node == ACTIVE_COMPONENT).then_some(role),
        ))
        .with_semantic_declaration(appearance_semantic_declaration(
            CANDIDATE_COMPONENT,
            (attached_node == CANDIDATE_COMPONENT).then_some(role),
        ));
    let input = worth_ui_dsl::WorthUiRustAuthoredArtifactInput::from_modules([module]);
    lower_rust_submission(
        crate::runtime::WorthUiSourceProvider::rust_authored(source_name)
            .with_rust_authored_input(input),
        [crate::runtime::WorthUiWatcherEvent::provider_revision(
            source_name,
        )],
        capabilities,
    )
}

pub(crate) fn appearance_candidate_submission(
    session: &crate::facade::WorthUiActiveApplicationSession,
    source_name: &str,
    attachment: Option<&worth_ui_dsl::UiAppearanceRoleDeclaration>,
) -> crate::runtime::WorthUiWatchedCandidateSubmission {
    appearance_submission(
        source_name,
        ACTIVE_COMPONENT,
        attachment,
        session.capabilities(),
    )
}

pub(crate) fn attached_appearance_candidate_submission(
    session: &crate::facade::WorthUiActiveApplicationSession,
    source_name: &str,
    component: &str,
) -> crate::runtime::WorthUiWatchedCandidateSubmission {
    let role = validation_background_role(APPEARANCE_TOKEN);
    appearance_submission(source_name, component, Some(&role), session.capabilities())
}

fn appearance_submission(
    source_name: &str,
    component: &str,
    attachment: Option<&worth_ui_dsl::UiAppearanceRoleDeclaration>,
    capabilities: &crate::capability::CapabilitySnapshot,
) -> crate::runtime::WorthUiWatchedCandidateSubmission {
    let declaration = appearance_semantic_declaration(component, attachment);
    let input = worth_ui_dsl::WorthUiRustAuthoredArtifactInput::from_modules([
        worth_ui_dsl::WorthUiRustAuthoredArtifactInputModule::new("appearance/consumer")
            .with_semantic_declaration(declaration),
    ]);
    lower_rust_submission(
        crate::runtime::WorthUiSourceProvider::rust_authored(source_name)
            .with_rust_authored_input(input),
        [crate::runtime::WorthUiWatcherEvent::provider_revision(
            source_name,
        )],
        capabilities,
    )
}

fn appearance_semantic_declaration(
    component: &str,
    role: Option<&worth_ui_dsl::UiAppearanceRoleDeclaration>,
) -> worth_ui_dsl::WorthUiSemanticArtifactDeclaration {
    let declaration = worth_ui_dsl::WorthUiSemanticArtifactDeclaration::new(
        worth_ui_dsl::UiDslSemanticKey::new(component),
        worth_ui_dsl::UiDslSemanticFamily::Control,
    )
    .with_structural_token(worth_ui_dsl::UiDslStructuralToken::new(
        "control:appearance-consumer",
    ))
    .with_component_reference(worth_ui_dsl::UiDslComponentReference::new(component).unwrap())
    .unwrap();
    role.map_or(declaration.clone(), |role| {
        declaration
            .clone()
            .with_appearance_role_attachment(
                worth_ui_dsl::UiAppearanceRoleAttachmentDeclaration::new(
                    role.role().clone(),
                    role.revision(),
                ),
            )
            .unwrap()
    })
}

pub(crate) fn appearance_component_builder(
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
) -> crate::facade::entry::WorthUiApplicationBuilder {
    appearance_component_builder_with_contract_and_static_token(
        role,
        component_appearance_contract(),
        APPEARANCE_TOKEN,
    )
}

pub(crate) fn legacy_static_paint_appearance_component_builder(
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
) -> crate::facade::entry::WorthUiApplicationBuilder {
    appearance_component_builder_with_contract_and_static_token(
        role,
        component_appearance_contract(),
        LEGACY_STATIC_PAINT_TOKEN,
    )
}

pub(crate) fn six_axis_appearance_component_builder(
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
) -> crate::facade::entry::WorthUiApplicationBuilder {
    appearance_component_builder_with_contract_and_static_token(
        role,
        six_axis_component_appearance_contract(),
        APPEARANCE_TOKEN,
    )
    .with_focus_policy_defaults(crate::declaration::UiFocusPolicy::workbench())
    .with_selection_policy_defaults(crate::declaration::UiSelectionPolicy::single())
}

fn appearance_component_builder_with_contract_and_static_token(
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
    appearance_contract: worth_ui_dsl::UiAppearanceAspectContract,
    static_paint_token: &str,
) -> crate::facade::entry::WorthUiApplicationBuilder {
    let (_, _, world_profile) =
        crate::evidence::measurement::projection::fact_test_support::display_field_projection_context(
            "appearance-consumer-active-session",
        );
    let appearance_token = crate::capability::ThemeTokenId::new(APPEARANCE_TOKEN).unwrap();
    let static_paint_token = crate::capability::ThemeTokenId::new(static_paint_token).unwrap();
    let builder = crate::facade::WorthUi::app()
        .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
        .with_graph_world_profile(world_profile)
        .register_component(
            static_paint_component_with_contract(
                ACTIVE_COMPONENT,
                static_paint_token.clone(),
                appearance_contract.clone(),
            )
            .unwrap(),
        )
        .register_component(
            static_paint_component_with_contract(
                CANDIDATE_COMPONENT,
                static_paint_token.clone(),
                appearance_contract,
            )
            .unwrap(),
        )
        .register_appearance_role(role.clone())
        .unwrap()
        .register_theme_token(appearance_theme_token(appearance_token))
        .register_mosaic_region_kind(source_backed_package_region())
        .register_mosaic_sizing_contract(source_backed_package_sizing());
    if static_paint_token.as_str() != APPEARANCE_TOKEN {
        builder.register_theme_token(appearance_theme_token(static_paint_token))
    } else {
        builder
    }
}

fn component_appearance_contract() -> worth_ui_dsl::UiAppearanceAspectContract {
    worth_ui_dsl::UiAppearanceAspectContract::component(
        [worth_ui_dsl::UiAppearanceAspect::Background],
        [],
    )
    .unwrap()
}

fn six_axis_component_appearance_contract() -> worth_ui_dsl::UiAppearanceAspectContract {
    worth_ui_dsl::UiAppearanceAspectContract::component(
        [
            worth_ui_dsl::UiAppearanceAspect::Background,
            worth_ui_dsl::UiAppearanceAspect::Foreground,
            worth_ui_dsl::UiAppearanceAspect::Border,
            worth_ui_dsl::UiAppearanceAspect::Radius,
            worth_ui_dsl::UiAppearanceAspect::Opacity,
            worth_ui_dsl::UiAppearanceAspect::Outline,
        ],
        [],
    )
    .unwrap()
}

pub(crate) fn appearance_theme_token(
    token: crate::capability::ThemeTokenId,
) -> crate::capability::ThemeTokenDescriptor {
    crate::capability::ThemeTokenDescriptor::define(
        token,
        crate::capability::ThemeTokenFamily::surface(),
        crate::capability::ThemeTokenSource::application(),
        crate::capability::ThemeTokenValue::color(
            crate::capability::ThemeColorValue::hex("#112233").unwrap(),
        ),
    )
}

pub(crate) fn static_paint_component(
    identity: &str,
    token: crate::capability::ThemeTokenId,
) -> crate::capability::ComponentDescriptor {
    static_paint_component_with_contract(identity, token, component_appearance_contract()).unwrap()
}

fn static_paint_component_with_contract(
    identity: &str,
    token: crate::capability::ThemeTokenId,
    appearance_contract: worth_ui_dsl::UiAppearanceAspectContract,
) -> Result<
    crate::capability::ComponentDescriptor,
    crate::capability::ComponentAppearanceAspectContractDenial,
> {
    source_backed_package_component(identity)
        .with_static_paint(
            crate::capability::ComponentStaticPaintContract::opaque_fill(
                token,
                crate::capability::ComponentStaticPaintOrder::back_to_front(0),
            ),
            crate::capability::ComponentAllocationMeasurementContract::fill_viewport(),
        )
        .with_appearance_aspect_contract(appearance_contract)
}

#[test]
fn component_descriptor_rejects_the_actual_backdrop_contract() {
    let token = crate::capability::ThemeTokenId::new(APPEARANCE_TOKEN).unwrap();
    let result = static_paint_component_with_contract(
        ACTIVE_COMPONENT,
        token,
        worth_ui_dsl::UiAppearanceAspectContract::backdrop(),
    );
    assert_eq!(
        result,
        Err(
            crate::capability::ComponentAppearanceAspectContractDenial::BackdropContractOnComponent
        )
    );
}

pub(crate) fn appearance_fixture(
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
) -> crate::facade::WorthUiRustAuthoredDeclarationFixture {
    let attachment = worth_ui_dsl::UiAppearanceRoleAttachmentDeclaration::new(
        role.role().clone(),
        role.revision(),
    );
    crate::facade::WorthUiRustAuthoredDeclarationFixture::named("appearance-consumer-current")
        .with_appearance_role("appearance/consumer", role.clone())
        .with_component_appearance_role("appearance/consumer", ACTIVE_COMPONENT, attachment)
}

fn appearance_fixture_without_attachment() -> crate::facade::WorthUiRustAuthoredDeclarationFixture {
    crate::facade::WorthUiRustAuthoredDeclarationFixture::named("appearance-capable-current")
        .with_semantic_artifact_spec(
            worth_ui_dsl::UiDslSemanticArtifactSpec::new(
                worth_ui_dsl::UiDslSemanticKey::new(ACTIVE_COMPONENT),
                worth_ui_dsl::UiDslSemanticFamily::Control,
                worth_ui_dsl::UiDslSourceProvenance::rust_authored("appearance/consumer", 0),
            )
            .with_structural_token(worth_ui_dsl::UiDslStructuralToken::new(
                "control:appearance-consumer",
            ))
            .with_component_reference(
                worth_ui_dsl::UiDslComponentReference::new(ACTIVE_COMPONENT).unwrap(),
            )
            .unwrap(),
        )
}

pub(crate) fn validation_background_role(slot: &str) -> worth_ui_dsl::UiAppearanceRoleDeclaration {
    validation_background_role_for(
        slot,
        worth_ui_dsl::UiAppearanceRoleApplicability::AnyComponent,
        worth_ui_dsl::UiAppearanceStateAxis::Validation,
    )
}

pub(crate) fn validation_background_role_with_axis(
    slot: &str,
    axis: worth_ui_dsl::UiAppearanceStateAxis,
) -> worth_ui_dsl::UiAppearanceRoleDeclaration {
    validation_background_role_for(
        slot,
        worth_ui_dsl::UiAppearanceRoleApplicability::AnyComponent,
        axis,
    )
}

fn validation_background_role_for(
    slot: &str,
    applicability: worth_ui_dsl::UiAppearanceRoleApplicability,
    axis: worth_ui_dsl::UiAppearanceStateAxis,
) -> worth_ui_dsl::UiAppearanceRoleDeclaration {
    let contract = worth_ui_dsl::UiAppearanceAspectContract::component(
        [worth_ui_dsl::UiAppearanceAspect::Background],
        [],
    )
    .unwrap();
    let partition = worth_ui_dsl::UiAppearancePartitionAuthoring::new([
        worth_ui_dsl::UiAppearanceAxisDomain::complete(axis),
    ])
    .with_cell(
        worth_ui_dsl::UiAppearanceCell::when([worth_ui_dsl::UiAppearanceAxisPredicate::any(axis)])
            .uses_slot(
                worth_ui_dsl::UiThemeSlotIdentity::new(slot).unwrap(),
                worth_ui_dsl::UiThemeValueKind::Color,
            ),
    )
    .compile(worth_ui_dsl::UiAppearanceAspect::Background)
    .unwrap();
    worth_ui_dsl::UiAppearanceRoleDeclaration::admit(
        worth_ui_dsl::UiAppearanceRoleIdentity::new("test.validation-background").unwrap(),
        worth_ui_dsl::UiAppearanceRoleRevision::new(1).unwrap(),
        applicability,
        &contract,
        [(worth_ui_dsl::UiAppearanceAspect::Background, partition)],
    )
    .unwrap()
}
