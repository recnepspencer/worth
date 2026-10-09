use super::super::{admit_provider_session, WorthQueryProviderAttemptRegistrationContext};
use crate::domain_computation::primary_graph::application_attempt::provider_execution::phase::{
    prepare_application_commit, start_managed_application_commit,
    WorthQueryApplicationCommitPreparation, WorthQueryApplicationCommitPreparationRequest,
    WorthQueryRunningApplicationCommit,
};
use crate::domain_computation::primary_graph::tests::application_attempt::preimage_evidence::{
    retained_status_program, RetentionMutationBreadth,
};
use crate::domain_computation::primary_graph::tests::application_attempt::{
    authenticated_principal, idempotency, resolved_account,
};
use crate::domain_computation::primary_graph::tests::fixture::{
    installed_authorization_world, live_scope,
};

#[test]
fn primary_comparison_scope_preserves_registration_cancellation_cleanup_and_retry() {
    let world = installed_authorization_world(true);
    let foreign = installed_authorization_world(true);
    let cancellation =
        worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource::new();
    let request =
        worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope::new(
            std::time::Instant::now() + std::time::Duration::from_secs(60),
            cancellation.token(),
        );
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let program = retained_status_program(
        &world,
        &principal,
        &account,
        &request,
        "overlay-owner",
        RetentionMutationBreadth::Narrow,
    );
    world
        .application
        .with_application_advancement(
            &crate::domain_computation::primary_graph::tests::fixture::live_scope(),
            |active_phase| {
                let phase = &active_phase;
                let prepared = prepare_application_commit(
                    phase,
                    &world.application,
                    WorthQueryApplicationCommitPreparationRequest::new(
                        program,
                        idempotency(187, 188),
                        None,
                        None,
                    ),
                );
                let WorthQueryApplicationCommitPreparation::Ready(prepared) = prepared else {
                    panic!("overlay fixture must reach ordinary prepared posture")
                };
                let running = start_managed_application_commit(phase, &world.application, prepared)
                    .unwrap_or_else(|outcome| panic!("overlay fixture must start: {outcome:?}"));
                let WorthQueryRunningApplicationCommit {
                    admission,
                    lease,
                    provider_attempt,
                    mut authorization,
                    idempotency: retained_idempotency,
                    mut running,
                    mutation_run,
                    attempt_basis,
                    aftermath_causality,
                    outcome_identity,
                    workflow_settlement_publication: _workflow_settlement_publication,
                } = running;
                let admitted_session = admit_provider_session(
                    phase,
                    &mut running,
                    &world.application.primary_graph_authority,
                    attempt_basis.retained_product(),
                    mutation_run,
                )
                .unwrap_or_else(|_| panic!("overlay fixture must admit its real provider session"));
                let super::WorthQueryRegisteredProviderSession {
                    registered,
                    mutation_run,
                } = admitted_session
                    .register(
                        &mut authorization,
                        provider_attempt,
                        attempt_basis,
                        WorthQueryProviderAttemptRegistrationContext::new(
                            &world.application.primary_provider,
                            &admission,
                            retained_idempotency,
                            outcome_identity,
                            aftermath_causality.as_ref(),
                        ),
                        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                    )
                    .unwrap_or_else(|_| panic!("overlay fixture must register"));
                registered.assert_scoped_comparison(&world, &foreign, &request, &cancellation);
                mutation_run
                    .finish(
                        running,
                        crate::domain_computation::WorthQueryManagedRunTerminalKind::Failed,
                        lease.release_custody(),
                    )
                    .unwrap();
            },
        )
        .unwrap();
    // The retry is a separate host call after the interrupted attempt closed.
    let fresh = live_scope();
    let principal = authenticated_principal(&world, &fresh);
    let account = resolved_account(&world, "open", &fresh);
    let retry = retained_status_program(
        &world,
        &principal,
        &account,
        &fresh,
        "after-retry",
        RetentionMutationBreadth::Narrow,
    );
    assert!(
        matches!(
            world.application.compare_and_commit_application(
                retry,
                idempotency(187, 188),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation
            ),
            crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome::Committed(
                _
            )
        ),
        "interrupted comparison must leave the key reusable"
    );
}
