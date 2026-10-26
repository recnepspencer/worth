use crate::basis::WorthQueryProductBranchCreateError;
use crate::domain_computation::execution_runtime::product_world::{
    test_product_world_resources, WorthQueryProductBranchCloseScope,
    WorthQueryProductBranchCreationDenial, WorthQueryProductBranchOwnerCleanupDenial,
    WorthQueryProductWorldResources,
};
use crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world_with_product_resources;

fn evidence(error: WorthQueryProductBranchCreateError) -> (usize, usize) {
    let WorthQueryProductBranchCreateError::Creation(
        WorthQueryProductBranchCreationDenial::CoordinationCapacityExhausted { limit, occupancy },
    ) = error
    else {
        panic!("expected the activation reservation refusal, got {error:?}");
    };
    (limit, occupancy)
}

#[test]
fn capacity_refusal_evidence_distinguishes_own_heads_outside_and_retired_holders() {
    let world = installed_authorization_world_with_product_resources(capacity_resources(5));
    let root = world.application.product_runtime().root_product_branch();
    let fork = || {
        world
            .application
            .with_host_advancement(|phase| {
                world
                    .application
                    .branches()
                    .fork(root)
                    .components(|components| {
                        components
                            .reuse_exact_relational_basis()
                            .reuse_exact_signal_basis()
                    })
                    .create(&phase)
            })
            .unwrap()
    };
    // One holder fills the installed policy: the root, three heads and its spare.
    let mut own_heads = (0..3).map(|_| fork().unwrap()).collect::<Vec<_>>();
    let spare = fork().unwrap();
    let first = evidence(fork().unwrap_err());
    assert_eq!(first, (5, 5));
    assert_eq!(first, evidence(fork().unwrap_err()));
    assert_eq!(first.1 - (1 + own_heads.len() + 1), 0);
    let runtime = world.application.product_runtime();
    let spare_lease = runtime
        .admit_product_occurrence(spare.occurrence())
        .unwrap();
    let retained_gate = runtime
        .activations
        .gate(spare_lease.branch_identity())
        .unwrap();
    drop(spare_lease);
    // World retirement has happened, while the pending close and a real gate
    // holder still exist. Coordination reports the retained allocation once.
    let pending = runtime
        .begin_product_branch_close(spare, WorthQueryProductBranchCloseScope::Workspace)
        .unwrap();
    assert_eq!(evidence(fork().unwrap_err()), first);
    assert_eq!(evidence(fork().unwrap_err()), first);
    pending.finish().unwrap();
    assert_eq!(evidence(fork().unwrap_err()), first);
    drop(retained_gate);
    // The first holder now has room for another model plus its spare, but an
    // independent holder takes both free slots in the same application.
    runtime
        .begin_product_branch_close(
            own_heads.pop().unwrap(),
            WorthQueryProductBranchCloseScope::Workspace,
        )
        .unwrap()
        .finish()
        .unwrap();
    let outside = fork().unwrap();
    assert!(!own_heads.contains(&outside));
    let outside_lease = runtime
        .admit_product_occurrence(outside.occurrence())
        .unwrap();
    assert_eq!(
        outside_lease.branch_identity().name().as_str(),
        "query-product-5"
    );
    drop(outside_lease);
    let outside_sibling = fork().unwrap();
    assert_ne!(outside, outside_sibling);
    let second = evidence(fork().unwrap_err());
    assert_eq!(second, first);
    assert_eq!(own_heads.len(), 2);
    assert_eq!(second.1 - (1 + own_heads.len()), 2);
    let pending_outside = runtime
        .begin_product_branch_close(outside, WorthQueryProductBranchCloseScope::Workspace)
        .unwrap();
    let replacement_spare = fork().unwrap();
    // Pending owner cleanup alone does not retain an activation gate.
    pending_outside.finish().unwrap();
    assert_ne!(replacement_spare, spare);
    assert_eq!(evidence(fork().unwrap_err()), first);
}

