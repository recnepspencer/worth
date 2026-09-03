use super::MountedAppearanceFixture;

pub(super) fn why(
    fixture: &MountedAppearanceFixture,
) -> worth_ui_inspection::UiAppearanceInspectionExplanation {
    why_for(fixture, worth_ui_dsl::UiAppearanceAspect::Background)
}

pub(super) fn why_for(
    fixture: &MountedAppearanceFixture,
    aspect: worth_ui_dsl::UiAppearanceAspect,
) -> worth_ui_inspection::UiAppearanceInspectionExplanation {
    let world = worth_ui_inspection::UiAppearanceInspectionWorld::new(
        fixture.session.session_identity().as_u64(),
        fixture
            .session
            .active_generation_identity()
            .prepared_generation()
            .semantic_package_identity()
            .narrowing_fingerprint(),
        fixture.surface.diagnostic_value(),
    );
    let query = worth_ui_inspection::UiAppearanceInspectionQuery::new(
        world,
        fixture.graph_node.digest(),
        aspect,
    );
    match fixture.session.why_appearance(query) {
        worth_ui_inspection::UiAppearanceInspectionOutcome::Found(explanation) => explanation,
        outcome => panic!("receipt outcome was not inspected: {outcome:?}"),
    }
}

pub(super) fn shutdown(session: crate::facade::WorthUiActiveApplicationSession) {
    let _ = session.shutdown();
}
