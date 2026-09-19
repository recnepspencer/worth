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
    let world = fixture.session.appearance_inspection_world(fixture.surface);
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
