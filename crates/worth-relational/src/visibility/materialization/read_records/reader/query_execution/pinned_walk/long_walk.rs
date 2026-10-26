//! A real authored chain crosses any legacy scope-widening depth bound.
use super::*;
use crate::facade::{
    identity::PartitionId,
    runtime::RelationalRuntimeApi,
    transactions::{
        AspectFieldPatch, CreateIntent, CreatedEntityRef, EntityReference, EntitySpec,
        MutationIntent, RelationSpec, WorkerIntentBatch,
    },
};
use crate::symbols::data::ClientKey;
use crate::tests::support::{
    release_test_commit_snapshot, test_owner_begin_transaction_for_main, AspectSchemaFixture,
};
use worth_execution::ExecutionAllocationPolicy;

#[test]
fn pinned_thousand_hops_complete_in_both_directions() {
    let host = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(AspectSchemaFixture::default().build_registry())
        .build();
    let creates: Vec<_> = (0..1001)
        .map(|n| CreatedEntityRef {
            partition_id: PartitionId::main(),
            kind_id: KindId(1),
            client_key: ClientKey::raw(format!("long-{n}")),
        })
        .collect();
    let mut batch = WorkerIntentBatch::new("long-chain");
    for created in &creates {
        batch
            .intents
            .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: created.partition_id,
                kind_id: created.kind_id,
                client_key: created.client_key.clone(),
                fields: AspectFieldPatch::default(),
            })));
    }
    for n in 0..1000 {
        batch
            .intents
            .push(MutationIntent::Create(CreateIntent::Relation(
                RelationSpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(2),
                    client_key: ClientKey::raw(format!("long-edge-{n}")),
                    source: EntityReference::Created(creates[n].clone()),
                    target: EntityReference::Created(creates[n + 1].clone()),
                    fields: AspectFieldPatch::default(),
                },
            )));
    }
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(batch, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let committed = transaction.commit(&execution, &runtime).unwrap();
    let ids: Vec<_> = creates
        .iter()
        .map(|created| committed.created_entity(created).unwrap())
        .collect();
    release_test_commit_snapshot(&runtime, &committed);
    let pin = runtime
        .read_truth()
        .pin_read_capability(&runtime.visibility_authority().snapshot(), &[], &execution)
        .unwrap();
    for (direction, root) in [
        (RelationDirection::Forward, ids[0]),
        (RelationDirection::Reverse, ids[1000]),
    ] {
        let walk = pin
            .complete_relation_walk(
                &execution,
                PinnedStructuralWalkScope::new(&[root], KindId(1), KindId(2), direction),
            )
            .unwrap();
        assert_eq!(walk.visits().len(), 1001);
        assert_eq!(walk.relations().len(), 1000);
        for (distance, visit) in walk.visits().iter().enumerate() {
            let index = match direction {
                RelationDirection::Forward => distance,
                RelationDirection::Reverse => 1000 - distance,
            };
            assert_eq!(
                (visit.entity.entity_id, visit.distance),
                (ids[index], distance)
            );
        }
        assert_eq!(walk.reads().entity_records, 1001);
        assert_eq!(walk.reads().adjacency_lists, 1001);
        assert_eq!(walk.reads().relation_records, 1000);
    }
}
