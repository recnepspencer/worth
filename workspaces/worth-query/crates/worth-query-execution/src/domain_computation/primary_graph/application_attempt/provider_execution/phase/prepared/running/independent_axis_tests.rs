use super::{start_managed_application_commit, WorthQueryRunningApplicationCommit};
use crate::domain_computation::primary_graph::application_attempt::provider_execution::{
    application_attempt_affinity::{
        WorthQueryApplicationAttemptAffinityMismatch as M, WorthQueryApplicationAttemptAffinityView,
    },
    phase::{
        prepare_application_commit, WorthQueryApplicationCommitPreparation,
        WorthQueryApplicationCommitPreparationRequest,
    },
};
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
use crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding;
use crate::domain_computation::WorthQueryManagedRunTerminalKind;

type RetainedProgram = WorthQueryApplicationEffectProgram<
    IdentityExecutionSchema,
    ExactStatusRetentionOperation,
    ExactStatusRetentionInput,
    Account,
>;

#[derive(Clone)]
struct TestAffinityView {
    runtime: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
    installed_operation: String,
    resource_binding: String,
    operation_attempt:
        Option<crate::domain_computation::authorization::WorthQueryOperationAdmissionIdentity>,
    operation_slot: Option<String>,
    schema_binding: Option<worth_query_installation::facade::ApplicationSchemaBindingIdentity>,
    snapshot: Option<worth_relational::facade::snapshots::SnapshotHandle>,
    graph_work_session: Option<u64>,
    graph_work_managed_run: Option<u64>,
}

impl TestAffinityView {
    fn capture(terminal: &WorthQueryProviderSessionTerminalBinding) -> Self {
        let plan = terminal.plan();
        Self {
            runtime: plan.runtime_authority(),
            installed_operation: plan.operation_identity().to_owned(),
            resource_binding: plan.binding_identity().to_owned(),
            operation_attempt: plan.application_operation_attempt(),
            operation_slot: plan.application_operation_slot().map(str::to_owned),
            schema_binding: plan.application_schema_binding().cloned(),
            snapshot: plan.application_snapshot().cloned(),
            graph_work_session: plan.graph_work_session_identity(),
            graph_work_managed_run: plan.graph_work_managed_run_identity(),
        }
    }
}

impl WorthQueryApplicationAttemptAffinityView for TestAffinityView {
    fn runtime_authority(
        &self,
    ) -> crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity {
        self.runtime
    }
    fn installed_operation(&self) -> &str {
        &self.installed_operation
    }
    fn resource_binding(&self) -> &str {
        &self.resource_binding
    }
    fn operation_attempt(
        &self,
    ) -> Option<crate::domain_computation::authorization::WorthQueryOperationAdmissionIdentity>
    {
        self.operation_attempt
    }
    fn operation_slot(&self) -> Option<&str> {
        self.operation_slot.as_deref()
    }
    fn schema_binding(
        &self,
    ) -> Option<&worth_query_installation::facade::ApplicationSchemaBindingIdentity> {
        self.schema_binding.as_ref()
    }
    fn snapshot(&self) -> Option<&worth_relational::facade::snapshots::SnapshotHandle> {
        self.snapshot.as_ref()
    }
    fn graph_work_session(&self) -> Option<u64> {
        self.graph_work_session
    }
    fn graph_work_managed_run(&self) -> Option<u64> {
        self.graph_work_managed_run
    }
}

