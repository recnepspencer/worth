pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::progressed::finish_application_commit;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::commit_preparation::prepare_application_commit;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::progress_application_commit;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::start_managed_application_commit;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::WorthQueryApplicationCommitPreparation;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::WorthQueryApplicationCommitPreparationRequest;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::WorthQueryRunningApplicationCommit;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::preimage_evidence::retained_status_program;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::preimage_evidence::RetainedAccount;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::preimage_evidence::RetentionMutationBreadth;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::preimage_evidence::RetentionOutputs;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::ExactStatusRetentionInput;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::ExactStatusRetentionOperation;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialStage;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitTerminalKind;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationEffectProgram;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationIdempotencyBinding;
pub use worth_query_execution::facade::domain_computation::WorthQueryPrimaryGraphApplicationRuntime;
pub use worth_execution::ExecutionAllocationPolicy;
pub use std::time::Duration;
pub use std::time::Instant;
pub use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
pub use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

pub type RetainedProgram = WorthQueryApplicationEffectProgram<
    IdentityExecutionSchema,
    ExactStatusRetentionOperation,
    ExactStatusRetentionInput,
    Account,
>;

#[test]
fn preparation_denial_cleanup_preserves_the_interleaved_peer() {
    assert_interleaved_terminal(
        |world| world.faults.reject_next_session_prepare(),
        |outcome| {
            assert!(matches!(
                outcome,
                WorthQueryApplicationCommitOutcome::Denied(denial)
                    if denial.stage() == WorthQueryApplicationCommitDenialStage::ProviderPlan
            ));
        },
    );
}

#[test]
fn owner_admission_denial_cleanup_preserves_the_interleaved_peer() {
    assert_interleaved_terminal(
        |world| world.faults.skip_next_invariant_owner_execution(),
        |outcome| {
            assert!(matches!(
                outcome,
                WorthQueryApplicationCommitOutcome::Denied(denial)
                    if denial.stage() == WorthQueryApplicationCommitDenialStage::InvariantExecution
            ))
        },
    );
}

#[test]
fn relational_invariant_denial_cleanup_preserves_the_interleaved_peer() {
    assert_interleaved_terminal(
        |world| world.faults.violate_next_relational_invariant(),
        |outcome| {
            assert!(matches!(
                outcome,
                WorthQueryApplicationCommitOutcome::Denied(denial)
                    if denial.stage() == WorthQueryApplicationCommitDenialStage::InvariantExecution
            ));
        },
    );
}

#[test]
fn pretransaction_abort_cleanup_preserves_the_interleaved_peer() {
    assert_interleaved_terminal(
        |world| world.faults.reject_next_commit_before_transaction(),
        |outcome| {
            assert!(matches!(
                outcome,
                WorthQueryApplicationCommitOutcome::Aborted
            ))
        },
    );
}

#[test]
fn response_loss_cleanup_preserves_the_interleaved_peer() {
    assert_interleaved_terminal(
        |world| world.faults.lose_next_commit_response(),
        |outcome| {
            assert!(matches!(
                outcome,
                WorthQueryApplicationCommitOutcome::Committed(_)
            ))
        },
    );
}

#[test]
fn post_commit_snapshot_recovery_cleanup_preserves_the_interleaved_peer() {
    assert_interleaved_terminal(
        |world| world.faults.fail_next_post_commit_snapshot(),
        |outcome| {
            assert!(matches!(
                outcome,
                WorthQueryApplicationCommitOutcome::Committed(_)
            ))
        },
    );
}

pub fn assert_interleaved_terminal(
    inject: impl FnOnce(&AuthorizationWorld),
    assert_victim: impl FnOnce(&WorthQueryApplicationCommitOutcome),
) {
    let world =
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(
            true,
        );
    world
        .application
        .with_host_advancement(|active_phase| {
            let phase = &active_phase;
            let baseline = world
                .invariant
                .active_snapshot_count_for_test(&active_phase.borrowed_request_for_test());
            let (victim, peer) = equivalent_programs(phase, &world, "interleaved-fault");
            let victim = start(phase, &world.application, victim, idempotency(145, 146));
            let peer = start(phase, &world.application, peer, idempotency(145, 146));
            let both_attempts = world
                .invariant
                .active_snapshot_count_for_test(&active_phase.borrowed_request_for_test());
            inject(&world);
            let victim = finish_application_commit(
                &phase.borrowed_request_for_test(),
                phase,
                &world.application,
                progress_application_commit(
                    &phase.borrowed_request_for_test(),
                    phase,
                    &world.application,
                    victim,
                    None,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                ),
            );
            assert_victim(&victim);
            assert_only_peer_remains(
                &active_phase.borrowed_request_for_test(),
                &world,
                baseline,
                both_attempts,
            );
            let expected_peer =
                if matches!(victim, WorthQueryApplicationCommitOutcome::Committed(_)) {
                    PeerExpectation::AlreadyCommitted
                } else {
                    PeerExpectation::Committed
                };
            finish_peer(phase, &world, peer, baseline, expected_peer);
        })
        .expect("fixture owner admits its advancement");
}

