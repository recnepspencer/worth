use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::elevated::RequestElevationInput;
use worth_query_execution::facade::domain_computation::WorthQueryElevationRequestOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryRequestedElevation;

pub type World = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;

pub fn commit_exact_request(
    world: &World,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> WorthQueryRequestedElevation {
    commit_request(world, request, crate::domain_computation::primary_graph::tests::authorization::capability_elevation::request_transition::honest_input())
}

pub fn commit_request(
    world: &World,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    input: RequestElevationInput,
) -> WorthQueryRequestedElevation {
    let program = world
        .application
        .with_application_advancement(request, |phase| {
            let custody = phase.test_host_request();
            let phase_custody = &custody;
            let principal =
                authenticated_principal(&(phase_custody).test_host_request(), world, request);
            crate::domain_computation::primary_graph::tests::authorization::capability_elevation::request_transition::request_reads(
                phase_custody,
                world,
                &principal,
                request,
                input,
            )
            .materialize_elevation_request_program()
            .unwrap()
        })
        .expect("the request materialization owns its host call");
    match world
        .application
        .compare_and_commit_elevation_request(program, idempotency(171, 171))
    {
        WorthQueryElevationRequestOutcome::Requested(requested) => requested,
        unexpected => panic!("the canonical request prerequisite must commit: {unexpected:?}"),
    }
}

pub fn resolve_exact_request_identities(
    world: &World,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) {
    world
        .application
        .with_application_advancement(request, |phase| {
            let custody = phase.test_host_request();
            let phase_custody = &custody;
            crate::domain_computation::primary_graph::tests::authorization::capability_elevation::request_transition::resolve_created_identities(
                phase_custody,
                &phase_custody.borrowed_request_for_test(),
                world,
                request,
            );
        })
        .expect("the identity read owns its host call");
}
