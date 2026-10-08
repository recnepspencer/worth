use std::sync::{Arc, Mutex};

use crate::facade::{
    BridgeRouteRequest, CommittedPatchSource, RelationalBridgeRecordIdentityParts,
    RelationalCommittedPatchRequest, RuntimeBridgeBuilder, SnapshotReadContract,
    SnapshotReadPacket, SnapshotReadRequest, SnapshotReadSource, TruthCommitIdentity,
};
use crate::relational_source::relational_test_support::{
    aspect_key, create_entity_outcome, field_key, single_string_aspect_field_patch,
};
use worth_foundational::facade::ScalarAspectType;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::transactions::{
    CreateIntent, EntitySpec, MutationIntent, WorkerIntentBatch,
};

use super::super::RuntimeBridgeRelationalSource;
use super::support::{
    exact_aspect_registration, exact_registration, register_remaining_patch_items,
    runtime_with_test_schema, TestSink,
};

#[test]
fn runtime_bridge_relational_source_exposes_latest_publication_bundle_authoritatively() {
    let runtime = runtime_with_test_schema();
    create_entity_outcome(&runtime, "alice");

    let bundle = runtime
        .publication()
        .latest_bundle()
        .expect("runtime publication bundle")
        .clone();
    let branch_identity = runtime
        .branch_identity(&bundle.commit.branch_id)
        .expect("committed branch identity");
    let expected_commit_identity = crate::facade::RelationalCommittedPatchRequest::new(
        TruthCommitIdentity::from_relational_commit_id(bundle.commit.commit_id.0),
    );

    let source = RuntimeBridgeRelationalSource::for_graph_role(Arc::new(runtime), "model")
        .expect("test graph role");
    let (_, basis) = source
        .observe_branch_basis(&branch_identity)
        .expect("owner-admitted publication basis");
    let lease = source
        .retain_branch_basis_for_bridge(&basis)
        .expect("retained publication observation");
    let expected_snapshot_identity = lease.snapshot_identity().clone();
    let envelope = source
        .load_committed_patch(expected_commit_identity)
        .expect("runtime bridge committed patch");
    let reader = source
        .open_snapshot(&expected_snapshot_identity)
        .expect("runtime bridge snapshot reader");

    assert_eq!(envelope.snapshot_identity(), &expected_snapshot_identity);
    assert_eq!(reader.snapshot_identity(), expected_snapshot_identity);
    assert!(envelope
        .patch_body()
        .canonical_items()
        .iter()
        .any(|item| item.aspect_key().as_str() == "name"));
}

#[test]
fn relational_source_graph_role_is_explicit_and_validated() {
    let runtime = Arc::new(runtime_with_test_schema());
    assert!(matches!(
        RuntimeBridgeRelationalSource::for_graph_role(runtime, " model "),
        Err(super::super::RelationalBridgeSourceConfigurationError::InvalidGraphRole)
    ));
}

