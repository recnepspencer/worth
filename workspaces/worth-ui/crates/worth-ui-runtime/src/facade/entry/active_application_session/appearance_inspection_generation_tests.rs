use crate::runtime::tests::appearance_component_session_test_support as support;

#[test]
fn evidence_only_rebind_changes_exact_inspection_generation_without_world_aliasing() {
    let role = super::role_support::single_aspect_role(
        "test.inspection-generation",
        worth_ui_dsl::UiAppearanceAspect::Background,
        support::APPEARANCE_TOKEN,
    );
    let mut fixture = super::mounted_fixture(&role, &[], false);
    let predecessor = fixture.session.active_generation_identity().clone();
    let predecessor_world = fixture.session.appearance_inspection_world(fixture.surface);
    let query = worth_ui_inspection::UiAppearanceInspectionQuery::new(
        predecessor_world,
        fixture.graph_node.digest(),
        worth_ui_dsl::UiAppearanceAspect::Background,
    );
    assert!(matches!(
        fixture.session.why_appearance(query),
        worth_ui_inspection::UiAppearanceInspectionOutcome::Found(_)
    ));

    let candidate = support::appearance_candidate_submission(
        &fixture.session,
        "appearance-inspection-generation-successor",
        Some(&role),
    );
    let mut observation = fixture.session.begin_observation_turn().unwrap();
    observation.admit_source(candidate).unwrap();
    let observations = observation.seal().unwrap();
    let evidence = match fixture.session.classify_observations(observations).unwrap() {
        crate::runtime::observation::UiChangeClassificationOutcome::EvidenceOnly(evidence) => {
            evidence
        }
        _ => panic!("equal authored semantics must produce evidence-only rebind"),
    };
    let plan = fixture
        .session
        .compile_preservation_rebind(
            evidence,
            crate::runtime::rebind::UiRebindExecutionPolicy::ordinary(),
        )
        .unwrap();
    let successor_prepared = plan.basis().candidate_generation().clone();
    let prepared = fixture
        .session
        .prepare_rebind(
            plan,
            crate::runtime::rebind::UiRebindExecutionRequest::new(1),
        )
        .unwrap();
    let receipt = match prepared.execute(1) {
        crate::runtime::rebind::UiRebindOutcome::Published(receipt) => receipt,
        _ => panic!("evidence-only rebind must publish"),
    };
    let successor = fixture.session.active_generation_identity().clone();

    assert_ne!(successor, predecessor);
    assert_eq!(
        predecessor
            .prepared_generation()
            .semantic_package_identity()
            .narrowing_fingerprint(),
        successor
            .prepared_generation()
            .semantic_package_identity()
            .narrowing_fingerprint()
    );
    assert_eq!(receipt.active_generation(), &successor_prepared);
    let successor_world = fixture.session.appearance_inspection_world(fixture.surface);
    assert_ne!(
        predecessor_world.evidence_generation(),
        successor_world.evidence_generation()
    );
    assert_eq!(
        fixture.session.why_appearance(query),
        worth_ui_inspection::UiAppearanceInspectionOutcome::WrongWorld
    );

    super::refresh_appearance_owner_snapshot(
        &mut fixture.session,
        &role,
        "appearance-inspection-generation-frame",
    );
    super::publish_frame(&mut fixture.session, 3);
    let successor_query = worth_ui_inspection::UiAppearanceInspectionQuery::new(
        successor_world,
        fixture.graph_node.digest(),
        worth_ui_dsl::UiAppearanceAspect::Background,
    );
    assert!(matches!(
        fixture.session.why_appearance(successor_query),
        worth_ui_inspection::UiAppearanceInspectionOutcome::Found(_)
    ));
    assert_eq!(
        fixture.session.why_appearance(query),
        worth_ui_inspection::UiAppearanceInspectionOutcome::WrongWorld
    );
    drop(receipt);
    super::query_support::shutdown(fixture.session);
}
