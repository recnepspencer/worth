use super::{
    start_managed_application_commit, WorthQueryApplicationAttemptBasis,
    WorthQueryRunningApplicationCommit,
};
use crate::domain_computation::primary_graph::application_attempt::provider_execution::phase::{
    prepare_application_commit, WorthQueryApplicationCommitPreparation,
    WorthQueryApplicationCommitPreparationRequest,
};
use crate::domain_computation::primary_graph::application_attempt::provider_execution::application_attempt_affinity::WorthQueryApplicationAttemptAffinityMismatch as M;
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
    WorthQueryApplicationEffectProgram, WorthQueryApplicationIdempotencyBinding,
    WorthQueryApplicationSnapshotLease,
};
use crate::domain_computation::WorthQueryManagedRunTerminalKind;

type RetainedProgram = WorthQueryApplicationEffectProgram<
    IdentityExecutionSchema,
    ExactStatusRetentionOperation,
    ExactStatusRetentionInput,
    Account,
>;

#[test]
fn exact_affinity_accepts_its_real_session_and_rejects_runtime_snapshot_and_branch_drift() {
    let world = installed_authorization_world(true);
    let foreign_world = installed_authorization_world(true);
    let running = start(
        &world,
        retained_program(&world, "axis-owner"),
        idempotency(187, 188),
    );
    let WorthQueryRunningApplicationCommit {
        admission,
        lease,
        provider_attempt: _provider_attempt,
        authorization: _authorization,
        idempotency: _idempotency,
        mut running,
        mutation_run,
        attempt_basis,
        aftermath_causality: _aftermath_causality,
    } = running;
    let staged = real_terminal_session(&world, &mut running);
    assert!(staged.bind_application_attempt(attempt_basis).is_ok());

    assert!(WorthQueryApplicationAttemptBasis::capture(
        &foreign_world.application,
        &admission,
        lease.snapshot(),
        lease.product_publication(),
    )
    .is_err());

    let substitute_lease = WorthQueryApplicationSnapshotLease::acquire(
        lease.handle().clone(),
        std::sync::Arc::clone(&lease.layout),
        lease.product_publication().clone(),
    )
    .expect("the real branch must issue a second exact snapshot handle");
    assert_ne!(substitute_lease.snapshot(), lease.snapshot());
    let substitute_snapshot_basis = WorthQueryApplicationAttemptBasis::capture(
        &world.application,
        &admission,
        substitute_lease.snapshot(),
        substitute_lease.product_publication(),
    )
    .expect("same-runtime same-branch snapshot remains a valid pre-session basis");
    assert!(staged
        .bind_application_attempt(substitute_snapshot_basis)
        .is_err());
    assert!(substitute_lease.release());

    let foreign_branch_snapshot = lease.handle().with_runtime_mut(|runtime| {
        let foreign =
            worth_relational::facade::history::BranchId("foreign-affinity-branch".to_owned());
        let (_, fork_basis) = runtime
            .observe_fork_source(lease.snapshot().branch_id())
            .expect("test runtime can observe the exact source basis");
        runtime
            .fork_branch(foreign.clone(), fork_basis)
            .expect("test runtime can issue a real sibling branch");
        let identity = runtime.branch_identity(&foreign).unwrap();
        let (_, basis) = runtime.observe_branch(&identity).unwrap();
        runtime
            .snapshots()
            .snapshot_for_observation(&basis.observation())
            .unwrap()
    });
    assert!(WorthQueryApplicationAttemptBasis::capture(
        &world.application,
        &admission,
        &foreign_branch_snapshot,
        lease.product_publication(),
    )
    .is_err());
    assert!(lease.handle().with_runtime_mut(|runtime| {
        runtime
            .snapshots()
            .release_snapshot(&foreign_branch_snapshot)
            .is_ok()
    }));
    let _ = staged.abort();
    finish_uncommitted(mutation_run, running, lease);
}

#[test]
fn a_real_peer_plan_and_session_cannot_substitute_for_the_captured_attempt() {
    let world = installed_authorization_world(true);
    let first = start(
        &world,
        retained_program(&world, "plan-owner"),
        idempotency(189, 190),
    );
    let second = start(
        &world,
        retained_program(&world, "plan-peer"),
        idempotency(191, 192),
    );
    let WorthQueryRunningApplicationCommit {
        admission: _first_admission,
        lease: first_lease,
        provider_attempt: _first_provider_attempt,
        authorization: _first_authorization,
        idempotency: _first_idempotency,
        running: mut first_running,
        mutation_run: first_mutation_run,
        attempt_basis: first_basis,
        aftermath_causality: _first_aftermath,
    } = first;
    let WorthQueryRunningApplicationCommit {
        admission: _second_admission,
        lease: second_lease,
        provider_attempt: _second_provider_attempt,
        authorization: _second_authorization,
        idempotency: _second_idempotency,
        running: mut second_running,
        mutation_run: second_mutation_run,
        attempt_basis: second_basis,
        aftermath_causality: _second_aftermath,
    } = second;
    let first_staged = real_terminal_session(&world, &mut first_running);
    let second_staged = real_terminal_session(&world, &mut second_running);
    let first_terminal = first_staged.provider_session_terminal_binding();
    let second_terminal = second_staged.provider_session_terminal_binding();
    assert_ne!(first_terminal, second_terminal);
    let mismatches = first_basis.affinity_mismatches(&second_terminal);
    assert_eq!(
        mismatches,
        [
            M::ResourceBinding,
            M::OperationAttempt,
            M::Snapshot,
            M::GraphWorkSession,
            M::GraphWorkManagedRun
        ]
        .into_iter()
        .collect()
    );
    assert!(second_staged.bind_application_attempt(first_basis).is_err());
    assert!(first_staged.bind_application_attempt(second_basis).is_err());
    let _ = first_staged.abort();
    let _ = second_staged.abort();
    finish_uncommitted(first_mutation_run, first_running, first_lease);
    finish_uncommitted(second_mutation_run, second_running, second_lease);
}

fn real_terminal_session<'run>(
    world: &AuthorizationWorld,
    running: &'run mut crate::domain_computation::WorthQueryRunningDirectRun,
) -> crate::domain_computation::WorthQuerySessionBoundReadsAndEffects<'run> {
    running
        .admit_provider_execution_plan(&world.application.primary_graph_authority)
        .expect("real application authorities must admit their provider plan")
        .readmit()
        .expect("the real provider must readmit its exact plan")
        .prepare()
        .expect("the real provider must prepare its exact session")
        .bind_reads_and_effects()
}

fn finish_uncommitted(
    mutation_run: crate::domain_computation::provider_session::WorthQueryMutationRunBinding,
    running: crate::domain_computation::WorthQueryRunningDirectRun,
    lease: WorthQueryApplicationSnapshotLease,
) {
    mutation_run
        .finish(
            running,
            WorthQueryManagedRunTerminalKind::Failed,
            lease.release(),
        )
        .expect("affinity probe must release managed-run resources");
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
        panic!("affinity fixture must reach ordinary prepared posture")
    };
    start_managed_application_commit(&world.application, prepared)
        .unwrap_or_else(|outcome| panic!("affinity fixture must start: {outcome:?}"))
}

fn retained_program(world: &AuthorizationWorld, replacement: &str) -> RetainedProgram {
    let request = live_scope();
    let principal = authenticated_principal(world, &request);
    let account = resolved_account(world, "open", &request);
    retained_status_program(
        world,
        &principal,
        &account,
        &request,
        replacement,
        RetentionMutationBreadth::Narrow,
    )
}
