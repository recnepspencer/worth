use std::sync::Arc;

use crate::facade::{
    RelationalBridgeRecordIdentityParts, SnapshotReadContract, SnapshotReadPacket,
    SnapshotReadRequest, SnapshotReadSource,
};
use worth_foundational::facade::{AspectValue, InternedString, ScalarAspectType};

use crate::relational_source::relational_test_support::{
    aspect_key, create_entity, field_key, fork_branch, runtime_with_declared_aspect_schema,
    single_string_aspect_field_patch, test_owner_begin_transaction_for_branch,
};
use worth_relational::facade::config::CascadeDeletePolicy;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::transactions::{
    EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent, WorkerIntentBatch,
};

use super::super::RuntimeBridgeRelationalSource;

#[test]
fn a_retained_observation_reads_its_own_root_after_the_branch_moves_on() {
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let entity = create_entity(&runtime, "before");
    let storm = BranchId("storm".to_owned());
    fork_branch(&runtime, storm.clone(), &BranchId("main".to_owned()));
    let identity = runtime.branch_identity(&storm).unwrap();
    let (_, before_basis) = runtime.observe_branch(&identity).unwrap();
    rename_on_branch(&runtime, entity, "after", storm);
    let (_, after_basis) = runtime.observe_branch(&identity).unwrap();

    let source =
        RuntimeBridgeRelationalSource::for_graph_role(Arc::new(runtime), "read-cutover").unwrap();
    let before = source
        .retain_branch_basis_for_bridge(&before_basis)
        .unwrap();
    let after = source.retain_branch_basis_for_bridge(&after_basis).unwrap();

    assert_eq!(
        read_name(&source, before.snapshot_identity(), entity),
        "before"
    );
    assert_eq!(
        read_name(&source, after.snapshot_identity(), entity),
        "after"
    );
}

fn rename_on_branch(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    entity: EntityId,
    name: &str,
    branch: BranchId,
) {
    let mut transaction = test_owner_begin_transaction_for_branch(runtime, branch);
    transaction
        .push_batch(
            WorkerIntentBatch::new("rename").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: entity,
                    fields: single_string_aspect_field_patch(
                        aspect_key("name"),
                        field_key("name"),
                        name,
                    ),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    transaction
        .commit(
            runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
}

fn read_name(
    source: &RuntimeBridgeRelationalSource,
    snapshot: &crate::facade::TruthSnapshotIdentity,
    entity: EntityId,
) -> String {
    let reader = source.open_snapshot(snapshot).unwrap();
    let packet = SnapshotReadPacket::new(vec![SnapshotReadRequest::for_relational_record(
        RelationalBridgeRecordIdentityParts::entity(
            entity.partition_id.0,
            entity.local_slot.0,
            entity.generation.0,
        ),
        SnapshotReadContract::scalar(aspect_key("name"), ScalarAspectType::String),
    )]);
    let result = reader.read_packet(&packet).unwrap();
    match result.records()[0].scalar_aspect_value() {
        Some(AspectValue::String(InternedString::Raw(name))) => name.clone(),
        other => panic!("the name reads as a raw string: {other:?}"),
    }
}
