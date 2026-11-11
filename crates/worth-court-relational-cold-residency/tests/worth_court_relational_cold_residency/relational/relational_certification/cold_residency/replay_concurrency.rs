//! Public replay references retain and release through their actual owner.
use worth_relational::facade::{identity::*, transactions::*};
use worth_testkit_relational::{
    aspect_storage::publications::{begin_main_transaction, release_test_commit_snapshot},
    durable_publication::{field_patch, memory_runtime},
};
fn name_field_patch(value: &str) -> AspectFieldPatch {
    field_patch("name", "name", value)
}
#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "Court harness contention: eight acquisition workers then eight release workers; peak eight plus caller and two registry threads"
)]
fn concurrent_replay_references_release_each_original_reference_once() {
    let backing = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&backing);
    let runtime = memory_runtime();
    let keys = (0..500)
        .map(|id| worth_relational::facade::symbols::ClientKey::raw(format!("concurrent-pin-{id}")))
        .collect::<Vec<_>>();
    let mut transaction = begin_main_transaction(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("concurrent-pin-population").push(MutationIntent::Create(
                CreateIntent::BulkEntities(
                    worth_relational::facade::transactions::BulkEntityCreateIntent {
                        partition_id: PartitionId::main(),
                        kind_id: KindId(1),
                        field_patches: keys.iter().map(|_| name_field_patch("pin-data")).collect(),
                        client_keys: keys,
                    },
                ),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
    let committed = transaction.commit(execution, &runtime).unwrap();
    release_test_commit_snapshot(&runtime, &committed);
    let version = committed.version_id;
    let start = std::sync::Barrier::new(8);
    std::thread::scope(|scope| {
        let workers = (0..8)
            .map(|_| {
                scope.spawn(|| {
                    let backing = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
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
    assert!(!runtime
        .history_authority()
        .release_version_replay_retention(version));
}