pub fn assert_only_peer_remains(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    baseline: usize,
    both_attempts: usize,
) {
    let after_victim = world.invariant.active_snapshot_count_for_test(request_execution);
    let owned = both_attempts
        .checked_sub(baseline)
        .expect("interleaved attempts cannot reduce baseline snapshot ownership");
    assert_eq!(
        owned % 2,
        0,
        "the two symmetric attempts must establish equal snapshot ownership"
    );
    let one_attempt = owned / 2;
    assert!(one_attempt > 0, "each attempt must own live snapshots");
    assert_eq!(
        after_victim,
        baseline + one_attempt,
        "victim cleanup must release exactly one attempt's ownership"
    );
}

pub fn finish_peer(
    phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,

    world: &AuthorizationWorld,
    peer: WorthQueryRunningApplicationCommit<
        IdentityExecutionSchema,
        ExactStatusRetentionOperation,
        ExactStatusRetentionInput,
        Account,
    >,
    baseline: usize,
    expectation: PeerExpectation,
) {
    let peer = finish_application_commit(
        &phase.borrowed_request_for_test(),
        phase,
        &world.application,
        progress_application_commit(
            &phase.borrowed_request_for_test(),
            phase,
            &world.application,
            peer,
            None,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        ),
    );
    match (expectation, peer) {
        (PeerExpectation::Committed, WorthQueryApplicationCommitOutcome::Committed(receipt)) => {
            assert_eq!(
                receipt.terminal().kind(),
                WorthQueryApplicationCommitTerminalKind::Executed
            );
        }
        (
            PeerExpectation::AlreadyCommitted,
            WorthQueryApplicationCommitOutcome::AlreadyCommitted(receipt),
        ) => {
            assert_eq!(
                receipt.terminal().kind(),
                WorthQueryApplicationCommitTerminalKind::Recovered
            );
        }
        (PeerExpectation::ProductBasisStale, outcome) => {
            worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::application_attempt::decision_currentness::assert_product_basis_stale(
                outcome,
                "the interleaved peer bound to the product before the winner",
            );
        }
        (expected, actual) => panic!("peer must reach {expected:?}, got {actual:?}"),
    }
    assert_eq!(
        world
            .invariant
            .active_snapshot_count_for_test(&phase.borrowed_request_for_test()),
        baseline
    );
    assert_eq!(
        world.application.provider_session_resource_count_for_test(),
        0
    );
}

#[derive(Clone, Copy, Debug)]
pub enum PeerExpectation {
    Committed,
    AlreadyCommitted,
    ProductBasisStale,
}

pub fn start(
    phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,

    application: &WorthQueryPrimaryGraphApplicationRuntime<IdentityExecutionSchema>,
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
        application,
        WorthQueryApplicationCommitPreparationRequest::new_for_test(program, idempotency, None, None),
    );
    let WorthQueryApplicationCommitPreparation::Ready(prepared) = prepared else {
        panic!("association fixture must reach ordinary prepared posture")
    };
    start_managed_application_commit(phase, application, prepared)
        .unwrap_or_else(|outcome| panic!("association fixture must start: {outcome:?}"))
}

pub fn equivalent_programs(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    world: &AuthorizationWorld,
    replacement: &str,
) -> (RetainedProgram, RetainedProgram) {
    let request = live_scope();
    let principal = authenticated_principal(&(phase_custody).test_host_request_for_test(), world, &request);
    let account = resolved_account(
        &(phase_custody).test_host_request_for_test(),
        world,
        "open",
        &request,
    );
    (
        retained_status_program(
            &(phase_custody).test_host_request_for_test(),
            world,
            &principal,
            &account,
            &request,
            replacement,
            RetentionMutationBreadth::Narrow,
        ),
        retained_status_program(
            &(phase_custody).test_host_request_for_test(),
            world,
            &principal,
            &account,
            &request,
            replacement,
            RetentionMutationBreadth::Narrow,
        ),
    )
}

#[path = "abandoned_running_attempt_preserves_the_interleaved_peer/mod.rs"]
pub mod abandoned_running_attempt_preserves_the_interleaved_peer;
#[path = "cancelled_cleanup_preserves_the_interleaved_peer/mod.rs"]
pub mod cancelled_cleanup_preserves_the_interleaved_peer;
#[path = "equivalent_retry/mod.rs"]
pub mod equivalent_retry;
#[path = "stale_read_set_cleanup_preserves_the_interleaved_peer/mod.rs"]
pub mod stale_read_set_cleanup_preserves_the_interleaved_peer;
