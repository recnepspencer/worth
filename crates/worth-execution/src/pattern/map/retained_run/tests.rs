use super::*;
use crate::{
    CancellationToken, ExecutionMap, ExecutionRequest, ExecutionWorkCeiling, KeylessPartition,
    MapKernelContext, MapKernelFailure, MapOutcome, SerialMemoryBudget, SerialRequest,
};
use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicUsize, Ordering},
};
use worth_proof::CanonicalUniqueVec;
mod refusals;

fn serial(memory: &SerialMemoryBudget) -> SerialRequest {
    SerialRequest::from_memory(memory.clone(), CancellationToken::new(), None)
}
fn entries(
    ids: impl IntoIterator<Item = u64>,
) -> BTreeMap<PartitionIdentity, KeylessPartition<u64>> {
    ids.into_iter()
        .map(|id| {
            (
                PartitionIdentity::new(id),
                KeylessPartition {
                    value: id,
                    kernel_scratch_bytes: 0,
                    max_result_bytes: 0,
                },
            )
        })
        .collect()
}
fn empty_ids() -> CanonicalUniqueVec<PartitionIdentity> {
    CanonicalUniqueVec::try_from_sorted_unique(Vec::new()).unwrap()
}
fn kernel(
    input: &u64,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<u64, MapKernelFailure<()>> {
    for units in [2, 0, 3] {
        context.checkpoint(units)?;
    }
    Ok(input * 10)
}
fn complete(
    outcome: RetainedRunOutcome<u64, ()>,
) -> (ExecutionRetainedRun<u64>, RetainedRunMetrics) {
    let RetainedRunOutcome::Complete { run, metrics, .. } = outcome else {
        panic!("actual native complete")
    };
    (run, metrics)
}
fn seed(count: u64, request: &SerialRequest) -> ExecutionRetainedRun<u64> {
    let map = ExecutionMap::<_, ()>::from_keyless_partitions(entries(1..=count)).unwrap();
    complete(
        map.run_retained_run(
            ExecutionRequest::serial(request),
            ExecutionWorkCeiling::new(Some(100_000)),
            kernel,
        )
        .unwrap(),
    )
    .0
}

#[test]
fn one_changed_input_uses_native_ranges_and_same_cells_as_population_grows() {
    for count in [1, 10, 100] {
        let memory = SerialMemoryBudget::new(Some(10_000_000));
        let request = serial(&memory);
        let prior = seed(count, &request);
        let changed = count / 2 + 1;
        let mut changes = entries([changed]);
        changes
            .get_mut(&PartitionIdentity::new(changed))
            .unwrap()
            .value += 7;
        let called = AtomicUsize::new(0);
        let outcome = prior
            .advance(
                ExecutionRequest::serial(&request),
                ExecutionWorkCeiling::new(Some(100_000)),
                changes,
                empty_ids(),
                |value, context| {
                    called.fetch_add(1, Ordering::SeqCst);
                    kernel(value, context)
                },
            )
            .unwrap();
        let (next, metrics) = complete(outcome);
        assert_eq!(called.load(Ordering::SeqCst), 1);
        assert_eq!(
            (
                next.partition_count(),
                next.charged_work(),
                next.charged_span()
            ),
            (count as usize, count * 5, 5)
        );
        assert_eq!(metrics.supplied_inputs, 1);
        assert_eq!(metrics.carried_slots, count - 1);
        assert_eq!(metrics.carried_slot_visits, 0);
        assert!(
            metrics.range_visits < 40 && metrics.copied_nodes < 60,
            "{count}: {metrics:?}"
        );
        // Independent full native execution consumes its own ordinary roster.
        let mut full = entries(1..=count);
        full.get_mut(&PartitionIdentity::new(changed))
            .unwrap()
            .value += 7;
        let oracle = ExecutionMap::<_, ()>::from_keyless_partitions(full).unwrap();
        let MapOutcome::Complete { values, report } = ExecutionWorkCeiling::new(Some(100_000))
            .run_serial(&request, || oracle.run(None, kernel))
            .unwrap()
            .0
        else {
            panic!("full native oracle")
        };
        assert_eq!(
            (report.charged_work(), report.charged_span()),
            (next.charged_work(), next.charged_span())
        );
        for (index, value) in values.iter().enumerate() {
            let id = PartitionIdentity::new(index as u64 + 1);
            assert_eq!(next.result(id).unwrap().value(), value);
            if id != PartitionIdentity::new(changed) {
                assert!(std::ptr::eq(
                    prior.result(id).unwrap().value(),
                    next.result(id).unwrap().value()
                ));
            }
        }
        eprintln!("population={count} {metrics:?}");
    }
}

#[test]
fn every_tight_native_prefix_matches_full_execution_with_changed_slot() {
    let memory = SerialMemoryBudget::new(Some(10_000_000));
    let request = serial(&memory);
    let prior = seed(4, &request);
    let varied = |input: &u64, context: &mut MapKernelContext<'_, '_>| {
        for units in [2, 0, if *input == 30 { 7 } else { 3 }] {
            context.checkpoint(units)?;
        }
        Ok::<_, MapKernelFailure<()>>(*input * 10)
    };
    for ceiling in 0..=26 {
        let mut full_inputs = entries(1..=4);
        full_inputs
            .get_mut(&PartitionIdentity::new(3))
            .unwrap()
            .value = 30;
        let ordinary = ExecutionMap::<_, ()>::from_keyless_partitions(full_inputs).unwrap();
        let full = ExecutionWorkCeiling::new(Some(ceiling))
            .run_serial(&request, || ordinary.run(None, varied))
            .unwrap()
            .0;
        let mut changed = entries([3]);
        changed.get_mut(&PartitionIdentity::new(3)).unwrap().value = 30;
        let sparse = prior
            .advance(
                ExecutionRequest::serial(&request),
                ExecutionWorkCeiling::new(Some(ceiling)),
                changed,
                empty_ids(),
                varied,
            )
            .unwrap();
        assert_eq!(
            (full.report().charged_work(), full.report().charged_span()),
            (
                sparse.report().charged_work(),
                sparse.report().charged_span()
            ),
            "ceiling={ceiling}"
        );
        match (full, sparse) {
            (MapOutcome::Complete { values, .. }, RetainedRunOutcome::Complete { run, .. }) => {
                assert_eq!(values.len(), run.partition_count());
                assert_eq!((run.charged_work(), run.charged_span()), (24, 9));
                let recarried = run
                    .advance::<u64, (), _>(
                        ExecutionRequest::serial(&request),
                        ExecutionWorkCeiling::new(Some(ceiling)),
                        BTreeMap::new(),
                        empty_ids(),
                        varied,
                    )
                    .unwrap();
                assert_eq!(
                    (
                        recarried.report().charged_work(),
                        recarried.report().charged_span()
                    ),
                    (24, 9)
                );
                assert_eq!(
                    (
                        recarried.metrics().supplied_inputs,
                        recarried.metrics().carried_slot_visits
                    ),
                    (0, 0)
                );
                for (index, value) in values.iter().enumerate() {
                    assert_eq!(
                        run.result(PartitionIdentity::new(index as u64 + 1))
                            .unwrap()
                            .value(),
                        value
                    );
                }
            }
            (
                MapOutcome::Stopped {
                    completed_prefix,
                    boundary,
                    reason,
                    ..
                },
                RetainedRunOutcome::Stopped {
                    completed_prefix: sparse,
                    boundary: actual,
                    reason: cause,
                    ..
                },
            ) => {
                assert_eq!((boundary, reason), (actual, cause), "ceiling={ceiling}");
                assert_eq!(completed_prefix.len(), sparse.partition_count());
                for (index, value) in completed_prefix.iter().enumerate() {
                    assert_eq!(
                        sparse
                            .result(PartitionIdentity::new(index as u64 + 1))
                            .unwrap()
                            .value(),
                        value
                    );
                }
            }
            _ => panic!("different prefix at {ceiling}"),
        }
    }
}

#[test]
fn native_membership_move_delete_last_member_and_reinsert_have_no_history_chain() {
    let memory = SerialMemoryBudget::new(Some(10_000_000));
    let request = serial(&memory);
    let prior = seed(3, &request);
    let removed =
        CanonicalUniqueVec::try_from_sorted_unique(vec![PartitionIdentity::new(2)]).unwrap();
    let (moved, _) = complete(
        prior
            .advance(
                ExecutionRequest::serial(&request),
                ExecutionWorkCeiling::new(Some(100)),
                entries([7]),
                removed,
                kernel,
            )
            .unwrap(),
    );
    assert_eq!(moved.partition_count(), 3);
    assert!(moved.result(PartitionIdentity::new(2)).is_none());
    assert_eq!(
        *moved.result(PartitionIdentity::new(7)).unwrap().value(),
        70
    );
    let removed = CanonicalUniqueVec::try_from_sorted_unique(vec![
        PartitionIdentity::new(1),
        PartitionIdentity::new(3),
        PartitionIdentity::new(7),
    ])
    .unwrap();
    let (empty, _) = complete(
        moved
            .advance::<u64, (), _>(
                ExecutionRequest::serial(&request),
                ExecutionWorkCeiling::new(Some(100)),
                BTreeMap::new(),
                removed,
                kernel,
            )
            .unwrap(),
    );
    assert_eq!((empty.partition_count(), empty.charged_work()), (0, 0));
    drop(moved);
    drop(prior);
    let (restored, _) = complete(
        empty
            .advance(
                ExecutionRequest::serial(&request),
                ExecutionWorkCeiling::new(Some(100)),
                entries([2]),
                empty_ids(),
                kernel,
            )
            .unwrap(),
    );
    assert_eq!(
        *restored.result(PartitionIdentity::new(2)).unwrap().value(),
        20
    );
    drop(restored);
    drop(empty);
    drop(request);
    assert_eq!(memory.charged_memory_bytes(), 0);
}
