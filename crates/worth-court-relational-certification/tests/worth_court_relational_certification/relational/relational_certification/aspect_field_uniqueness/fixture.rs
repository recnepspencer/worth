use super::*;
use worth_relational::facade::runtime::{
    InvariantCatalog, InvariantRegistration, InvariantRule, RelationalRuntimeApi,
};
use worth_testkit_relational::aspect_storage::{
    entity_field_aspect, entity_summary_struct_aspect, publications::string_aspect_field_patch,
    AspectSchemaFixture,
};

pub(super) fn summary_contract() -> worth_foundational::facade::AspectContract {
    entity_summary_struct_aspect(aspect_key("summary"), field_key("summary")).contract
}

fn summary_runtime(registration: InvariantRegistration) -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .schema_registry(
            AspectSchemaFixture {
                entity_aspects: vec![
                    entity_field_aspect(aspect_key("name"), field_key("name")),
                    entity_summary_struct_aspect(aspect_key("summary"), field_key("summary")),
                ],
                ..AspectSchemaFixture::default()
            }
            .build_registry(),
        )
        .invariant_catalog(InvariantCatalog {
            registrations: vec![registration],
        })
        .build()
        .expect("valid native runtime configuration")
}

pub(super) fn runtime_with_summary_title_uniqueness() -> RelationalRuntime {
    summary_runtime(InvariantRegistration::mutation_sensitive_blocking(
        InvariantRule::unique_entity_aspect_field(aspect_key("summary"), field_key("title")),
    ))
}

pub(super) fn runtime_with_summary_title_commit_boundary_uniqueness() -> RelationalRuntime {
    summary_runtime(InvariantRegistration::commit_boundary_blocking(
        InvariantRule::unique_entity_aspect_field(aspect_key("summary"), field_key("title")),
    ))
}

pub(super) fn admitted_snapshot(
    runtime: &RelationalRuntime,
) -> worth_relational::facade::snapshots::SnapshotHandle {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .expect("main basis is owner-admitted");
    runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .expect("owner-admitted observation opens its exact snapshot")
}

pub(super) fn entity_slot_count(runtime: &RelationalRuntime) -> usize {
    runtime
        .observe_partition_slot_census(PartitionId::main())
        .expect("the fixture's partition holds the published entity slots")
        .entity_slot_count
}

pub(super) fn commit_entity_with_summary(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    client_key: &str,
    title: &str,
    status: &str,
) -> Result<CommitResult, TransactionCommitError> {
    let mut txn = begin_main_transaction(runtime);
    txn.push_batch(
        WorkerIntentBatch::new(format!("entity-{client_key}")).push(MutationIntent::Create(
            CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: ClientKey::raw(client_key),
                fields: string_aspect_field_patch([
                    (aspect_key("name"), field_key("name"), client_key),
                    (aspect_key("summary"), field_key("title"), title),
                    (aspect_key("summary"), field_key("status"), status),
                ]),
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    txn.commit(execution, runtime)
}
