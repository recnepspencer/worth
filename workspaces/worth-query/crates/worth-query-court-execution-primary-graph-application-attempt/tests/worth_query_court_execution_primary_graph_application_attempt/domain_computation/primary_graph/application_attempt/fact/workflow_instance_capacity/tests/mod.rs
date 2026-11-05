use worth_relational::facade::identity::EntityId;
use worth_relational::facade::identity::KindId;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationAdjacencyDirection;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedRelation;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::adjacency::observe_adjacency;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::workflow_instance_capacity::remains_equal;
use worth_relational::facade::config::CascadeDeletePolicy;
use worth_relational::facade::config::CrossContextPolicy;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::runtime::RelationalRuntimeApi;
use worth_relational::facade::schema::EntityKindRegistration;
use worth_relational::facade::schema::KindAspectContractDeclarations;
use worth_relational::facade::schema::RelationIntegrityDeclarations;
use worth_relational::facade::schema::RelationKindRegistration;
use worth_relational::facade::schema::RelationalSchemaRegistry;
use worth_relational::facade::schema::SchemaId;
use worth_relational::facade::schema::SchemaVersionId;
use worth_relational::facade::snapshots::SnapshotHandle;
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::AspectFieldPatch;
use worth_relational::facade::transactions::CommitResult;
use worth_relational::facade::transactions::CreateIntent;
use worth_relational::facade::transactions::CreatedEntityRef;
use worth_relational::facade::transactions::EntityReference;
use worth_relational::facade::transactions::EntitySpec;
use worth_relational::facade::transactions::MutationIntent;
use worth_relational::facade::transactions::RelationSpec;
use worth_relational::facade::transactions::WorkerIntentBatch;

pub const NODE: KindId = KindId::new(1);
pub const LIVE: KindId = KindId::new(2);

/// A start commits only while the lineage's live membership is exactly what
/// it read and still leaves room for one more instance. Two starts prepared
/// against the same membership cannot both commit, and neither can a start
/// that read a full lineage.
#[test]
fn a_membership_that_gained_an_instance_or_is_full_is_not_current() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = capacity_runtime();
    let [lineage, first, second] =
        commit_entities(execution, &runtime, ["lineage", "first", "second"]);
    let read = commit_live(execution, &runtime, "first-live", first, lineage);
    let expected = incoming(execution, &runtime, &read.snapshot, lineage);
    assert_eq!(expected.len(), 1);
    assert!(remains_equal(
        &runtime,
        execution,
        &read.snapshot,
        LIVE,
        lineage,
        2,
        &expected
    )
    .unwrap());
    assert!(
        !remains_equal(
            &runtime,
            execution,
            &read.snapshot,
            LIVE,
            lineage,
            1,
            &expected
        )
        .unwrap(),
        "a membership that already fills the lineage admits no start",
    );

    let raced = commit_live(execution, &runtime, "second-live", second, lineage);
    assert!(
        !remains_equal(
            &runtime,
            execution,
            &raced.snapshot,
            LIVE,
            lineage,
            2,
            &expected
        )
        .unwrap(),
        "a start that read one member cannot commit once another joined",
    );
    assert!(
        !remains_equal(
            &runtime,
            execution,
            &raced.snapshot,
            LIVE,
            lineage,
            8,
            &expected
        )
        .unwrap(),
        "the raced membership differs however much room is left",
    );
    for snapshot in [&read.snapshot, &raced.snapshot] {
        worth_query_execution::court_doors_for_test::relational_snapshot_release::release_query_snapshot(
            &runtime,
            snapshot,
        );
    }
}

pub fn incoming(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    lineage: EntityId,
) -> Vec<WorthQueryApplicationObservedRelation> {
    observe_adjacency(
        runtime,
        execution,
        snapshot,
        LIVE,
        lineage,
        WorthQueryApplicationAdjacencyDirection::Incoming,
        8,
    )
    .unwrap()
    .expect("the fixture membership is readable")
}

pub fn capacity_runtime() -> RelationalRuntime {
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
        .expect("valid native runtime configuration")
}

pub fn commit_entities<const N: usize>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
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
    let committed = commit(execution, runtime, "entities", intents);
    let issued = created.map(|created| {
        committed
            .created_entity(&created)
            .expect("the fixture commit issued each entity")
    });
    worth_query_execution::court_doors_for_test::relational_snapshot_release::release_query_snapshot(
        runtime,
        &committed.snapshot,
    );
    issued
}

pub fn commit_live(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    key: &str,
    instance: EntityId,
    lineage: EntityId,
) -> CommitResult {
    commit(
        execution,
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

pub fn commit<const N: usize>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
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
        .commit(execution, runtime)
        .expect("the fixture commits")
}
