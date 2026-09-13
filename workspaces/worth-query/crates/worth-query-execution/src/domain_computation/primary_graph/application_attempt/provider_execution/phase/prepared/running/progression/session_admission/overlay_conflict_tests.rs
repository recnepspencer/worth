use super::super::{
    admit_provider_session, registered, WorthQueryProviderAttemptRegistrationContext,
};
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
fn second_real_overlay_is_rejected_without_orphaning_the_first_overlay() {
    let world = installed_authorization_world(true);
    let request = live_scope();
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
    let prepared = prepare_application_commit(
        &world.application,
        WorthQueryApplicationCommitPreparationRequest::new(
            program,
            idempotency(185, 186),
            None,
            None,
        ),
    );
    let WorthQueryApplicationCommitPreparation::Ready(prepared) = prepared else {
        panic!("overlay fixture must reach ordinary prepared posture")
    };
    let running = start_managed_application_commit(&world.application, prepared)
        .unwrap_or_else(|outcome| panic!("overlay fixture must start: {outcome:?}"));
    let WorthQueryRunningApplicationCommit {
        admission,
        lease: _lease,
        provider_attempt,
        mut authorization,
        idempotency,
        mut running,
        mutation_run,
        attempt_basis,
        aftermath_causality,
    } = running;
    let admitted_session = admit_provider_session(
        &mut running,
        &world.application.primary_graph_authority,
        attempt_basis.retained_product(),
        mutation_run,
    )
    .unwrap_or_else(|_| panic!("overlay fixture must admit its real provider session"));
    let registered_session = admitted_session
        .register(
            &mut authorization,
            provider_attempt,
            attempt_basis,
            WorthQueryProviderAttemptRegistrationContext::new(
                &world.application.primary_provider,
                &admission,
                idempotency,
                aftermath_causality.as_ref(),
            ),
        )
        .unwrap_or_else(|_| panic!("overlay fixture must register"));
    registered::assert_second_real_overlay_is_rejected(registered_session.registered, &world);
}
