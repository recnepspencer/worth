use crate::application::{
    assert_declaration_aspect_projections, WorthQueryDeclarationRelationalRoutingChecked,
};

use super::support::{
    domain::{
        admitted_handle, ConflictingAspectFamily, ExpandedAspectFamily, MissingAspectFamily,
        RoutingInput, RuntimeFamily,
    },
    proof::checked_from_progressed,
};

#[test]
fn routed_relational_artifacts_expose_authority_scoped_aspect_state() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let routing = admitted_handle(request_execution, "primary")
        .declare_review_progress_describe_plan_receipt_envelope_and_route_relational_truth(
            RoutingInput::<RuntimeFamily>::new("edge:42"),
        )
        .unwrap_or_else(|_| panic!("relational routing should succeed"));

    assert_declaration_aspect_projections(
        routing.aspect_contract().required(),
        &[
            "selection.active_face",
            "selection.neighborhood.local_topology",
        ],
    );
    assert_eq!(
        routing.aspect_coverage_basis(),
        crate::application::WorthQueryDeclarationAspectCoverageBasis::EnvelopePublishedCoverage
    );
    assert_eq!(
        routing.aspect_fit(),
        crate::application::WorthQueryDeclarationAspectFit::CompatibleSuperset
    );
}

#[test]
fn relational_routing_denies_missing_and_conflicting_authority_slices() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    match checked_from_progressed(
        &admitted_handle(request_execution, "primary"),
        RoutingInput::<MissingAspectFamily>::new("edge:42"),
    ) {
        WorthQueryDeclarationRelationalRoutingChecked::Denied(denial) => assert_eq!(
            denial.cause(),
            crate::application::WorthQueryDeclarationRelationalRoutingDenialCause::RelationalAspectGap
        ),
        _ => panic!("missing relational aspect coverage should deny"),
    }

    match checked_from_progressed(
        &admitted_handle(request_execution, "primary"),
        RoutingInput::<ConflictingAspectFamily>::new("edge:42"),
    ) {
        WorthQueryDeclarationRelationalRoutingChecked::Denied(denial) => assert_eq!(
            denial.cause(),
            crate::application::WorthQueryDeclarationRelationalRoutingDenialCause::RelationalAspectGap
        ),
        _ => panic!("conflicting relational aspect coverage should deny"),
    }
}

#[test]
fn relational_routing_digest_changes_with_authority_aspect_truth() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let base = admitted_handle(request_execution, "primary")
        .declare_review_progress_describe_plan_receipt_envelope_and_route_relational_truth(
            RoutingInput::<RuntimeFamily>::new("edge:42"),
        )
        .unwrap_or_else(|_| panic!("base relational routing should succeed"));
    let expanded = admitted_handle(request_execution, "primary")
        .declare_review_progress_describe_plan_receipt_envelope_and_route_relational_truth(
            RoutingInput::<ExpandedAspectFamily>::new("edge:42"),
        )
        .unwrap_or_else(|_| panic!("expanded relational routing should succeed"));

    assert_ne!(
        base.relational_routing_digest(),
        expanded.relational_routing_digest()
    );
}

#[test]
fn relational_support_rows_expose_aspect_gap_mismatch() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let support = admitted_handle(request_execution, "primary")
        .relational_truth_support::<RoutingInput<MissingAspectFamily>>();
    let row = &support.rows()[0];

    assert_eq!(
        row.aspect_mismatch(),
        Some(crate::application::WorthQueryDeclarationAuthorityAspectMismatch::AuthorityAspectGap)
    );
}
