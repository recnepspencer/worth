use super::fixtures::*;
use crate::branch::{RelationalEntityMaterialization, RelationalRelationMaterialization};
use crate::tests::support::*;

#[test]
fn native_storage_owner_suspends_and_restores_private_records_without_renaming() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let runtime = runtime();
    let born = commit(execution, &runtime, "main", birth(&[1, 0]));
    let rows = records(execution, &runtime, &born.snapshot);
    let read = runtime
        .read_truth()
        .read_snapshot(&born.snapshot, execution)
        .unwrap()
        .unwrap();
    let relations = read
        .relations()
        .into_iter()
        .filter(|edge| edge.kind.kind_id == KindId(5))
        .map(|edge| RelationalRelationMaterialization {
            relation_id: edge.relation_id,
            kind_id: edge.kind.kind_id,
            source: edge.source,
            target: edge.target,
            fields: Default::default(),
        })
        .collect();
    let services = runtime.owner_component_services();
    let identity = runtime.main_branch_identity();
    let (_, basis) = services.basis_port().observe_branch(&identity).unwrap();
    let pending = services
        .materialization_port()
        .prepare_generated_materialization_suspension(execution, &basis, &[rows[0].0])
        .unwrap();
    let (candidate, completion) = pending.into_parts();
    let suspended = completion
        .complete(
            runtime
                .publish_prepared_candidate(execution, candidate)
                .unwrap(),
        )
        .unwrap();
    assert!(runtime
        .read_truth()
        .read_snapshot(&suspended.commit.snapshot, execution)
        .unwrap()
        .is_none());
    let (_, basis) = services.basis_port().observe_branch(&identity).unwrap();
    let pending = services
        .materialization_port()
        .prepare_generated_rematerialization(
            execution,
            &basis,
            suspended.custody,
            vec![RelationalEntityMaterialization {
                entity_id: rows[0].0,
                kind_id: KindId(3),
                fields: AspectFieldPatch::new(rows[0].1.fields()),
            }],
            relations,
        )
        .unwrap();
    let (candidate, completion, _) = pending.into_parts();
    let restored = completion
        .complete(
            runtime
                .publish_prepared_candidate(execution, candidate)
                .unwrap(),
        )
        .unwrap();
    assert_eq!(records(execution, &runtime, &restored.snapshot), rows);
}
