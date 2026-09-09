use super::super::{
    WorthQueryApplicationCommitProgressionAuthority, WorthQueryProgressedApplicationCommit,
    WorthQueryProviderProgressionCompletion,
};
use super::admit_provider_session;
use crate::domain_computation::primary_graph::application_attempt::provider_execution::phase::{
    finish_application_commit, prepare_application_commit, start_managed_application_commit,
    WorthQueryApplicationCommitPreparation, WorthQueryApplicationCommitPreparationRequest,
    WorthQueryRunningApplicationCommit,
};
use crate::domain_computation::primary_graph::application_attempt::provider_execution::WorthQueryApplicationAttemptBasis;
use crate::domain_computation::primary_graph::tests::application_attempt::preimage_evidence::{
    retained_status_program, RetentionMutationBreadth,
};
use crate::domain_computation::primary_graph::tests::application_attempt::{
    authenticated_principal, idempotency, resolved_account,
};
use crate::domain_computation::primary_graph::tests::fixture::{
    installed_authorization_world, live_scope, Account, AuthorizationWorld,
    ExactStatusRetentionInput, ExactStatusRetentionOperation, IdentityExecutionSchema,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitDenialStage, WorthQueryApplicationCommitOutcome,
    WorthQueryApplicationEffectProgram, WorthQueryApplicationIdempotencyBinding,
    WorthQueryProviderAttemptRegistrationContext,
};

type RetainedProgram = WorthQueryApplicationEffectProgram<
    IdentityExecutionSchema,
    ExactStatusRetentionOperation,
    ExactStatusRetentionInput,
    Account,
>;

type RunningRetainedCommit = WorthQueryRunningApplicationCommit<
    IdentityExecutionSchema,
    ExactStatusRetentionOperation,
    ExactStatusRetentionInput,
    Account,
>;

#[test]
fn occupied_real_registration_cleans_the_conflict_and_preserves_an_interleaved_peer() {
    let world = installed_authorization_world(true);
    let (victim, peer) = equivalent_programs(&world, "occupied-registration");
    let victim = start(&world, victim, idempotency(181, 182));
    let peer = start(&world, peer, idempotency(183, 184));
    let victim = while_peer_is_registered(&world, peer, || {
        reject_occupied_registration(&world, victim)
    });
    assert!(matches!(
        victim,
        WorthQueryApplicationCommitOutcome::Denied(ref denial)
            if denial.stage() == WorthQueryApplicationCommitDenialStage::ProviderPlan
    ));
    assert_eq!(world.application.provider_session_resource_count(), 0);
}

fn reject_occupied_registration(
    world: &AuthorizationWorld,
    victim: RunningRetainedCommit,
) -> WorthQueryApplicationCommitOutcome {
    let WorthQueryRunningApplicationCommit {
        admission,
        lease,
        provider_attempt,
        mut authorization,
        idempotency,
        mut running,
        mutation_run: unbound_run,
        attempt_basis,
        aftermath_causality,
    } = victim;
    let reservation_basis = WorthQueryApplicationAttemptBasis::capture(
        &world.application,
        &admission,
        lease.snapshot(),
        lease.product(),
    )
    .expect("the real running attempt must recapture its own exact basis");
    let admitted_session = admit_provider_session(
        &mut running,
        &world.application.primary_graph_authority,
        attempt_basis.retained_product(),
        unbound_run,
    )
    .unwrap_or_else(|_| panic!("the real victim session must reach provider registration"));
    let reserved_affinity = admitted_session
        .staged
        .bind_application_attempt(reservation_basis)
        .expect("the real terminal session must bind its captured attempt basis");
    let reservation = world
        .application
        .primary_provider
        .reserve_application_attempt(&reserved_affinity)
        .expect("the rightful reservation must occupy the actual attempt store");
    assert_eq!(
        world
            .application
            .primary_provider
            .application_attempt_resource_count(),
        2,
        "the peer registration and victim reservation must coexist before cleanup"
    );

    let failure = match admitted_session.register(
        &mut authorization,
        provider_attempt,
        attempt_basis,
        WorthQueryProviderAttemptRegistrationContext::new(
            &world.application.primary_provider,
            &admission,
            idempotency,
            aftermath_causality.as_ref(),
        ),
    ) {
        Ok(_) => panic!("the actual occupied store must reject duplicate registration"),
        Err(failure) => failure,
    };
    let WorthQueryProviderProgressionCompletion { outcome, cleanup } = failure.into_completion();
    assert!(matches!(
        outcome,
        crate::domain_computation::primary_graph::WorthQueryProviderProgressionOutcome::Denied(
            ref denial
        ) if denial.stage() == WorthQueryApplicationCommitDenialStage::ProviderPlan
    ));
    assert_eq!(
        world
            .application
            .primary_provider
            .application_attempt_resource_count(),
        1,
        "duplicate-registration cleanup must release only its exact stale reservation"
    );
    drop(reservation);

    finish_application_commit(
        &world.application,
        WorthQueryProgressedApplicationCommit {
            outcome,
            lease,
            running,
            cleanup,
        },
    )
}

