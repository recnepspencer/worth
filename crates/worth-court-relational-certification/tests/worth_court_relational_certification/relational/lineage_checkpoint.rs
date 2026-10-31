use worth_execution::ExecutionAllocationPolicy as Allocation;
use worth_relational::facade::{
    durability::RecoveryVerificationMode,
    identity::{EntityId, KindId, PartitionId},
    lineage::LineageEventKind,
    publication::RecordStructuralChange,
    runtime::RelationalRuntime,
    symbols::ClientKey,
    transactions::{
        CommitResult, CreateIntent, EntityMutationIntent, EntitySpec, MutationIntent, RecordRef,
        ReplaceEntityIntent, WorkerIntentBatch,
    },
};
use worth_testkit_relational::aspect_storage::{
    publications::{begin_main_transaction, changed_entities},
    AspectSchemaFixture,
};

#[test]
fn reverse_partition_create_drafts_restore_the_original_published_lineage() {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let runtime = AspectSchemaFixture::default().build_runtime();
    let source = create_source(execution, &runtime);
    let committed = commit_reverse_partition_creates(execution, &runtime, source);
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint(execution, Allocation::SystemAllocation)
        .expect("genuine owner capture before any lineage projection read");
    let mut recovered = AspectSchemaFixture::default().build_runtime();
    let restored = recovered
        .durability_recovery()
        .restore_native_checkpoint(execution, &checkpoint)
        .expect("canonical create lineage restores despite reverse draft partition order");
    assert_eq!(restored.latest_commit, Some(committed.commit.clone()));
    assert_original_lineage(execution, &runtime, &recovered, &committed);
}

#[test]
fn checkpoint_tail_replay_preserves_canonical_create_and_replace_lineage() {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let runtime = AspectSchemaFixture::default().build_runtime();
    let source = create_source(execution, &runtime);
    runtime
        .durability_authority()
        .checkpoint(execution)
        .unwrap();
    let committed = commit_reverse_partition_creates(execution, &runtime, source);
    let plan = runtime
        .durability()
        .recovery_plan(
            execution,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let mut recovered = AspectSchemaFixture::default().build_runtime();
    let restored = recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    assert_eq!(restored.coverage.checkpoint_commits, 1);
    assert_eq!(restored.coverage.replayed_tail_commits, 1);
    assert_eq!(restored.latest_commit, Some(committed.commit.clone()));
    assert_original_lineage(execution, &runtime, &recovered, &committed);
}

fn assert_original_lineage(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    recovered: &RelationalRuntime,
    committed: &CommitResult,
) {
    let original = runtime
        .replay()
        .canonical_commit_envelope(execution, committed.commit.commit_id)
        .unwrap()
        .expect("published original envelope");
    let restored = recovered
        .replay()
        .canonical_commit_envelope(execution, committed.commit.commit_id)
        .unwrap()
        .expect("restored original envelope");
    assert_eq!(original.lineage_events(), restored.lineage_events());
    assert_eq!(
        original.lineage_decision_log(),
        restored.lineage_decision_log()
    );
    assert_eq!(
        original.lineage_digest_basis(),
        restored.lineage_digest_basis()
    );
    let created = original
        .patch
        .authoritative_record_patches
        .iter()
        .filter_map(|patch| match (&patch.target, patch.structural_change) {
            (RecordRef::Entity(entity), RecordStructuralChange::Created) => Some(*entity),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(created.len(), 2);
    let [lower, higher] = [created[0], created[1]];
    assert_eq!(lower.partition_id, PartitionId::main());
    assert_eq!(higher.partition_id, PartitionId(2));
    assert!(
        higher > lower,
        "the create precedes the replacement in draft order"
    );
    assert!(matches!(
        original.merged_plan.merged_intents.as_slice(),
        [
            MutationIntent::Create(_),
            MutationIntent::Entity(EntityMutationIntent::Replace(_)),
        ]
    ));
    let read = recovered.read_truth();
    let lower_lineage = read
        .visible_entity_at_version(execution, lower, committed.version_id)
        .unwrap()
        .unwrap()
        .lineage_id
        .unwrap();
    let higher_lineage = read
        .visible_entity_at_version(execution, higher, committed.version_id)
        .unwrap()
        .unwrap()
        .lineage_id
        .unwrap();
    let events = original.lineage_events();
    assert_eq!(
        events.iter().map(|event| event.kind()).collect::<Vec<_>>(),
        vec![
            LineageEventKind::Create,
            LineageEventKind::Create,
            LineageEventKind::Replace
        ]
    );
    assert_eq!(events[0].targets(), &[lower_lineage]);
    assert_eq!(events[1].targets(), &[higher_lineage]);
    assert_eq!(events[2].targets(), &[lower_lineage]);
}

fn commit_reverse_partition_creates(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    source: EntityId,
) -> CommitResult {
    let mut transaction = begin_main_transaction(runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("mixed-create-replace")
                .push(MutationIntent::Create(CreateIntent::Entity(entity_spec(
                    PartitionId(2),
                    "higher",
                ))))
                .push(MutationIntent::Entity(EntityMutationIntent::Replace(
                    ReplaceEntityIntent {
                        entity_id: source,
                        replacement: entity_spec(PartitionId::main(), "lower"),
                    },
                ))),
            Allocation::SystemAllocation,
        )
        .unwrap();
    transaction.commit(execution, runtime).unwrap()
}

fn create_source(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) -> EntityId {
    let mut transaction = begin_main_transaction(runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("source").push(MutationIntent::Create(CreateIntent::Entity(
                entity_spec(PartitionId::main(), "source"),
            ))),
            Allocation::SystemAllocation,
        )
        .unwrap();
    changed_entities(&transaction.commit(execution, runtime).unwrap())[0]
}

fn entity_spec(partition_id: PartitionId, name: &str) -> EntitySpec {
    EntitySpec {
        partition_id,
        kind_id: KindId(1),
        client_key: ClientKey::raw(name),
        fields: Default::default(),
    }
}