#[test]
fn partition_source_filters_the_real_commit_and_retains_exact_partition_provenance() {
    let runtime = runtime_with_test_schema();
    let entity = |partition_id, key: &str| {
        MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id,
            kind_id: worth_relational::facade::identity::KindId(1),
            client_key: worth_relational::facade::symbols::ClientKey::raw(key),
            fields: single_string_aspect_field_patch(
                crate::relational_source::relational_test_support::aspect_key("name"),
                field_key("name"),
                key,
            ),
        }))
    };
    let mut transaction =
        crate::relational_source::relational_test_support::test_owner_begin_transaction_for_main(
            &runtime,
        );
    transaction
        .push_batch(
            WorkerIntentBatch::new("partition-publication")
                .push(entity(PartitionId::main(), "main"))
                .push(entity(PartitionId::new(7), "secondary")),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let committed = transaction
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let secondary = committed
        .changed_records
        .iter()
        .find_map(|record| match record {
            worth_relational::facade::transactions::RecordRef::Entity(entity)
                if entity.partition_id == PartitionId::new(7) =>
            {
                Some(entity)
            }
            _ => None,
        })
        .unwrap();
    let truth_partition =
        worth_foundational::facade::TruthPartitionRole::new("model-main").unwrap();
    let branch_identity = runtime
        .branch_identity(&committed.commit.branch_id)
        .expect("committed branch identity");
    let source = RuntimeBridgeRelationalSource::for_graph_partition(
        Arc::new(runtime),
        "model",
        PartitionId::main(),
        truth_partition.clone(),
    )
    .unwrap();
    let (_, basis) = source
        .observe_branch_basis(&branch_identity)
        .expect("owner-admitted partition basis");
    let _lease = source
        .retain_branch_basis_for_bridge(&basis)
        .expect("retained partition observation");
    let envelope = source
        .load_committed_patch(RelationalCommittedPatchRequest::new(
            TruthCommitIdentity::from_relational_commit_id(committed.commit.commit_id.0),
        ))
        .unwrap();

    let provenance = envelope.producer_metadata().authoritative_source().unwrap();
    assert_eq!(provenance.partition_role(), Some(&truth_partition));
    let counters = envelope.patch_summary().authoritative_lowering();
    assert_eq!(counters.source_record_patches_examined, 2);
    assert_eq!(counters.source_record_patches_filtered_out, 1);
    assert_eq!(counters.record_patches_inspected, 1);
    assert!(envelope.patch_body().canonical_items().iter().all(|item| {
        item.relational_record_identity_parts()
            .is_some_and(|record| record.partition_id() == 0)
    }));

    let reader = source.open_snapshot(envelope.snapshot_identity()).unwrap();
    let packet = SnapshotReadPacket::new(vec![SnapshotReadRequest::for_relational_record(
        RelationalBridgeRecordIdentityParts::entity(
            secondary.partition_id.0,
            secondary.local_slot.0,
            secondary.generation.0,
        ),
        SnapshotReadContract::scalar(aspect_key("name"), ScalarAspectType::String),
    )]);
    assert!(reader.read_packet(&packet).is_err());
}

#[test]
fn runtime_bridge_relational_source_drives_public_bridge_delivery_with_canonical_snapshot_authority(
) {
    let runtime = runtime_with_test_schema();
    create_entity_outcome(&runtime, "alice");

    let bundle = runtime
        .publication()
        .latest_bundle()
        .expect("runtime publication bundle")
        .clone();
    let commit_identity = TruthCommitIdentity::from_relational_commit_id(bundle.commit.commit_id.0);
    let branch_identity = runtime
        .branch_identity(&bundle.commit.branch_id)
        .expect("committed branch identity");

    let source = RuntimeBridgeRelationalSource::for_graph_role(Arc::new(runtime), "model")
        .expect("test graph role");
    let (_, basis) = source
        .observe_branch_basis(&branch_identity)
        .expect("owner-admitted delivery basis");
    let lease = source
        .retain_branch_basis_for_bridge(&basis)
        .expect("retained delivery observation");
    let expected_snapshot_identity = lease.snapshot_identity().clone();
    let envelope = source
        .load_committed_patch(RelationalCommittedPatchRequest::new(
            commit_identity.clone(),
        ))
        .expect("runtime bridge committed patch");
    let first_patch_item = envelope
        .patch_body()
        .canonical_items()
        .first()
        .expect("runtime bridge envelope should contain at least one patch item");
    let mut builder = RuntimeBridgeBuilder::new()
        .with_relational_source(source.clone())
        .with_signal_sink(TestSink)
        .with_continuity_lineage_source(source.clone())
        .register_mapping(exact_registration(
            "runtime-publication-item-0",
            first_patch_item,
        ))
        .register_aspect_mapping(exact_aspect_registration(
            "runtime-publication-item-field-0",
            first_patch_item,
        ));
    register_remaining_patch_items!(builder, envelope, "runtime-publication");
    let bridge = builder
        .build()
        .expect("runtime bridge should build from runtime-backed relational source");

    let route = bridge
        .plan_committed_patch(BridgeRouteRequest::for_commit(commit_identity))
        .expect("runtime-backed relational bridge route");
    let result = bridge
        .deliver_invalidation(route)
        .expect("runtime-backed relational bridge delivery");

    assert_eq!(
        result.result_summary().snapshot_identity(),
        &expected_snapshot_identity
    );
    assert_eq!(
        result.receipt().snapshot_identity(),
        &expected_snapshot_identity
    );
}

