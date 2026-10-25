//! Real embedded readmission measured through the process allocator. The
//! isolated child excludes allocations from concurrently running courts.
use crate::identity::data::{KindId, PartitionId};
use crate::tests::support::*;
use worth_execution::ExecutionRequest;

#[test]
fn embedded_restore_allocation_grows_linearly_in_relation_bearing_regions() {
    const PROBE: &str = "WORTH_RELATIONAL_EMBEDDED_RESTORE_ALLOCATION_PROBE";
    if std::env::var_os(PROBE).is_none() {
        let name = concat!(
            module_path!(),
            "::embedded_restore_allocation_grows_linearly_in_relation_bearing_regions"
        );
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                name.trim_start_matches("worth_relational::"),
                "--nocapture",
            ])
            .env(PROBE, "1")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            String::from_utf8_lossy(&output.stdout).contains("embedded restore allocations:"),
            "child must execute the probe"
        );
        return;
    }
    let narrow = measured_restore(8);
    let wide = measured_restore(32);
    println!("embedded restore allocations: 8 regions={narrow}, 32 regions={wide}");
    assert!(
        wide <= narrow.saturating_mul(7) + 256 * 1024,
        "four times the regions must not cause quadratic allocation: {narrow} -> {wide}"
    );
}

fn measured_restore(partitions: u32) -> usize {
    let host = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = durable_runtime_with_test_schema();
    for id in 1..=partitions {
        let mut txn = test_owner_begin_transaction_for_main(&runtime);
        txn.push_batch(
            WorkerIntentBatch::new("embedded-region").push(
                crate::transactions::data::MutationIntent::Create(
                    crate::transactions::data::CreateIntent::BulkEntities(
                        crate::transactions::data::BulkEntityCreateIntent {
                            partition_id: PartitionId(id),
                            kind_id: KindId(1),
                            client_keys: (0..128)
                                .map(|n| {
                                    crate::symbols::data::ClientKey::raw(format!("entity-{n}"))
                                })
                                .collect(),
                            field_patches: (0..128)
                                .map(|_| name_field_patch("same payload"))
                                .collect(),
                        },
                    ),
                ),
            ),
            execution.allocation_policy(),
        )
        .unwrap();
        let outcome = txn.commit(&execution, &runtime).unwrap();
        let entities = changed_entities(&outcome);
        release_test_commit_snapshot(&runtime, &outcome);
        create_relation_in_partition(
            &execution,
            &runtime,
            entities[0],
            entities[1],
            "edge",
            PartitionId(id),
        );
    }
    let native = runtime
        .durability_authority()
        .native_checkpoint(execution.allocation_policy())
        .unwrap();
    let mut receiving = durable_runtime_with_test_schema();
    let allocations = stats_alloc::Region::new(&stats_alloc::INSTRUMENTED_SYSTEM);
    receiving
        .durability_recovery()
        .restore_native_checkpoint(&execution, &native)
        .unwrap();
    allocations.change().bytes_allocated
}
