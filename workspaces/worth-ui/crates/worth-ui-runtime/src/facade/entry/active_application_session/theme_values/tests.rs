#[path = "test_support.rs"]
mod support;

#[test]
fn appearance_theme_value_truth_real_session_observes_b_to_a_without_appearance_host_emission() {
    let role = crate::runtime::tests::appearance_component_session_test_support::
        validation_background_role(
            crate::runtime::tests::appearance_component_session_test_support::APPEARANCE_TOKEN,
        );
    let (mut session, host, surface, graph_node) = support::mounted_session(&role);
    let initial_view = session
        .presentation
        .appearance_theme_resolution_view(
            session.capabilities(),
            &role,
            surface,
            &session.active_generation_identity(),
        )
        .unwrap();
    assert_eq!(
        initial_view
            .resolve(
                &worth_ui_dsl::UiThemeSlotIdentity::new(support::theme_token().as_str()).unwrap(),
                worth_ui_dsl::UiThemeValueKind::Color,
            )
            .unwrap()
            .value(),
        support::typed_color([68, 85, 102, 255])
    );
    let initial_world = worth_ui_inspection::UiAppearanceInspectionWorld::new(
        session.session_identity().as_u64(),
        session
            .active_generation_identity()
            .prepared_generation()
            .semantic_package_identity()
            .narrowing_fingerprint(),
        surface.diagnostic_value(),
    );
    let initial_explanation =
        match session.why_appearance(worth_ui_inspection::UiAppearanceInspectionQuery::new(
            initial_world,
            graph_node.digest(),
            worth_ui_dsl::UiAppearanceAspect::Background,
        )) {
            worth_ui_inspection::UiAppearanceInspectionOutcome::Found(explanation) => explanation,
            outcome => panic!("initial B appearance result was not inspected: {outcome:?}"),
        };
    assert_eq!(
        initial_explanation.value(),
        worth_ui_inspection::UiAppearanceInspectionValue::Resolved(support::typed_color([
            68, 85, 102, 255
        ]),)
    );

    let token = support::theme_token();
    let change = support::theme_change(token, 0, support::legacy_color("#112233"));
    session.admit_application_theme_values(&[change]).unwrap();
    assert!(session
        .presentation
        .appearance_invalidation_batch()
        .is_some_and(|batch| batch.selected_count() > 0));
    host.push_native_display_settled_without_effects();
    let appearance_successor = support::publish_frame(&mut session, 2);
    assert_eq!(
        appearance_successor
            .cost_report()
            .adapter()
            .presented_surfaces(),
        0
    );

    let world = worth_ui_inspection::UiAppearanceInspectionWorld::new(
        session.session_identity().as_u64(),
        session
            .active_generation_identity()
            .prepared_generation()
            .semantic_package_identity()
            .narrowing_fingerprint(),
        surface.diagnostic_value(),
    );
    let explanation =
        match session.why_appearance(worth_ui_inspection::UiAppearanceInspectionQuery::new(
            world,
            graph_node.digest(),
            worth_ui_dsl::UiAppearanceAspect::Background,
        )) {
            worth_ui_inspection::UiAppearanceInspectionOutcome::Found(explanation) => explanation,
            outcome => panic!("B-to-A appearance result was not inspected: {outcome:?}"),
        };
    assert_eq!(
        explanation.value(),
        worth_ui_inspection::UiAppearanceInspectionValue::Resolved(support::typed_color([
            17, 34, 51, 255
        ]),)
    );
    assert!(!explanation.input_evidence_changed());
    assert!(explanation.semantic_projection_changed());
    assert!(explanation.resolved_aspect_value_changed());
    assert!(explanation.mounted_mechanical_output_changed());
    assert!(!explanation.equal_output_suppressed());
    assert!(!explanation.denied_before_effects());
    assert_eq!(explanation.denial_posture(), None);
    assert!(explanation.cost().consumers_selected() > 0);
    assert_eq!(explanation.cost().theme_slots_compared(), 1);
    assert_eq!(host.presentation_calls(), 3);
    assert!(host
        .last_filled_rect_colors()
        .iter()
        .all(|color| color.channels() == [17, 34, 51, 255]));
    let _ = session.shutdown();
}
