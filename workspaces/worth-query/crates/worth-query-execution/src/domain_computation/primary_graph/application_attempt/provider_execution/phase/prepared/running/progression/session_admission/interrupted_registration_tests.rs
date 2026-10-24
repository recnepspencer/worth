//! A request stopped before registration admits its decision-fact binding
//! keeps that control outcome and releases every attempt-local resource.
use super::super::{
    WorthQueryProgressedApplicationCommit, WorthQueryProviderProgressionCompletion,
};
use super::admit_provider_session;
use super::registration_conflict_tests::{equivalent_programs, start};
use crate::domain_computation::primary_graph::application_attempt::provider_execution::phase::{
    finish_application_commit, WorthQueryRunningApplicationCommit,
};
use crate::domain_computation::primary_graph::tests::application_attempt::idempotency;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitOutcome, WorthQueryProviderAttemptRegistrationContext,
    WorthQueryProviderProgressionOutcome,
};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

#[test]
fn cancellation_at_registration_binding_is_cancelled_not_denied() {
    let world =
        crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world(
            true,
        );
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(None, cancellation.token());
    world
        .application
        .with_host_advancement(|active_phase| {
            let host_custody = active_phase.test_host_request();
            let phase = &active_phase;
            let head = || {
                world
                    .selected_product(&host_custody)
                    .product()
                    .selected_commit()
                    .clone()
            };
            let head_before = head();
            let (program, _) = equivalent_programs(
                &host_custody.phase_for_application(&world.application),
                &world,
                &request,
                "interrupted-registration",
            );
            let WorthQueryRunningApplicationCommit {
                admission,
                lease,
                provider_attempt,
                mut authorization,
                idempotency,
                mut running,
                mutation_run,
                attempt_basis,
                aftermath_causality,
                outcome_identity,
                workflow_settlement_publication: _,
            } = start(phase, &world, program, idempotency(185, 186));
            let admitted = admit_provider_session(
                phase,
                &mut running,
                &world.application.primary_graph_authority,
                attempt_basis.retained_product(),
                mutation_run,
            )
            .unwrap_or_else(|_| panic!("the live session must reach provider registration"));
            // The request stops after session admission, so the first owner
            // consult to observe it is registration's decision-fact binding.
            cancellation.cancel();
            let Err(failure) = admitted.register(
                &mut authorization,
                provider_attempt,
                attempt_basis,
                WorthQueryProviderAttemptRegistrationContext::new(
                    &world.application.primary_provider,
                    &admission,
                    idempotency,
                    outcome_identity,
                    aftermath_causality.as_ref(),
                ),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            ) else {
                panic!("a cancelled request must not register its provider attempt")
            };
            let WorthQueryProviderProgressionCompletion { outcome, cleanup } =
                failure.into_completion();
            assert!(
                matches!(outcome, WorthQueryProviderProgressionOutcome::Cancelled),
                "the binding's request stop is a control outcome, never a denial"
            );
            let finished = finish_application_commit(
                &phase.borrowed_request_for_test(),
                phase,
                &world.application,
                WorthQueryProgressedApplicationCommit {
                    workflow_settlement_publication: None,
                    outcome,
                    lease,
                    running,
                    cleanup,
                    request: admission.publication_request().clone(),
                },
            );
            assert!(matches!(
                finished,
                WorthQueryApplicationCommitOutcome::Cancelled
            ));
            assert_eq!(
                world
                    .application
                    .primary_provider
                    .application_attempt_resource_count(),
                0
            );
            assert_eq!(
                world.application.provider_session_resource_count_for_test(),
                0
            );
            assert_eq!(head(), head_before, "nothing may commit");
        })
        .expect("fixture owner admits its advancement");
}
