//! Relation-bearing cold regions remain independently readable and writable.
use super::{region_publications::create_entity, region_source::recover_shape};
use std::sync::atomic::Ordering;
use worth_relational::facade::{
    change_source::RelationalObservationReadDenial, identity::PartitionId,
    storage::RelationalStorageSubject, transactions::RelationalExecutionDenialCause,
};
use worth_testkit_relational::{
    aspect_storage::publications::begin_main_transaction, cold_recovery::update_entity,
};
#[test]
fn a_relation_bearing_cold_commit_loads_only_its_partition() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let cold = recover_shape(execution, None, 16, false);
    create_entity(execution, &cold.runtime, "new", PartitionId(1));
    let reads = cold.source.reads.load(Ordering::Relaxed);
    assert!(reads < 12, "one-partition commit loaded {reads} members");
}

#[test]
fn missing_history_damages_only_its_own_region() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let cold = recover_shape(execution, Some(PartitionId(5)), 16, false);
    // Exercise the damaged cell first, then prove unrelated cells remain usable.
    let (_, basis) = cold
        .runtime
        .observe_branch(&cold.runtime.main_branch_identity())
        .unwrap();
    let read = || {
        cold.runtime
            .entity_record_at_observation(execution, &basis.observation(), cold.entities[4])
    };
    let first = read().unwrap_err();
    assert!(
        matches!(first, RelationalObservationReadDenial::Execution(RelationalExecutionDenialCause::StorageUnavailable(f))
        if f.subject==RelationalStorageSubject::Region(PartitionId(5)))
    );
    let reads = cold.source.reads.load(Ordering::Relaxed);
    assert_eq!(read().unwrap_err(), first);
    assert_eq!(cold.source.reads.load(Ordering::Relaxed), reads);
    for entity in cold
        .entities
        .iter()
        .filter(|entity| entity.partition_id != PartitionId(5))
    {
        let mut reader = begin_main_transaction(&cold.runtime);
        assert!(reader
            .read_entity(execution, *entity)
            .unwrap()
            .effective()
            .is_some());
        create_entity(execution, &cold.runtime, "healthy", entity.partition_id);
    }
}

#[test]
fn fork_of_a_cold_runtime_shares_cells_and_both_copies_can_read_and_write() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let cold = recover_shape(execution, None, 16, false);
    let fork = cold.runtime.fork().unwrap();
    assert_eq!(
        cold.source.reads.load(Ordering::Relaxed),
        0,
        "fork must perform no I/O"
    );
    for (runtime, id, name) in [(&cold.runtime, 1, "origin"), (&fork, 2, "fork")] {
        let mut reader = begin_main_transaction(runtime);
        assert!(reader
            .read_entity(execution, cold.entities[id - 1])
            .unwrap()
            .effective()
            .is_some());
        create_entity(execution, runtime, name, PartitionId(id as u32));
        update_entity(execution, runtime, cold.entities[id - 1], name);
    }
    for runtime in [&cold.runtime, &fork] {
        for entity in &cold.entities {
            let mut reader = begin_main_transaction(runtime);
            assert!(reader
                .read_entity(execution, *entity)
                .unwrap()
                .effective()
                .is_some());
        }
    }
}
