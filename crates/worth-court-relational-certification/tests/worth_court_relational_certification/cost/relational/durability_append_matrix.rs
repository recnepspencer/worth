//! Counted-work certification for durable canonical envelope appends over
//! `TestDurableMedium` (fresh and existing medium heads).
use worth_execution::{
    CancellationToken, ExecutionRequest, ExecutionWorkCeiling, SerialMemoryBudget, SerialRequest,
};
use worth_relational::facade::{
    config::{CascadeDeletePolicy, RelationalRuntimeProfile},
    identity::{KindId, PartitionId},
    runtime::RelationalRuntimeApi,
    symbols::ClientKey,
    transactions::{CreateIntent, EntitySpec, MutationIntent, WorkerIntentBatch},
};
use worth_testkit_relational::{
    aspect_storage::{
        aspect_key, field_key,
        publications::{
            begin_main_transaction, release_test_commit_snapshot, string_aspect_field_patch,
        },
        AspectSchemaFixture,
    },
    durable_medium::TestDurableMedium,
};

fn build_durable_runtime() -> (
    std::sync::Arc<TestDurableMedium>,
    worth_relational::facade::runtime::RelationalRuntime,
) {
    let fixture = AspectSchemaFixture::with_default_declared_aspects(
        CascadeDeletePolicy::CascadeDeleteRelations,
    );
    let medium = TestDurableMedium::new();
    let runtime = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(fixture.build_registry())
        .build_durable(medium.attachment())
        .expect("valid durable native runtime");
    (medium, runtime)
}

fn commit_entity(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    name: &str,
) -> u64 {
    let ((), report) = execution
        .run(ExecutionWorkCeiling::new(None), |_| {
            let mut txn = begin_main_transaction(runtime);
            txn.push_batch(
                WorkerIntentBatch::new("durability-append").push(MutationIntent::Create(
                    CreateIntent::Entity(EntitySpec {
                        partition_id: PartitionId::main(),
                        kind_id: KindId(1),
                        client_key: ClientKey::raw(name),
                        fields: string_aspect_field_patch([(
                            aspect_key("name"),
                            field_key("name"),
                            name,
                        )]),
                    }),
                )),
                execution.allocation_policy(),
            )
            .expect("stage entity create");
            let outcome = txn
                .commit(execution, runtime)
                .expect("durable commit succeeds");
            release_test_commit_snapshot(runtime, &outcome);
        })
        .expect("counted work run succeeds");
    report.charged_work()
}

#[test]
fn perf_durability_append_matrix() {
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationToken::new(),
        None,
    );
    let execution = &ExecutionRequest::serial(&host);

    const SAMPLES: usize = 3;

    // Fresh medium: first durable commit advances medium head to 1 and log to 1.
    let mut fresh_work = Vec::with_capacity(SAMPLES);
    for _ in 0..SAMPLES {
        let (medium, runtime) = build_durable_runtime();
        let work = commit_entity(execution, &runtime, "fresh-source");
        let committed = medium.committed_artifacts().expect("valid stored frames");
        assert_eq!(committed.len(), 1);
        assert_eq!(committed[0].position().0, 1);
        assert_eq!(runtime.durability().durable_commit_count(), 1);
        assert!(work > 0, "fresh durable append must charge counted work");
        fresh_work.push(work);
    }

    // Existing medium: second durable commit extends medium head to 2 and log to 2.
    let mut warm_work = Vec::with_capacity(SAMPLES);
    for _ in 0..SAMPLES {
        let (medium, runtime) = build_durable_runtime();
        let _ = commit_entity(execution, &runtime, "warm-source-a");
        let work = commit_entity(execution, &runtime, "warm-source-b");
        let committed = medium.committed_artifacts().expect("valid stored frames");
        assert_eq!(committed.len(), 2);
        assert_eq!(committed[1].position().0, 2);
        assert_eq!(runtime.durability().durable_commit_count(), 2);
        assert!(work > 0, "warm durable append must charge counted work");
        warm_work.push(work);
    }

    assert_eq!(
        fresh_work[0], fresh_work[1],
        "identical fresh appends must charge deterministic counted work"
    );
    assert_eq!(
        warm_work[0], warm_work[1],
        "identical warm appends must charge deterministic counted work"
    );
}
