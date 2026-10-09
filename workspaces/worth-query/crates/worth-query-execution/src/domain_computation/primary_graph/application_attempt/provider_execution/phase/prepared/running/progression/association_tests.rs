use crate::domain_computation::primary_graph::application_attempt::provider_execution::phase::{
    finish_application_commit, prepare_application_commit, progress_application_commit,
    start_managed_application_commit, WorthQueryApplicationCommitPreparation,
    WorthQueryApplicationCommitPreparationRequest, WorthQueryRunningApplicationCommit,
};
use crate::domain_computation::primary_graph::tests::application_attempt::preimage_evidence::{
    retained_status_program, RetainedAccount, RetentionMutationBreadth, RetentionOutputs,
};
use crate::domain_computation::primary_graph::tests::application_attempt::{
    authenticated_principal, idempotency, resolved_account,
};
use crate::domain_computation::primary_graph::tests::fixture::{
    live_scope, Account, AuthorizationWorld, ExactStatusRetentionInput,
    ExactStatusRetentionOperation, IdentityExecutionSchema,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitDenialStage, WorthQueryApplicationCommitOutcome,
    WorthQueryApplicationCommitTerminalKind, WorthQueryApplicationEffectProgram,
    WorthQueryApplicationIdempotencyBinding, WorthQueryPrimaryGraphApplicationRuntime,
};
use crate::facade::runtime::ExecutionAllocationPolicy;
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

fn assert_interleaved_terminal(
    inject: impl FnOnce(&AuthorizationWorld),
    assert_victim: impl FnOnce(&WorthQueryApplicationCommitOutcome),
) {
    let world =
        crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world(
            true,
        );
    world
        .application
        .with_host_advancement(|active_phase| {
            let phase = &active_phase;
            let baseline = world.invariant.active_snapshot_count();
            let (victim, peer) = equivalent_programs(&world, "interleaved-fault");
            let victim = start(phase, &world.application, victim, idempotency(145, 146));
            let peer = start(phase, &world.application, peer, idempotency(145, 146));
            let both_attempts = world.invariant.active_snapshot_count();
            inject(&world);
            let victim = finish_application_commit(
                phase,
                &world.application,
                progress_application_commit(
                    phase,
                    &world.application,
                    victim,
                    crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                ),
            );
            assert_victim(&victim);
            assert_only_peer_remains(&world, baseline, both_attempts);
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
    phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

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
        phase,
        &world.application,
        progress_application_commit(
            phase,
            &world.application,
            peer,
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
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
    phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

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
        phase,
        application,
        WorthQueryApplicationCommitPreparationRequest::new(program, idempotency, None, None),
    );
    let WorthQueryApplicationCommitPreparation::Ready(prepared) = prepared else {
        panic!("association fixture must reach ordinary prepared posture")
    };
    start_managed_application_commit(phase, application, prepared)
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

#[path = "association_tests/abandoned_running_attempt_preserves_the_interleaved_peer.rs"]
mod abandoned_running_attempt_preserves_the_interleaved_peer;

#[path = "association_tests/cancelled_cleanup_preserves_the_interleaved_peer.rs"]
mod cancelled_cleanup_preserves_the_interleaved_peer;

#[path = "association_tests/equivalent_retry.rs"]
mod equivalent_retry;
#[path = "association_tests/stale_read_set_cleanup_preserves_the_interleaved_peer.rs"]
mod stale_read_set_cleanup_preserves_the_interleaved_peer;
