//! Numeric topology only; no Native, Source slot or qualified root is fabricated.
use worth_execution::ExecutionRequest;
use worth_query_execution::facade::domain_computation::WorthQueryManagedIndexStop as Stop;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::managed_index::source_structure::cold::acyclic;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::managed_index::invocation::controlled;
use worth_query_execution::facade::domain_computation::WorthQueryManagedIndexWorkDenial as Denial;
use worth_execution::CancellationSource;
use worth_execution::ExecutionWorkCeiling;
use worth_execution::MapKernelStop;
use worth_execution::SerialMemoryBudget;
use worth_execution::SerialRequest;
use worth_execution::WorkCeilingDenial;

pub fn chain(
    count: usize,
    memory: Option<u64>,
    cancel_at: Option<usize>,
) -> (Result<(), Denial>, usize, u64) {
    let cancel = CancellationSource::new();
    let storage = SerialRequest::from_memory(SerialMemoryBudget::new(memory), cancel.token(), None);
    let execution = ExecutionRequest::serial(&storage);
    let mut reads = 0;
    let mut reads_by_node = vec![0; count];
    let (result, report) = ExecutionWorkCeiling::new(Some(32_000_000))
        .run_with_checkpoint(None, Some(&storage), |_| {
            controlled(&execution, |control| {
                acyclic(
                    (0..count).rev().map(|n| n * 1024),
                    count,
                    count.saturating_sub(1) * 1024 + usize::from(count > 0),
                    &execution,
                    control,
                    |key| {
                        reads += 1;
                        assert_eq!(key % 1024, 0);
                        assert!(key / 1024 < count);
                        reads_by_node[key / 1024] += 1;
                        if cancel_at == Some(reads) {
                            cancel.cancel();
                        }
                        Ok(key.checked_sub(1024))
                    },
                )
                .map_err(Into::into)
            })
        })
        .expect("predeclared memory admits the original SDK root before DFS");
    assert_eq!(
        storage.memory().unwrap().charged_memory_bytes(),
        0,
        "actual DFS scratch ownership released"
    );
    if result.is_ok() {
        assert!(
            reads_by_node.iter().all(|reads| *reads == 1),
            "each actual parent is read exactly once"
        );
    }
    (result, reads, report.charged_work())
}
#[test]
fn cold_parent_dfs_visits_sparse_chains_with_bounded_original_work() {
    let mut work = [0; 2];
    for count in [0, 1, 64, 128] {
        let (result, reads, invoice) = chain(count, Some(16 * 1024 * 1024), None);
        result.unwrap();
        assert!(
            reads == count,
            "actual parent callbacks, count={count}, reads={reads}"
        );
        match count {
            64 => work[0] = invoice,
            128 => work[1] = invoice,
            _ => {}
        }
    }
    let [work64, work128] = work;
    eprintln!("cold DFS accepted original SDK work: 64={work64}, 128={work128}");
    assert!(
        work64 > 0 && work128 < 3 * work64,
        "separate actual SDK bookkeeping growth"
    );
}

