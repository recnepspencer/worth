//! Read admission must never build a durable schema-authority projection.
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    identity::KindId,
    runtime::RelationalRuntimeApi,
    schema::{EntityKindRegistration, RelationalSchemaRegistry, SchemaId, SchemaVersionId},
};

#[test]
fn pinned_snapshot_and_view_admission_build_no_schema_projection() {
    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "read.entity".into(),
            schema_id: SchemaId("read".into()),
            schema_version_id: SchemaVersionId(7),
            aspect_contract_declarations: Default::default(),
        })
        .unwrap();
    let (_, positive) = RelationalSchemaRegistry::count_authority_projections_for_test(|| {
        registry.authority_snapshot()
    });
    assert!(
        positive > 0,
        "recorder must observe genuine projection construction"
    );
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .build()
        .unwrap();
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let (_, projections) = RelationalSchemaRegistry::count_authority_projections_for_test(|| {
        let (_, basis) = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap();
        let observation = basis.observation();
        let snapshot = runtime
            .snapshots()
            .snapshot_for_observation(&observation)
            .unwrap();
        runtime
            .read_truth()
            .read_observation(&execution, &observation)
            .unwrap()
            .unwrap();
        runtime
            .read_truth()
            .project_observation(&observation)
            .unwrap();
        assert_eq!(
            runtime.read_truth().snapshot_schema_version(&snapshot),
            Some(SchemaVersionId(7))
        );
        let pin = runtime
            .read_truth()
            .pin_read_capability(&snapshot, &[], &execution)
            .unwrap();
        let view = runtime.read_truth().project_snapshot(&snapshot).unwrap();
        let view_pin = runtime
            .read_truth()
            .pin_view_read_capability(&view, &[], &execution)
            .unwrap();
        assert_eq!(pin.read_context().schema_version, SchemaVersionId(7));
        assert_eq!(view_pin.read_context().schema_version, SchemaVersionId(7));
    });
    assert_eq!(
        projections, 0,
        "read admission reconstructed schema authority"
    );
}

#[test]
fn fixed_public_commit_work_does_not_follow_schema_width() {
    use worth_testkit_relational::maintenance::assert_change_bounded;
    let small = fixed_commit_projection_work(512);
    let large = fixed_commit_projection_work(1024);
    eprintln!("SCHEMA_PROJECTION_WORK installed=512:{small} 1024:{large}");
    assert_change_bounded(
        "public commit schema projection",
        "installed kinds",
        small.max(1),
        large.max(1),
    );
}

fn fixed_commit_projection_work(kinds: u32) -> u64 {
    use worth_relational::facade::{identity::PartitionId, symbols::ClientKey, transactions::*};
    let mut registry = RelationalSchemaRegistry::new();
    for kind in 1..=kinds {
        registry = registry
            .register_entity_kind(EntityKindRegistration {
                kind_id: KindId(kind),
                kind_name: format!("kind-{kind}"),
                schema_id: SchemaId("growth".into()),
                schema_version_id: SchemaVersionId(1),
                aspect_contract_declarations: Default::default(),
            })
            .unwrap();
    }
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .build()
        .unwrap();
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let commit = |key: &str| {
        let (_, basis) = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap();
        let mut tx = runtime
            .begin_branch_transaction(
                &basis,
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            )
            .unwrap();
        tx.push_batch(
            WorkerIntentBatch::new("one-record").push(MutationIntent::Create(
                CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId(7),
                    kind_id: KindId(1),
                    client_key: ClientKey::raw(key),
                    fields: Default::default(),
                }),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
        let result = tx.commit(&execution, &runtime).unwrap();
        runtime
            .snapshots()
            .release_snapshot(&result.snapshot)
            .unwrap();
    };
    commit("warm");
    RelationalSchemaRegistry::observe_authority_projection_work(|| commit("edit")).1
}

#[test]
fn fixed_transaction_read_does_not_replay_staged_history() {
    use worth_testkit_relational::maintenance::assert_change_bounded;
    let a = repeated_read_work(64);
    let b = repeated_read_work(128);
    assert_change_bounded(
        "transaction read projection",
        "staged history",
        a.max(1),
        b.max(1),
    );
}

fn repeated_read_work(stages: usize) -> u64 {
    use crate::content::historical_kind::fixture::World;
    use crate::leased_preparation::ordinary_context::support;
    use worth_relational::facade::{mvcc::*, transactions::*};
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let mut world = World::new();
    world.create(&execution, 7, "before");
    let entity_id = world.entities[0];
    let (_, basis) = world
        .runtime
        .observe_branch(&world.runtime.main_branch_identity())
        .unwrap();
    let mut tx = world
        .runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    for n in 0..stages {
        tx.push_batch(
            WorkerIntentBatch::new("append").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id,
                    fields: support::string_field_patch(
                        support::aspect_key("name"),
                        support::field_key("name"),
                        &format!("value-{n}"),
                    ),
                }),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
    }
    let (expected, cold_work) =
        BranchBoundRelationalTransaction::observe_read_projection_work(|| {
            tx.read_entity(&execution, entity_id).unwrap()
        });
    assert!(
        cold_work > 0,
        "the positive control must actually project the staged records"
    );
    let (actual, work) = BranchBoundRelationalTransaction::observe_read_projection_work(|| {
        tx.read_entity(&execution, entity_id).unwrap()
    });
    assert_eq!(actual, expected);
    eprintln!("TRANSACTION_PROJECTION_WORK staged={stages} cold={cold_work} repeated={work}");
    work
}
