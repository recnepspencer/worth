// A known empty existing-entity selection retains sparse creation and old graph data.
pub(crate) use crate::leased_preparation::isolated_context::preimage::source::facts;
pub(crate) use {
    crate::leased_preparation::isolated_context::preimage::check_snapshot,
    crate::leased_preparation::isolated_context::preimage::check_source,
    crate::leased_preparation::ordinary_context::authority,
    crate::leased_preparation::ordinary_context::lease_request,
    crate::leased_preparation::ordinary_context::serial,
    worth_relational::facade::identity::KindId, worth_relational::facade::identity::PartitionId,
    worth_relational::facade::transactions::EntitySpec,
};
pub(crate) use {
    worth_relational::facade::transactions::CreatedEntityRef,
    worth_relational::facade::transactions::RelationSpec,
};

#[test]
fn preimage_creation_only_graph_keeps_sparse_admission() {
    if worth_testkit_relational::test_process::reexecute_isolated_test() {
        return;
    }

    let _serial = serial();
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let seed_execution = worth_execution::ExecutionRequest::serial(&host);
    let source = crate::leased_preparation::isolated_context::preimage::source::Source::seed(
        &seed_execution,
        1,
        2,
    );
    let lease = authority()
        .request_lease(lease_request(
            32 * 1024 * 1024,
            worth_execution::CancellationToken::new(),
        ))
        .unwrap();
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let (_, basis) = source
        .runtime
        .observe_branch(&source.runtime.main_branch_identity())
        .unwrap();
    let mut transaction = source
        .runtime
        .begin_branch_transaction(
            &basis,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .unwrap();
    let refs = ["created-a", "created-b"].map(|key| CreatedEntityRef {
        partition_id: PartitionId(1),
        kind_id: KindId(3),
        client_key: worth_relational::facade::symbols::ClientKey::raw(key),
    });
    let mut batch = worth_relational::facade::transactions::WorkerIntentBatch::new("creation-only");
    for reference in &refs {
        batch = batch.push(
            worth_relational::facade::transactions::MutationIntent::Create(
                worth_relational::facade::transactions::CreateIntent::Entity(EntitySpec {
                    partition_id: reference.partition_id,
                    kind_id: reference.kind_id,
                    client_key: reference.client_key.clone(),
                    fields: facts("created", true),
                }),
            ),
        );
    }
    batch = batch.push(
        worth_relational::facade::transactions::MutationIntent::Create(
            worth_relational::facade::transactions::CreateIntent::Relation(RelationSpec {
                partition_id: worth_relational::facade::identity::PartitionId::main(),
                kind_id: KindId(2),
                client_key: worth_relational::facade::symbols::ClientKey::raw("new-edge"),
                source: worth_relational::facade::transactions::EntityReference::Created(
                    refs[0].clone(),
                ),
                target: worth_relational::facade::transactions::EntityReference::Created(
                    refs[1].clone(),
                ),
                fields: worth_relational::facade::transactions::AspectFieldPatch::from_locator(
                    crate::leased_preparation::ordinary_context::support::aspect_field_locator(
                        "label",
                    ),
                    worth_foundational::facade::AspectValue::String("new".into()),
                ),
            }),
        ),
    );
    transaction
        .push_batch(batch, execution.allocation_policy())
        .unwrap();
    let committed = transaction.commit(&execution, &source.runtime).unwrap();
    assert_eq!(committed.complexity_delta().partitions_cloned, 0);
    let [a, b] = refs.map(|reference| committed.created_entity(&reference).unwrap());
    let read = source
        .runtime
        .read_truth()
        .read_snapshot(&committed.snapshot, &execution)
        .unwrap()
        .unwrap();
    for entity in [a, b] {
        check_source(
            read.get_entity(entity)
                .unwrap()
                .authoritative_aspect_state
                .as_ref()
                .unwrap(),
            "created",
        );
    }
    let projection = source
        .runtime
        .read_truth()
        .project_snapshot(&committed.snapshot)
        .unwrap();
    let edges = projection
        .outgoing_relations_of_kind(&execution, a, KindId(2))
        .unwrap();
    assert_eq!(edges.len(), 1);
    assert_eq!((edges[0].source, edges[0].target), (a, b));
    check_snapshot(&source, &execution, &committed.snapshot, "original", 0);
    check_snapshot(
        &source,
        &execution,
        &source.original.snapshot,
        "original",
        0,
    );
    source
        .runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();
    source
        .runtime
        .snapshots()
        .release_snapshot(&source.original.snapshot)
        .unwrap();
}