fn while_peer_is_registered(
    world: &AuthorizationWorld,
    peer: RunningRetainedCommit,
    while_registered: impl FnOnce() -> WorthQueryApplicationCommitOutcome,
) -> WorthQueryApplicationCommitOutcome {
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
    } = peer;
    let admitted = admit_provider_session(
        &mut running,
        &world.application.primary_graph_authority,
        attempt_basis.retained_product(),
        mutation_run,
    )
    .unwrap_or_else(|_| panic!("the interleaved peer session must reach registration"));
    let registered = admitted
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
        .unwrap_or_else(|_| panic!("the interleaved peer must register in the actual store"));
    let authorization = authorization
        .finish_registration()
        .expect("the peer registration must preserve its authorization facts");
    assert_eq!(
        world
            .application
            .primary_provider
            .application_attempt_resource_count(),
        1,
        "the peer must occupy the actual application-attempt store"
    );

    let victim = while_registered();
    assert_eq!(
        world
            .application
            .primary_provider
            .application_attempt_resource_count(),
        1,
        "victim cleanup must preserve the exact registered peer"
    );

    let serialization = world
        .application
        .primary_provider
        .serialize_application_commit();
    let authority = WorthQueryApplicationCommitProgressionAuthority {
        application: &world.application,
        provider: &world.application.primary_provider,
        admission: &admission,
        authorization,
        idempotency,
        serialization: &serialization,
        aftermath_causality,
    };
    let peer = finish_application_commit(
        &world.application,
        registered.progress(&authority).finish(lease, running),
    );
    assert!(matches!(
        peer,
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    victim
}

fn start(
    world: &AuthorizationWorld,
    program: RetainedProgram,
    idempotency: WorthQueryApplicationIdempotencyBinding,
) -> WorthQueryRunningApplicationCommit<
    IdentityExecutionSchema,
    ExactStatusRetentionOperation,
    ExactStatusRetentionInput,
    Account,
> {
    let prepared = prepare_application_commit(
        &world.application,
        WorthQueryApplicationCommitPreparationRequest::new(program, idempotency, None, None),
    );
    let WorthQueryApplicationCommitPreparation::Ready(prepared) = prepared else {
        panic!("registration fixture must reach ordinary prepared posture")
    };
    start_managed_application_commit(&world.application, prepared)
        .unwrap_or_else(|outcome| panic!("registration fixture must start: {outcome:?}"))
}

fn equivalent_programs(
    world: &AuthorizationWorld,
    replacement: &str,
) -> (RetainedProgram, RetainedProgram) {
    let request = live_scope();
    let principal = authenticated_principal(world, &request);
    let account = resolved_account(world, "open", &request);
    (
        retained_status_program(
            world,
            &principal,
            &account,
            &request,
            replacement,
            RetentionMutationBreadth::Narrow,
        ),
        retained_status_program(
            world,
            &principal,
            &account,
            &request,
            replacement,
            RetentionMutationBreadth::Narrow,
        ),
    )
}
