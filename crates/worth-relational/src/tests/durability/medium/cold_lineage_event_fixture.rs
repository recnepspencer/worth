//! Actual replacement publications and unrelated ranges in an external home image.
use super::*;
use crate::history::data::BranchId;
use crate::lineage::data::{
    HistoricalLineageResolution, HistoricalResolutionBoundednessBasis, HistoricalResolutionRequest,
};
use crate::transactions::data::{
    BulkEntityCreateIntent, EntityMutationIntent, EntitySpec, ReplaceEntityIntent,
};

pub(super) struct EventFixture {
    pub(super) runtime: crate::runtime::RelationalRuntime,
    pub(super) source: Arc<MemberSource>,
    pub(super) seed: crate::identity::data::LineageId,
    pub(super) original: crate::identity::data::EntityId,
    pub(super) expected: HistoricalLineageResolution,
}

pub(super) fn request(seed: crate::identity::data::LineageId) -> HistoricalResolutionRequest {
    HistoricalResolutionRequest {
        branch_id: BranchId("main".into()),
        lineage_id: seed,
        boundedness_basis: HistoricalResolutionBoundednessBasis::BranchScopedLineageSeed,
    }
}

pub(super) fn reopened(execution: &worth_execution::ExecutionRequest<'_, '_>) -> EventFixture {
    reopened_with_source(execution, |source| source)
}

pub(super) fn reopened_with_source(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    wrap: impl FnOnce(
        Arc<MemberSource>,
    ) -> Arc<dyn crate::durability::medium::RelationalDurableColdSource>,
) -> EventFixture {
    let producer = durable_runtime_with_test_schema();
    let mut transaction = test_owner_begin_transaction_for_main(&producer);
    transaction
        .push_batch(
            WorkerIntentBatch::new("event-page-population").push(MutationIntent::Create(
                CreateIntent::BulkEntities(BulkEntityCreateIntent {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_keys: (0..601)
                        .map(|id| {
                            crate::symbols::data::ClientKey::raw(format!("event-node-{id:04}"))
                        })
                        .collect(),
                    field_patches: (0..601)
                        .map(|_| name_field_patch("event-payload"))
                        .collect(),
                }),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
    let committed = transaction.commit(execution, &producer).unwrap();
    let original = *changed_entities(&committed).last().unwrap();
    release_test_commit_snapshot(&producer, &committed);
    let seed = producer
        .lineage_access()
        .for_record(execution, original)
        .unwrap()
        .unwrap()
        .lineage_id;
    let mut current = original;
    for step in 0..2 {
        let mut transaction = test_owner_begin_transaction_for_main(&producer);
        transaction
            .push_batch(
                WorkerIntentBatch::new("event-page-replace").push(MutationIntent::Entity(
                    EntityMutationIntent::Replace(ReplaceEntityIntent {
                        entity_id: current,
                        replacement: EntitySpec {
                            partition_id: PartitionId::main(),
                            kind_id: KindId(1),
                            client_key: crate::symbols::data::ClientKey::raw(format!(
                                "event-replacement-{step}"
                            )),
                            fields: name_field_patch(&format!("replacement-{step}")),
                        },
                    }),
                )),
                execution.allocation_policy(),
            )
            .unwrap();
        let committed = transaction.commit(execution, &producer).unwrap();
        current = changed_entities(&committed)
            .into_iter()
            .find(|id| *id != current)
            .unwrap();
        release_test_commit_snapshot(&producer, &committed);
    }
    let expected = producer
        .lineage_access()
        .resolve_historical_lineage(execution, request(seed))
        .unwrap();
    assert_ne!(
        expected.resolved,
        vec![seed],
        "the producer really traversed replacement events"
    );
    assert!(expected.traversed_event_ids.len() >= 2);
    let (recovery, source, _) = checkpoint_fixture(execution, producer, current, false);
    source.damaged.store(false, Ordering::Relaxed);
    let recovery = RelationalDurableRecoverySource::new(
        recovery.checkpoint().cloned(),
        recovery.tail().to_vec(),
        recovery.transitions().to_vec(),
        wrap(source.clone()),
    );
    let mut runtime = durable_runtime_with_test_schema();
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            recovery,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    runtime
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    assert_eq!(
        source.reads.load(Ordering::Relaxed),
        0,
        "reopen reads no lineage or symbol members"
    );
    assert_eq!(runtime.lineage.event_count(), 0);
    assert_eq!(runtime.lineage.node_count(), 0);
    EventFixture {
        runtime,
        source,
        seed,
        original,
        expected,
    }
}
