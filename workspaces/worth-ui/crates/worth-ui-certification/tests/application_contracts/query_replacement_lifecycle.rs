use worth_ui::facade::app::WorthUiApplicationCutoverDenial;
use worth_ui_certification::scenario::application_authority_closure::candidate_catalog::admit_candidate_catalog;
use worth_ui_query_binding::WorthUiQueryWorkspaceExt;
use worth_ui_runtime::facade::application::{
    WorthUiPlanRegionTransition, WorthUiVirtualizedPlanAvailability,
};
use worth_ui_test_support::WorthUiActiveSessionCertificationExt;
#[path = "query_replacement_lifecycle/mixed_real_lifecycle/hostile_mounted_journey.rs"]
mod hostile_mounted_journey;
#[path = "query_replacement_lifecycle/mixed_real_lifecycle/mixed_real_world.rs"]
mod mixed_real_world;
#[path = "query_replacement_lifecycle/mixed_real_lifecycle/mounted_successor.rs"]
mod mounted_successor;
#[path = "query_replacement_lifecycle/precommit_rollback.rs"]
mod precommit_rollback;
#[path = "query_replacement_lifecycle/query_patch.rs"]
pub(crate) mod query_patch;
#[path = "query_replacement_lifecycle/reset_workspace.rs"]
pub(crate) mod reset_workspace;
#[path = "query_replacement_lifecycle/scenario.rs"]
pub(crate) mod scenario;
#[path = "query_replacement_lifecycle/settled_snapshot_preservation.rs"]
mod settled_snapshot_preservation;
#[path = "query_replacement_lifecycle/support.rs"]
pub(crate) mod support;

use scenario::{
    application, installed_workspace, submission, ACTIVE_COMPONENT, FIRST_VIEW, NEXT_COMPONENT,
    SECOND_VIEW,
};
use support::*;

const QUERY_REBIND_STORM_COUNT: usize = 256;

#[test]
fn public_semantic_no_op_preserves_the_exact_real_query_live_resource() {
    let mut workspace = installed_workspace("query-semantic-no-op");
    let installed = workspace.worth_ui().expect("Worth UI domain installed");
    let first = installed
        .live_measurement_view(FIRST_VIEW)
        .expect("first live view");
    let second = installed
        .live_measurement_view(SECOND_VIEW)
        .expect("second live view");
    let mut session = application(first.clone(), second, &mut workspace)
        .launch()
        .expect("Query application launch");
    admit_active_resource(&mut session, &first, &mut workspace);

    let prime = submission(
        "query-lifecycle-active",
        ACTIVE_COMPONENT,
        &[FIRST_VIEW],
        session.capabilities(),
    );
    let prime = lower_and_stage(&session, prepare_catalog(&session, prime));
    let prime = activate(&mut session, prime.0, prime.1);
    assert!(prime.operation_live_retirement().is_empty());
    let active_generation = session.generation_identity().clone();
    assert_visible_query_execution(&mut session);

    let equivalent = submission(
        "query-lifecycle-active",
        ACTIVE_COMPONENT,
        &[FIRST_VIEW],
        session.capabilities(),
    );
    let equivalent = lower_and_stage(&session, prepare_catalog(&session, equivalent));
    let boundary = activation_boundary(&mut session);
    let outcome = session
        .activate_prepared_replacement(equivalent.0, equivalent.1, boundary, None)
        .expect("equivalent Query candidate reaches the public decision");
    let no_op = outcome
        .semantic_no_op()
        .expect("complete Query-backed executable and allocation equality is a semantic no-op");

    assert_eq!(no_op.active_generation(), &active_generation);
    assert_eq!(session.generation_identity(), &active_generation);
    assert_eq!(no_op.work().activation_publication_count(), 0);
    assert_visible_query_execution(&mut session);
    assert_active_operation_live_resource(&session);
    close_retirement(
        session.shutdown().into_operation_live_retirement(),
        &mut workspace,
    );
    close(open_resource(&first, &mut workspace), &mut workspace);
}

