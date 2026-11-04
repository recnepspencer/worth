pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::start_managed_application_commit;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::application_attempt_affinity::WorthQueryApplicationAttemptBasis;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::WorthQueryRunningApplicationCommit;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::commit_preparation::prepare_application_commit;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::WorthQueryApplicationCommitPreparation;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::WorthQueryApplicationCommitPreparationRequest;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::application_attempt_affinity::WorthQueryApplicationAttemptAffinityMismatch as M;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::preimage_evidence::retained_status_program;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::preimage_evidence::RetentionMutationBreadth;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::ExactStatusRetentionInput;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::ExactStatusRetentionOperation;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationEffectProgram;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationIdempotencyBinding;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease;
pub use worth_query_execution::facade::domain_computation::WorthQueryManagedRunTerminalKind;

pub type RetainedProgram = WorthQueryApplicationEffectProgram<
    IdentityExecutionSchema,
    ExactStatusRetentionOperation,
    ExactStatusRetentionInput,
    Account,
>;

#[test]
fn exact_affinity_accepts_its_real_session_and_rejects_runtime_snapshot_and_branch_drift() {
    let world =
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(
            true,
        );
    let foreign_world =
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(
            true,
        );
    world
        .application
        .with_host_advancement(|active_phase| {
            let phase = &active_phase;
            let execution = phase;

            let running = start(
                phase,
                &world,
                retained_program(phase, &world, "axis-owner"),
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
                outcome_identity: _outcome_identity,
                workflow_settlement_publication: _workflow_settlement_publication,
            } = running;
            let staged = real_terminal_session(
                execution,
                &world,
                &mut running,
                lease.product().retained_clone_for_test(),
            );
            assert!(staged.bind_application_attempt_for_test(attempt_basis).is_ok());

            assert!(WorthQueryApplicationAttemptBasis::capture(
                &foreign_world.application,
                &admission,
                lease.snapshot_for_test(),
                lease.product(),
            )
            .is_err());

            let substitute_lease = WorthQueryApplicationSnapshotLease::acquire_for_test(
                &phase
                    .execution_request_for_for_test(&(*world.application.product_runtime_for_test()))
                    .expect("fixture issuer"),
                lease.handle_for_test().clone(),
                std::sync::Arc::clone(&(*lease.layout_for_test())),
                lease.product().retained_clone_for_test(),
            )
            .expect("the real branch must issue a second exact snapshot handle");
            assert_ne!(substitute_lease.snapshot_for_test(), lease.snapshot_for_test());
            let substitute_snapshot_basis = WorthQueryApplicationAttemptBasis::capture(
                &world.application,
                &admission,
                substitute_lease.snapshot_for_test(),
                substitute_lease.product(),
            )
            .expect("same-runtime same-branch snapshot remains a valid pre-session basis");
            assert!(staged
                .bind_application_attempt_for_test(substitute_snapshot_basis)
                .is_err());
            assert!(substitute_lease.release_for_test());

            let foreign_branch_snapshot = lease.handle_for_test().with_runtime_mut(
                &phase
                    .execution_request_for_for_test(&(*world.application.product_runtime_for_test()))
                    .expect("fixture issuer"),
                |runtime| {
                    let foreign = worth_relational::facade::history::BranchId(
                        "foreign-affinity-branch".to_owned(),
                    );
                    let (_, fork_basis) = runtime
                        .observe_fork_source(lease.snapshot_for_test().branch_id())
                        .expect("test runtime can observe the exact source basis");
                    runtime
                        .fork_branch(
                            &phase
                                .execution_request_for_for_test(&(*world.application.product_runtime_for_test()))
                                .expect("fixture issuer"),
                            foreign.clone(),
                            fork_basis,
                        )
                        .expect("test runtime can issue a real sibling branch");
                    let identity = runtime.branch_identity(&foreign).unwrap();
                    let (_, basis) = runtime.observe_branch(&identity).unwrap();
                    runtime
                        .snapshots()
                        .snapshot_for_observation(&basis.observation())
                        .unwrap()
                },
            );
            assert!(WorthQueryApplicationAttemptBasis::capture(
                &world.application,
                &admission,
                &foreign_branch_snapshot,
                lease.product(),
            )
            .is_err());
            assert!(lease.handle_for_test().with_runtime_mut(
                &phase
                    .execution_request_for_for_test(&(*world.application.product_runtime_for_test()))
                    .expect("fixture issuer"),
                |runtime| {
                    runtime
                        .snapshots()
                        .release_snapshot(&foreign_branch_snapshot)
                        .is_ok()
                }
            ));
            let _ = staged.abort();
            finish_uncommitted(mutation_run, running, lease);
        })
        .expect("fixture owner admits its advancement");
}

