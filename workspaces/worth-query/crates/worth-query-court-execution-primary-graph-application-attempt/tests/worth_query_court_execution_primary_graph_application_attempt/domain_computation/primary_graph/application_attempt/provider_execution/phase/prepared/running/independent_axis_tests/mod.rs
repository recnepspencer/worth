use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::start_managed_application_commit;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::WorthQueryRunningApplicationCommit;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::application_attempt_affinity::WorthQueryApplicationAttemptAffinityMismatch as M;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::application_attempt_affinity::WorthQueryApplicationAttemptAffinityView;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::entry_custody::preparation::prepare_application_commit;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::WorthQueryApplicationCommitPreparation;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::WorthQueryApplicationCommitPreparationRequest;
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
use worth_query_execution::facade::domain_computation::WorthQueryApplicationEffectProgram;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationIdempotencyBinding;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease;
use worth_query_execution::court_doors_for_test::domain_computation::provider_session::protocol::terminal_binding::WorthQueryProviderSessionTerminalBinding;
use worth_query_execution::facade::domain_computation::WorthQueryManagedRunTerminalKind;

pub type RetainedProgram = WorthQueryApplicationEffectProgram<
    IdentityExecutionSchema,
    ExactStatusRetentionOperation,
    ExactStatusRetentionInput,
    Account,
>;

#[derive(Clone)]
pub struct TestAffinityView {
    pub runtime: worth_query_execution::facade::domain_computation::WorthQueryRuntimeAuthorityIdentity,
    pub installed_operation: String,
    pub resource_binding: String,
    pub operation_attempt:
        Option<worth_query_execution::court_doors_for_test::domain_computation::authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation::WorthQueryOperationAdmissionIdentity>,
    pub operation_slot: Option<String>,
    pub schema_binding: Option<worth_query_installation::facade::ApplicationSchemaBindingIdentity>,
    pub snapshot: Option<worth_relational::facade::snapshots::SnapshotHandle>,
    pub graph_work_session: Option<u64>,
    pub graph_work_managed_run: Option<u64>,
}

impl TestAffinityView {
    pub fn capture(terminal: &WorthQueryProviderSessionTerminalBinding) -> Self {
        let plan = terminal.plan();
        Self {
            runtime: plan.runtime_authority_for_test(),
            installed_operation: plan.operation_identity().to_owned(),
            resource_binding: plan.binding_identity().to_owned(),
            operation_attempt: plan.application_operation_attempt_for_test(),
            operation_slot: plan
                .application_operation_slot_for_test()
                .map(str::to_owned),
            schema_binding: plan.application_schema_binding_for_test().cloned(),
            snapshot: plan.application_snapshot_for_test().cloned(),
            graph_work_session: plan.graph_work_session_identity(),
            graph_work_managed_run: plan.graph_work_managed_run_identity(),
        }
    }
}

impl WorthQueryApplicationAttemptAffinityView for TestAffinityView {
    fn runtime_authority(
        &self,
    ) -> worth_query_execution::facade::domain_computation::WorthQueryRuntimeAuthorityIdentity {
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
    ) -> Option<worth_query_execution::court_doors_for_test::domain_computation::authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation::WorthQueryOperationAdmissionIdentity>
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
    let world =
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(true, worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());
    let foreign_world =
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(true, worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());
    // Foreign evidence is data; its provider executes only under its own opener.
    let foreign_view = foreign_world
        .application
        .with_host_advancement(|phase| {
            let foreign = start(
                &phase,
                &foreign_world,
                "independent-foreign",
                idempotency(205, 206),
            );
            let (_basis, mut running, mutation_run, lease) = release(foreign);
            let staged = real_terminal_session(&phase, &foreign_world, &mut running);
            let view =
                TestAffinityView::capture(&staged.provider_session_terminal_binding_for_test());
            let _ = staged.abort();
            finish(mutation_run, running, lease);
            view
        })
        .expect("foreign fixture owner admits its own advancement");
    world
        .application
        .with_host_advancement(|active_phase| {
            let phase = &active_phase;
            let execution = phase;

            let owner = start(phase, &world, "independent-owner", idempotency(201, 202));
            let peer = start(phase, &world, "independent-peer", idempotency(203, 204));

            let (basis, mut owner_running, owner_run, owner_lease) = release(owner);
            let (_peer_basis, mut peer_running, peer_run, peer_lease) = release(peer);
            let owner_staged = real_terminal_session(execution, &world, &mut owner_running);
            let peer_staged = real_terminal_session(execution, &world, &mut peer_running);
            let owner_terminal = owner_staged.provider_session_terminal_binding_for_test();
            let peer_view = TestAffinityView::capture(
                &peer_staged.provider_session_terminal_binding_for_test(),
            );
            let base = TestAffinityView::capture(&owner_terminal);
            assert!(basis.affinity_mismatches_view_for_test(&base).is_empty());

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
            finish(owner_run, owner_running, owner_lease);
            finish(peer_run, peer_running, peer_lease);
        })
        .expect("fixture owner admits its advancement");
}

