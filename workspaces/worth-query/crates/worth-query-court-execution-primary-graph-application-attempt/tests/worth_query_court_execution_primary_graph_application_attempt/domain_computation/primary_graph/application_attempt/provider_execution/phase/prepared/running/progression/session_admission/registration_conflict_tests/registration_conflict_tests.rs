use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::WorthQueryApplicationCommitProgressionAuthority;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::WorthQueryProgressedApplicationCommit;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::WorthQueryProviderProgressionCompletion;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::session_admission::admit_provider_session;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::progressed::finish_application_commit;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::commit_preparation::prepare_application_commit;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::start_managed_application_commit;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::WorthQueryApplicationCommitPreparation;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::WorthQueryApplicationCommitPreparationRequest;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::WorthQueryRunningApplicationCommit;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::application_attempt_affinity::WorthQueryApplicationAttemptBasis;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::preimage_evidence::retained_status_program;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::preimage_evidence::RetentionMutationBreadth;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::ExactStatusRetentionInput;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::ExactStatusRetentionOperation;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialStage;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationEffectProgram;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationIdempotencyBinding;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::registered::WorthQueryProviderAttemptRegistrationContext;

pub type RetainedProgram = WorthQueryApplicationEffectProgram<
    IdentityExecutionSchema,
    ExactStatusRetentionOperation,
    ExactStatusRetentionInput,
    Account,
>;

pub type RunningRetainedCommit = WorthQueryRunningApplicationCommit<
    IdentityExecutionSchema,
    ExactStatusRetentionOperation,
    ExactStatusRetentionInput,
    Account,
>;

#[test]
fn occupied_real_registration_cleans_the_conflict_and_preserves_an_interleaved_peer() {
    let world =
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(
            true,
        );
    world
        .application
        .with_host_advancement(|active_phase| {
            let host_custody = active_phase.test_host_request_for_test();
            let phase = &active_phase;

            let (victim, peer) = equivalent_programs(
                &host_custody.phase_for_application_for_test(&world.application),
                &world,
                &live_scope(),
                "occupied-registration",
            );
            let victim = start(phase, &world, victim, idempotency(181, 182));
            let peer = start(phase, &world, peer, idempotency(183, 184));
            let victim = while_peer_is_registered(phase, &world, peer, || {
                reject_occupied_registration(phase, &world, victim)
            });
            assert!(matches!(
                victim,
                WorthQueryApplicationCommitOutcome::Denied(ref denial)
                    if denial.stage() == WorthQueryApplicationCommitDenialStage::ProviderPlan
            ));
            assert_eq!(
                world.application.provider_session_resource_count_for_test(),
                0
            );
        })
        .expect("fixture owner admits its advancement");
}

pub fn reject_occupied_registration(
    phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,

    world: &AuthorizationWorld,
    victim: RunningRetainedCommit,
) -> WorthQueryApplicationCommitOutcome {
    let execution = phase;

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
        outcome_identity,
        workflow_settlement_publication: _workflow_settlement_publication,
    } = victim;
    let reservation_basis = WorthQueryApplicationAttemptBasis::capture(
        &world.application,
        &admission,
        lease.snapshot_for_test(),
        lease.product(),
    )
    .expect("the real running attempt must recapture its own exact basis");
    let admitted_session = admit_provider_session(
        execution,
        &mut running,
        &(*world.application.primary_graph_authority_for_test()),
        attempt_basis.retained_product(),
        unbound_run,
    )
    .unwrap_or_else(|_| panic!("the real victim session must reach provider registration"));
    let reserved_affinity = (*admitted_session
        .staged_for_test())
        .bind_application_attempt_for_test(reservation_basis)
        .expect("the real terminal session must bind its captured attempt basis");
    let reservation = (*world
        .application
        .primary_provider_for_test())
        .reserve_application_attempt_for_test(&reserved_affinity)
        .expect("the rightful reservation must occupy the actual attempt store");
    assert_eq!(
        (*world
            .application
            .primary_provider_for_test())
            .application_attempt_resource_count_for_test(),
        2,
        "the peer registration and victim reservation must coexist before cleanup"
    );

    let failure = match admitted_session.register_for_test(
        &mut authorization,
        provider_attempt,
        attempt_basis,
        WorthQueryProviderAttemptRegistrationContext::new_for_test(
            &(*world.application.primary_provider_for_test()),
            &admission,
            idempotency,
            outcome_identity,
            aftermath_causality.as_ref(),
        ),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    ) {
        Ok(_) => panic!("the actual occupied store must reject duplicate registration"),
        Err(failure) => failure,
    };
    let WorthQueryProviderProgressionCompletion { outcome, cleanup } = failure.into_completion_for_test();
    assert!(matches!(
        outcome,
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::outcome::WorthQueryProviderProgressionOutcome::Denied(
            ref denial
        ) if denial.stage() == WorthQueryApplicationCommitDenialStage::ProviderPlan
    ));
    assert_eq!(
        (*world
            .application
            .primary_provider_for_test())
            .application_attempt_resource_count_for_test(),
        1,
        "duplicate-registration cleanup must release only its exact stale reservation"
    );
    drop(reservation);

    finish_application_commit(
        &phase.borrowed_request_for_test(),
        phase,
        &world.application,
        WorthQueryProgressedApplicationCommit {
            workflow_settlement_publication: None,
            outcome,
            lease,
            running,
            cleanup,
            request: admission.publication_request_for_test().clone(),
        },
    )
}

