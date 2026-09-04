use worth_ui_dsl::{UiAppearanceAspect, UiThemeColor, UiThemeValue, UiThemeValueKind};

use super::super::state::{
    UiAppearanceNodeRoleBinding, UiAppearanceNodeRoleBindingDenial, UiAppearanceStateVector,
    UiAppearanceStateVectorDenial, UiAppearanceTarget,
};
use super::super::theme::UiThemeResolutionView;
use super::UiAppearanceResolver;

#[test]
fn resolver_is_deterministic_and_emits_no_host_commands() {
    super::tests::run_on_appearance_fixture_stack(|| {
        let (session, binding, target, vector, theme) = inputs();
        assert_eq!(binding.basis().graph_node(), target.graph_node());
        assert_eq!(binding.basis().role(), binding.role().role());
        assert_eq!(binding.basis().revision(), binding.role().revision());
        assert_eq!(
            binding.basis().aspect_contract(),
            binding.role().aspect_contract()
        );
        assert_eq!(vector.binding(), Some(binding.basis()));
        let resolver = UiAppearanceResolver::new();
        let first = resolver
            .resolve_node(
                session.graph().snapshot(),
                session.capabilities(),
                &binding,
                &vector,
                &theme,
            )
            .expect("sealed appearance inputs should resolve");
        let second = resolver
            .resolve_node(
                session.graph().snapshot(),
                session.capabilities(),
                &binding,
                &vector,
                &theme,
            )
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

#[test]
fn resolver_rejects_an_unbound_vector_before_any_effect() {
    super::tests::run_on_appearance_fixture_stack(|| {
        let (session, binding, target, _bound_vector, theme) = inputs();
        let snapshot = session.appearance_owner_snapshot_for_test().unwrap();
        let unbound_vector = UiAppearanceStateVector::seal(snapshot, &target)
            .expect("the same sealed owner snapshot should produce an unbound control vector");
        let evidence = UiAppearanceResolver::new()
            .resolve_node(
                session.graph().snapshot(),
                session.capabilities(),
                &binding,
                &unbound_vector,
                &theme,
            )
            .expect_err("resolver must require the binding-carrying vector");

        assert_eq!(
            evidence.denial(),
            super::UiAppearanceResolutionDenial::VectorRoleBindingMismatch
        );
        assert_eq!(
            evidence.subject(),
            super::UiAppearanceResolutionSubject::GraphNode(target.graph_node())
        );
        assert!(session
            .inspect_mounted_identity()
            .frame_receipts()
            .is_empty());
        let _ = session.shutdown();
    });
}

#[test]
fn resolver_uses_the_canonical_node_index_without_linear_fallback() {
    super::tests::run_on_appearance_fixture_stack(|| {
        let (session, binding, target, vector, theme) = inputs();
        let graph = session.graph().snapshot();
        let attached = target.graph_node();
        let peer = graph
            .nodes()
            .iter()
            .find(|node| {
                node.graph_node_identity() != attached
                    && node.appearance_role_attachment().is_none()
            })
            .expect("appearance fixture should provide an unattached peer")
            .graph_node_identity();
        let stale_index = crate::graph::adversarial_snapshot_with_swapped_node_index_for_test(
            graph, attached, peer,
        );
        let missing = UiAppearanceNodeRoleBindingDenial::MissingRoleAttachment;

        assert_eq!(stale_index.authority_digest(), graph.authority_digest());
        assert_eq!(
            stale_index
                .core_indexes()
                .node_identity()
                .node(stale_index.nodes(), attached)
                .map(|node| node.graph_node_identity()),
            Some(peer)
        );
        assert_eq!(
            UiAppearanceNodeRoleBinding::from_current_graph(
                &stale_index,
                session.capabilities(),
                &target,
            ),
            Err(missing)
        );
        assert_eq!(
            UiAppearanceStateVector::seal_for_binding(
                session.appearance_owner_snapshot_for_test().unwrap(),
                &stale_index,
                session.capabilities(),
                &binding,
            ),
            Err(UiAppearanceStateVectorDenial::RoleBinding(missing))
        );
        let evidence = UiAppearanceResolver::new()
            .resolve_node(
                &stale_index,
                session.capabilities(),
                &binding,
                &vector,
                &theme,
            )
            .expect_err("resolver must use the canonical indexed node lookup");
        assert_eq!(
            evidence.denial(),
            super::UiAppearanceResolutionDenial::NodeRoleBinding(missing,)
        );
        assert!(session
            .inspect_mounted_identity()
            .frame_receipts()
            .is_empty());
        let _ = session.shutdown();
    });
}

pub(super) fn inputs() -> (
    crate::facade::WorthUiActiveApplicationSession,
    UiAppearanceNodeRoleBinding,
    UiAppearanceTarget,
    UiAppearanceStateVector,
    UiThemeResolutionView,
) {
    use crate::runtime::tests::appearance_component_session_test_support::{
        source_backed_two_node_appearance_session, two_node_appearance_candidate_submission,
        validation_background_role, APPEARANCE_NODE_A,
    };

    let role = validation_background_role("theme.appearance_consumer");
    let mut session = source_backed_two_node_appearance_session(&role);
    let candidate = two_node_appearance_candidate_submission(
        &session,
        "appearance-resolver-test",
        &role,
        APPEARANCE_NODE_A,
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
    let binding = UiAppearanceNodeRoleBinding::from_current_graph(
        session.graph().snapshot(),
        session.capabilities(),
        &target,
    )
    .unwrap();
    let target = binding.target().clone();
    let vector = UiAppearanceStateVector::seal_for_binding(
        snapshot,
        session.graph().snapshot(),
        session.capabilities(),
        &binding,
    )
    .unwrap();
    let theme = theme_view(&session, binding.role(), surface);
    (session, binding, target, vector, theme)
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
