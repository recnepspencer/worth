//! Real embedded readmission measured through the process allocator. The
//! isolated child excludes allocations from concurrently running courts.
use worth_execution::{CancellationToken, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::{
    config::CascadeDeletePolicy,
    identity::{KindId, PartitionId},
    symbols::ClientKey,
    transactions::{
        BulkEntityCreateIntent, CreateIntent, EntityReference, MutationIntent, RecordRef,
        RelationSpec, WorkerIntentBatch,
    },
};
use worth_testkit_relational::aspect_storage::{
    aspect_key, field_key,
    publications::{
        begin_main_transaction, release_test_commit_snapshot, string_aspect_field_patch,
    },
    AspectSchemaFixture,
};

#[test]
fn embedded_restore_allocation_grows_linearly_in_relation_bearing_regions() {
    const PROBE: &str = "WORTH_RELATIONAL_EMBEDDED_RESTORE_ALLOCATION_PROBE";
    if std::env::var_os(PROBE).is_none() {
        let name = concat!(
            module_path!(),
            "::embedded_restore_allocation_grows_linearly_in_relation_bearing_regions"
        );
        let filter = name
            .strip_prefix("worth_court_relational_certification::")
            .unwrap_or(name);
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", filter, "--test-threads=1", "--nocapture"])
            .env(PROBE, "1")
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "{stdout}\n{stderr}");
        assert!(
            stdout.contains("embedded restore allocations:") && stdout.contains("1 passed"),
            "child must execute the probe:\n{stdout}\n{stderr}"
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
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationToken::new(),
        None,
    );
    let execution = ExecutionRequest::serial(&host);
    let fixture = AspectSchemaFixture::with_default_declared_aspects(
        CascadeDeletePolicy::CascadeDeleteRelations,
    );
    let runtime = fixture.build_runtime();
    for id in 1..=partitions {
        let mut txn = begin_main_transaction(&runtime);
        txn.push_batch(
            WorkerIntentBatch::new("embedded-region").push(MutationIntent::Create(
                CreateIntent::BulkEntities(BulkEntityCreateIntent {
                    partition_id: PartitionId(id),
                    kind_id: KindId(1),
                    client_keys: (0..128)
                        .map(|n| ClientKey::raw(format!("entity-{n}")))
                        .collect(),
                    field_patches: (0..128)
                        .map(|_| {
                            string_aspect_field_patch([(
                                aspect_key("name"),
                                field_key("name"),
                                "same payload",
                            )])
                        })
                        .collect(),
                }),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
        let outcome = txn.commit(&execution, &runtime).unwrap();
        let entities = outcome
            .changed_records
            .iter()
            .filter_map(|record| match record {
                RecordRef::Entity(entity_id) => Some(*entity_id),
                RecordRef::Relation(_) => None,
            })
            .collect::<Vec<_>>();
        release_test_commit_snapshot(&runtime, &outcome);

        let mut rel_txn = begin_main_transaction(&runtime);
        rel_txn
            .push_batch(
                WorkerIntentBatch::new("embedded-region-edge").push(MutationIntent::Create(
                    CreateIntent::Relation(RelationSpec {
                        partition_id: PartitionId(id),
                        kind_id: KindId(2),
                        client_key: ClientKey::raw("edge"),
                        source: EntityReference::Existing(entities[0]),
                        target: EntityReference::Existing(entities[1]),
                        fields: string_aspect_field_patch([(
                            aspect_key("label"),
                            field_key("label"),
                            "edge",
                        )]),
                    }),
                )),
                execution.allocation_policy(),
            )
            .unwrap();
        let rel_outcome = rel_txn.commit(&execution, &runtime).unwrap();
        release_test_commit_snapshot(&runtime, &rel_outcome);
    }
    let native = runtime
        .durability_authority()
        .native_checkpoint(&execution, execution.allocation_policy())
        .unwrap();
    let mut receiving = fixture.build_runtime();
    let allocations = stats_alloc::Region::new(&stats_alloc::INSTRUMENTED_SYSTEM);
    receiving
        .durability_recovery()
        .restore_native_checkpoint(&execution, &native)
        .unwrap();
    allocations.change().bytes_allocated
}
