use super::*;
use worth_relational::facade::{
    config::{CascadeDeletePolicy, CrossContextPolicy},
    identity::PartitionId,
    runtime::{RelationalRuntime, RelationalRuntimeApi},
    schema::{
        EntityKindRegistration, KindAspectContractDeclarations, RelationIntegrityDeclarations,
        RelationKindRegistration, RelationalSchemaRegistry, SchemaId, SchemaVersionId,
    },
    snapshots::SnapshotHandle,
    symbols::ClientKey,
    transactions::{
        AspectFieldPatch, CommitResult, CreateIntent, CreatedEntityRef, EntityReference,
        EntitySpec, MutationIntent, RelationSpec, WorkerIntentBatch,
    },
};

const NODE: KindId = KindId::new(1);
const LIVE: KindId = KindId::new(2);

/// A start commits only while the lineage's live membership is exactly what
/// it read and still leaves room for one more instance. Two starts prepared
/// against the same membership cannot both commit, and neither can a start
/// that read a full lineage.
#[test]
fn a_membership_that_gained_an_instance_or_is_full_is_not_current() {
    let mut runtime = capacity_runtime();
    let [lineage, first, second] = commit_entities(&mut runtime, ["lineage", "first", "second"]);
    let read = commit_live(&mut runtime, "first-live", first, lineage);
    let expected = incoming(&runtime, &read.snapshot, lineage);
    assert_eq!(expected.len(), 1);
    assert!(remains_equal(
        &runtime,
        &read.snapshot,
        LIVE,
        lineage,
        2,
        &expected
    ));
    assert!(
        !remains_equal(&runtime, &read.snapshot, LIVE, lineage, 1, &expected),
        "a membership that already fills the lineage admits no start",
    );

    let raced = commit_live(&mut runtime, "second-live", second, lineage);
    assert!(
        !remains_equal(&runtime, &raced.snapshot, LIVE, lineage, 2, &expected),
        "a start that read one member cannot commit once another joined",
    );
    assert!(
        !remains_equal(&runtime, &raced.snapshot, LIVE, lineage, 8, &expected),
        "the raced membership differs however much room is left",
    );
    for snapshot in [&read.snapshot, &raced.snapshot] {
        crate::relational_snapshot_release::release_query_snapshot(&mut runtime, snapshot);
    }
}

fn incoming(
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    lineage: EntityId,
) -> Vec<WorthQueryApplicationObservedRelation> {
    observe_adjacency(
        runtime,
        snapshot,
        LIVE,
        lineage,
        WorthQueryApplicationAdjacencyDirection::Incoming,
        8,
    )
    .expect("the fixture membership is readable")
}

fn capacity_runtime() -> RelationalRuntime {
    let schema = SchemaId("workflow-instance-capacity".to_owned());
    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: NODE,
            kind_name: "capacity.node".to_owned(),
            schema_id: schema.clone(),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
        })
        .and_then(|registry| {
            registry.register_relation_kind(RelationKindRegistration {
                kind_id: LIVE,
                kind_name: "capacity.live".to_owned(),
                schema_id: schema,
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
                relation_integrity: RelationIntegrityDeclarations::default(),
            })
        })
        .expect("the fixture schema is valid");
    RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .build()
}

fn commit_entities<const N: usize>(
    runtime: &mut RelationalRuntime,
    keys: [&str; N],
) -> [EntityId; N] {
    let created = keys.map(|key| CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: NODE,
        client_key: ClientKey::raw(key),
    });
    let intents = created.clone().map(|created| {
        MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: created.partition_id,
            kind_id: created.kind_id,
            client_key: created.client_key,
            fields: AspectFieldPatch::default(),
        }))
    });
    let committed = commit(runtime, "entities", intents);
    let issued = created.map(|created| {
        committed
            .created_entity(&created)
            .expect("the fixture commit issued each entity")
    });
    crate::relational_snapshot_release::release_query_snapshot(runtime, &committed.snapshot);
    issued
}

fn commit_live(
    runtime: &mut RelationalRuntime,
    key: &str,
    instance: EntityId,
    lineage: EntityId,
) -> CommitResult {
    commit(
        runtime,
        key,
        [MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: LIVE,
                client_key: ClientKey::raw(key),
                source: EntityReference::Existing(instance),
                target: EntityReference::Existing(lineage),
                fields: AspectFieldPatch::default(),
            },
        ))],
    )
}

fn commit<const N: usize>(
    runtime: &mut RelationalRuntime,
    name: &str,
    intents: [MutationIntent; N],
) -> CommitResult {
    let basis = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .expect("the main branch admits a basis");
    let mut transaction = runtime
        .begin_branch_transaction(
            &basis,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .expect("the fixture transaction begins");
    transaction
        .push_batch(
            intents
                .into_iter()
                .fold(WorkerIntentBatch::new(name), WorkerIntentBatch::push),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("the fixture batch stages");
    transaction
        .commit(
            runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("the fixture commits")
}
