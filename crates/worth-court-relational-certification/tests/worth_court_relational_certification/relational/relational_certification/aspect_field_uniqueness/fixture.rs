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

pub(super) fn summary_runtime_with_kinds(registration: Option<InvariantRegistration>, other_kind: bool) -> RelationalRuntime {
    let fixture = AspectSchemaFixture {
        entity_aspects: vec![
            entity_field_aspect(aspect_key("name"), field_key("name")),
            entity_summary_struct_aspect(aspect_key("summary"), field_key("summary")),
        ],
        ..AspectSchemaFixture::default()
    };
    let mut registry = fixture.build_registry();
    if other_kind {
        use worth_relational::facade::schema::{EntityKindRegistration, KindAspectContractDeclarations};
        registry = registry.register_entity_kind(EntityKindRegistration {
            kind_id: KindId(17),
            kind_name: "other-summary-kind".into(),
            schema_id: fixture.schema_id,
            schema_version_id: fixture.schema_version_id,
            aspect_contract_declarations: KindAspectContractDeclarations::new(fixture.entity_aspects),
        }).expect("the second kind shares the same declared aspect and field");
    }
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .invariant_catalog(InvariantCatalog {
            registrations: registration.iter().cloned().collect(),
        })
        .build()
        .expect("valid native runtime configuration");
    if registration.is_some() {
        register_summary_title_index(&runtime, KindId(1));
    }
    runtime
}

pub(super) fn runtime_with_summary_title_uniqueness() -> RelationalRuntime {
    summary_runtime_with_kinds(Some(InvariantRegistration::mutation_sensitive_blocking(
        InvariantRule::unique_entity_aspect_field(worth_relational::facade::identity::KindId(1), worth_relational::facade::indexes::DerivedIndexId(1), aspect_key("summary"), field_key("title")),
    )), false)
}

pub(super) fn runtime_with_summary_title_commit_boundary_uniqueness() -> RelationalRuntime {
    summary_runtime_with_kinds(Some(InvariantRegistration::commit_boundary_blocking(
        InvariantRule::unique_entity_aspect_field(worth_relational::facade::identity::KindId(1), worth_relational::facade::indexes::DerivedIndexId(1), aspect_key("summary"), field_key("title")),
    )), false)
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
    stage_summary(runtime, KindId(1), client_key, title, status).commit(execution, runtime)
}

pub(super) fn register_summary_title_index(runtime: &RelationalRuntime, kind: KindId) -> worth_relational::facade::indexes::DerivedIndexId {
    register_summary_index(runtime, kind, "title")
}

pub(super) fn register_summary_index(runtime: &RelationalRuntime, kind: KindId, field: &str) -> worth_relational::facade::indexes::DerivedIndexId {
    use worth_relational::facade::indexes::{DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind};
    runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: format!("{}.summary.{field}", kind.0),
        kind: DerivedIndexKind::EntityField {
            field_locator: worth_testkit_relational::aspect_storage::aspect_field_locator(aspect_key("summary"), field_key(field)),
        },
        branch_scoped: false,
        unique_entity_kind: Some(kind),
    }).unwrap().index_id
}

pub(super) fn stage_summary(
    runtime: &RelationalRuntime,
    kind: KindId,
    client_key: &str,
    title: &str,
    status: &str,
) -> worth_relational::facade::mvcc::BranchBoundRelationalTransaction {
    let mut txn = begin_main_transaction(runtime);
    txn.push_batch(
        WorkerIntentBatch::new(format!("entity-{client_key}"))
            .push(create_summary(kind, client_key, title, status)),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    txn
}

pub(super) fn create_summary(kind: KindId, client_key: &str, title: &str, status: &str) -> MutationIntent {
    MutationIntent::Create(CreateIntent::Entity(EntitySpec {
        partition_id: PartitionId::main(),
        kind_id: kind,
        client_key: ClientKey::raw(client_key),
        fields: string_aspect_field_patch([
            (aspect_key("name"), field_key("name"), client_key),
            (aspect_key("summary"), field_key("title"), title),
            (aspect_key("summary"), field_key("status"), status),
        ]),
    }))
}