#[test]
fn retained_cleanup_capacity_evidence_counts_gates_but_not_blocked_component_work() {
    let world = installed_authorization_world_with_product_resources(capacity_resources(5));
    let runtime = world.application.product_runtime();
    let root = runtime.root_product_branch();
    let spare = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .branches()
                .fork(root)
                .components(|components| components.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
        })
        .unwrap()
        .unwrap();
    let fork = || {
        world
            .application
            .with_host_advancement(|phase| {
                world
                    .application
                    .branches()
                    .fork(root)
                    .components(|components| {
                        components
                            .reuse_exact_relational_basis()
                            .reuse_exact_signal_basis()
                    })
                    .create(&phase)
            })
            .unwrap()
    };
    let own_heads = (0..3).map(|_| fork().unwrap()).collect::<Vec<_>>();
    assert_eq!(evidence(fork().unwrap_err()), (5, 5));
    let lease = runtime
        .admit_product_occurrence(spare.occurrence())
        .unwrap();
    let retained_gate = runtime.activations.gate(lease.branch_identity()).unwrap();
    drop(lease);
    let policy = crate::test_request_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&policy);
    let integration = world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .integration_handle();
    let active_transaction = integration.with_runtime(&execution, |relational| {
        let identity = relational
            .branch_identity(&worth_relational::facade::history::BranchId(
                "query-product-1-relational".to_owned(),
            ))
            .unwrap();
        let (_, basis) = relational.observe_branch(&identity).unwrap();
        relational
            .begin_branch_transaction(
                &basis,
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            )
            .unwrap()
    });
    let pending = runtime
        .begin_product_branch_close(spare, WorthQueryProductBranchCloseScope::Workspace)
        .unwrap();
    let (retired, cleanup) = pending.into_cleanup();
    assert_eq!(retired, spare);
    assert_eq!(cleanup.pending_work().len(), 1);
    let failure = cleanup
        .retry()
        .expect_err("the native transaction still holds the fork");
    assert_eq!(
        failure.denial(),
        WorthQueryProductBranchOwnerCleanupDenial::RelationalOperationsStillActive
    );
    assert_eq!(evidence(fork().unwrap_err()), (5, 5));
    // The retired gate is a capacity holder, independently of component cleanup.
    drop(retained_gate);
    let replacement = fork().expect("blocked component cleanup alone holds no activation slot");
    assert!(!own_heads.contains(&replacement));
    assert_eq!(evidence(fork().unwrap_err()), (5, 5));
    drop(active_transaction);
    let receipt = failure.into_cleanup().retry().unwrap();
    assert_eq!(receipt.retired_component_count(), 1);
    assert_eq!(evidence(fork().unwrap_err()), (5, 5));
}

/// Install a distinct branch ceiling through the ordinary composition input;
/// retain the fixture's unrelated policies from their actual owner.
pub(super) fn capacity_resources(maximum: u64) -> WorthQueryProductWorldResources {
    use worth_runtime_world::facade::{
        RuntimeWorldBranchBudgetInstallation, RuntimeWorldBudgetInstallation,
        RuntimeWorldBudgetLimit, RuntimeWorldCustodyBudgetInstallation,
        RuntimeWorldHistoryBudgetInstallation, RuntimeWorldObservationBudgetInstallation,
        RuntimeWorldPublicationBudgetInstallation, RuntimeWorldRecoveryBudgetInstallation,
        RuntimeWorldRetentionBudgetInstallation,
    };
    let (budgets, clock, invalidation, execution) = test_product_world_resources().into_parts();
    let installed = |limit: RuntimeWorldBudgetLimit| {
        u64::try_from(limit.get().expect("the fixture installs finite policies")).unwrap()
    };
    WorthQueryProductWorldResources::install(
        RuntimeWorldBudgetInstallation {
            branches: RuntimeWorldBranchBudgetInstallation {
                live_product_branches: maximum,
            },
            history: RuntimeWorldHistoryBudgetInstallation {
                retained_composite_commits: installed(budgets.retained_composite_commits()),
                history_metadata_bytes: installed(budgets.history_metadata_bytes()),
            },
            observations: RuntimeWorldObservationBudgetInstallation {
                active_observations: installed(budgets.active_observations()),
            },
            publication: RuntimeWorldPublicationBudgetInstallation {
                active_publication_attempts: installed(budgets.active_publication_attempts()),
            },
            recovery: RuntimeWorldRecoveryBudgetInstallation {
                retained_product_unpublished_records: installed(
                    budgets.retained_product_unpublished_records(),
                ),
                retained_partial_metadata_bytes: installed(
                    budgets.retained_partial_metadata_bytes(),
                ),
            },
            retention: RuntimeWorldRetentionBudgetInstallation {
                unique_exact_component_pins: installed(budgets.unique_exact_component_pins()),
                in_flight_pin_acquisition_reservations: installed(
                    budgets.in_flight_pin_acquisition_reservations(),
                ),
            },
            custody: RuntimeWorldCustodyBudgetInstallation {
                owner_created_component_custody_records: installed(
                    budgets.owner_created_component_custody_records(),
                ),
            },
        },
        clock,
        invalidation,
        execution.policy,
    )
    .unwrap()
}
