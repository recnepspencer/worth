use super::*;

pub fn retained_program(
    phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    world: &AuthorizationWorld,
    replacement: &str,
) -> RetainedProgram {
    let request = live_scope();
    let principal = authenticated_principal(
        phase_custody,
        world,
        &request,
    );
    let account = resolved_account(
        phase_custody,
        world,
        "open",
        &request,
    );
    retained_status_program(
        phase_custody,
        world,
        &principal,
        &account,
        &request,
        replacement,
        RetentionMutationBreadth::Narrow,
    )
}