#[test]
fn a_real_peer_plan_and_session_cannot_substitute_for_the_captured_attempt() {
    let world =
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(
            true,
        );
    world
        .application
        .with_host_advancement(|active_phase| {
            let phase = &active_phase;
            let execution = phase;

            let first = start(
                phase,
                &world,
                retained_program(phase, &world, "plan-owner"),
                idempotency(189, 190),
            );
            let second = start(
                phase,
                &world,
                retained_program(phase, &world, "plan-peer"),
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
                outcome_identity: _first_outcome_identity,
                workflow_settlement_publication: _first_workflow_settlement_publication,
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
                outcome_identity: _second_outcome_identity,
                workflow_settlement_publication: _second_workflow_settlement_publication,
            } = second;
            let first_staged = real_terminal_session(
                execution,
                &world,
                &mut first_running,
                first_lease.product().retained_clone_for_test(),
            );
            let second_staged = real_terminal_session(
                execution,
                &world,
                &mut second_running,
                second_lease.product().retained_clone_for_test(),
            );
            let first_terminal = first_staged.provider_session_terminal_binding_for_test();
            let second_terminal = second_staged.provider_session_terminal_binding_for_test();
            assert_ne!(first_terminal, second_terminal);
            let mismatches = first_basis.affinity_mismatches_for_test(&second_terminal);
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
            assert!(second_staged.bind_application_attempt_for_test(first_basis).is_err());
            assert!(first_staged.bind_application_attempt_for_test(second_basis).is_err());
            let _ = first_staged.abort();
            let _ = second_staged.abort();
            finish_uncommitted(first_mutation_run, first_running, first_lease);
            finish_uncommitted(second_mutation_run, second_running, second_lease);
        })
        .expect("fixture owner admits its advancement");
}

pub fn fork_product(
    world: &AuthorizationWorld,
    source: &worth_query_execution::facade::domain_computation::WorthQueryProductBranchLease,
) -> worth_runtime_world::facade::ProductBranchIdentity {
    let intent = worth_runtime_world::facade::ProductBranchCreationIntent::from_source(
        "provider-affinity-sibling",
        worth_runtime_world::facade::ProductBranchCreationPlans::new(
            worth_runtime_world::facade::RelationalBranchCreationPlan::ReuseExact,
            worth_runtime_world::facade::SignalBranchCreationPlan::ReuseExact,
        ),
    )
    .expect("the sibling intent is valid");
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let outcome = world
        .application
        .product_runtime()
        .create_product_branch_for_test(
            execution,
            source,
            None,
            intent,
            &worth_runtime_world::facade::RuntimeWorldCancellationSource::new().token(),
        )
        .expect("the product owner accepts the sibling request");
    let worth_runtime_world::facade::RuntimeWorldBranchCreationOutcome::Performed(observation) =
        outcome
    else {
        panic!("the product owner must create an exact sibling: {outcome:?}")
    };
    observation.branch_identity().clone()
}

pub fn real_terminal_session<'run>(
    execution: &'run worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,

    world: &AuthorizationWorld,
    running: &'run mut worth_query_execution::facade::domain_computation::WorthQueryRunningDirectRun,
    product: worth_query_execution::facade::domain_computation::WorthQueryProductBranchLease,
) -> worth_query_execution::facade::domain_computation::WorthQuerySessionBoundReadsAndEffects<'run>
{
    running
        .admit_provider_execution_plan(&(*world.application.primary_graph_authority_for_test()))
        .expect("real application authorities must admit their provider plan")
        .bind_application_product_for_test(product)
        .expect("the exact selected product must bind its provider plan")
        .readmit(execution)
        .expect("the real provider must readmit its exact plan")
        .prepare()
        .expect("the real provider must prepare its exact session")
        .bind_reads_and_effects()
}

pub fn finish_uncommitted(
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
        .expect("affinity probe must release managed-run resources");
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
        panic!("affinity fixture must reach ordinary prepared posture")
    };
    start_managed_application_commit(phase, &world.application, prepared)
        .unwrap_or_else(|outcome| panic!("affinity fixture must start: {outcome:?}"))
}

pub fn retained_program(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    world: &AuthorizationWorld,
    replacement: &str,
) -> RetainedProgram {
    let request = live_scope();
    let principal = authenticated_principal(&(phase_custody).test_host_request_for_test(), world, &request);
    let account = resolved_account(
        &(phase_custody).test_host_request_for_test(),
        world,
        "open",
        &request,
    );
    retained_status_program(
        &(phase_custody).test_host_request_for_test(),
        world,
        &principal,
        &account,
        &request,
        replacement,
        RetentionMutationBreadth::Narrow,
    )
}

#[path = "foreign_phase/mod.rs"]
pub mod foreign_phase;
#[path = "selected_product/mod.rs"]
pub mod selected_product;
