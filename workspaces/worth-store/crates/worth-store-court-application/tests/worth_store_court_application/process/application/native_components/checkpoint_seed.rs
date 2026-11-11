//! A real fieldless-schema Home with one acknowledged Native seed.
use worth_execution::{ExecutionAllocationPolicy, ExecutionRequest};
use worth_relational::facade::runtime::RelationalRuntimeApi;

pub(super) struct SeededCheckpointHome {
    pub(super) fixture:
        worth_store_testkit_application_oracle::facade::world_home::WorldHomeFixture,
    // Preserve the original court's admitted-basis and released-result lifetimes.
    _basis: worth_relational::facade::branch::AdmittedRelationalBranchBasis,
    _committed: worth_relational::facade::transactions::CommitResult,
}

pub(super) fn seeded(
    location: &std::path::Path,
    execution: &ExecutionRequest<'_, '_>,
) -> SeededCheckpointHome {
    use worth_relational::facade::{
        identity::{KindId, PartitionId},
        mvcc::RelationalTransactionIntent,
        schema::{
            EntityKindRegistration, KindAspectContractDeclarations, RelationalSchemaRegistry,
            SchemaId, SchemaVersionId,
        },
        symbols::ClientKey,
        transactions::{CreateIntent, EntitySpec, MutationIntent, WorkerIntentBatch},
    };
    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "checkpoint.seed".into(),
            schema_id: SchemaId("checkpoint".into()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(Vec::new()),
        })
        .unwrap();
    let fixture = super::coupled_home::open(
        location,
        RelationalRuntimeApi::builder().schema_setup(move |schema| {
            schema.schema_registry(registry);
        }),
        true,
        execution,
    );
    let home = &fixture.home;
    let (_, basis) = home
        .relational()
        .runtime()
        .observe_branch(&home.relational().runtime().main_branch_identity())
        .unwrap();
    let mut transaction = home
        .relational()
        .runtime()
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new("seed").push(MutationIntent::Create(CreateIntent::Entity(
                EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: ClientKey::raw("seed"),
                    fields: Default::default(),
                },
            ))),
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let committed = transaction
        .commit(execution, home.relational().runtime())
        .unwrap();
    home.relational()
        .runtime()
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();
    SeededCheckpointHome {
        fixture,
        _basis: basis,
        _committed: committed,
    }
}
