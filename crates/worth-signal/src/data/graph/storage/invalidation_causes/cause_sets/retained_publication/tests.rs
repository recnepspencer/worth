use super::*;
use crate::data::handle::NodeId;
use crate::data::proof::invalidation::binding::{
    DependencyRevision, OutputCommitOrdinal, ResolvedDependencyCause,
};
use crate::data::retained_storage::{RetainedStorageForkGrowth, RetainedStorageForkPreparation};
use crate::logic::evaluation::EvaluationWork;
use crate::runtime_policy::{SignalConditionalEvaluationBudget, SignalRuntimePolicy};

fn causes(node: u32) -> NormalizedCauseSet {
    NormalizedCauseSet::prepare(
        vec![ResolvedDependencyCause::new(
            1,
            NodeId::new(node, 0),
            DependencyRevision(1),
            NodeId::new(9, 0),
            crate::data::aspect::Aspect::new(1),
            None,
            0,
            OutputCommitOrdinal(1),
            1,
            Default::default(),
        )],
        &mut EvaluationWork::Ordinary,
    )
    .unwrap()
}

fn large_causes() -> NormalizedCauseSet {
    NormalizedCauseSet::prepare(
        vec![ResolvedDependencyCause::new(
            1,
            NodeId::new(5, 0),
            DependencyRevision(1),
            NodeId::new(9, 0),
            crate::data::aspect::Aspect::new(1),
            Some(crate::data::output::PartitionSubscription::whole_partition(
                "scope".repeat(2_048),
            )),
            0,
            OutputCommitOrdinal(1),
            1,
            Default::default(),
        )],
        &mut EvaluationWork::Ordinary,
    )
    .unwrap()
}

#[test]
fn retained_epoch_virtual_final_empty_matches_sequential_reuse() {
    let mut source = CanonicalCauseSetStore::default();
    let producer = source.insert_normalized(causes(0));
    let mut work = Work::new(1_000_000);
    source.prepare_fork_charge(&mut work).unwrap();
    source.prepare_fork_growth(&mut work).unwrap();
    let mut sequential = source.fork_persistent();
    sequential.release(producer).unwrap();
    let transient = sequential.insert_normalized(causes(1));
    sequential.release(transient).unwrap();
    let sibling = sequential.insert_normalized(causes(2));

    let mut cursor = source.prepare_cause_slots().unwrap();
    let mut ordinary = EvaluationWork::Ordinary;
    cursor.release(producer, &mut ordinary).unwrap();
    let first_empty = cursor
        .replacement(PendingCauseSetId::EMPTY, true, &mut ordinary)
        .unwrap();
    let live = cursor
        .epoch_replacement_after(first_empty.handle(), false, &mut ordinary)
        .unwrap();
    let final_empty = cursor
        .epoch_replacement_after(live.handle(), true, &mut ordinary)
        .unwrap();
    let sibling_slot = cursor
        .replacement(PendingCauseSetId::EMPTY, false, &mut ordinary)
        .unwrap();
    assert_eq!(sibling_slot.handle(), sibling);
    assert_eq!(final_empty.handle(), PendingCauseSetId::EMPTY);

    let ledger = SignalConditionalRetentionLedger::new(
        SignalConditionalEvaluationBudget {
            maximum_retained_slots: 4,
            maximum_retained_bytes: 16 * 1024 * 1024,
            maximum_attempt_visits: 1_000_000,
        },
        SignalRuntimePolicy::development().conditional_temporal_budget,
    );
    let mut draft = source
        .begin_retained_publication(&ledger, Charge::capacity::<u8>(8 * 1024 * 1024).unwrap())
        .unwrap();
    draft
        .release_prepared_producer(producer, &mut work)
        .unwrap();
    for slot in [first_empty, live, final_empty, sibling_slot] {
        draft.apply_epoch_virtual_slot(slot, &mut work).unwrap();
    }
    draft
        .finish_epoch_virtual_slot(sibling, causes(2), &mut work)
        .unwrap();
    let prepared = draft.finish().unwrap();
    assert_eq!(
        prepared
            .store
            .slot_generations
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        sequential
            .slot_generations
            .iter()
            .copied()
            .collect::<Vec<_>>()
    );
    assert_eq!(
        prepared
            .store
            .free_indices
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        sequential.free_indices.iter().copied().collect::<Vec<_>>()
    );
    assert_eq!(
        prepared.store.occupied_set_count,
        sequential.occupied_set_count
    );
    assert_eq!(prepared.store.output_commit_reference_count_for_test(1), 1);
    assert_eq!(
        prepared.store.get(sibling).unwrap()[0].key.consumer,
        NodeId::new(2, 0)
    );
}

#[test]
fn retained_epoch_virtual_payload_growth_denies_before_live_publication() {
    let mut source = CanonicalCauseSetStore::default();
    let mut work = Work::new(1_000_000);
    source.prepare_fork_charge(&mut work).unwrap();
    source.prepare_fork_growth(&mut work).unwrap();
    let ledger = SignalConditionalRetentionLedger::new(
        SignalConditionalEvaluationBudget {
            maximum_retained_slots: 4,
            maximum_retained_bytes: 16 * 1024 * 1024,
            maximum_attempt_visits: 1_000_000,
        },
        SignalRuntimePolicy::development().conditional_temporal_budget,
    );
    let slot = source
        .prepare_cause_slots()
        .unwrap()
        .replacement(
            PendingCauseSetId::EMPTY,
            false,
            &mut EvaluationWork::Ordinary,
        )
        .unwrap();
    let mut funded = source
        .begin_retained_publication(&ledger, Charge::capacity::<u8>(8 * 1024 * 1024).unwrap())
        .unwrap();
    funded.apply_epoch_virtual_slot(slot, &mut work).unwrap();
    // The retained owner admits the staging peak, which can exceed the final
    // metadata root while a persistent page is being replaced.
    let metadata_peak = funded
        .admitted_payload
        .checked_sub(Charge::capacity::<usize>(2).unwrap())
        .unwrap();
    drop(funded);
    let baseline_custody = ledger.usage();

    let mut constrained = source
        .begin_retained_publication(&ledger, metadata_peak)
        .unwrap();
    constrained
        .apply_epoch_virtual_slot(slot, &mut work)
        .unwrap();
    assert!(matches!(
        constrained.finish_epoch_virtual_slot(slot.handle(), large_causes(), &mut work),
        Err(SignalError::EvaluationStorageCapacityExhausted)
    ));
    drop(constrained);
    assert_eq!(source.sets.len(), 0);
    assert_eq!(source.free_indices.len(), 0);
    assert_eq!(source.output_commit_reference_count_for_test(1), 0);
    assert_eq!(ledger.usage(), baseline_custody);
}
