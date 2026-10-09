use std::sync::{Arc, Mutex};

use crate::facade::{
    RelationalBridgeRecordIdentityParts, SnapshotReadContract, SnapshotReadSource,
};
use worth_foundational::facade::{AspectKey, AspectValue, ScalarAspectType};

use crate::relational_source::relational_test_support::{
    changed_entities, create_entity_outcome, field_key, runtime_with_declared_aspect_schema,
    single_string_aspect_field_patch,
};
use worth_relational::facade::config::CascadeDeletePolicy;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::transactions::{
    EntityMutationIntent, MutationIntent, ReplaceEntityIntent, WorkerIntentBatch,
};

use super::RuntimeBridgeRelationalSource;

#[test]
fn runtime_bridge_snapshot_reader_prefers_retained_observation_over_later_commit_id_collision() {
    let runtime = Arc::new(Mutex::new(runtime_with_test_schema()));
    let source =
        RuntimeBridgeRelationalSource::for_shared_graph_role(Arc::clone(&runtime), "model")
            .expect("test graph role");
    let created = {
        let runtime = runtime.lock().expect("test runtime lock");
        create_entity_outcome(&runtime, "alice")
    };
    let active_entity_identity = active_entity_identity(&created);
    let branch_identity = runtime
        .lock()
        .expect("test runtime lock")
        .branch_identity(&created.commit.branch_id)
        .expect("committed branch identity");
    let (_, basis) = source
        .observe_branch_basis(&branch_identity)
        .expect("owner-admitted active basis");
    let lease = source
        .retain_branch_basis_for_bridge(&basis)
        .expect("retained active observation");
    let active_snapshot_identity = lease.snapshot_identity().clone();

    replace_entity_after_snapshot(&mut runtime.lock().expect("test runtime lock"), &created);

    let reader = source
        .open_snapshot(&active_snapshot_identity)
        .expect("active snapshot should remain bridge-readable after later commit id collision");
    drop(basis);
    assert!(lease.release().released());
    assert!(source.open_snapshot(&active_snapshot_identity).is_err());
    // The already opened reader owns the selected root. This proves exact
    // reading after registration removal, not retention accounting or close.
    let packet = crate::facade::SnapshotReadPacket::new(vec![
        crate::facade::SnapshotReadRequest::for_relational_record(
            active_entity_identity,
            SnapshotReadContract::scalar(aspect_key("name"), ScalarAspectType::String),
        ),
    ]);
    let result = reader
        .read_packet(
            &packet,
            worth_execution::ExecutionRequest::serial(&crate::snapshot::test_serial_request()),
        )
        .expect("bridge snapshot packet should read from active binding");

    assert_eq!(result.snapshot_identity(), &active_snapshot_identity);
    assert_eq!(result.records().len(), 1);
    assert_eq!(
        result.records()[0].scalar_aspect_value(),
        Some(&AspectValue::String("alice".into()))
    );
}

