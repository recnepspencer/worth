use super::*;

pub fn retained_program(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    world: &AuthorizationWorld,
    replacement: &str,
) -> RetainedProgram {
    let request = live_scope();
    let principal = authenticated_principal(
        &(phase_custody).test_host_request_for_test(),
        world,
        &request,
    );
    let account = resolved_account(
        &(phase_custody).test_host_request_for_test(),
        world,
        "open",
        &request,
    );
    retained_status_program(
        &(phase_custody).test_host_request_for_test(),
        world,
        &principal,
        &account,
        &request,
        replacement,
        RetentionMutationBreadth::Narrow,
    )
}