pub fn assert_one(
    basis: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::application_attempt_affinity::WorthQueryApplicationAttemptBasis,
    base: &TestAffinityView,
    expected: M,
    substitute: impl FnOnce(&mut TestAffinityView),
) {
    let mut candidate = base.clone();
    substitute(&mut candidate);
    assert_eq!(
        basis.affinity_mismatches_view_for_test(&candidate),
        [expected].into_iter().collect()
    );
}

pub fn release(
    running: WorthQueryRunningApplicationCommit<
        IdentityExecutionSchema,
        ExactStatusRetentionOperation,
        ExactStatusRetentionInput,
        Account,
    >,
) -> (
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::application_attempt_affinity::WorthQueryApplicationAttemptBasis,
    worth_query_execution::facade::domain_computation::WorthQueryRunningDirectRun,
    worth_query_execution::court_doors_for_test::domain_computation::provider_session::graph_obligation::mutation_progression::WorthQueryMutationRunBinding,
    WorthQueryApplicationSnapshotLease,
){
    let (
        _coupled_outputs,
        _admission_rest,
        lease,
        _provider_attempt_rest,
        _outcome_identity_rest,
        _workflow_settlement_publication_rest,
        _authorization_rest,
        _idempotency_rest,
        running,
        mutation_run,
        attempt_basis,
        _aftermath_causality_rest,
    ) = running.into_parts_for_test();
    (attempt_basis, running, mutation_run, lease)
}

pub fn real_terminal_session<'run>(
    execution: &'run worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,

    world: &AuthorizationWorld,
    running: &'run mut worth_query_execution::facade::domain_computation::WorthQueryRunningDirectRun,
) -> worth_query_execution::facade::domain_computation::WorthQuerySessionBoundReadsAndEffects<'run>
{
    running
        .admit_provider_execution_plan(
            (*world.application.runtime_for_test()).primary_graph_authority_for_test(),
        )
        .unwrap()
        .readmit(execution)
        .unwrap()
        .prepare()
        .unwrap()
        .bind_reads_and_effects()
}

pub fn finish(
    mutation_run: worth_query_execution::court_doors_for_test::domain_computation::provider_session::graph_obligation::mutation_progression::WorthQueryMutationRunBinding,
    running: worth_query_execution::facade::domain_computation::WorthQueryRunningDirectRun,
    lease: WorthQueryApplicationSnapshotLease,
) {
    mutation_run
        .finish_for_test(
            running,
            WorthQueryManagedRunTerminalKind::Failed,
            lease.release_custody(),
            false,
        )
        .unwrap();
}

pub fn start(
    phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,

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
    let principal = authenticated_principal(&(phase).test_host_request_for_test(), world, &request);
    let account = resolved_account(
        &(phase).test_host_request_for_test(),
        world,
        "open",
        &request,
    );
    let program: RetainedProgram = retained_status_program(
        &(phase).test_host_request_for_test(),
        world,
        &principal,
        &account,
        &request,
        replacement,
        RetentionMutationBreadth::Narrow,
    );
    let WorthQueryApplicationCommitPreparation::Ready(prepared) = prepare_application_commit(
        &phase.borrowed_request_for_test(),
        WorthQueryApplicationCommitPreparationRequest::new_for_test(
            world
                .application
                .validate_commit_program_in_advancement_for_test(phase, program)
                .unwrap_or_else(|outcome| panic!("fixture admission: {outcome:?}")),
            idempotency,
            None,
            None,
        ),
    ) else {
        panic!("affinity fixture must prepare")
    };
    start_managed_application_commit(phase, &world.application, prepared).unwrap()
}
