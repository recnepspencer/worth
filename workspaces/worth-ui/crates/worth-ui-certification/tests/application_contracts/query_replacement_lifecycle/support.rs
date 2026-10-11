use worth_query::facade::{domain, runtime};
use worth_ui::facade::app::{
    WorthUiActiveApplicationSession, WorthUiPreparedApplicationReplacement, WorthUiVisibleRange,
};
use worth_ui::facade::source::WorthUiWatchedCandidateSubmission;
use worth_ui_certification::scenario::application_authority_closure::candidate_catalog::admit_candidate_catalog;
use worth_ui_query_binding::{
    WorthUiCollectionAllocationPolicy, WorthUiInstalledLiveQueryView,
    WorthUiOperationLiveCloseOutcome, WorthUiOperationLiveOpenRequest,
    WorthUiOperationLiveResource, WorthUiOperationLiveRetirement,
    WorthUiOperationLiveRetirementCloseOutcome, WorthUiQueryAllocationDetail,
    WorthUiQueryConsumerRequirements, WorthUiQueryDenialPresentation,
    WorthUiQueryInspectionRelevance, WorthUiQueryViewShape,
};
use worth_ui_runtime::facade::application::WorthUiVirtualizedPlanSummaryRequest;
use worth_ui_runtime::facade::execution::WorthUiFrameBoundary;
use worth_ui_test_support::{
    WorthUiActiveSessionCertificationExt, WorthUiFrameworkTurnCertificationExt,
};

pub(crate) fn prepare_catalog(
    session: &WorthUiActiveApplicationSession,
    submission: WorthUiWatchedCandidateSubmission,
) -> (
    Box<WorthUiPreparedApplicationReplacement>,
    worth_ui::facade::graph::UiAdmittedAllocationCatalogDelta,
) {
    let mut prepared = session
        .prepare_replacement(submission)
        .expect("replacement prepares");
    let catalog = admit_candidate_catalog(session, &mut prepared);
    (prepared, catalog)
}

pub(crate) fn lower_and_stage(
    session: &WorthUiActiveApplicationSession,
    prepared: (
        Box<WorthUiPreparedApplicationReplacement>,
        worth_ui::facade::graph::UiAdmittedAllocationCatalogDelta,
    ),
) -> (
    worth_ui::facade::app::WorthUiPendingApplicationCutover,
    worth_ui::facade::graph::UiAdmittedAllocationCatalogDelta,
) {
    let lowered = session
        .lower_prepared_replacement(*prepared.0)
        .expect("candidate lowers");
    let pending = session
        .stage_prepared_replacement(lowered)
        .expect("candidate stages");
    (pending, prepared.1)
}

pub(crate) fn activate(
    session: &mut WorthUiActiveApplicationSession,
    pending: worth_ui::facade::app::WorthUiPendingApplicationCutover,
    catalog: worth_ui::facade::graph::UiAdmittedAllocationCatalogDelta,
) -> worth_ui::facade::app::WorthUiApplicationCutoverReceipt {
    let boundary = activation_boundary(session);
    session
        .activate_prepared_replacement(pending, catalog, boundary, None)
        .expect("candidate cuts over")
        .into_activation()
        .expect("changed Query-backed meaning publishes a successor")
}

pub(crate) fn activation_boundary(
    session: &mut WorthUiActiveApplicationSession,
) -> WorthUiFrameBoundary {
    session
        .execute_framework_turn(|_| {})
        .expect("no mounted presentation lease is active")
        .into_completion()
        .into_execution()
        .unwrap_or_else(|_| panic!("empty turn publishes an execution boundary"))
        .into_activation_boundary()
}

pub(crate) fn admit_active_resource(
    session: &mut WorthUiActiveApplicationSession,
    view: &WorthUiInstalledLiveQueryView,
    workspace: &mut runtime::WorthQueryWorkspace,
) -> worth_ui_query_binding::WorthUiInstalledQueryBindingReference {
    let resource = open_resource(view, workspace);
    let reference = resource.installed_reference().clone();
    let mut admitted = false;
    let completion = session
        .execute_framework_turn(|turn| {
            turn.query_projection(|source| {
                admitted = source.admit_operation_live(resource).is_ok();
            });
        })
        .expect("no mounted presentation lease is active");
    drop(completion.into_completion());
    assert!(admitted);
    reference
}

pub(super) fn admit_candidate_resource(
    candidate: &mut WorthUiPreparedApplicationReplacement,
    view: &WorthUiInstalledLiveQueryView,
    workspace: &mut runtime::WorthQueryWorkspace,
) {
    let resource = open_resource(view, workspace);
    candidate
        .admit_candidate_operation_live(resource)
        .expect("candidate owns its Query resource before publication");
}

pub(super) fn assert_visible_query_execution(session: &mut WorthUiActiveApplicationSession) {
    let summary = session
        .inspect_virtualized_plan(WorthUiVirtualizedPlanSummaryRequest::first_view())
        .expect("active Query summary");
    let target = summary.target(WorthUiVisibleRange::rows(0, 1).expect("visible range"));
    session
        .execute_framework_turn(|_| {})
        .expect("no mounted presentation lease is active")
        .into_execution()
        .unwrap_or_else(|_| panic!("empty turn executes"))
        .execute_virtualized_data_frame(target)
        .expect("visible Query frame executes");
}

pub(crate) fn open_resource(
    view: &WorthUiInstalledLiveQueryView,
    workspace: &mut runtime::WorthQueryWorkspace,
) -> WorthUiOperationLiveResource {
    view.open_operation(operation_live_request(), workspace)
        .unwrap_or_else(|stopped| panic!("live resource open stopped: {stopped:?}"))
}

pub(crate) fn assert_active_operation_live_resource(session: &WorthUiActiveApplicationSession) {
    let residue = session.inspect_query_state_residue();
    assert_eq!(residue.scanned_live_resources(), 1);
    assert!(residue.is_clean());
}

pub(super) fn close(
    resource: WorthUiOperationLiveResource,
    workspace: &mut runtime::WorthQueryWorkspace,
) {
    assert!(matches!(
        resource.close(workspace),
        WorthUiOperationLiveCloseOutcome::Closed(_)
    ));
}

pub(crate) fn close_retirement(
    retirement: WorthUiOperationLiveRetirement,
    workspace: &mut runtime::WorthQueryWorkspace,
) {
    let WorthUiOperationLiveRetirementCloseOutcome::Closed(receipt) = retirement.close(workspace)
    else {
        panic!("retired Query resource must close")
    };
    assert_eq!(receipt.closed_resource_count(), 1);
    let query_receipt = receipt
        .query_close_receipts()
        .next()
        .expect("Query owns the exact close proof");
    assert!(query_receipt.owner_terminal());
    assert_eq!(query_receipt.counters().close_completions, 1);
}

fn operation_live_request() -> WorthUiOperationLiveOpenRequest {
    WorthUiOperationLiveOpenRequest::new(
        WorthUiQueryConsumerRequirements::new(
            domain::WorthQueryConsumerBoundaryRequirements {
                presentation: domain::WorthQueryConsumerPresentationPosture::Interactive,
                allocation: domain::WorthQueryConsumerAllocationPosture::Borrowed,
            },
            WorthUiQueryAllocationDetail::BorrowedFactSlice,
            WorthUiQueryViewShape::Collection,
            WorthUiQueryDenialPresentation::StructuredStatus,
            WorthUiQueryInspectionRelevance::Relevant,
        ),
        domain::WorthQueryCollectionWindowBreadth::new(1, 0, 0, 1).unwrap(),
        WorthUiCollectionAllocationPolicy::PreserveAdmittedRows,
    )
}
