use crate::domain_computation::primary_graph::application_attempt::provider_execution::phase::{
    finish_application_commit, prepare_application_commit, progress_application_commit,
    start_managed_application_commit, WorthQueryApplicationCommitPreparation,
    WorthQueryApplicationCommitPreparationRequest, WorthQueryRunningApplicationCommit,
};
use crate::domain_computation::primary_graph::tests::application_attempt::preimage_evidence::{
    retained_status_program, RetentionMutationBreadth, RETAINED_ACCOUNT_OUTPUT,
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
    WorthQueryApplicationCommitTerminalKind, WorthQueryApplicationEffectProgram,
    WorthQueryApplicationIdempotencyBinding, WorthQueryPrimaryGraphApplicationRuntime,
};
use std::time::{Duration, Instant};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

type RetainedProgram = WorthQueryApplicationEffectProgram<
    IdentityExecutionSchema,
    ExactStatusRetentionOperation,
    ExactStatusRetentionInput,
    Account,
>;

#[test]
fn genuinely_interleaved_equivalent_sessions_validate_retry_cleanup_separately() {
    let world = installed_authorization_world(true);
    let snapshot_baseline = world.invariant.active_snapshot_count();
    let (left, right) = equivalent_programs(&world, "racing-equivalent");
    let binding = idempotency(139, 140);
    let left = start(&world.application, left, binding);
    let right = start(&world.application, right, binding);
    let left = progress_application_commit(&world.application, left);
    let right = progress_application_commit(&world.application, right);
    assert_eq!(
        world.invariant.active_snapshot_count(),
        snapshot_baseline + 2
    );
    let left = finish_application_commit(&world.application, left);
    assert_eq!(
        world.invariant.active_snapshot_count(),
        snapshot_baseline + 1,
        "finishing one attempt must preserve its interleaved peer's snapshot lease"
    );
    let right = finish_application_commit(&world.application, right);
    assert_eq!(world.invariant.active_snapshot_count(), snapshot_baseline);
    assert_eq!(world.application.provider_session_resource_count(), 0);

    let (executed, recovered) = match (left, right) {
        (
            WorthQueryApplicationCommitOutcome::Committed(executed),
            WorthQueryApplicationCommitOutcome::AlreadyCommitted(recovered),
        )
        | (
            WorthQueryApplicationCommitOutcome::AlreadyCommitted(recovered),
            WorthQueryApplicationCommitOutcome::Committed(executed),
        ) => (executed, recovered),
        unexpected => panic!("expected one executed and one recovered commit: {unexpected:?}"),
    };
    assert!(executed.is_same_authoritative_commit(&recovered));
    assert_eq!(executed.retained_preimage(), recovered.retained_preimage());
    assert_eq!(executed.dispatch_outbox(), recovered.dispatch_outbox());
    let executed_output = executed
        .output_correspondence()
        .entity(RETAINED_ACCOUNT_OUTPUT)
        .expect("fresh receipt must retain the bound output role");
    let recovered_output = recovered
        .output_correspondence()
        .entity(RETAINED_ACCOUNT_OUTPUT)
        .expect("idempotent receipt must recover the same output role");
    assert_eq!(executed_output.entity_id(), recovered_output.entity_id());
    assert_eq!(executed.terminal().attempt_resources_released(), Some(true));
    assert_eq!(
        recovered.terminal().attempt_resources_released(),
        Some(true)
    );
    assert_eq!(
        executed.terminal().kind(),
        WorthQueryApplicationCommitTerminalKind::Executed
    );
    assert_eq!(
        recovered.terminal().kind(),
        WorthQueryApplicationCommitTerminalKind::Recovered
    );
}

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
fn index_publication_recovery_cleanup_preserves_the_interleaved_peer() {
    assert_interleaved_terminal(
        |world| world.faults.fail_next_index_publication(),
        |outcome| {
            assert!(matches!(
                outcome,
                WorthQueryApplicationCommitOutcome::Committed(_)
            ))
        },
    );
}

