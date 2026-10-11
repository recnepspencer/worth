use super::*;
use crate::data::handle::NodeId;
use crate::data::proof::invalidation::binding::{
    DependencyRevision, OutputCommitOrdinal, ResolvedDependencyCause,
};
use crate::data::retained_storage::RetainedStoragePreparation as Work;
use crate::data::retained_storage::{RetainedStorageForkGrowth, RetainedStorageForkPreparation};

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
fn empty() -> NormalizedCauseSet {
    NormalizedCauseSet::prepare(Vec::new(), &mut EvaluationWork::Ordinary).unwrap()
}
fn prepare(
    store: &CanonicalCauseSetStore,
    handles: [PendingCauseSetId; 3],
    work: &mut Work,
) -> Result<Vec<PreparedCauseSlot>, SignalError> {
    let mut work = EvaluationWork::Conditional(work);
    let mut cursor = store.prepare_cause_slots()?;
    cursor.release(handles[0], &mut work)?;
    let mut slots = vec![
        cursor.replacement(handles[1], false, &mut work)?,
        cursor.replacement(handles[2], true, &mut work)?,
    ];
    for _ in 0..4 {
        slots.push(cursor.replacement(PendingCauseSetId::EMPTY, false, &mut work)?);
    }
    Ok(slots)
}

#[test]
fn prepared_slots_follow_producer_release_consumer_release_free_tail_and_append() {
    let mut source = CanonicalCauseSetStore::default();
    let producer = source.insert_normalized(causes(0));
    let keep = source.insert_normalized(causes(1));
    let release = source.insert_normalized(causes(2));
    let free = source.insert_normalized(causes(3));
    source.release(free).unwrap();
    let mut store = source.fork_persistent();
    let mut measured = Work::new(usize::MAX);
    let slots = prepare(&store, [producer, keep, release], &mut measured).unwrap();
    let reused = |id: PendingCauseSetId| PendingCauseSetId {
        index: id.index,
        generation: id.generation.wrapping_add(1),
    };
    let expected = [
        keep,
        PendingCauseSetId::EMPTY,
        reused(release),
        reused(producer),
        reused(free),
        PendingCauseSetId {
            index: NonZeroU32::new(5),
            generation: 0,
        },
    ];
    assert_eq!(
        slots
            .iter()
            .map(PreparedCauseSlot::handle)
            .collect::<Vec<_>>(),
        expected
    );
    for available in [0, measured.visits() - 1, measured.visits()] {
        let mut work = Work::new(measured.visits() + 7);
        work.reserve_visits(measured.visits() + 7 - available)
            .unwrap();
        let result = prepare(&store, [producer, keep, release], &mut work);
        assert_eq!(result.is_ok(), available == measured.visits());
        assert!(store.shares_storage_with(&source));
        assert_eq!(store.get(producer).unwrap().len(), 1);
    }
    store.release(producer).unwrap();
    for (n, slot) in slots.into_iter().enumerate() {
        let value = if n == 1 {
            empty()
        } else {
            causes(n as u32 + 10)
        };
        assert_eq!(
            store.publish_prepared_cause_slot(slot, value).unwrap(),
            expected[n]
        );
    }
    assert_eq!(store.sets.len(), 5);
    assert_eq!(store.occupied_set_count, 5);
    assert!(store.get(producer).is_err());
    assert_eq!(source.get(producer).unwrap().len(), 1);
    assert_eq!(source.sets.len(), 4);
}

#[test]
fn slot_preparation_rejects_duplicate_owners_and_stale_publication() {
    let mut store = CanonicalCauseSetStore::default();
    let current = store.insert_normalized(causes(0));
    let mut cursor = store.prepare_cause_slots().unwrap();
    cursor
        .release(current, &mut EvaluationWork::Ordinary)
        .unwrap();
    assert!(cursor
        .replacement(current, false, &mut EvaluationWork::Ordinary)
        .is_err());
    drop(cursor);
    store.release(current).unwrap();
    let slot = store
        .prepare_cause_slots()
        .unwrap()
        .replacement(
            PendingCauseSetId::EMPTY,
            false,
            &mut EvaluationWork::Ordinary,
        )
        .unwrap();
    let intervening = store.insert_normalized(causes(1));
    assert!(store.publish_prepared_cause_slot(slot, causes(2)).is_err());
    assert_eq!(
        store.get(intervening).unwrap()[0].key.consumer,
        NodeId::new(1, 0)
    );
    store.slot_generations.clear();
    assert!(store.prepare_cause_slots().is_err());
}

