use crate::runtime::{
    WorthQueryGraphReadAccessAdmissionPosture, WorthQueryGraphReadAccessDenialKind,
    WorthQueryGraphReadAccessRequirementKind, WorthQueryGraphReadCheckpointInterval,
    WorthQueryGraphReadMaterializationPolicy, WorthQueryGraphReadMaterializationRequest,
    WorthQueryGraphReadMaterializationRequestError,
};

use crate::runtime::tests::graph_read_access::support::graph_index_inventory::read_families::predicate_collection_family;
use crate::runtime::tests::graph_read_access::support::graph_index_inventory::runtime_profiles::{
    profile_with_graph_support_temporarily_unavailable, workspace_with_graph_support,
};
use crate::runtime::tests::graph_read_access::support::graph_read_access::async_materialization::{
    async_materialization_workspace, async_required_graph_read_family, inline_graph_read_family,
};

#[test]
fn async_required_admission_derives_materialization_request_with_budget_proof() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = async_materialization_workspace(
        request_execution,
        "graph-read-access.async-materialization.request-proof",
    );
    let family = async_required_graph_read_family(&mut workspace, "async-request-proof");
    let admission = workspace
        .read_family_intent(&family)
        .review()
        .expect("large read should be reviewable")
        .graph_read_access_admission()
        .expect("graph read admission should exist")
        .clone();

    assert_eq!(
        admission
            .denial()
            .expect("large read should require async materialization")
            .kind(),
        &WorthQueryGraphReadAccessDenialKind::BudgetExceeded
    );
    assert_eq!(
        admission.denial().unwrap().suggested_posture(),
        &WorthQueryGraphReadAccessAdmissionPosture::AsyncMaterializationRequired
    );

    let request = WorthQueryGraphReadMaterializationRequest::from_required_admission(
        &admission,
        WorthQueryGraphReadMaterializationPolicy::bounded()
            .with_max_resident_bytes(64 * 1024)
            .with_checkpoint_interval(WorthQueryGraphReadCheckpointInterval::frontier_pages(4)),
    )
    .expect("async-required admission should derive request");

    assert_eq!(request.admission_digest(), admission.digest());
    assert_eq!(
        request.admission_denial_kind(),
        &WorthQueryGraphReadAccessDenialKind::BudgetExceeded
    );
    assert_eq!(
        request.requirement_set_digest(),
        admission.requirement_set().digest().render_support_hex()
    );
    assert_eq!(
        request.inventory_match_report_digest(),
        admission.graph_index_inventory_match_report().digest()
    );
    assert_eq!(
        request.budget_digest(),
        admission.budget_check().budget_digest()
    );
    assert_eq!(
        request.estimated_touched_edges(),
        admission.cost_estimate().intrinsic().edge_touches()
    );
    assert_eq!(
        request.estimated_resident_bytes(),
        admission.cost_estimate().supported().memory().total_bytes()
    );
    assert_eq!(
        request.estimated_emitted_rows(),
        admission
            .cost_estimate()
            .intrinsic()
            .intermediate_set_size()
    );
}

#[test]
fn inline_admission_cannot_be_smuggled_into_async_materialization() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = async_materialization_workspace(
        request_execution,
        "graph-read-access.async-materialization.reject-inline",
    );
    let family = inline_graph_read_family(&mut workspace, "async-reject-inline");
    let admission = workspace
        .read_family_intent(&family)
        .review()
        .expect("inline read should be reviewable")
        .graph_read_access_admission()
        .expect("graph read admission should exist")
        .clone();

    assert!(admission.is_admitted());
    assert_eq!(
        WorthQueryGraphReadMaterializationRequest::from_required_admission(
            &admission,
            WorthQueryGraphReadMaterializationPolicy::bounded(),
        )
        .expect_err("inline admission must not become async request"),
        WorthQueryGraphReadMaterializationRequestError::MissingAsyncMaterializationDenial
    );
}

#[test]
fn required_async_materialization_denial_runs_through_materialization_lifecycle() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = workspace_with_graph_support(
        request_execution,
        "graph-read-access.async-materialization.required-async",
        profile_with_graph_support_temporarily_unavailable(
            WorthQueryGraphReadAccessRequirementKind::PredicateSupport,
        ),
    );
    let family = predicate_collection_family(&mut workspace, "async-required-support");
    let admission = workspace
        .read_family_intent(&family)
        .review()
        .expect("temporarily unavailable support should be reviewable")
        .graph_read_access_admission()
        .expect("graph read admission should exist")
        .clone();

    assert_eq!(
        admission
            .denial()
            .expect("temporary support should require async materialization")
            .kind(),
        &WorthQueryGraphReadAccessDenialKind::RequiredAsyncMaterialization
    );

    let request = WorthQueryGraphReadMaterializationRequest::from_required_admission(
        &admission,
        WorthQueryGraphReadMaterializationPolicy::bounded(),
    )
    .expect("required-async denial should derive materialization request");
    assert_eq!(
        request.admission_denial_kind(),
        &WorthQueryGraphReadAccessDenialKind::RequiredAsyncMaterialization
    );

    let receipt = workspace
        .graph_read_materializations()
        .admit(request_execution, request)
        .expect("required-async request should admit")
        .start()
        .expect("job should start")
        .complete();

    assert_eq!(receipt.admission_digest(), admission.digest());
    assert!(receipt.checkpoint_count() > 0);
}