#[test]
fn runtime_bridge_replays_historical_commit_after_newer_publication_arrives() {
    let runtime = Arc::new(Mutex::new(runtime_with_test_schema()));
    let source =
        RuntimeBridgeRelationalSource::for_shared_graph_role(Arc::clone(&runtime), "model")
            .expect("test graph role");
    let historical_commit = {
        let runtime = runtime.lock().expect("test runtime lock");
        create_entity_outcome(&runtime, "alice").commit.clone()
    };
    let branch_identity = runtime
        .lock()
        .expect("test runtime lock")
        .branch_identity(&historical_commit.branch_id)
        .expect("historical branch identity");
    let (_, historical_basis) = source
        .observe_branch_basis(&branch_identity)
        .expect("owner-admitted historical basis");
    let historical_lease = source
        .retain_branch_basis_for_bridge(&historical_basis)
        .expect("retained historical observation");
    let historical_commit_id = historical_commit.commit_id;

    {
        let runtime = runtime.lock().expect("test runtime lock");
        let mut txn = crate::relational_source::relational_test_support::test_owner_begin_transaction_for_main(&runtime);
        txn.push_batch(
            WorkerIntentBatch::new("update").push(MutationIntent::Create(
                worth_relational::facade::transactions::CreateIntent::Entity(
                    worth_relational::facade::transactions::EntitySpec {
                        partition_id: PartitionId::main(),
                        kind_id: worth_relational::facade::identity::KindId(1),
                        client_key: worth_relational::facade::symbols::ClientKey::raw("bob"),
                        fields: single_string_aspect_field_patch(
                            crate::relational_source::relational_test_support::aspect_key("name"),
                            field_key("name"),
                            "bob",
                        ),
                    },
                ),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
        txn.commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("second commit should publish");
    }
    let historical_commit_identity = RelationalCommittedPatchRequest::new(
        TruthCommitIdentity::from_relational_commit_id(historical_commit_id.0),
    );
    let envelope = source
        .load_committed_patch(historical_commit_identity.clone())
        .expect("historical bridge committed patch");
    let expected_snapshot_identity = envelope.snapshot_identity().clone();
    assert_eq!(
        &expected_snapshot_identity,
        historical_lease.snapshot_identity(),
    );
    let first_patch_item = envelope
        .patch_body()
        .canonical_items()
        .first()
        .expect("historical bridge envelope should contain at least one patch item");
    let mut builder = RuntimeBridgeBuilder::new()
        .with_relational_source(source.clone())
        .with_signal_sink(TestSink)
        .with_continuity_lineage_source(source.clone())
        .register_mapping(exact_registration(
            "historical-publication-item-0",
            first_patch_item,
        ))
        .register_aspect_mapping(exact_aspect_registration(
            "historical-publication-item-field-0",
            first_patch_item,
        ));
    register_remaining_patch_items!(builder, envelope, "historical-publication");
    let bridge = builder
        .build()
        .expect("runtime bridge should build from runtime-backed relational source");

    let planned = bridge
        .plan_committed_patch(BridgeRouteRequest::for_commit(
            historical_commit_identity.commit_identity().clone(),
        ))
        .expect("historical route should still plan after newer publication");
    let result = bridge
        .deliver_invalidation(planned)
        .expect("historical route should still deliver after newer publication");
    let canonical = bridge
        .diagnostics()
        .last_canonical_route_record()
        .expect("historical route record");
    let replay = bridge
        .replay_canonical_record(&canonical)
        .expect("historical replay should remain reconstructable");

    assert_eq!(
        result.result_summary().snapshot_identity(),
        &expected_snapshot_identity
    );
    assert_eq!(
        result.receipt().snapshot_identity(),
        &expected_snapshot_identity
    );
    assert_eq!(replay.source_snapshot(), &expected_snapshot_identity);
}