#[test]
fn ordinary_slot_preparation_keeps_single_owner_bookkeeping_inline() {
    const CHILD: &str = "WORTH_SIGNAL_CAUSE_SLOT_INLINE_ALLOCATION_CHILD";
    const TEST: &str = "data::graph::storage::invalidation_causes::cause_sets::slot_preparation::tests::ordinary_slot_preparation_keeps_single_owner_bookkeeping_inline";
    if std::env::var_os(CHILD).is_none() {
        let output = crate::process_deadline::output(
            std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", TEST, "--nocapture", "--test-threads=1"])
                .env(CHILD, "1"),
        )
        .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains(TEST) && stdout.contains("1 passed; 0 failed"));
        return;
    }
    let mut store = CanonicalCauseSetStore::default();
    let current = store.insert_normalized(causes(0));
    let other = store.insert_normalized(causes(1));
    let vacant = store.insert_normalized(causes(2));
    store.release(vacant).unwrap();
    // The same cursor serves ordinary replacement and output packets. Measure
    // preparation only: payload construction and store publication have their
    // own allocation lifecycle and are intentionally outside this assertion.
    for (owner, empty) in [
        (current, false),
        (current, true),
        (PendingCauseSetId::EMPTY, false),
        (PendingCauseSetId::EMPTY, true),
    ] {
        let region = stats_alloc::Region::new(&stats_alloc::INSTRUMENTED_SYSTEM);
        let mut cursor = store.prepare_cause_slots().unwrap();
        let slot = cursor
            .replacement(owner, empty, &mut EvaluationWork::Ordinary)
            .unwrap();
        std::hint::black_box(slot);
        drop(cursor);
        let measured = region.change();
        assert_eq!(measured.allocations, 0);
        assert_eq!(measured.reallocations, 0);
    }
    let region = stats_alloc::Region::new(&stats_alloc::INSTRUMENTED_SYSTEM);
    let mut cursor = store.prepare_cause_slots().unwrap();
    cursor
        .replacement(current, false, &mut EvaluationWork::Ordinary)
        .unwrap();
    cursor
        .replacement(other, false, &mut EvaluationWork::Ordinary)
        .unwrap();
    assert!(
        region.change().allocations > 0,
        "multiple owners exercise heap tracking"
    );
    // Both inline and overflow membership remain authoritative for duplicates.
    for duplicate in [current, other] {
        assert!(cursor
            .replacement(duplicate, false, &mut EvaluationWork::Ordinary)
            .is_err());
    }
    drop(cursor);
    assert_eq!(store.get(current).unwrap().len(), 1);
    assert_eq!(store.get(other).unwrap().len(), 1);
}

