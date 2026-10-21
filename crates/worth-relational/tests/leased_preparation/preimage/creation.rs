//! A known empty existing-entity selection retains sparse creation and old graph data.
use super::*;
use source::facts;
use worth_relational::facade::transactions::{CreatedEntityRef, EntityReference, RelationSpec};

#[test]
fn preimage_creation_only_graph_keeps_sparse_admission() {
    let _serial = serial();
    let host = crate::shared_relational_certification_test_execution_policy::serial_request();
    let seed_execution = worth_execution::ExecutionRequest::serial(&host);
    let source = Source::seed(&seed_execution, 1, 2);
    let lease = authority()
        .request_lease(lease_request(32 * 1024 * 1024, CancellationToken::new()))
        .unwrap();
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let (_, basis) = source
        .runtime
        .observe_branch(&source.runtime.main_branch_identity())
        .unwrap();
    let mut transaction = source
        .runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    let refs = ["created-a", "created-b"].map(|key| CreatedEntityRef {
        partition_id: PartitionId(1),
        kind_id: KindId(3),
        client_key: ClientKey::raw(key),
    });
    let mut batch = WorkerIntentBatch::new("creation-only");
    for reference in &refs {
        batch = batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: reference.partition_id,
            kind_id: reference.kind_id,
            client_key: reference.client_key.clone(),
            fields: facts("created", true),
        })));
    }
    batch = batch.push(MutationIntent::Create(CreateIntent::Relation(
        RelationSpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(2),
            client_key: ClientKey::raw("new-edge"),
            source: EntityReference::Created(refs[0].clone()),
            target: EntityReference::Created(refs[1].clone()),
            fields: AspectFieldPatch::from_locator(
                support::aspect_field_locator("label"),
                AspectValue::String("new".into()),
            ),
        },
    )));
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