#[test]
fn every_plan_session_and_attempt_axis_is_rejected_independently() {
    let world = installed_authorization_world(true);
    let foreign_world = installed_authorization_world(true);
    let owner = start(&world, "independent-owner", idempotency(201, 202));
    let peer = start(&world, "independent-peer", idempotency(203, 204));
    let foreign = start(&foreign_world, "independent-foreign", idempotency(205, 206));

    let (basis, mut owner_running, owner_run, owner_lease) = release(owner);
    let (_peer_basis, mut peer_running, peer_run, peer_lease) = release(peer);
    let (_foreign_basis, mut foreign_running, foreign_run, foreign_lease) = release(foreign);
    let owner_staged = real_terminal_session(&world, &mut owner_running);
    let peer_staged = real_terminal_session(&world, &mut peer_running);
    let foreign_staged = real_terminal_session(&foreign_world, &mut foreign_running);
    let owner_terminal = owner_staged.provider_session_terminal_binding();
    let peer_view = TestAffinityView::capture(&peer_staged.provider_session_terminal_binding());
    let foreign_view =
        TestAffinityView::capture(&foreign_staged.provider_session_terminal_binding());
    let base = TestAffinityView::capture(&owner_terminal);
    assert!(basis.affinity_mismatches_view(&base).is_empty());

    assert_one(&basis, &base, M::Runtime, |view| {
        view.runtime = foreign_view.runtime
    });
    assert_one(&basis, &base, M::InstalledOperation, |view| {
        view.installed_operation.push_str("-foreign")
    });
    assert_one(&basis, &base, M::ResourceBinding, |view| {
        view.resource_binding.push_str("-foreign")
    });
    assert_one(&basis, &base, M::OperationAttempt, |view| {
        view.operation_attempt = peer_view.operation_attempt
    });
    assert_one(&basis, &base, M::OperationSlot, |view| {
        view.operation_slot = Some("foreign-slot".to_owned())
    });
    assert_one(&basis, &base, M::SchemaBinding, |view| {
        view.schema_binding = foreign_view.schema_binding.clone()
    });
    assert_one(&basis, &base, M::Snapshot, |view| {
        view.snapshot = peer_view.snapshot.clone()
    });
    assert_one(&basis, &base, M::GraphWorkSession, |view| {
        view.graph_work_session = peer_view.graph_work_session
    });
    assert_one(&basis, &base, M::GraphWorkManagedRun, |view| {
        view.graph_work_managed_run = peer_view.graph_work_managed_run
    });

    let _ = owner_staged.abort();
    let _ = peer_staged.abort();
    let _ = foreign_staged.abort();
    finish(owner_run, owner_running, owner_lease);
    finish(peer_run, peer_running, peer_lease);
    finish(foreign_run, foreign_running, foreign_lease);
}

fn assert_one(
    basis: &crate::domain_computation::primary_graph::application_attempt::provider_execution::WorthQueryApplicationAttemptBasis,
    base: &TestAffinityView,
    expected: M,
    substitute: impl FnOnce(&mut TestAffinityView),
) {
    let mut candidate = base.clone();
    substitute(&mut candidate);
    assert_eq!(
        basis.affinity_mismatches_view(&candidate),
        [expected].into_iter().collect()
    );
}

fn release(
    running: WorthQueryRunningApplicationCommit<
        IdentityExecutionSchema,
        ExactStatusRetentionOperation,
        ExactStatusRetentionInput,
        Account,
    >,
) -> (
    crate::domain_computation::primary_graph::application_attempt::provider_execution::WorthQueryApplicationAttemptBasis,
    crate::domain_computation::WorthQueryRunningDirectRun,
    crate::domain_computation::provider_session::WorthQueryMutationRunBinding,
    WorthQueryApplicationSnapshotLease,
){
    let WorthQueryRunningApplicationCommit {
        lease,
        running,
        mutation_run,
        attempt_basis,
        ..
    } = running;
    (attempt_basis, running, mutation_run, lease)
}

fn real_terminal_session<'run>(
    world: &AuthorizationWorld,
    running: &'run mut crate::domain_computation::WorthQueryRunningDirectRun,
) -> crate::domain_computation::WorthQuerySessionBoundReadsAndEffects<'run> {
    running
        .admit_provider_execution_plan(&world.application.primary_graph_authority)
        .unwrap()
        .readmit()
        .unwrap()
        .prepare()
        .unwrap()
        .bind_reads_and_effects()
}

fn finish(
    mutation_run: crate::domain_computation::provider_session::WorthQueryMutationRunBinding,
    running: crate::domain_computation::WorthQueryRunningDirectRun,
    lease: WorthQueryApplicationSnapshotLease,
) {
    mutation_run
        .finish(
            running,
            WorthQueryManagedRunTerminalKind::Failed,
            lease.release_custody(),
        )
        .unwrap();
}

fn start(
    world: &AuthorizationWorld,
    replacement: &str,
    idempotency: WorthQueryApplicationIdempotencyBinding,
) -> WorthQueryRunningApplicationCommit<
    IdentityExecutionSchema,
    ExactStatusRetentionOperation,
    ExactStatusRetentionInput,
    Account,
> {
    let request = live_scope();
    let principal = authenticated_principal(world, &request);
    let account = resolved_account(world, "open", &request);
    let program: RetainedProgram = retained_status_program(
        world,
        &principal,
        &account,
        &request,
        replacement,
        RetentionMutationBreadth::Narrow,
    );
    let WorthQueryApplicationCommitPreparation::Ready(prepared) = prepare_application_commit(
        &world.application,
        WorthQueryApplicationCommitPreparationRequest::new(program, idempotency, None, None),
    ) else {
        panic!("affinity fixture must prepare")
    };
    start_managed_application_commit(&world.application, prepared).unwrap()
}
