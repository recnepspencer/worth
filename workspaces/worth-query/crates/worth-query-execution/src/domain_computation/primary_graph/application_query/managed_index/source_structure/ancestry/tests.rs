//! Numeric relation only: actual SDK grants, no constructed Source or qualified root.
use super::*;
use crate::domain_computation::primary_graph::application_query::managed_index::{
    invocation::controlled, WorthQueryManagedIndexWorkDenial as Denial,
};
use std::cell::Cell;
use worth_execution::{
    CancellationSource, ExecutionWorkCeiling, MapKernelStop, SerialMemoryBudget, SerialRequest,
    WorkCeilingDenial,
};

fn chain(count: usize, memory: u64, cancel_after_ready: bool) -> (Result<(), Denial>, usize, u64) {
    let cancel = CancellationSource::new();
    let storage =
        SerialRequest::from_memory(SerialMemoryBudget::new(Some(memory)), cancel.token(), None);
    let execution = ExecutionRequest::serial(&storage);
    let reads = Cell::new(0);
    let (result, report) = ExecutionWorkCeiling::new(Some(64_000_000))
        .run_with_checkpoint(None, Some(&storage), |_| {
            controlled(&execution, |control| {
                let mut memo = Ancestry::default();
                let mut parent = |node: usize| {
                    assert_eq!(node % 1024, 0);
                    assert!(node / 1024 < count);
                    reads.set(reads.get() + 1);
                    Ok(node.checked_sub(1024))
                };
                assert!(memo.contains(
                    0,
                    (count - 1) * 1024,
                    Some(0),
                    count,
                    &execution,
                    control,
                    &mut parent
                )?);
                assert_eq!(
                    reads.get(),
                    count,
                    "first path visits every actual numeric parent once"
                );
                if cancel_after_ready {
                    cancel.cancel();
                }
                for _ in 0..128 {
                    assert!(memo.contains(
                        (count / 2) * 1024,
                        (count - 1) * 1024,
                        Some(0),
                        count,
                        &execution,
                        control,
                        &mut parent
                    )?);
                }
                assert_eq!(
                    reads.get(),
                    count,
                    "repeated membership reuses the path without more parent callbacks"
                );
                Ok(())
            })
        })
        .expect("predeclared memory admits the original root before ancestry construction");
    assert_eq!(
        storage.memory().unwrap().charged_memory_bytes(),
        0,
        "all pending/ready maps, actual jump arrays and Arc grants release"
    );
    (result, reads.get(), report.charged_work())
}

#[test]
fn summary_ancestry_reuses_sparse_parent_paths_with_original_paid_jumps() {
    let mut work = [0; 2];
    for (position, count) in [64, 128].into_iter().enumerate() {
        let (result, reads, invoice) = chain(count, 16 * 1024 * 1024, false);
        result.unwrap();
        assert_eq!(reads, count);
        work[position] = invoice;
    }
    eprintln!("ancestry chain plus128 repeated lookups accepted SDK work64/128={work:?}");
    assert!(
        work[0] > 0 && work[1] < 3 * work[0],
        "separate original SDK bookkeeping growth, not parent-read counts"
    );
}

#[test]
fn summary_ancestry_rejects_ready_siblings_unvisited_regions_and_false_roots() {
    let storage = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(16 * 1024 * 1024)),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&storage);
    controlled(&execution, |control| {
        let parents = [None, Some(0), Some(0), Some(1), Some(2)];
        let reads = Cell::new(0);
        let mut parent = |node: usize| {
            reads.set(reads.get() + 1);
            Ok(parents[node / 1024].map(|p| p * 1024))
        };
        let mut memo = Ancestry::default();
        for (region, source, expected) in [
            (0, 3, true),
            (2, 3, false),
            (0, 4, true),
            (3, 4, false),
            (1, 4, false),
            (2, 4, true),
            (4, 4, true),
            (4, 2, false),
        ] {
            assert_eq!(
                memo.contains(
                    region * 1024,
                    source * 1024,
                    Some(0),
                    parents.len(),
                    &execution,
                    control,
                    &mut parent
                )?,
                expected
            );
            if source == 3 {
                assert_eq!(
                    reads.get(),
                    3,
                    "an off-path region does not initialize an unrelated path"
                );
            }
        }
        assert_eq!(reads.get(), parents.len());
        Ok(())
    })
    .unwrap();
    assert_eq!(storage.memory().unwrap().charged_memory_bytes(), 0);
    for (parents, expected) in [
        (vec![None, None], Stop::SourceStructureIncomplete),
        (vec![None, Some(2), Some(1)], Stop::SourceStructureCycle),
    ] {
        let storage = SerialRequest::from_memory(
            SerialMemoryBudget::new(Some(16 * 1024 * 1024)),
            CancellationSource::new().token(),
            None,
        );
        let execution = ExecutionRequest::serial(&storage);
        let result = controlled(&execution, |control| {
            Ancestry::default()
                .contains(
                    0,
                    parents.len() - 1,
                    Some(0),
                    parents.len(),
                    &execution,
                    control,
                    |node| Ok(parents[node]),
                )
                .map(|_| ())
                .map_err(Into::into)
        });
        assert!(
            matches!(result, Err(Denial::Controlled(ref cause)) if *cause == expected),
            "{result:?}"
        );
        assert_eq!(storage.memory().unwrap().charged_memory_bytes(), 0);
    }
}

#[test]
fn summary_ancestry_releases_partial_and_ready_stops_then_retries_same_graph() {
    let (stopped, reads, _) = chain(64, 16 * 1024 * 1024, true);
    assert_eq!(
        reads, 64,
        "actual Ready tables were populated before cancellation"
    );
    assert!(matches!(
        stopped,
        Err(Denial::Controlled(Stop::Request(
            WorkCeilingDenial::Stopped(MapKernelStop::Cancelled)
        )))
    ));
    chain(64, 16 * 1024 * 1024, false).0.unwrap();
    let (limited, reads, _) = chain(128, 4096, false);
    eprintln!("ancestry4KiB partial actual parent reads={reads}, refusal={limited:?}");
    assert!(
        reads > 0 && reads < 128,
        "original memory refuses actual partial scratch, not root entry"
    );
    assert!(matches!(
        limited,
        Err(Denial::Controlled(Stop::Request(
            WorkCeilingDenial::Admission(worth_execution::LeaseDenial::MemoryExhausted(_))
        )))
    ));
    chain(128, 16 * 1024 * 1024, false).0.unwrap();
}
