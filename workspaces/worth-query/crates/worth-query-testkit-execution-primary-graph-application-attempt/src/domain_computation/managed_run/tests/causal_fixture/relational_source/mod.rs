//! The causal fixture's relational source, its lease, and the entities it creates.
use super::*;

pub fn relational_source_and_lease(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    matching_snapshot: bool,
) -> (
    RuntimeBridgeRelationalSource,
    WorthQueryManagedRelationalObservation,
    Option<WorthQueryManagedRelationalObservation>,
    TruthBranchIdentity,
    TruthSnapshotIdentity,
) {
    let mut runtime = relational_runtime();
    let committed = create_fixture_entity(execution, &mut runtime);
    let branch_identity = runtime.main_branch_identity();
    let (_, bridge_basis) = runtime.observe_branch(&branch_identity).unwrap();
    let substitute_basis = runtime
        .readmit_branch_basis(bridge_basis.descriptor())
        .unwrap();
    assert!(runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .is_ok());
    let source = RuntimeBridgeRelationalSource::for_graph_role(Arc::new(runtime), "model")
        .expect("model should be a valid graph role");
    let bridge_basis =
        WorthQueryManagedRelationalObservation::retain_for_test(&source, bridge_basis, true)
            .expect("Relational source should retain the bridge execution basis");
    let substitute = if matching_snapshot {
        None
    } else {
        Some(
            WorthQueryManagedRelationalObservation::retain_for_test(
                &source,
                substitute_basis,
                true,
            )
            .expect("same version should admit an independent execution basis"),
        )
    };
    let snapshot = bridge_basis.identity().snapshot_identity().clone();
    let branch = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_branch::primary_truth_branch_identity();
    (source, bridge_basis, substitute, branch, snapshot)
}

pub fn create_fixture_entity(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &mut RelationalRuntime,
) -> worth_relational::facade::transactions::CommitResult {
    let mut transaction = {
        let transaction_validation_input = runtime
            .admit_branch_basis(&runtime.main_branch_identity())
            .expect("main branch binding");
        runtime
            .begin_branch_transaction(
                &transaction_validation_input,
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            )
            .expect("owner-admitted transaction context")
    };
    transaction
        .push_batch(
            WorkerIntentBatch::new("managed-run-fixture").push(MutationIntent::Create(
                CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: ClientKey::raw("managed-run-entity"),
                    fields: AspectFieldPatch::new(BTreeMap::new()),
                }),
            )),
            NativeAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    transaction
        .commit(execution, runtime)
        .expect("managed-run fixture entity should commit")
}