#[test]
fn public_cutover_preserves_and_retires_exact_real_query_resources() {
    let mut workspace = installed_workspace("query-replacement-success");
    let installed = workspace.worth_ui().expect("Worth UI domain installed");
    let first = installed
        .live_measurement_view(FIRST_VIEW)
        .expect("first live view");
    let second = installed
        .live_measurement_view(SECOND_VIEW)
        .expect("second live view");
    let app = application(first.clone(), second.clone(), &mut workspace);
    let mut session = app.launch().expect("Query application launch");
    admit_active_resource(&mut session, &first, &mut workspace);
    assert_visible_query_execution(&mut session);
    let initial_generation = session.generation_identity().clone();

    let preserve_submission = submission(
        "query-lifecycle-active",
        ACTIVE_COMPONENT,
        &[FIRST_VIEW],
        session.capabilities(),
    );
    let preserve = prepare_catalog(&session, preserve_submission);
    let preserve = lower_and_stage(&session, preserve);
    let cutover = activate(&mut session, preserve.0, preserve.1);

    assert_ne!(cutover.active_generation(), &initial_generation);
    assert!(cutover.operation_live_retirement().is_empty());
    assert_visible_query_execution(&mut session);
    assert_active_operation_live_resource(&session);

    let switch_submission = submission(
        "query-lifecycle-active",
        ACTIVE_COMPONENT,
        &[SECOND_VIEW],
        session.capabilities(),
    );
    let mut switch = prepare_catalog(&session, switch_submission);
    admit_candidate_resource(&mut switch.0, &second, &mut workspace);
    let switch = lower_and_stage(&session, switch);
    let retirement = activate(&mut session, switch.0, switch.1).into_operation_live_retirement();

    assert_eq!(retirement.len(), 1);
    assert_visible_query_execution(&mut session);
    close_retirement(retirement, &mut workspace);
    close(open_resource(&first, &mut workspace), &mut workspace);
    assert_active_operation_live_resource(&session);

    let remove_submission = submission(
        "query-lifecycle-active",
        ACTIVE_COMPONENT,
        &[],
        session.capabilities(),
    );
    let remove = lower_and_stage(&session, prepare_catalog(&session, remove_submission));
    let retirement = activate(&mut session, remove.0, remove.1).into_operation_live_retirement();

    assert_eq!(retirement.len(), 1);
    assert_eq!(
        session.virtualized_plan_availability(),
        WorthUiVirtualizedPlanAvailability::QueryFree
    );
    close_retirement(retirement, &mut workspace);
    close(open_resource(&second, &mut workspace), &mut workspace);
    assert!(session.shutdown().operation_live_retirement().is_empty());
}

#[test]
fn denied_candidate_reaps_only_candidate_query_resources() {
    let mut workspace = installed_workspace("query-replacement-rollback");
    let installed = workspace.worth_ui().expect("Worth UI domain installed");
    let first = installed
        .live_measurement_view(FIRST_VIEW)
        .expect("first live view");
    let second = installed
        .live_measurement_view(SECOND_VIEW)
        .expect("second live view");
    let app = application(first.clone(), second.clone(), &mut workspace);
    let mut session = app.launch().expect("Query application launch");
    admit_active_resource(&mut session, &first, &mut workspace);

    let mut candidate = session
        .prepare_replacement(submission(
            "query-lifecycle-active",
            NEXT_COMPONENT,
            &[SECOND_VIEW],
            session.capabilities(),
        ))
        .expect("candidate replacement prepares");
    admit_candidate_resource(&mut candidate, &second, &mut workspace);
    let candidate_catalog = admit_candidate_catalog(&session, &mut candidate);
    let pending = lower_and_stage(&session, (candidate, candidate_catalog)).0;

    let foreign_submission = submission(
        "query-lifecycle-active",
        NEXT_COMPONENT,
        &[SECOND_VIEW],
        session.capabilities(),
    );
    let (_, foreign_catalog) = prepare_catalog(&session, foreign_submission);
    let boundary = activation_boundary(&mut session);
    let denial =
        match session.activate_prepared_replacement(pending, foreign_catalog, boundary, None) {
            Ok(_) => panic!("foreign candidate catalog cannot publish"),
            Err(denial) => denial,
        };
    assert!(matches!(
        denial,
        WorthUiApplicationCutoverDenial::PreparedApplicationGraphMismatch
    ));

    close(open_resource(&second, &mut workspace), &mut workspace);
    assert_active_operation_live_resource(&session);
    assert_visible_query_execution(&mut session);
    close_retirement(
        session.shutdown().into_operation_live_retirement(),
        &mut workspace,
    );
}

