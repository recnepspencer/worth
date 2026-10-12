use crate::application::{
    assert_declaration_aspect_projections, test_declaration_aspect_key,
    WorthQueryDeclarationAspectFit, WorthQueryDeclarationRoutePlanChecked,
    WorthQueryDeclarationRoutePlanDenialCause,
};

use super::domain::{
    admitted_handle, route_input, AspectRichRouteFamily, ConflictAspectRouteFamily,
    MissingAspectRouteFamily, RelationalRouteFamily, RouteInput,
};

#[test]
fn route_plans_expose_route_scoped_aspect_contract_fit_and_publication() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let plan = admitted_handle(request_execution, "primary")
        .declare_review_progress_describe_and_plan(RouteInput::<AspectRichRouteFamily>::new(
            "edge:42",
        ))
        .unwrap_or_else(|_| panic!("aspect-rich route plan should succeed"));

    assert_eq!(
        plan.aspect_fit(),
        WorthQueryDeclarationAspectFit::CompatibleSuperset
    );
    assert_declaration_aspect_projections(
        plan.aspect_contract().required(),
        &["selection.active_edge"],
    );
    assert_declaration_aspect_projections(
        plan.aspect_contract().preserved(),
        &["selection.local_topology"],
    );
    assert!(plan.aspect_contract().published().is_empty());
    assert!(!plan
        .aspect_publication()
        .present()
        .contains(&test_declaration_aspect_key("selection.material_edit")));
    assert!(plan
        .aspect_publication()
        .masked()
        .contains(&test_declaration_aspect_key("selection.private_authority")));
}

#[test]
fn route_planning_denies_when_required_aspects_are_missing() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "primary");

    match handle.plan_routes_checked(route_input(
        &handle,
        RouteInput::<MissingAspectRouteFamily>::new("edge:42"),
    )) {
        WorthQueryDeclarationRoutePlanChecked::Denied(denial) => {
            assert_eq!(
                denial.cause(),
                WorthQueryDeclarationRoutePlanDenialCause::MissingRequiredAspect
            );
        }
        _ => panic!("missing required aspects should deny route planning"),
    }
}

#[test]
fn route_planning_denies_when_aspect_contract_conflicts_with_coverage() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "primary");

    match handle.plan_routes_checked(route_input(
        &handle,
        RouteInput::<ConflictAspectRouteFamily>::new("edge:42"),
    )) {
        WorthQueryDeclarationRoutePlanChecked::Denied(denial) => {
            assert_eq!(
                denial.cause(),
                WorthQueryDeclarationRoutePlanDenialCause::AspectConflict
            );
        }
        _ => panic!("conflicting aspect coverage should deny route planning"),
    }
}

#[test]
fn route_plan_digest_changes_when_route_scoped_aspect_truth_changes() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let aspectful = admitted_handle(request_execution, "primary")
        .declare_review_progress_describe_and_plan(RouteInput::<AspectRichRouteFamily>::new(
            "edge:42",
        ))
        .unwrap_or_else(|_| panic!("aspect-rich route plan should succeed"));
    let plain = admitted_handle(request_execution, "primary")
        .declare_review_progress_describe_and_plan(RouteInput::<RelationalRouteFamily>::new(
            "edge:42",
        ))
        .unwrap_or_else(|_| panic!("plain route plan should succeed"));

    assert_ne!(aspectful.route_plan_digest(), plain.route_plan_digest());
}
