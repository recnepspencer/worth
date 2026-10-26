//! Concurrent replay references have one physical pin owner.
use super::*;

#[test]
fn concurrent_replay_references_share_one_physical_pin_set() {
    let backing = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&backing);
    let runtime = runtime_with_test_schema();
    let keys = (0..500)
        .map(|id| crate::symbols::data::ClientKey::raw(format!("concurrent-pin-{id}")))
        .collect::<Vec<_>>();
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("concurrent-pin-population").push(MutationIntent::Create(
                CreateIntent::BulkEntities(crate::transactions::data::BulkEntityCreateIntent {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    field_patches: keys.iter().map(|_| name_field_patch("pin-data")).collect(),
                    client_keys: keys,
                }),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
    let committed = transaction.commit(execution, &runtime).unwrap();
    let entity = changed_entities(&committed)[0];
    release_test_commit_snapshot(&runtime, &committed);
    let version = runtime.current_version_id();
    let start = std::sync::Barrier::new(8);
    std::thread::scope(|scope| {
        let workers = (0..8)
            .map(|_| {
                scope.spawn(|| {
                    let backing = crate::test_execution_policy::serial_request();
                    let execution = &worth_execution::ExecutionRequest::serial(&backing);
                    start.wait();
                    assert_eq!(
                        runtime
                            .history_authority()
                            .retain_version_for_replay(execution, version),
                        Ok(true)
                    );
                })
            })
            .collect::<Vec<_>>();
        for worker in workers {
            worker.join().unwrap();
        }
    });
    let count = || {
        runtime
            .partitions
            .partition(entity.partition_id)
            .unwrap()
            .entity_arena
            .replay_pin_count(entity.slot_index())
            .unwrap()
    };
    assert_eq!(
        count(),
        1,
        "concurrent references share exactly one physical pin set"
    );
    std::thread::scope(|scope| {
        let workers = (0..8)
            .map(|_| {
                scope.spawn(|| {
                    start.wait();
                    assert!(runtime
                        .history_authority()
                        .release_version_replay_retention(version));
                })
            })
            .collect::<Vec<_>>();
        for worker in workers {
            worker.join().unwrap();
        }
    });
    assert_eq!(count(), 0);
    assert!(!runtime
        .history_authority()
        .release_version_replay_retention(version));
}
