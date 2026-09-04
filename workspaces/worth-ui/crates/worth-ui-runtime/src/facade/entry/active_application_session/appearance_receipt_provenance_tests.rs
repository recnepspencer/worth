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
    assert_eq!(
        explanation.invalidation_cause(),
        worth_ui_inspection::UiAppearanceInspectionInvalidationCause::InputEvidenceChanged
    );
    super::query_support::shutdown(fixture.session);
}
