use crate::runtime::tests::appearance_component_session_test_support::{
    appearance_candidate_submission, appearance_component_builder, appearance_fixture,
    validation_background_role, APPEARANCE_NODE_A as ACTIVE_COMPONENT, APPEARANCE_TOKEN,
};
use crate::runtime::tests::appearance_theme_test_support;
#[test]
fn authored_role_attachment_replacement_installs_exact_successor_consumers() {
    attachment_rebind(validation_background_role(APPEARANCE_TOKEN), true);
}

#[test]
fn authored_unconditional_attachment_resolves_appearance_in_first_successor_publication() {
    let contract = worth_ui_dsl::UiAppearanceAspectContract::component(
        [worth_ui_dsl::UiAppearanceAspect::Background],
        [],
    )
    .unwrap();
    let partition = worth_ui_dsl::UiAppearancePartitionAuthoring::new([])
        .with_cell(worth_ui_dsl::UiAppearanceCell::when([]).uses_slot(
            worth_ui_dsl::UiThemeSlotIdentity::new(APPEARANCE_TOKEN).unwrap(),
            worth_ui_dsl::UiThemeValueKind::Color,
        ))
        .compile(worth_ui_dsl::UiAppearanceAspect::Background)
        .unwrap();
    let role = worth_ui_dsl::UiAppearanceRoleDeclaration::admit(
        worth_ui_dsl::UiAppearanceRoleIdentity::new("test.unconditional-background").unwrap(),
        worth_ui_dsl::UiAppearanceRoleRevision::new(1).unwrap(),
        worth_ui_dsl::UiAppearanceRoleApplicability::AnyComponent,
        &contract,
        [(worth_ui_dsl::UiAppearanceAspect::Background, partition)],
    )
    .unwrap();
    attachment_rebind(role, true);
}

fn attachment_rebind(
    blue: worth_ui_dsl::UiAppearanceRoleDeclaration,
    assert_first_successor_paint: bool,
) {
    let green = worth_ui_dsl::UiAppearanceRoleDeclaration::admit(
        worth_ui_dsl::UiAppearanceRoleIdentity::new("test.green-background").unwrap(),
        blue.revision(),
        blue.applicability().clone(),
        blue.aspect_contract(),
        blue.partitions().iter().cloned(),
    )
    .unwrap();
    let host = crate::certification_support::ScriptedPresentationHost::native_display();
    host.set_capabilities(worth_ui_host_native::appearance_capability_report());
    let mut session = appearance_component_builder(&blue)
        .register_appearance_role(green.clone())
        .unwrap()
        .register_appearance_theme_bundle(appearance_theme_test_support::bundle())
        .unwrap()
        .with_rust_authored_declaration_fixture(appearance_fixture(&blue))
        .freeze()
        .map(|app| {
            crate::facade::entry::WorthUiCertificationApplicationTransition::activate_test_host(
                app,
                host.clone(),
            )
        })
        .unwrap()
        .launch()
        .unwrap();
    let (_surface, _) = super::mounting_fixture::mount(&mut session, 1_000);

    // Identical aspect demand and slot values cannot make an explicit role
    // attachment edit a no-op. The accepted graph must name the new role and
    // only that role may select the component in the successor consumer index.
    for (source, selected, absent) in [
        ("appearance-green-edit", &green, &blue),
        ("appearance-blue-recovery", &blue, &green),
    ] {
        let predecessor = session.active_generation_identity();
        let candidate = appearance_candidate_submission(&session, source, Some(selected));
        let mut observation = session.begin_observation_turn().unwrap();
        observation.admit_source(candidate).unwrap();
        let observations = observation.seal().unwrap();
        let changed = match session.classify_observations(observations).unwrap() {
            crate::runtime::observation::UiChangeClassificationOutcome::Changed(changed) => changed,
            _ => panic!("watched attachment edits cannot be evidence-only"),
        };
        assert_eq!(changed.facts().len(), 1);
        let fact = changed.facts()[0].authored_source().unwrap();
        assert_eq!(
            fact.kind(),
            crate::fact_contract::UiAuthoredFactKind::SemanticsChanged
        );
        assert_eq!(
            fact.selector(),
            &crate::fact_contract::UiAuthoredFactSelector::node(format!(
                "component:{ACTIVE_COMPONENT}"
            ),)
        );
        let lifecycle = session
            .resolve_affected_scope(changed)
            .unwrap()
            .resolve_identity_lifecycle()
            .unwrap();
        let plan = session
            .compile_rebind_plan(
                lifecycle,
                crate::runtime::rebind::UiRebindExecutionPolicy::ordinary(),
            )
            .unwrap();
        host.push_native_display_settled_without_effects();
        let receipt = match session
            .prepare_rebind(
                plan,
                crate::runtime::rebind::UiRebindExecutionRequest::new(1),
            )
            .unwrap()
            .execute(1)
        {
            crate::runtime::rebind::UiRebindOutcome::Published(receipt) => receipt,
            crate::runtime::rebind::UiRebindOutcome::RejectedBeforeEffects(denial) => panic!(
                "attachment rebind denied: {:?}, {:?}",
                denial.cause(),
                denial.host_rejections()
            ),
            _ => panic!("the classified attachment edit must publish"),
        };
        assert!(receipt.application_publication().is_some());
        assert!(receipt.mounted_publication().is_some());
        assert_ne!(session.active_generation_identity(), predecessor);
        if assert_first_successor_paint {
            let projection = session
                .mounted
                .current_unpublished_appearance()
                .expect("replacement appearance lowering succeeds")
                .expect("the first successor publication contains resolved appearance");
            super::test_support::assert_unpublished_surface(projection, [17, 34, 51, 255]);
        }

        let prepared = session.application.prepared_authority();
        let nodes = prepared.graph_snapshot().nodes();
        let node = nodes
            .iter()
            .find(|node| {
                node.component_reference()
                    .is_some_and(|id| id.as_str() == ACTIVE_COMPONENT)
            })
            .unwrap();
        assert_eq!(
            node.appearance_role_attachment().unwrap().role(),
            selected.role()
        );
        let index = prepared.consumed_fact_index();
        assert!(index.has_appearance_attachment(node.graph_node_identity()));
        let required_roles = index.appearance_required_role_identities();
        assert!(required_roles.contains(selected.role()));
        assert!(!required_roles.contains(absent.role()));
    }
    let _ = session.shutdown();
}
