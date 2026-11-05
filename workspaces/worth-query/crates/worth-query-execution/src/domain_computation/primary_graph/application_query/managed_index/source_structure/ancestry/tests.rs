//! Numeric relation only; actual SDK control, no fabricated Source or cached fact.
use super::*;
use std::cell::Cell;
use worth_execution::{
    CancellationSource, ExecutionWorkCeiling, MapKernelStop, SerialMemoryBudget, SerialRequest,
    WorkCeilingDenial,
};
use {
    crate::domain_computation::primary_graph::application_query::managed_index::invocation::controlled,
    crate::domain_computation::vocabulary::primary_graph::invocation::WorthQueryManagedIndexWorkDenial as Denial,
};
#[derive(Clone, Copy)]
enum CancelAt {
    Never,
    Parent8,
    Cached,
}
fn chain(
    count: usize,
    ceiling: u64,
    cancel_at: CancelAt,
) -> (Result<(), Denial>, usize, bool, u64) {
    let cancel = CancellationSource::new();
    let storage =
        SerialRequest::from_memory(SerialMemoryBudget::new(Some(4096)), cancel.token(), None);
    let execution = ExecutionRequest::serial(&storage);
    let reads = Cell::new(0);
    let warmed = Cell::new(false);
    let (result, report) = ExecutionWorkCeiling::new(Some(ceiling))
        .run_with_checkpoint(None, Some(&storage), |_| {
            controlled(&execution, |control| {
                let mut parent = |node: usize| {
                    assert_eq!(node % 1024, 0);
                    assert!(node / 1024 < count);
                    reads.set(reads.get() + 1);
                    if matches!(cancel_at, CancelAt::Parent8) && reads.get() == 8 {
                        cancel.cancel();
                    }
                    Ok(node.checked_sub(1024))
                };
                let (middle, last) = ((count / 2) * 1024, (count - 1) * 1024);
                let mut root = Ancestry::new(0);
                assert!(root.contains(middle, Some(0), count, control, &mut parent)?);
                assert!(root.contains(last, Some(0), count, control, &mut parent)?);
                assert_eq!(
                    reads.get(),
                    count - 1,
                    "a known descendant can finish a new paid path by transitivity"
                );
                let mut other = Ancestry::new(middle);
                assert!(other.contains(last, Some(0), count, control, &mut parent)?);
                assert_eq!(
                    reads.get(),
                    count + count / 2 - 2,
                    "another receiving region proves its own relation"
                );
                warmed.set(true);
                if matches!(cancel_at, CancelAt::Cached) {
                    cancel.cancel();
                }
                for _ in 0..128 {
                    assert!(root.contains(last, Some(0), count, control, &mut parent)?);
                    assert!(other.contains(last, Some(0), count, control, &mut parent)?);
                }
                assert_eq!(
                    reads.get(),
                    count + count / 2 - 2,
                    "repeated positive facts in each receiver add no parent reads"
                );
                Ok(())
            })
        })
        .expect("original root admission precedes the bounded numeric workload");
    assert_eq!(
        storage.memory().unwrap().charged_memory_bytes(),
        0,
        "bounded scratch retains no SDK heap grant"
    );
    (result, reads.get(), warmed.get(), report.charged_work())
}
#[test]
fn summary_membership_cache_reuses_region_facts_with_bounded_original_scratch() {
    let mut work = [0; 2];
    for (position, count) in [64, 128].into_iter().enumerate() {
        let (result, reads, warmed, invoice) = chain(count, 64_000_000, CancelAt::Never);
        result.unwrap();
        assert!(warmed);
        assert_eq!(reads, count + count / 2 - 2);
        work[position] = invoice;
    }
    eprintln!("bounded membership4KiB accepted whole numeric SDK work64/128={work:?}");
    assert!(work[0] > 0 && work[1] < 3 * work[0]);
}
#[test]
fn summary_membership_cache_separates_regions_and_keeps_saturated_fallback_lawful() {
    let storage = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(4096)),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&storage);
    controlled(&execution, |control| {
        let parents = [
            None,
            Some(0),
            Some(0),
            Some(1),
            Some(1),
            Some(2),
            Some(2),
            Some(3),
            Some(3),
        ];
        let reads = Cell::new(0);
        let mut parent = |node: usize| {
            reads.set(reads.get() + 1);
            Ok(parents[node])
        };
        let mut left = Ancestry::new(1);
        let mut right = Ancestry::new(2);
        assert!(left.contains(3, Some(0), parents.len(), control, &mut parent)?);
        assert!(
            !right.contains(3, Some(0), parents.len(), control, &mut parent)?,
            "another receiver cannot inherit the warmed fact"
        );
        assert!(right.contains(5, Some(0), parents.len(), control, &mut parent)?);
        assert!(!left.contains(5, Some(0), parents.len(), control, &mut parent)?);
        let before = reads.get();
        assert!(left.contains(1, Some(0), parents.len(), control, &mut parent)?);
        assert_eq!(reads.get(), before);
        let mut root = Ancestry::new(0);
        for source in [4, 5, 6, 7, 8] {
            assert!(root.contains(source, Some(0), parents.len(), control, &mut parent)?);
        }
        let before_fallback = reads.get();
        assert!(root.contains(4, Some(0), parents.len(), control, &mut parent)?);
        assert!(
            reads.get() > before_fallback,
            "saturation evicts an acceleration fact and uses the actual paid fallback"
        );
        let after_fallback = reads.get();
        assert!(root.contains(4, Some(0), parents.len(), control, &mut parent)?);
        assert_eq!(
            reads.get(),
            after_fallback,
            "the newly proved same-region fact is reused"
        );
        Ok(())
    })
    .unwrap();
    assert_eq!(storage.memory().unwrap().charged_memory_bytes(), 0);
    for (parents, expected) in [
        (vec![None, None], Stop::SourceStructureIncomplete),
        (vec![None, Some(2), Some(1)], Stop::SourceStructureCycle),
    ] {
        let result = controlled(&execution, |control| {
            Ancestry::new(0)
                .contains(parents.len() - 1, Some(0), parents.len(), control, |node| {
                    Ok(parents[node])
                })
                .map(|_| ())
                .map_err(Into::into)
        });
        assert!(
            matches!(result,Err(Denial::Controlled(ref cause)) if *cause==expected),
            "{result:?}"
        );
    }
}
#[test]
fn summary_membership_cache_hits_honor_original_stops_and_same_graph_retries() {
    let (cancelled, reads, warmed, _) = chain(128, 64_000_000, CancelAt::Parent8);
    assert_eq!(reads, 8);
    assert!(!warmed);
    assert!(matches!(
        cancelled,
        Err(Denial::Controlled(Stop::Request(
            WorkCeilingDenial::Stopped(MapKernelStop::Cancelled)
        )))
    ));
    let (cancelled, reads, warmed, _) = chain(128, 64_000_000, CancelAt::Cached);
    assert!(warmed);
    assert_eq!(reads, 190);
    assert!(matches!(
        cancelled,
        Err(Denial::Controlled(Stop::Request(
            WorkCeilingDenial::Stopped(MapKernelStop::Cancelled)
        )))
    ));
    let (limited, reads, warmed, _) = chain(128, 20_000, CancelAt::Never);
    assert!(warmed);
    assert_eq!(
        reads, 190,
        "work refusal occurs after the facts populated and adds no hidden parent read"
    );
    assert!(
        matches!(
            limited,
            Err(Denial::Controlled(Stop::Request(
                WorkCeilingDenial::Stopped(MapKernelStop::WorkCeiling)
            )))
        ),
        "{limited:?}"
    );
    chain(128, 64_000_000, CancelAt::Never).0.unwrap();
}
