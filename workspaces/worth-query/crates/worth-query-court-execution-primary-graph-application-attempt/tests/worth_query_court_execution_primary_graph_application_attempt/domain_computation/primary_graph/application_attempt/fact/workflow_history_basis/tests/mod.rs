use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::snapshots::SnapshotHandle;
use worth_query_execution::domain_computation::primary_graph::application_attempt::fact::workflow_history_basis::WorkflowHistoryRequirement;
use worth_query_execution::domain_computation::primary_graph::application_attempt::fact::workflow_history_basis::remains_equal;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::runtime::RelationalRuntimeApi;
use worth_relational::facade::schema::EntityKindRegistration;
use worth_relational::facade::schema::KindAspectContractDeclarations;
use worth_relational::facade::schema::RelationalSchemaRegistry;
use worth_relational::facade::schema::SchemaId;
use worth_relational::facade::schema::SchemaVersionId;
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::AspectFieldPatch;
use worth_relational::facade::transactions::CreateIntent;
use worth_relational::facade::transactions::EntitySpec;
use worth_relational::facade::transactions::MutationIntent;
use worth_relational::facade::transactions::WorkerIntentBatch;

#[test]
fn history_basis_requires_same_owner_branch_version_and_live_candidate() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = history_runtime();
    let first = commit(execution, &mut runtime, "first");
    let retained = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let second_lease = runtime
        .snapshots()
        .snapshot_for_observation(&retained.observation())
        .unwrap();
    assert_ne!(first.snapshot_id(), second_lease.snapshot_id());
    assert!(remains_equal(
        &runtime,
        &second_lease,
        &first,
        WorkflowHistoryRequirement::Exact,
        1
    ));
    assert!(!remains_equal(
        &runtime,
        &second_lease,
        &first,
        WorkflowHistoryRequirement::ReplayOnly,
        1
    ));
    assert!(remains_equal(
        &runtime,
        &second_lease,
        &first,
        WorkflowHistoryRequirement::Append {
            maximum_transitions: 2
        },
        1
    ));
    assert!(!remains_equal(
        &runtime,
        &second_lease,
        &first,
        WorkflowHistoryRequirement::Append {
            maximum_transitions: 1
        },
        1
    ));
    let (_, source) = runtime.observe_fork_source(first.branch_id()).unwrap();
    let fork = runtime
        .fork_branch(execution, BranchId("history-fork".to_owned()), source)
        .unwrap();
    let fork_basis = runtime.admit_branch_basis(fork.target_identity()).unwrap();
    let fork_snapshot = runtime
        .snapshots()
        .snapshot_for_observation(&fork_basis.observation())
        .unwrap();
    assert_eq!(fork_snapshot.version_id(), first.version_id());
    assert!(!remains_equal(
        &runtime,
        &fork_snapshot,
        &first,
        WorkflowHistoryRequirement::Append {
            maximum_transitions: 2
        },
        1
    ));
    let changed = commit(execution, &mut runtime, "second");
    for (owner, candidate) in [(&runtime, &fork_snapshot), (&runtime, &changed)] {
        assert!(!remains_equal(
            owner,
            candidate,
            &first,
            WorkflowHistoryRequirement::Exact,
            1
        ));
    }
    assert!(!remains_equal(
        &runtime,
        &changed,
        &first,
        WorkflowHistoryRequirement::Append {
            maximum_transitions: 2
        },
        1
    ));
    assert!(remains_equal(
        &runtime,
        &second_lease,
        &first,
        WorkflowHistoryRequirement::Append {
            maximum_transitions: 2
        },
        1
    ));
    let mut foreign = history_runtime();
    let foreign_snapshot = commit(execution, &mut foreign, "first");
    assert!(!remains_equal(
        &foreign,
        &foreign_snapshot,
        &first,
        WorkflowHistoryRequirement::Exact,
        1
    ));
    assert_eq!(foreign_snapshot.version_id(), first.version_id());
    assert!(!remains_equal(
        &foreign,
        &foreign_snapshot,
        &first,
        WorkflowHistoryRequirement::Append {
            maximum_transitions: 2
        },
        1
    ));
    assert!(!remains_equal(
        &foreign,
        &second_lease,
        &first,
        WorkflowHistoryRequirement::Append {
            maximum_transitions: 2
        },
        1
    ));
    worth_query_execution::relational_snapshot_release::release_query_snapshot(
        &mut runtime,
        &second_lease,
    );
    assert!(!remains_equal(
        &runtime,
        &second_lease,
        &first,
        WorkflowHistoryRequirement::Exact,
        1
    ));
    assert!(!remains_equal(
        &runtime,
        &second_lease,
        &first,
        WorkflowHistoryRequirement::Append {
            maximum_transitions: 2
        },
        1
    ));
    // An invalid lease must be distinguished before reading any anchor.
    assert_eq!(
        worth_query_execution::domain_computation::primary_graph::application_attempt::fact::adjacency::observe_adjacency_checked(
            &runtime,
            execution,
            &second_lease,
            KindId::new(1),
            EntityId::new(PartitionId::main(), 0, 1),
            worth_query_execution::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationAdjacencyDirection::Outgoing,
            0,
        ),
        Err(worth_query_execution::domain_computation::primary_graph::application_attempt::fact::adjacency::AdjacencyObservationDenial::SnapshotUnavailable)
    );
    worth_query_execution::relational_snapshot_release::release_query_snapshot(
        &mut runtime,
        &first,
    );
    worth_query_execution::relational_snapshot_release::release_query_snapshot(
        &mut runtime,
        &changed,
    );
    worth_query_execution::relational_snapshot_release::release_query_snapshot(
        &mut runtime,
        &fork_snapshot,
    );
    worth_query_execution::relational_snapshot_release::release_query_snapshot(
        &mut foreign,
        &foreign_snapshot,
    );
}

pub fn history_runtime() -> RelationalRuntime {
    let schema = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId::new(1),
            kind_name: "history".to_owned(),
            schema_id: SchemaId("history-basis".to_owned()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
        })
        .unwrap();
    RelationalRuntimeApi::builder()
        .schema_registry(schema)
        .build()
        .expect("valid native runtime configuration")
}

pub fn commit(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &mut RelationalRuntime,
    name: &str,
) -> SnapshotHandle {
    let basis = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(
            &basis,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new(name).push(MutationIntent::Create(CreateIntent::Entity(
                EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId::new(1),
                    client_key: ClientKey::raw(name),
                    fields: AspectFieldPatch::default(),
                },
            ))),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    transaction
        .commit(execution, runtime)
        .unwrap()
        .snapshot
        .clone()
}