#[test]
fn runtime_bridge_snapshot_reader_requires_a_retained_branch_observation() {
    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&runtime, "managed");
    let branch_id = created.snapshot.branch_id().clone();
    let branch_identity = runtime
        .branch_identity(&branch_id)
        .expect("created branch identity is owner-issued");
    let entity_identity = active_entity_identity(&created);
    assert!(runtime
        .snapshots()
        .release_snapshot(&created.snapshot)
        .is_ok());
    let runtime = Arc::new(runtime);
    let source = RuntimeBridgeRelationalSource::for_graph_role(Arc::clone(&runtime), "model")
        .expect("test graph role");
    let before = retention(&runtime, &branch_identity);
    let (_, basis) = source
        .observe_branch_basis(&branch_identity)
        .expect("Relational owner should admit its exact branch basis");
    let lease = source
        .retain_branch_basis_for_bridge(&basis)
        .expect("Bridge should retain the admitted observation");
    let identity = lease.snapshot_identity().clone();

    let reader = source
        .open_snapshot(&identity)
        .expect("retained observation should authorize Bridge snapshot access");
    let second_reader = source.open_snapshot(&identity).unwrap();
    let opened = retention(&runtime, &branch_identity);
    assert_eq!(opened.observation_acquires, before.observation_acquires + 1);
    assert_eq!(
        opened.external_pin_acquires,
        before.external_pin_acquires + 1
    );
    drop(basis);
    let packet = crate::facade::SnapshotReadPacket::new(vec![
        crate::facade::SnapshotReadRequest::for_relational_record(
            entity_identity,
            SnapshotReadContract::scalar(aspect_key("name"), ScalarAspectType::String),
        ),
    ]);
    assert_eq!(
        reader
            .read_packet(
                &packet,
                worth_execution::ExecutionRequest::serial(&crate::snapshot::test_serial_request())
            )
            .expect("observation packet should read")
            .records()
            .len(),
        1
    );

    assert!(lease.release().released());
    assert!(source.open_snapshot(&identity).is_err());
    let unregistered = retention(&runtime, &branch_identity);
    assert_eq!(
        unregistered.external_pin_releases,
        opened.external_pin_releases + 1
    );
    assert_eq!(
        unregistered.observation_releases,
        opened.observation_releases
    );
    assert_eq!(
        second_reader
            .read_packet(
                &packet,
                worth_execution::ExecutionRequest::serial(&crate::snapshot::test_serial_request())
            )
            .unwrap()
            .records()[0]
            .scalar_aspect_value(),
        Some(&AspectValue::String("managed".into()))
    );
    drop(reader);
    assert_eq!(
        retention(&runtime, &branch_identity).observation_releases,
        opened.observation_releases
    );
    drop(second_reader);
    let released = retention(&runtime, &branch_identity);
    assert_eq!(
        released.observation_releases,
        opened.observation_releases + 1
    );
    assert_eq!(released.observation_acquires, opened.observation_acquires);
    assert_eq!(released.external_pin_acquires, opened.external_pin_acquires);
}

fn retention(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    branch: &worth_relational::facade::branch::RelationalBranchIdentity,
) -> worth_relational::facade::inspection::RelationalRetentionCostCounters {
    runtime
        .branch_retention_cost_counters(branch)
        .expect("owner-issued branch has retention counters")
}

fn active_entity_identity(
    result: &worth_relational::facade::transactions::CommitResult,
) -> RelationalBridgeRecordIdentityParts {
    let entity = changed_entities(result)[0];
    RelationalBridgeRecordIdentityParts::entity(
        entity.partition_id.0,
        entity.local_slot.0,
        entity.generation.0,
    )
}

fn replace_entity_after_snapshot(
    runtime: &mut worth_relational::facade::runtime::RelationalRuntime,
    created: &worth_relational::facade::transactions::CommitResult,
) {
    let mut txn =
        crate::relational_source::relational_test_support::test_owner_begin_transaction_for_main(
            runtime,
        );
    txn.push_batch(
        WorkerIntentBatch::new("update").push(MutationIntent::Entity(
            EntityMutationIntent::Replace(ReplaceEntityIntent {
                entity_id: changed_entities(created)[0],
                replacement: worth_relational::facade::transactions::EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: worth_relational::facade::identity::KindId(1),
                    client_key: worth_relational::facade::symbols::ClientKey::raw("alice"),
                    fields: single_string_aspect_field_patch(
                        crate::relational_source::relational_test_support::aspect_key("name"),
                        field_key("name"),
                        "alice-updated",
                    ),
                },
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    txn.commit(
        runtime,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("second commit should publish");
}

fn runtime_with_test_schema() -> worth_relational::facade::runtime::RelationalRuntime {
    runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations)
}

fn aspect_key(value: &str) -> AspectKey {
    AspectKey::new(value).expect("valid test aspect key")
}
