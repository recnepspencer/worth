use worth_ui_dsl::{UiAppearanceAspect, UiThemeColor, UiThemeValue, UiThemeValueKind};

use super::super::state::{UiAppearanceStateVector, UiAppearanceTarget};
use super::super::theme::UiThemeResolutionView;
use super::UiAppearanceResolver;

#[test]
fn resolver_is_deterministic_and_emits_no_host_commands() {
    super::tests::run_on_appearance_fixture_stack(|| {
        let (mut session, role, target, vector, theme) = inputs();
        let resolver = UiAppearanceResolver::new();
        let first = resolver
            .resolve_node(&target, &role, &vector, &theme)
            .expect("sealed appearance inputs should resolve");
        let second = resolver
            .resolve_node(&target, &role, &vector, &theme)
            .expect("the same sealed inputs should resolve identically");

        assert!(first.exactly_equivalent(&second));
        assert_eq!(first.semantic_digest(), second.semantic_digest());
        assert_eq!(first.aspects().len(), 1);
        let aspect = &first.aspects()[0];
        assert_eq!(aspect.aspect(), UiAppearanceAspect::Background);
        assert_eq!(
            aspect.value(),
            UiThemeValue::Color(UiThemeColor::from_channels([1, 2, 3, 255]))
        );
        assert_eq!(
            aspect.provenance().selected_slot().as_str(),
            "theme.appearance_consumer"
        );
        assert_eq!(
            aspect.provenance().terminal_slot().as_str(),
            "theme.appearance_consumer"
        );
        assert_eq!(
            aspect.support(),
            super::UiAppearanceSupportPosture::Supported
        );
        let inspection_world = worth_ui_inspection::UiAppearanceInspectionWorld::new(
            first.state().basis().session().as_u64(),
            first
                .state()
                .basis()
                .generation()
                .prepared_generation()
                .semantic_package_identity()
                .narrowing_fingerprint(),
            first.state().basis().surface().diagnostic_value(),
        );
        session.record_appearance_projection_for_inspection(&first, 1);
        let inspection =
            session.why_appearance(worth_ui_inspection::UiAppearanceInspectionQuery::new(
                inspection_world,
                target.graph_node().digest(),
                UiAppearanceAspect::Background,
            ));
        let worth_ui_inspection::UiAppearanceInspectionOutcome::Found(explanation) = inspection
        else {
            panic!("a recorded sealed projection should be inspectable")
        };
        assert_eq!(explanation.role(), role.role().as_str());
        assert_eq!(explanation.theme(), "theme.test");
        assert_eq!(
            explanation.value(),
            worth_ui_inspection::UiAppearanceInspectionValue::Resolved(aspect.value())
        );
        assert_eq!(explanation.query().world(), inspection_world);
        assert_eq!(
            explanation.matched_cell().ordinal(),
            aspect.decision_cell_ordinal()
        );
        assert_eq!(
            explanation.matched_cell().state_classes(),
            aspect.state_classes()
        );
        assert_eq!(
            explanation.source_span(),
            &worth_ui_inspection::UiAppearanceInspectionSourceSpan::Unavailable
        );
        assert_eq!(
            explanation.invalidation_cause(),
            worth_ui_inspection::UiAppearanceInspectionInvalidationCause::NotAttributed
        );
        assert_eq!(
            explanation.mounted_mechanic(),
            worth_ui_inspection::UiAppearanceInspectionMountedMechanic::NotEvaluated
        );
        assert_eq!(
            explanation.physical_suppression(),
            worth_ui_inspection::UiAppearanceInspectionPhysicalSuppression::NotEvaluated
        );
        assert_eq!(explanation.cost().consumers_selected(), 1);
        assert!(
            session
                .inspect_mounted_identity()
                .frame_receipts()
                .is_empty(),
            "resolver and inspection inputs must not publish a mounted host frame"
        );
        let _ = session.shutdown();
    });
}

