use std::sync::{Arc, Mutex};

use crate::facade::identity::PartitionId;
use crate::facade::transactions::{CreateIntent, EntitySpec, MutationIntent, WorkerIntentBatch};
use crate::tests::support::{aspect_key, field_key, single_string_aspect_field_patch};
use worth_foundational::ScalarAspectType;
use worth_runtime_bridge::facade::{
    CommittedPatchSource, RelationalBridgeRecordIdentityParts, RelationalCommittedPatchRequest,
    SnapshotReadContract, SnapshotReadPacket, SnapshotReadRequest, SnapshotReadSource,
    TruthCommitIdentity, TruthSnapshotReader,
};

use super::super::RuntimeBridgeRelationalSource;
use super::support::runtime_with_test_schema;

#[test]
fn shared_source_retains_the_live_runtime_authority_and_observes_later_commits() {
    let runtime = Arc::new(Mutex::new(runtime_with_test_schema()));
    let source =
        RuntimeBridgeRelationalSource::for_shared_graph_role(Arc::clone(&runtime), "model")
            .expect("shared source should accept the installed graph role");
    let expected_runtime_id = runtime
        .lock()
        .expect("test runtime lock")
        .runtime_instance_id();

    assert_eq!(
        source.authoritative_source_profile().runtime_instance_id(),
        expected_runtime_id
    );

    let committed = {
        let runtime = runtime.lock().expect("test runtime lock");
        let mut transaction =
            crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
        transaction
            .push_batch(WorkerIntentBatch::new("shared-authority-create").push(
                MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: crate::facade::identity::KindId(1),
                    client_key: crate::facade::symbols::ClientKey::raw("alice"),
                    fields: single_string_aspect_field_patch(
                        aspect_key("name"),
                        field_key("name"),
                        "alice",
                    ),
                })),
            ))
            .expect("test staging stays within configured resource budgets");
        transaction
            .commit(&runtime)
            .expect("real shared-runtime commit")
    };
    let entity = committed
        .changed_records
        .iter()
        .find_map(|record| match record {
            crate::facade::transactions::RecordRef::Entity(entity) => Some(entity),
            crate::facade::transactions::RecordRef::Relation(_) => None,
        })
        .expect("created entity");
    let branch_identity = runtime
        .lock()
        .expect("test runtime lock")
        .branch_identity(&committed.commit.branch_id)
        .expect("committed branch identity");
    let (_, basis) = source
        .observe_branch_basis(&branch_identity)
        .expect("source must observe the live owner basis");
    let lease = source
        .retain_branch_basis_for_bridge(&basis)
        .expect("source must retain the live owner observation");
    let envelope = source
        .load_committed_patch(RelationalCommittedPatchRequest::new(
            TruthCommitIdentity::from_relational_commit_id(committed.commit.commit_id.0),
        ))
        .expect("source must observe commits made after its construction");
    let reader = source
        .clone()
        .open_retained_snapshot(lease)
        .expect("a source clone must open the exact retained observation");
    assert_eq!(reader.snapshot_identity(), *envelope.snapshot_identity());
    let packet = SnapshotReadPacket::new(vec![SnapshotReadRequest::for_relational_record(
        RelationalBridgeRecordIdentityParts::entity(
            entity.partition_id.0,
            entity.local_slot.0,
            entity.generation.0,
        ),
        SnapshotReadContract::scalar(aspect_key("name"), ScalarAspectType::String),
    )]);
    let result = reader
        .read_packet(&packet)
        .expect("snapshot must be read through the same shared runtime");

    assert_eq!(result.records().len(), 1);
    assert_eq!(
        result.records()[0].scalar_aspect_value(),
        Some(&worth_foundational::facade::AspectValue::String(
            "alice".into()
        ))
    );
    assert!(source.open_snapshot(envelope.snapshot_identity()).is_ok());
    drop(reader);
    assert!(source.open_snapshot(envelope.snapshot_identity()).is_err());
}

#[test]
fn exact_reader_rejects_a_foreign_lease_despite_equal_snapshot_descriptors() {
    fn retained_source() -> (
        RuntimeBridgeRelationalSource,
        crate::facade::bridge::RelationalBridgeObservationLease,
    ) {
        let runtime = runtime_with_test_schema();
        crate::tests::support::create_entity_outcome(&runtime, "retained-reader");
        let identity = runtime
            .branch_identity(&crate::history::data::BranchId("main".to_owned()))
            .unwrap();
        let source =
            RuntimeBridgeRelationalSource::for_graph_role(Arc::new(runtime), "model").unwrap();
        let (_, basis) = source.observe_branch_basis(&identity).unwrap();
        let lease = source.retain_branch_basis_for_bridge(&basis).unwrap();
        (source, lease)
    }

    let (first, first_lease) = retained_source();
    let (second, second_lease) = retained_source();
    let snapshot = first_lease.snapshot_identity().clone();
    assert_eq!(snapshot, *second_lease.snapshot_identity());
    assert!(second.open_snapshot(&snapshot).is_ok());

    let denial = second.open_retained_snapshot(first_lease).unwrap_err();
    assert!(denial
        .to_string()
        .contains("another source registration owner"));
    assert!(first.open_snapshot(&snapshot).is_err());

    let valid = second.open_retained_snapshot(second_lease).unwrap();
    assert_eq!(valid.snapshot_identity(), snapshot);
    assert!(second.open_snapshot(&snapshot).is_ok());
    drop(valid);
    assert!(second.open_snapshot(&snapshot).is_err());
}
