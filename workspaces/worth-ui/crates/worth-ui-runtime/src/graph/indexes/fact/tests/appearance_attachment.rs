#[test]
fn appearance_role_is_explicit_per_node_not_a_component_default() {
    let app = super::static_paint_app();
    let snapshot = app.prepared_authority().graph_snapshot();
    let attached = super::graph_node_named(snapshot, super::STATIC_PAINT_COMPONENT);
    let peer = super::graph_node_named(snapshot, super::STATIC_PAINT_PEER);

    assert!(attached.appearance_role_attachment().is_some());
    assert!(peer.appearance_role_attachment().is_none());
    assert_eq!(attached.component_reference(), peer.component_reference());
    assert!(app
        .prepared_authority()
        .consumed_fact_index()
        .has_appearance_consumers());
}

#[test]
fn unattached_node_does_not_infer_appearance_demand_from_its_component() {
    let token = crate::capability::ThemeTokenId::new(super::STATIC_PAINT_TOKEN).unwrap();
    let role = crate::runtime::tests::appearance_component_session_test_support::validation_background_role(
        super::STATIC_PAINT_TOKEN,
    );
    let app = crate::facade::WorthUi::app()
        .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
        .register_component(
            crate::runtime::tests::appearance_component_session_test_support::static_paint_component(
                super::STATIC_PAINT_COMPONENT,
                token.clone(),
            ),
        )
        .register_appearance_role(role)
        .unwrap()
        .register_theme_token(crate::capability::ThemeTokenDescriptor::define(
            token,
            crate::capability::ThemeTokenFamily::surface(),
            crate::capability::ThemeTokenSource::application(),
            crate::capability::ThemeTokenValue::color(
                crate::capability::ThemeColorValue::hex("#112233").unwrap(),
            ),
        ))
        .with_rust_authored_declaration_fixture(
            crate::facade::WorthUiRustAuthoredDeclarationFixture::named("unattached-static-paint")
                .with_semantic_artifact_spec(
                    worth_ui_dsl::UiDslSemanticArtifactSpec::new(
                        worth_ui_dsl::UiDslSemanticKey::new(super::STATIC_PAINT_COMPONENT),
                        worth_ui_dsl::UiDslSemanticFamily::Control,
                        worth_ui_dsl::UiDslSourceProvenance::file_authored("app/unattached.wui", 0),
                    )
                    .with_structural_token(worth_ui_dsl::UiDslStructuralToken::new(
                        "control:unattached-static-paint",
                    ))
                    .with_component_reference(
                        worth_ui_dsl::UiDslComponentReference::new(super::STATIC_PAINT_COMPONENT)
                            .unwrap(),
                    )
                    .unwrap(),
                ),
        )
        .freeze()
        .map(crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host)
        .expect("unattached static-paint fixture should prepare");

    assert!(super::graph_node_named(
        app.prepared_authority().graph_snapshot(),
        super::STATIC_PAINT_COMPONENT,
    )
    .appearance_role_attachment()
    .is_none());
    assert!(!app
        .prepared_authority()
        .consumed_fact_index()
        .has_appearance_consumers());
}

#[test]
fn appearance_consumer_queries_reconstruct_from_the_existing_index_without_host_work() {
    let app = super::static_paint_app();
    let authority = app.prepared_authority();
    let index = authority.consumed_fact_index();
    let role = crate::runtime::tests::appearance_component_session_test_support::validation_background_role(
        super::STATIC_PAINT_TOKEN,
    );
    let slot = worth_ui_dsl::UiThemeSlotIdentity::new(super::STATIC_PAINT_TOKEN).unwrap();
    let node = super::graph_node_named(authority.graph_snapshot(), super::STATIC_PAINT_COMPONENT)
        .graph_node_identity();

    let state = crate::runtime::appearance::UiAppearanceConsumerSelection::for_state(
        index,
        worth_ui_dsl::UiAppearanceStateAxis::Validation,
    );
    let role_selection =
        crate::runtime::appearance::UiAppearanceConsumerSelection::for_role(index, role.role());
    let slot_selection =
        crate::runtime::appearance::UiAppearanceConsumerSelection::for_slot(index, &slot);

    for selection in [&state, &role_selection, &slot_selection] {
        assert!(selection.is_reconstructible());
        assert_eq!(selection.selected_count(), 1);
        assert_eq!(selection.consumers(), [node]);
    }
    assert!(
        authority
            .consumed_fact_index()
            .appearance_slot_consumer_nodes(&slot)
            .iter()
            .all(|selected| *selected == node),
        "the prepared-only proof has no host command publisher to invoke"
    );
}