pub(super) fn inputs() -> (
    crate::facade::WorthUiActiveApplicationSession,
    worth_ui_dsl::UiAppearanceRoleDeclaration,
    UiAppearanceTarget,
    UiAppearanceStateVector,
    UiThemeResolutionView,
) {
    use crate::runtime::tests::appearance_component_session_test_support::{
        attached_appearance_candidate_submission, source_backed_static_paint_consumer_session,
        validation_background_role, APPEARANCE_TOKEN,
    };

    let mut session = source_backed_static_paint_consumer_session();
    let role = validation_background_role(APPEARANCE_TOKEN);
    let candidate = attached_appearance_candidate_submission(
        &session,
        "appearance-resolver-test",
        "workspace.component.active_session_current",
    );
    let mut turn = session.begin_observation_turn().unwrap();
    turn.admit_source(candidate).unwrap();
    let observations = turn.seal().unwrap();
    session.classify_observations(observations).unwrap();
    let snapshot = session.appearance_owner_snapshot_for_test().unwrap();
    let graph_node = session
        .graph()
        .snapshot()
        .nodes()
        .iter()
        .find(|node| node.appearance_role_attachment().is_some())
        .expect("resolver fixture has an attached node")
        .graph_node_identity();
    let surface = worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let mounted_instance =
        worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap();
    let frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
    let node_receipt = worth_ui_host_contract::UiMountedNodeReceiptIssuer::mint_for(frame)
        .unwrap()
        .receipt_for(mounted_instance);
    let incarnation = worth_ui_host_contract::UiMountIncarnation::mint_unbound().unwrap();
    let target = UiAppearanceTarget::new(
        session.session_identity(),
        surface,
        graph_node,
        mounted_instance,
        incarnation,
        node_receipt,
    )
    .unwrap();
    let vector = UiAppearanceStateVector::seal_for_role(&snapshot, &target, &role).unwrap();
    let theme = theme_view(&session, &role, surface);
    (session, role, target, vector, theme)
}

pub(super) fn run_on_appearance_fixture_stack(test: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .name("worth-ui-appearance-test".to_owned())
        .stack_size(8 * 1024 * 1024)
        .spawn(test)
        .expect("appearance fixture test thread should start")
        .join()
        .expect("appearance fixture test should not panic");
}

fn theme_view(
    session: &crate::facade::WorthUiActiveApplicationSession,
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
) -> UiThemeResolutionView {
    let token = crate::capability::ThemeTokenId::new("theme.appearance_consumer").unwrap();
    let catalog = crate::capability::UiThemeSlotCatalog::admit(
        1,
        [crate::capability::UiThemeSlotDeclaration::new(
            token.clone(),
            crate::capability::ThemeTokenFamily::surface(),
            UiThemeValueKind::Color,
            crate::capability::ThemeTokenSource::application(),
            crate::capability::UiThemeSlotDisclosure::Public,
            crate::capability::UiThemeSlotSuccessorCompatibility::ExactMeaning,
            None,
        )],
    )
    .unwrap();
    let definition_identity =
        crate::capability::UiThemeDefinitionIdentity::new("theme.test").unwrap();
    let definition = crate::capability::UiThemeDefinition::admit(
        definition_identity.clone(),
        1,
        &catalog,
        [(
            token,
            UiThemeValue::Color(UiThemeColor::from_channels([1, 2, 3, 255])),
        )],
    )
    .unwrap();
    let bundle =
        crate::capability::FrozenAppearanceThemeCapabilities::admit(catalog, vec![definition])
            .unwrap();
    let host_profile = worth_ui_host_contract::UiHostAppearanceProfileContract::admit(
        "resolver-test-host",
        1,
        worth_ui_host_contract::UiHostAppearanceMechanicFamily::ALL,
        Some(worth_ui_host_contract::UiHostPrimaryPointerKind::Mouse),
    )
    .unwrap();
    let capability =
        crate::runtime::appearance::theme::UiThemeCapabilityAdmission::from_frozen_capabilities(
            &bundle,
            &definition_identity,
            session.capabilities().appearance_roles(),
            &host_profile,
        )
        .unwrap()
        .issue(
            [role.role().clone()],
            surface,
            session.active_generation_identity(),
        )
        .unwrap();
    UiThemeResolutionView::from_capability(&capability, &bundle).unwrap()
}
