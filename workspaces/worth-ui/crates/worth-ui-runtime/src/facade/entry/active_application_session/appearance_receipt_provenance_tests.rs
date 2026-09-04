use std::collections::BTreeMap;

use super::role_support::{radius_role, single_aspect_role};
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
fn mounted_resolution_denial_preserves_prior_theme_comparison_work() {
    let role = radius_role("test.receipt-partial-resolution");
    let mut fixture = super::mounted_fixture(&role, &[], true);
    super::role_support::update_radius_at_revision(&mut fixture.session, [i32::MAX - 1; 4], 0);
    fixture
        .session
        .presentation
        .replace_appearance_theme_values_for_test(
            fixture.surface,
            BTreeMap::from([(
                crate::capability::ThemeTokenId::new(support::APPEARANCE_TOKEN).unwrap(),
                worth_ui_dsl::UiThemeValue::Color(worth_ui_dsl::UiThemeColor::from_channels([
                    9, 9, 9, 255,
                ])),
            )]),
        );
    super::publish_validation_class(
        &mut fixture.session,
        fixture.graph_node,
        crate::runtime::intent::UiValidationAppearanceClass::Valid,
        None,
    );
    super::refresh_appearance_owner_snapshot(
        &mut fixture.session,
        &role,
        "appearance-receipt-partial-resolution",
    );
    super::publish_frame(&mut fixture.session, 1);

    let explanation =
        super::query_support::why_for(&fixture, worth_ui_dsl::UiAppearanceAspect::Background);
    assert_eq!(
        explanation.denial_posture(),
        Some(worth_ui_inspection::UiAppearanceInspectionDenialPosture::Resolution)
    );
    assert!(!explanation.input_evidence_changed());
    assert!(!explanation.semantic_projection_changed());
    assert!(!explanation.resolved_aspect_value_changed());
    assert!(!explanation.mounted_mechanical_output_changed());
    assert!(!explanation.equal_output_suppressed());
    assert!(explanation.denied_before_effects());
    assert_eq!(explanation.cost().theme_slots_compared(), 2);

    fixture
        .session
        .presentation
        .remove_appearance_theme_values_for_test(fixture.surface);
    super::query_support::shutdown(fixture.session);
}