#[test]
fn epoch_virtual_transitions_match_sequential_slot_reuse_and_final_causes() {
    let mut source = CanonicalCauseSetStore::default();
    let producer = source.insert_normalized(causes(0));
    let mut fork_work = Work::new(usize::MAX);
    source.prepare_fork_charge(&mut fork_work).unwrap();
    source.prepare_fork_growth(&mut fork_work).unwrap();
    let mut sequential = source.fork_persistent();
    sequential.release(producer).unwrap();
    // The first empty admission owns no slot. A later admission takes the
    // producer's release, settles, then a sibling takes that same slot.
    let transient = sequential.insert_normalized(causes(1));
    sequential.release(transient).unwrap();
    let sibling = sequential.insert_normalized(causes(2));
    let consumer = sequential.insert_normalized(causes(3));

    let mut cursor = source.prepare_cause_slots().unwrap();
    let mut work = EvaluationWork::Ordinary;
    cursor.release(producer, &mut work).unwrap();
    let first_empty = cursor
        .replacement(PendingCauseSetId::EMPTY, true, &mut work)
        .unwrap();
    let first_live = cursor
        .epoch_replacement_after(first_empty.handle(), false, &mut work)
        .unwrap();
    let settled = cursor
        .epoch_replacement_after(first_live.handle(), true, &mut work)
        .unwrap();
    let sibling_slot = cursor
        .replacement(PendingCauseSetId::EMPTY, false, &mut work)
        .unwrap();
    let final_slot = cursor
        .epoch_replacement_after(settled.handle(), false, &mut work)
        .unwrap();
    assert_eq!(sibling_slot.handle(), sibling);
    assert_eq!(final_slot.handle(), consumer);

    let mut draft = source.fork_persistent();
    let mut retained = Work::new(usize::MAX);
    draft
        .release_epoch_accounted(producer, &mut retained)
        .unwrap();
    for slot in [first_empty, first_live, settled, sibling_slot, final_slot] {
        draft.apply_epoch_virtual_slot(slot, &mut retained).unwrap();
    }
    draft
        .finish_epoch_virtual_slot(sibling, causes(2), &mut retained)
        .unwrap();
    draft
        .finish_epoch_virtual_slot(consumer, causes(3), &mut retained)
        .unwrap();
    assert_eq!(
        draft.slot_generations.iter().copied().collect::<Vec<_>>(),
        sequential
            .slot_generations
            .iter()
            .copied()
            .collect::<Vec<_>>()
    );
    assert_eq!(
        draft.free_indices.iter().copied().collect::<Vec<_>>(),
        sequential.free_indices.iter().copied().collect::<Vec<_>>()
    );
    assert_eq!(draft.occupied_set_count, sequential.occupied_set_count);
    assert_eq!(draft.output_commit_reference_count_for_test(1), 2);
    assert_eq!(sequential.output_commit_reference_count_for_test(1), 2);
    for (handle, owner) in [(sibling, NodeId::new(2, 0)), (consumer, NodeId::new(3, 0))] {
        assert_eq!(draft.get(handle).unwrap()[0].key.consumer, owner);
        assert_eq!(sequential.get(handle).unwrap()[0].key.consumer, owner);
    }
}

#[test]
fn epoch_virtual_continuation_checks_work_before_touching_slot_state() {
    let source = CanonicalCauseSetStore::default();
    let mut measured_cursor = source.prepare_cause_slots().unwrap();
    let first = measured_cursor
        .replacement(
            PendingCauseSetId::EMPTY,
            false,
            &mut EvaluationWork::Ordinary,
        )
        .unwrap();
    let mut measured = Work::new(usize::MAX);
    let measured_slot = measured_cursor
        .epoch_replacement_after(
            first.handle(),
            true,
            &mut EvaluationWork::Conditional(&mut measured),
        )
        .unwrap();
    assert!(measured.visits() > 0);

    let mut denied_cursor = source.prepare_cause_slots().unwrap();
    let first = denied_cursor
        .replacement(
            PendingCauseSetId::EMPTY,
            false,
            &mut EvaluationWork::Ordinary,
        )
        .unwrap();
    let mut denied = Work::new(measured.visits() - 1);
    assert!(denied_cursor
        .epoch_replacement_after(
            first.handle(),
            true,
            &mut EvaluationWork::Conditional(&mut denied),
        )
        .is_err());
    assert!(denied_cursor.released.is_empty());
    assert!(denied_cursor
        .allocated
        .contains(first.handle().index.unwrap().get()));
    let mut funded = Work::new(measured.visits());
    let retried_slot = denied_cursor
        .epoch_replacement_after(
            first.handle(),
            true,
            &mut EvaluationWork::Conditional(&mut funded),
        )
        .unwrap();
    assert_eq!(retried_slot.handle(), measured_slot.handle());
    assert_eq!(denied_cursor.released, measured_cursor.released);
    assert_eq!(source.sets.len(), 0);
    assert_eq!(source.free_indices.len(), 0);
}
