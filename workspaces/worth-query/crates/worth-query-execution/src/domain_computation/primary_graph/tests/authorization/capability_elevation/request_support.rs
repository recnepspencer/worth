use super::super::super::application_attempt::{authenticated_principal, idempotency};
use super::super::super::fixture::RequestElevationInput;
use crate::domain_computation::primary_graph::{
    WorthQueryElevationRequestOutcome, WorthQueryRequestedElevation,
};

type World = super::super::super::fixture::AuthorizationWorld;

pub(super) fn commit_exact_request(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    world: &World,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> WorthQueryRequestedElevation {
    commit_request(
        &(phase_custody).test_host_request(),
        world,
        request,
        super::request_transition::honest_input(),
    )
}

pub(super) fn commit_request(
    phase_custody: &crate::domain_computation::primary_graph::TestHostRequest<'_>,
    world: &World,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    input: RequestElevationInput,
) -> WorthQueryRequestedElevation {
    let principal = authenticated_principal(&(phase_custody).test_host_request(), world, request);
    let program =
        super::request_transition::request_reads(phase_custody, world, &principal, request, input)
            .materialize_elevation_request_program()
            .unwrap();
    match world
        .application
        .compare_and_commit_elevation_request(program, idempotency(171, 171))
    {
        WorthQueryElevationRequestOutcome::Requested(requested) => requested,
        unexpected => panic!("the canonical request prerequisite must commit: {unexpected:?}"),
    }
}

pub(super) fn resolve_exact_request_identities(
    phase_custody: &crate::domain_computation::primary_graph::TestHostRequest<'_>,
    world: &World,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) {
    super::request_transition::resolve_created_identities(
        phase_custody,
        &phase_custody.borrowed_request_for_test(),
        world,
        request,
    );
}