#[test]
fn cold_walk_tags_match_independent_exhaustive_functional_graph_cycles() {
    use std::collections::BTreeSet;
    for count in 0usize..=4 {
        for mut encoding in 0..(count + 1).pow(count as u32) {
            let parents: Vec<_> = (0..count)
                .map(|_| {
                    let digit = encoding % (count + 1);
                    encoding /= count + 1;
                    (digit < count).then_some(digit)
                })
                .collect();
            let cyclic = (0..count).any(|seed| {
                let mut seen = BTreeSet::new();
                let mut next = Some(seed);
                while let Some(node) = next {
                    if !seen.insert(node) {
                        return true;
                    }
                    next = parents[node];
                }
                false
            });
            let storage = SerialRequest::from_memory(
                SerialMemoryBudget::new(Some(16 * 1024 * 1024)),
                CancellationSource::new().token(),
                None,
            );
            let execution = ExecutionRequest::serial(&storage);
            let mut reads = vec![0; count];
            let result = controlled(&execution, |control| {
                acyclic(
                    (0..count).chain((0..count).rev()),
                    count,
                    count,
                    &execution,
                    control,
                    |node| {
                        reads[node] += 1;
                        Ok(parents[node])
                    },
                )
                .map_err(Into::into)
            });
            if cyclic {
                assert!(
                    matches!(result, Err(Denial::Controlled(Stop::SourceStructureCycle))),
                    "{parents:?}: {result:?}"
                );
            } else {
                result.unwrap();
                assert!(
                    reads.iter().all(|reads| *reads == 1),
                    "{parents:?}: {reads:?}"
                );
            }
            assert_eq!(storage.memory().unwrap().charged_memory_bytes(), 0);
        }
    }
}
#[test]
fn cold_parent_dfs_refuses_self_and_disconnected_cycles() {
    let mut long_cycle = vec![None];
    long_cycle.extend((1..=64).map(|node| Some(if node == 64 { 1 } else { node + 1 })));
    for parents in [vec![Some(0)], vec![None, Some(2), Some(1)], long_cycle] {
        let storage = SerialRequest::from_memory(
            SerialMemoryBudget::new(Some(16 * 1024 * 1024)),
            CancellationSource::new().token(),
            None,
        );
        let execution = ExecutionRequest::serial(&storage);
        let result = controlled(&execution, |control| {
            acyclic(
                0..parents.len(),
                parents.len(),
                parents.len(),
                &execution,
                control,
                |key| Ok(parents[key]),
            )
            .map_err(Into::into)
        });
        assert!(matches!(
            result,
            Err(Denial::Controlled(Stop::SourceStructureCycle))
        ));
        assert_eq!(storage.memory().unwrap().charged_memory_bytes(), 0);
    }
}

#[test]
fn cold_walk_refuses_coordinates_outside_its_actual_frontier() {
    for (seed, parent) in [(2, None), (0, Some(2))] {
        let storage = SerialRequest::from_memory(
            SerialMemoryBudget::new(Some(16 * 1024 * 1024)),
            CancellationSource::new().token(),
            None,
        );
        let execution = ExecutionRequest::serial(&storage);
        let result = controlled(&execution, |control| {
            acyclic([seed], 1, 2, &execution, control, |_| Ok(parent)).map_err(Into::into)
        });
        assert!(matches!(
            result,
            Err(Denial::Controlled(Stop::SourceStructureIncomplete))
        ));
        assert_eq!(storage.memory().unwrap().charged_memory_bytes(), 0);
    }
}
#[test]
fn cold_parent_dfs_releases_stopped_scratch_and_retries_with_fresh_request() {
    let (cancelled, reads, _) = chain(64, Some(16 * 1024 * 1024), Some(8));
    assert_eq!(reads, 8);
    assert!(matches!(
        cancelled,
        Err(Denial::Controlled(Stop::Request(
            WorkCeilingDenial::Stopped(MapKernelStop::Cancelled)
        )))
    ));
    let (retry, reads, _) = chain(64, Some(16 * 1024 * 1024), None);
    retry.unwrap();
    assert!(reads <= 128);
    let (limited, partial_reads, _) = chain(128, Some(4096), None);
    eprintln!("cold DFS4KiB partial parent reads={partial_reads}, refusal={limited:?}");
    assert!(
        partial_reads == 0,
        "the original memory must admit the complete bounded tag column before visiting parents"
    );
    assert!(
        matches!(
            limited,
            Err(Denial::Controlled(Stop::Storage(
                worth_execution::ExecutionRequestArrayDenial::Request(
                    WorkCeilingDenial::Admission(worth_execution::LeaseDenial::MemoryExhausted(_))
                )
            )))
        ),
        "actual original-memory refusal: {limited:?}"
    );
    let (retry, reads, _) = chain(128, Some(16 * 1024 * 1024), None);
    retry.unwrap();
    assert!(reads <= 256);
}
