use worth_ui_inspection::{UiInspectionQuery, UiInspectionScope, UiInspectionTarget};

use crate::facade::{WorthUiActiveApplicationSession, WorthUiApplicationReplacementOutcome};
use crate::runtime::tests::active_application_session_test_support::admit_candidate_catalog;
use crate::runtime::tests::source_ingress_boundary_test_support::lower_file_submission;
use crate::runtime::{WorthUiSourceProvider, WorthUiWatcherEvent};

const CANDIDATE_COMPONENT: &str =
    "component workspace.component.active_session_candidate { region workspace.region.primary { sizing workspace.sizing.mosaic_support; } }";

/// Cuts `session` over to a successor generation whose authored source is the
/// candidate component plus `declarations`, through the real replacement path.
pub(crate) fn cut_over(session: &mut WorthUiActiveApplicationSession, declarations: &str) {
    let _receipt = replace(session, format!("{CANDIDATE_COMPONENT}\n{declarations}"))
        .into_activation()
        .expect("changed executable meaning publishes a successor");
}

/// Replaces the authored source of `session` with `source` through the real
/// replacement path, and returns what the plan decision made of it.
pub(crate) fn replace(
    session: &mut WorthUiActiveApplicationSession,
    source: String,
) -> WorthUiApplicationReplacementOutcome {
    let submission = lower_file_submission(
        WorthUiSourceProvider::in_memory("expression-successor").with_file("app/main.wui", source),
        [WorthUiWatcherEvent::provider_revision(
            "expression-successor",
        )],
        session.capabilities(),
    );
    let mut prepared = session
        .prepare_replacement(submission)
        .expect("successor candidate prepares");
    let catalog = admit_candidate_catalog(&mut prepared);
    let _ = prepared.inspect_candidate(UiInspectionQuery::new(
        UiInspectionTarget::product_root(),
        UiInspectionScope::graph(),
    ));
    let lowered = session
        .lower_prepared_replacement(*prepared)
        .expect("prepared successor lowers");
    let pending = session
        .stage_prepared_replacement(lowered)
        .expect("lowered successor stages");
    let boundary = session
        .execute_framework_turn(|_| {})
        .expect("no mounted presentation lease is active")
        .into_completion()
        .into_execution()
        .expect("an empty framework turn yields an activation boundary")
        .into_activation_boundary();
    session
        .activate_prepared_replacement(pending, catalog, boundary, None)
        .expect("the candidate-owned catalog cuts over atomically")
}
