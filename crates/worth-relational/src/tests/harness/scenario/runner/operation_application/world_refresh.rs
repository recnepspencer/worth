use super::*;

pub(crate) fn refresh_live_world(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    entities: &mut Vec<EntityId>,
    relations: &mut Vec<ActiveRelation>,
) {
    let snapshot = runtime.visibility_authority().snapshot();
    let read = runtime
        .read_truth()
        .read_snapshot(&snapshot, execution)
        .unwrap();
    *entities = read
        .entities()
        .iter()
        .map(|record| record.entity_id)
        .collect();
    *relations = read
        .relations()
        .iter()
        .map(|record| ActiveRelation {
            relation_id: record.relation_id,
            source: record.source,
            target: record.target,
            partition: record.relation_id.partition_id,
        })
        .collect();
    drop(read);
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&snapshot)
        .is_ok());
}
