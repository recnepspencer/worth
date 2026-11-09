//! Genuine bulk births, replacement events and independent cold receiver fixtures.
use super::selected_directory::{self, Directory};
use std::sync::{atomic::Ordering, Arc};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    durability::*, identity::*, lineage::*, runtime::RelationalRuntime, symbols::ClientKey,
    transactions::*,
};
use worth_testkit_relational::{
    aspect_storage::publications::{
        begin_main_transaction, changed_entities, release_test_commit_snapshot,
    },
    cold_recovery::{capture_source, ColdRecoverySourceFixture},
    durable_medium::{immutable_members::ImmutableCheckpointMemberSource, TestDurableMedium},
    durable_publication::{durable_runtime, field_patch},
};

pub(super) struct EventFixture {
    pub runtime: RelationalRuntime,
    pub source: Arc<ImmutableCheckpointMemberSource>,
    pub medium: Arc<TestDurableMedium>,
    pub seed: LineageId,
    pub original: EntityId,
    pub expected: HistoricalLineageResolution,
    pub directory: Directory,
}
pub(super) fn reopened(request: &ExecutionRequest<'_, '_>) -> EventFixture {
    reopened_with_source(request, |source| source)
}
pub(super) fn reopened_with_source(
    request: &ExecutionRequest<'_, '_>,
    wrap: impl FnOnce(Arc<ImmutableCheckpointMemberSource>) -> Arc<dyn RelationalDurableColdSource>,
) -> EventFixture {
    let medium = TestDurableMedium::new();
    let producer = durable_runtime(&medium);
    let mut transaction = begin_main_transaction(&producer);
    transaction
        .push_batch(
            WorkerIntentBatch::new("event-page-population").push(MutationIntent::Create(
                CreateIntent::BulkEntities(BulkEntityCreateIntent {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_keys: (0..601)
                        .map(|id| ClientKey::raw(format!("event-node-{id:04}")))
                        .collect(),
                    field_patches: (0..601)
                        .map(|_| field_patch("name", "name", "event-payload"))
                        .collect(),
                }),
            )),
            request.allocation_policy(),
        )
        .unwrap();
    let committed = transaction.commit(request, &producer).unwrap();
    let original = *changed_entities(&committed).last().unwrap();
    release_test_commit_snapshot(&producer, &committed);
    let seed = {
        let (_, basis) = producer
            .observe_branch(&producer.main_branch_identity())
            .unwrap();
        producer
            .lineage_node_at_observation(request, &basis.observation(), original)
            .unwrap()
            .unwrap()
            .node()
            .lineage_id()
    };
    let mut current = original;
    for step in 0..2 {
        let mut transaction = begin_main_transaction(&producer);
        transaction
            .push_batch(
                WorkerIntentBatch::new("event-page-replace").push(MutationIntent::Entity(
                    EntityMutationIntent::Replace(ReplaceEntityIntent {
                        entity_id: current,
                        replacement: EntitySpec {
                            partition_id: PartitionId::main(),
                            kind_id: KindId(1),
                            client_key: ClientKey::raw(format!("event-replacement-{step}")),
                            fields: field_patch("name", "name", &format!("replacement-{step}")),
                        },
                    }),
                )),
                request.allocation_policy(),
            )
            .unwrap();
        let committed = transaction.commit(request, &producer).unwrap();
        current = changed_entities(&committed)
            .into_iter()
            .find(|id| *id != current)
            .unwrap();
        release_test_commit_snapshot(&producer, &committed);
    }
    let expected = producer
        .discover_authored_lineage_history(request, &producer.main_branch_identity(), seed)
        .unwrap();
    assert_ne!(
        expected.resolved,
        vec![seed],
        "the producer really traversed replacement events"
    );
    assert!(expected.traversed_event_ids.len() >= 2);
    let ColdRecoverySourceFixture { input, source, .. } =
        capture_source(request, producer, medium, current, false);
    source.damaged.store(false, Ordering::Relaxed);
    let directory = selected_directory::read(&input);
    let input = RelationalDurableRecoverySource::new(
        input.checkpoint().cloned(),
        input.tail().to_vec(),
        input.transitions().to_vec(),
        wrap(source.clone()),
    );
    let medium = TestDurableMedium::new();
    let mut runtime = durable_runtime(&medium);
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            request,
            input,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    runtime
        .durability_recovery()
        .recover(request, plan)
        .unwrap();
    assert_eq!(
        source.reads.load(Ordering::Relaxed),
        0,
        "reopen reads no lineage or symbol members"
    );
    EventFixture {
        runtime,
        source,
        medium,
        seed,
        original,
        expected,
        directory,
    }
}