#[test]
fn late_foreign_boundary_denial_releases_query_resource_when_retry_is_abandoned() {
    let mut workspace = installed_workspace("query-replacement-late-rollback");
    let installed = workspace.worth_ui().expect("Worth UI domain installed");
    let first = installed
        .live_measurement_view(FIRST_VIEW)
        .expect("first live view");
    let second = installed
        .live_measurement_view(SECOND_VIEW)
        .expect("second live view");
    let mut session = application(first.clone(), second.clone(), &mut workspace)
        .launch()
        .expect("Query application launch");
    let mut foreign_session = application(first.clone(), second.clone(), &mut workspace)
        .launch()
        .expect("equal-looking foreign application launch");
    admit_active_resource(&mut session, &first, &mut workspace);
    let active_generation = session.generation_identity().clone();

    let mut candidate = prepare_catalog(
        &session,
        submission(
            "query-lifecycle-active",
            NEXT_COMPONENT,
            &[SECOND_VIEW],
            session.capabilities(),
        ),
    );
    admit_candidate_resource(&mut candidate.0, &second, &mut workspace);
    let (pending, catalog) = lower_and_stage(&session, candidate);
    let foreign_boundary = activation_boundary(&mut foreign_session);

    let denial =
        match session.activate_prepared_replacement(pending, catalog, foreign_boundary, None) {
            Ok(_) => panic!("foreign frame authority must deny the late commit"),
            Err(denial) => denial,
        };
    assert!(matches!(
        denial,
        WorthUiApplicationCutoverDenial::FrameBoundaryUnavailable { .. }
    ));
    assert_eq!(session.generation_identity(), &active_generation);
    drop(denial);
    close(open_resource(&second, &mut workspace), &mut workspace);
    assert_active_operation_live_resource(&session);
    assert_visible_query_execution(&mut session);
    close_retirement(
        session.shutdown().into_operation_live_retirement(),
        &mut workspace,
    );
    let _ = foreign_session.shutdown();
}

#[test]
fn bounded_query_rebind_storm_retires_each_predecessor_resource_exactly_once() {
    let mut workspace = installed_workspace("query-replacement-storm");
    let installed = workspace.worth_ui().expect("Worth UI domain installed");
    let first = installed
        .live_measurement_view(FIRST_VIEW)
        .expect("first view");
    let second = installed
        .live_measurement_view(SECOND_VIEW)
        .expect("second view");
    let mut session = application(first.clone(), second.clone(), &mut workspace)
        .launch()
        .expect("Query application launch");
    admit_active_resource(&mut session, &first, &mut workspace);

    for step in 0..QUERY_REBIND_STORM_COUNT {
        let (next_view, next_id) = if step % 2 == 0 {
            (&second, SECOND_VIEW)
        } else {
            (&first, FIRST_VIEW)
        };
        let next = submission(
            "query-lifecycle-active",
            ACTIVE_COMPONENT,
            &[next_id],
            session.capabilities(),
        );
        let mut next = prepare_catalog(&session, next);
        admit_candidate_resource(&mut next.0, next_view, &mut workspace);
        let next = lower_and_stage(&session, next);
        let cutover = activate(&mut session, next.0, next.1);
        let transitions = cutover.structural_reuse().transitions();
        assert!(transitions
            .iter()
            .any(|transition| transition.transition() == WorthUiPlanRegionTransition::Retired));
        assert!(transitions
            .iter()
            .any(|transition| transition.transition() == WorthUiPlanRegionTransition::Inserted));
        assert!(
            transitions.len() <= 3,
            "Query replacement stays closure-bounded"
        );
        let retirement = cutover.into_operation_live_retirement();
        assert_eq!(retirement.len(), 1);
        close_retirement(retirement, &mut workspace);
        assert_visible_query_execution(&mut session);
        assert_active_operation_live_resource(&session);
    }

    let remove = submission(
        "query-lifecycle-active",
        ACTIVE_COMPONENT,
        &[],
        session.capabilities(),
    );
    let remove = lower_and_stage(&session, prepare_catalog(&session, remove));
    close_retirement(
        activate(&mut session, remove.0, remove.1).into_operation_live_retirement(),
        &mut workspace,
    );
    let _ = session.shutdown();
}