pub fn while_peer_is_registered(
    phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,

    world: &AuthorizationWorld,
    peer: RunningRetainedCommit,
    while_registered: impl FnOnce() -> WorthQueryApplicationCommitOutcome,
) -> WorthQueryApplicationCommitOutcome {
    let execution = phase;

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
        workflow_settlement_publication: _workflow_settlement_publication,
    } = peer;
    let product = attempt_basis.retained_product();
    let admitted = admit_provider_session(
        execution,
        &mut running,
        &(*world.application.primary_graph_authority_for_test()),
        product.retained_clone_for_test(),
        mutation_run,
    )
    .unwrap_or_else(|_| panic!("the interleaved peer session must reach registration"));
    let registered = admitted
        .register_for_test(
            &mut authorization,
            provider_attempt,
            attempt_basis,
            WorthQueryProviderAttemptRegistrationContext::new_for_test(
                &(*world.application.primary_provider_for_test()),
                &admission,
                idempotency,
                outcome_identity,
                aftermath_causality.as_ref(),
            ),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap_or_else(|_| panic!("the interleaved peer must register in the actual store"));
    let authorization = authorization
        .finish_registration()
        .expect("the peer registration must preserve its authorization facts");
    assert_eq!(
        (*world
            .application
            .primary_provider_for_test())
            .application_attempt_resource_count_for_test(),
        1,
        "the peer must occupy the actual application-attempt store"
    );

    let victim = while_registered();
    assert_eq!(
        (*world
            .application
            .primary_provider_for_test())
            .application_attempt_resource_count_for_test(),
        1,
        "victim cleanup must preserve the exact registered peer"
    );

    let commit_lane = (*world
        .application
        .primary_provider_for_test())
        .application_branch_commit_lane_for_test(product.observation_for_test());
    let coordination = commit_lane.enter_for_test();
    let authority = WorthQueryApplicationCommitProgressionAuthority {
        application: &world.application,
        provider: &(*world.application.primary_provider_for_test()),
        admission: &admission,
        authorization,
        idempotency,
        coordination: &coordination,
        product: &product,
        aftermath_causality,
        history: None,
    };
    let peer = finish_application_commit(
        &phase.borrowed_request_for_test(),
        phase,
        &world.application,
        registered
            .progress_for_test(
                &phase.borrowed_request_for_test(),
                &authority,
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .finish_for_test(
                lease,
                running,
                None,
                admission.publication_request_for_test().clone(),
            ),
    );
    assert!(matches!(
        peer,
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    victim
}

pub fn start(
    phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,

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
        &phase.borrowed_request_for_test(),
        phase,
        &world.application,
        WorthQueryApplicationCommitPreparationRequest::new_for_test(program, idempotency, None, None),
    );
    let WorthQueryApplicationCommitPreparation::Ready(prepared) = prepared else {
        panic!("registration fixture must reach ordinary prepared posture")
    };
    start_managed_application_commit(phase, &world.application, prepared)
        .unwrap_or_else(|outcome| panic!("registration fixture must start: {outcome:?}"))
}

pub fn equivalent_programs(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    world: &AuthorizationWorld,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    replacement: &str,
) -> (RetainedProgram, RetainedProgram) {
    let principal = authenticated_principal(&(phase_custody).test_host_request_for_test(), world, request);
    let account = resolved_account(&(phase_custody).test_host_request_for_test(), world, "open", request);
    (
        retained_status_program(
            &(phase_custody).test_host_request_for_test(),
            world,
            &principal,
            &account,
            request,
            replacement,
            RetentionMutationBreadth::Narrow,
        ),
        retained_status_program(
            &(phase_custody).test_host_request_for_test(),
            world,
            &principal,
            &account,
            request,
            replacement,
            RetentionMutationBreadth::Narrow,
        ),
    )
}