#[test]
fn stale_read_set_cleanup_preserves_the_interleaved_peer() {
    let world = installed_authorization_world(true);
    let baseline = world.invariant.active_snapshot_count();
    let (victim, peer) = equivalent_programs(&world, "stale-interleaved");
    let victim = start(&world.application, victim, idempotency(147, 148));
    let peer = start(&world.application, peer, idempotency(149, 150));
    let both_attempts = world.invariant.active_snapshot_count();
    let winner = equivalent_programs(&world, "stale-winner").0;
    assert!(matches!(
        world
            .application
            .compare_and_commit_application(winner, idempotency(151, 152)),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    let victim = finish_application_commit(
        &world.application,
        progress_application_commit(&world.application, victim),
    );
    crate::domain_computation::primary_graph::tests::application_attempt::assert_product_basis_stale(
        victim,
        "the interleaved victim bound to the product before the winner",
    );
    assert_only_peer_remains(&world, baseline, both_attempts);
    finish_peer(&world, peer, baseline, PeerExpectation::ProductBasisStale);
}

#[test]
fn cancelled_cleanup_preserves_the_interleaved_peer() {
    let world = installed_authorization_world(true);
    let baseline = world.invariant.active_snapshot_count();
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(60),
        cancellation.token(),
    );
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let victim = retained_status_program(
        &world,
        &principal,
        &account,
        &request,
        "cancelled-victim",
        RetentionMutationBreadth::Narrow,
    );
    let peer = equivalent_programs(&world, "cancelled-victim").0;
    let victim = start(&world.application, victim, idempotency(141, 142));
    let peer = start(&world.application, peer, idempotency(141, 142));
    let both_attempts = world.invariant.active_snapshot_count();
    cancellation.cancel();
    let victim = finish_application_commit(
        &world.application,
        progress_application_commit(&world.application, victim),
    );
    assert!(
        matches!(victim, WorthQueryApplicationCommitOutcome::Cancelled),
        "cancelled victim must remain cancelled, got {victim:?}"
    );
    assert_only_peer_remains(&world, baseline, both_attempts);
    finish_peer(&world, peer, baseline, PeerExpectation::Committed);
}

#[test]
fn abandoned_running_attempt_preserves_the_interleaved_peer() {
    let world = installed_authorization_world(true);
    let baseline = world.invariant.active_snapshot_count();
    let (victim, peer) = equivalent_programs(&world, "abandoned-victim");
    let victim = start(&world.application, victim, idempotency(143, 144));
    let peer = start(&world.application, peer, idempotency(143, 144));
    let both_attempts = world.invariant.active_snapshot_count();
    drop(victim);
    assert_only_peer_remains(&world, baseline, both_attempts);
    finish_peer(&world, peer, baseline, PeerExpectation::Committed);
}

fn assert_interleaved_terminal(
    inject: impl FnOnce(&AuthorizationWorld),
    assert_victim: impl FnOnce(&WorthQueryApplicationCommitOutcome),
) {
    let world = installed_authorization_world(true);
    let baseline = world.invariant.active_snapshot_count();
    let (victim, peer) = equivalent_programs(&world, "interleaved-fault");
    let victim = start(&world.application, victim, idempotency(145, 146));
    let peer = start(&world.application, peer, idempotency(145, 146));
    let both_attempts = world.invariant.active_snapshot_count();
    inject(&world);
    let victim = finish_application_commit(
        &world.application,
        progress_application_commit(&world.application, victim),
    );
    assert_victim(&victim);
    assert_only_peer_remains(&world, baseline, both_attempts);
    let expected_peer = if matches!(victim, WorthQueryApplicationCommitOutcome::Committed(_)) {
        PeerExpectation::AlreadyCommitted
    } else {
        PeerExpectation::Committed
    };
    finish_peer(&world, peer, baseline, expected_peer);
}

fn assert_only_peer_remains(world: &AuthorizationWorld, baseline: usize, both_attempts: usize) {
    let after_victim = world.invariant.active_snapshot_count();
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

fn finish_peer(
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
        &world.application,
        progress_application_commit(&world.application, peer),
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
            crate::domain_computation::primary_graph::tests::application_attempt::assert_product_basis_stale(
                outcome,
                "the interleaved peer bound to the product before the winner",
            );
        }
        (expected, actual) => panic!("peer must reach {expected:?}, got {actual:?}"),
    }
    assert_eq!(world.invariant.active_snapshot_count(), baseline);
    assert_eq!(world.application.provider_session_resource_count(), 0);
}

#[derive(Clone, Copy, Debug)]
enum PeerExpectation {
    Committed,
    AlreadyCommitted,
    ProductBasisStale,
}

fn start(
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
        application,
        WorthQueryApplicationCommitPreparationRequest::new(program, idempotency, None, None),
    );
    let WorthQueryApplicationCommitPreparation::Ready(prepared) = prepared else {
        panic!("association fixture must reach ordinary prepared posture")
    };
    start_managed_application_commit(application, prepared)
        .unwrap_or_else(|outcome| panic!("association fixture must start: {outcome:?}"))
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
