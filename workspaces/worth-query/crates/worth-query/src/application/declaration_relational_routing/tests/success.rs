use crate::application::{
    WorthQueryDeclarationRelationalAuthorityFamily, WorthQueryDeclarationRelationalRoutingChecked,
    WorthQueryDeclarationRelationalRoutingInput, WorthQueryDeclarationRelationalTruthClaim,
};

use super::support::{
    domain::{
        admitted_handle, BridgeSourceFamily, GroupedFamily, HistoryFamily, MixedAuthorityFamily,
        MixedFamily, RoutingInput, RuntimeFamily, StrategyFamily,
    },
    proof::checked_from_progressed,
};

#[test]
fn relational_routing_common_lane_reads_like_intent() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let routing = admitted_handle(request_execution, "primary")
        .declare_review_progress_describe_plan_receipt_envelope_and_route_relational_truth(
            RoutingInput::<GroupedFamily>::new("edge:42"),
        )
        .unwrap_or_else(|_| panic!("common relational routing should succeed"));

    assert_eq!(
        routing.truth_claim(),
        WorthQueryDeclarationRelationalTruthClaim::GroupedTruth
    );
    assert_eq!(
        routing.authority_family(),
        WorthQueryDeclarationRelationalAuthorityFamily::GroupedTruth
    );
}

#[test]
fn explicit_and_common_lanes_converge_on_one_routing_digest() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "primary");
    let progressed = handle
        .declare_review_and_progress(RoutingInput::<RuntimeFamily>::new("edge:42"))
        .unwrap_or_else(|_| panic!("progression should admit"));
    let envelope = handle
        .envelope_routes_from_progressed(progressed)
        .unwrap_or_else(|_| panic!("envelope should succeed"));
    let explicit = handle
        .route_relational_truth(WorthQueryDeclarationRelationalRoutingInput::enveloped(
            envelope,
        ))
        .unwrap_or_else(|_| panic!("explicit relational routing should succeed"));
    let common = handle
        .declare_review_progress_describe_plan_receipt_envelope_and_route_relational_truth(
            RoutingInput::<RuntimeFamily>::new("edge:42"),
        )
        .unwrap_or_else(|_| panic!("common relational routing should succeed"));

    assert_eq!(
        explicit.relational_routing_digest(),
        common.relational_routing_digest()
    );
}

#[test]
fn mixed_routes_only_lower_the_relational_slice() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    match checked_from_progressed(
        &admitted_handle(request_execution, "primary"),
        RoutingInput::<MixedFamily>::new("edge:42"),
    ) {
        WorthQueryDeclarationRelationalRoutingChecked::Routed(routing) => {
            assert!(matches!(
                routing.class(),
                crate::application::WorthQueryDeclarationRelationalRoutingClass::MixedAuthorityRelationalTruth
            ));
            assert!(routing.explain().mixed_origin());
        }
        _ => panic!("mixed route plans should still route the relational slice"),
    }
}

#[test]
fn mixed_authority_families_keep_the_common_relational_lane() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let routing = admitted_handle(request_execution, "primary")
        .declare_review_progress_describe_plan_receipt_envelope_and_route_relational_truth(
            RoutingInput::<MixedAuthorityFamily>::new("edge:42"),
        )
        .unwrap_or_else(|_| panic!("mixed-authority relational routing should succeed"));

    assert!(matches!(
        routing.class(),
        crate::application::WorthQueryDeclarationRelationalRoutingClass::MixedAuthorityRelationalTruth
    ));
}

#[test]
fn authority_family_bindings_stay_distinct() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let grouped = admitted_handle(request_execution, "primary")
        .declare_review_progress_describe_plan_receipt_envelope_and_route_relational_truth(
            RoutingInput::<GroupedFamily>::new("edge:42"),
        )
        .unwrap_or_else(|_| panic!("grouped routing should succeed"));
    let history = admitted_handle(request_execution, "primary")
        .declare_review_progress_describe_plan_receipt_envelope_and_route_relational_truth(
            RoutingInput::<HistoryFamily>::new("edge:42"),
        )
        .unwrap_or_else(|_| panic!("history routing should succeed"));
    let strategy = admitted_handle(request_execution, "primary")
        .declare_review_progress_describe_plan_receipt_envelope_and_route_relational_truth(
            RoutingInput::<StrategyFamily>::new("edge:42"),
        )
        .unwrap_or_else(|_| panic!("strategy routing should succeed"));
    let bridge_source = admitted_handle(request_execution, "primary")
        .declare_review_progress_describe_plan_receipt_envelope_and_route_relational_truth(
            RoutingInput::<BridgeSourceFamily>::new("edge:42"),
        )
        .unwrap_or_else(|_| panic!("bridge-source routing should succeed"));

    assert_ne!(
        grouped.relational_routing_digest(),
        history.relational_routing_digest()
    );
    assert_ne!(
        history.relational_routing_digest(),
        strategy.relational_routing_digest()
    );
    assert_ne!(
        strategy.relational_routing_digest(),
        bridge_source.relational_routing_digest()
    );
}

#[test]
fn routing_digest_changes_when_admitted_world_changes() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let primary = admitted_handle(request_execution, "primary")
        .declare_review_progress_describe_plan_receipt_envelope_and_route_relational_truth(
            RoutingInput::<RuntimeFamily>::new("edge:42"),
        )
        .unwrap_or_else(|_| panic!("primary routing should succeed"));
    let alternate = admitted_handle(request_execution, "alternate")
        .declare_review_progress_describe_plan_receipt_envelope_and_route_relational_truth(
            RoutingInput::<RuntimeFamily>::new("edge:42"),
        )
        .unwrap_or_else(|_| panic!("alternate routing should succeed"));

    assert_ne!(
        primary.relational_routing_digest(),
        alternate.relational_routing_digest()
    );
}

#[test]
fn relational_support_report_tracks_claim_and_status() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let report = admitted_handle(request_execution, "primary")
        .relational_truth_support::<RoutingInput<GroupedFamily>>();
    let row = report.rows().first().expect("support row should exist");

    assert_eq!(
        row.truth_claim(),
        WorthQueryDeclarationRelationalTruthClaim::GroupedTruth
    );
    assert_eq!(row.status().as_str(), "admitted");
}
