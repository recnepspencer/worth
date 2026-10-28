//! Physical output custody never transfers Native-owned orbit metadata.
use super::fixtures::*;
use crate::branch::{
    RelationalEntityMaterialization, RelationalMaterializationRecord,
    RelationalRelationMaterialization,
};
use crate::tests::support::*;
use crate::transactions::data::AspectFieldPatch;

#[test]
fn generated_payload_suspends_and_restores_while_native_orbit_custody_stays_materialized() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let runtime = runtime();
    let born = commit(execution, &runtime, "main", birth(&[1, 2, 0]));
    let members = members(&born, 3);
    let original = records(execution, &runtime, &born.snapshot);
    let read = runtime
        .read_truth()
        .read_snapshot(&born.snapshot, execution)
        .unwrap()
        .unwrap();
    let business: Vec<_> = read
        .relations()
        .iter()
        .filter(|edge| edge.kind.kind_id == KindId(2))
        .map(|edge| RelationalRelationMaterialization {
            relation_id: edge.relation_id,
            kind_id: KindId(2),
            source: edge.source,
            target: edge.target,
            fields: AspectFieldPatch::default(),
        })
        .collect();
    let native: Vec<_> = read
        .relations()
        .iter()
        .filter(|edge| edge.kind.kind_id == descriptor().membership_kind)
        .map(|edge| edge.relation_id)
        .collect();
    assert_eq!(business.len(), 3);
    assert_eq!(native.len(), 3);
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let prepared = runtime
        .owner_component_services()
        .materialization_port()
        .prepare_generated_materialization_suspension(execution, &basis, &members)
        .unwrap();
    let (candidate, completion) = prepared.into_parts();
    assert_eq!(completion.records.len(), 6, "three authored entities and three authored relations; Native metadata stays with its owner");
    assert!(completion.records.iter().all(|record| match record {
        RelationalMaterializationRecord::Entity { kind_id, .. } => *kind_id == KindId(1),
        RelationalMaterializationRecord::Relation { kind_id, .. } => *kind_id == KindId(2),
    }));
    let suspended = runtime
        .publish_prepared_candidate(execution, candidate)
        .unwrap();
    let suspended = completion.complete(suspended).unwrap();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    for relation in native {
        let slot = basis
            .inner
            .root
            .partition_state(relation.partition_id)
            .unwrap()
            .unwrap()
            .relation_arena
            .get(&relation)
            .unwrap();
        assert!(slot.is_live());
        assert!(
            !slot.is_materialization_unavailable(),
            "Native membership remains available during payload suspension"
        );
    }
    let entities = members
        .into_iter()
        .map(|entity_id| RelationalEntityMaterialization {
            entity_id,
            kind_id: KindId(1),
            fields: AspectFieldPatch::default(),
        })
        .collect();
    let prepared = runtime
        .owner_component_services()
        .materialization_port()
        .prepare_generated_rematerialization(
            execution,
            &basis,
            suspended.custody,
            entities,
            business,
        )
        .unwrap_or_else(|failure| {
            panic!("restore the exact authored payload: {:?}", failure.error)
        });
    let (candidate, completion, _) = prepared.into_parts();
    let restored = runtime
        .publish_prepared_candidate(execution, candidate)
        .unwrap();
    let restored = completion.complete(restored).unwrap();
    assert_eq!(records(execution, &runtime, &restored.snapshot), original);
}
