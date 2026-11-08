//! Actual unpublished candidate reservations survive as burned allocator frontiers.
use super::{begin_main_transaction, durable_runtime, field_patch, TestDurableMedium};
use worth_execution::{
    CancellationSource, ExecutionAllocationPolicy as Policy, ExecutionRequest, SerialMemoryBudget,
    SerialRequest,
};
use worth_relational::facade::{
    identity::{KindId, PartitionId},
    mvcc::PreparedRelationalCommitCandidate,
    runtime::RelationalRuntime,
    transactions::*,
};

fn prepare(
    runtime: &RelationalRuntime,
    request: &ExecutionRequest<'_, '_>,
    label: &str,
    relation_partition: u32,
) -> PreparedRelationalCommitCandidate {
    let left = CreatedEntityRef {
        partition_id: PartitionId(51),
        kind_id: KindId(1),
        client_key: format!("{label}-left").into(),
    };
    let right = CreatedEntityRef {
        client_key: format!("{label}-right").into(),
        ..left.clone()
    };
    let mut batch = WorkerIntentBatch::new(label);
    for entity in [&left, &right] {
        batch = batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: entity.partition_id,
            kind_id: entity.kind_id,
            client_key: entity.client_key.clone(),
            fields: field_patch("name", "name", label),
        })));
    }
    batch = batch.push(MutationIntent::Create(CreateIntent::Relation(
        RelationSpec {
            partition_id: PartitionId(relation_partition),
            kind_id: KindId(2),
            client_key: format!("{label}-edge").into(),
            source: EntityReference::Created(left),
            target: EntityReference::Created(right),
            fields: field_patch("label", "label", label),
        },
    )));
    let mut transaction = begin_main_transaction(runtime);
    transaction
        .push_batch(batch, request.allocation_policy())
        .unwrap();
    let proposal = transaction.validate(request, runtime).unwrap();
    runtime
        .prepare_validated_proposal(request, proposal)
        .unwrap()
}

#[test]
fn partition_high_water_includes_both_record_classes_pending_and_burned_namespaces() {
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(1024 * 1024)),
        CancellationSource::new().token(),
        None,
    );
    let request = ExecutionRequest::serial(&serial);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    assert_eq!(
        runtime
            .observe_record_partition_high_water(&request)
            .unwrap(),
        None
    );
    let observer = runtime
        .inspect_what_happened()
        .observe_record_reservations(6, request.allocation_policy())
        .unwrap();
    let first = prepare(&runtime, &request, "first", 3);
    assert_eq!(
        runtime
            .observe_record_partition_high_water(&request)
            .unwrap(),
        Some(PartitionId(51)),
        "lexicographic class order cannot hide the larger entity partition"
    );
    let sibling = prepare(&runtime, &request, "sibling", 72);
    assert_eq!(
        runtime
            .observe_record_partition_high_water(&request)
            .unwrap(),
        Some(PartitionId(72))
    );
    let report = observer.report(request.allocation_policy()).unwrap();
    let mut actual: Vec<_> = report
        .records()
        .map(|record| match record {
            RecordRef::Entity(entity) => (0, entity.partition_id.0),
            RecordRef::Relation(relation) => (1, relation.partition_id.0),
        })
        .collect();
    actual.sort_unstable();
    assert_eq!(
        actual,
        vec![(0, 51), (0, 51), (0, 51), (0, 51), (1, 3), (1, 72)]
    );
    drop((sibling, first));
    assert_eq!(
        runtime
            .observe_record_partition_high_water(&request)
            .unwrap(),
        Some(PartitionId(72)),
        "abandonment burns slots without erasing their namespace"
    );
    assert!(
        runtime
            .history()
            .branch_head_for_observation(
                &runtime
                    .observe_branch(&runtime.main_branch_identity())
                    .unwrap()
                    .1
                    .observation()
            )
            .unwrap()
            .is_none(),
        "no prepared record was published to make the frontier observable"
    );
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint(&request, Policy::SystemAllocation)
        .unwrap();
    let mut restored = durable_runtime(&medium);
    restored
        .durability_recovery()
        .restore_native_checkpoint(&request, &checkpoint)
        .unwrap();
    assert_eq!(
        restored
            .observe_record_partition_high_water(&request)
            .unwrap(),
        Some(PartitionId(72)),
        "the existing restored allocator frontier needs no record bodies"
    );
}
