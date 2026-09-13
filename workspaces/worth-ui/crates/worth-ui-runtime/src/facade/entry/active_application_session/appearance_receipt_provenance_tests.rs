use super::role_support::single_aspect_role;
use super::support;

#[test]
fn real_source_turn_reports_input_evidence_changed() {
    let role = single_aspect_role(
        "test.receipt-input",
        worth_ui_dsl::UiAppearanceAspect::Background,
        support::APPEARANCE_TOKEN,
    );
    let mut fixture = super::mounted_fixture(&role, &[], false);
    super::role_support::update_theme(&mut fixture.session, "#405060");
    let first_revision = super::publish_validation_class(
        &mut fixture.session,
        fixture.graph_node,
        crate::runtime::intent::UiValidationAppearanceClass::Valid,
        None,
    );
    super::refresh_appearance_owner_snapshot(
        &mut fixture.session,
        &role,
        "appearance-receipt-input-a",
    );
    super::publish_frame(&mut fixture.session, 1);
    let second_revision = super::publish_validation_class(
        &mut fixture.session,
        fixture.graph_node,
        crate::runtime::intent::UiValidationAppearanceClass::Valid,
        Some(first_revision),
    );
    assert_eq!(second_revision, first_revision + 1);
    super::refresh_appearance_owner_snapshot(
        &mut fixture.session,
        &role,
        "appearance-receipt-input-b",
    );
    super::publish_frame(&mut fixture.session, 2);

    let explanation = super::query_support::why(&fixture);
    assert_eq!(explanation.denial_posture(), None);
    assert_eq!(
        explanation.state_classes(),
        &[worth_ui_dsl::UiAppearanceAxisClass::ValidationValid]
    );
    assert!(explanation.input_evidence_changed());
    assert!(!explanation.semantic_projection_changed());
    assert!(!explanation.resolved_aspect_value_changed());
    assert!(!explanation.mounted_mechanical_output_changed());
    assert!(!explanation.equal_output_suppressed());
    assert!(!explanation.denied_before_effects());
    super::query_support::shutdown(fixture.session);
}

#[test]
fn radius_value_kind_mismatch_denies_theme_admission_before_resolution() {
    let themes = super::role_support::radius_theme_bundle();
    let token = crate::capability::ThemeTokenId::new(support::APPEARANCE_TOKEN).unwrap();
    let definition = crate::capability::UiThemeDefinition::admit(
        crate::capability::UiThemeDefinitionIdentity::new("theme.appearance.radii-invalid-kind")
            .unwrap(),
        1,
        themes.catalog(),
        themes
            .get(themes.initial_definition_identity())
            .unwrap()
            .values()
            .map(|(slot, value)| {
                (
                    slot.clone(),
                    if slot == &token {
                        worth_ui_dsl::UiThemeValue::Color(
                            worth_ui_dsl::UiThemeColor::from_channels([9, 9, 9, 255]),
                        )
                    } else {
                        *value
                    },
                )
            }),
    );
    assert_eq!(
        definition,
        Err(crate::capability::UiThemeDefinitionDenial::ValueKindMismatch(token))
    );
}
