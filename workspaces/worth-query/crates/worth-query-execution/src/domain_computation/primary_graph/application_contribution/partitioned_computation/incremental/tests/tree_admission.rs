//! Scope admission before construction and after the scheduled shape are distinct.

use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicUsize, Ordering};

use worth_execution::{
    ExecutionMap, KeylessPartition, MapKernelFailure, ReduceInputDenial, ReductionTree,
};

use super::*;
use crate::domain_computation::primary_graph::application_contribution::request_execution::{
    test_execution_authority, test_policy,
};

const PARTITIONS: usize = 16;
const MEMORY: u64 = 1 << 20;
const KERNEL_WORK: u64 = 3;
const VALUE_BYTES: u64 = 4096;
static COMBINES: AtomicUsize = AtomicUsize::new(0);

fn sum(left: &u64, right: &u64) -> u64 {
    COMBINES.fetch_add(1, Ordering::Relaxed);
    left + right
}

fn fill_room(execution: &QueryRequestExecution<'_>, before_map: bool) -> crate::domain_computation::primary_graph::application_contribution::request_execution::QueryMemoryReservation{
    let bytes = if before_map {
        MEMORY
    } else {
        // This successful hold leaves 64 KiB beside the tree's declared
        // build memory, before framework charges. The admitted map finishes;
        // the frontier's additional subtree result reservation cannot fit.
        let build = ReductionTree::<u64, fn(&u64, &u64) -> u64>::checked_build_memory_bound(
            PARTITIONS,
            VALUE_BYTES,
        )
        .unwrap();
        MEMORY - build - (64 << 10)
    };
    execution.reserve(bytes).unwrap()
}

#[test]
fn scope_admission_reports_shape_work_only_when_construction_ran() {
    let keys: Vec<_> = (0..PARTITIONS).map(|n| Id::new(n as u64)).collect();
    let shape = shape::Shape::from_sorted(&keys);
    let visits = shape::Shape::shape_work(&shape, PARTITIONS);
    for before_map in [true, false] {
        let request = live_scope();
        let execution = QueryRequestExecution::open(
            RuntimeWorldExecutionPlacement::Leased {
                authority: test_execution_authority(),
                policy: test_policy(NonZeroUsize::MIN, MEMORY),
            },
            &request,
        );
        let map = ExecutionMap::<u64, u64>::from_keyless_partitions(
            keys.iter()
                .map(|id| {
                    (
                        *id,
                        KeylessPartition {
                            value: 1,
                            kernel_scratch_bytes: 0,
                            max_result_bytes: 8,
                        },
                    )
                })
                .collect(),
        )
        .unwrap();
        let filled = Mutex::new(before_map.then(|| fill_room(&execution, true)));
        let entries = AtomicUsize::new(0);
        let mut tree_memory = execution.reserve(0).unwrap();
        let inputs = execution.reserve(0).unwrap();
        tree_runs();
        COMBINES.store(0, Ordering::Relaxed);
        let outcome = execution.dispatch().unwrap().reduce(
            10_000,
            &map,
            inputs,
            &mut tree_memory,
            |value, context| {
                context.checkpoint(KERNEL_WORK)?;
                if entries.fetch_add(1, Ordering::Relaxed) + 1 == PARTITIONS {
                    *filled.lock().unwrap() = Some(fill_room(&execution, false));
                }
                Ok::<u64, MapKernelFailure<u32>>(*value)
            },
            0,
            sum as fn(&u64, &u64) -> u64,
            VALUE_BYTES,
            Cause::NoProducerPrior,
        );
        let reports = tree_runs();
        assert_eq!(COMBINES.load(Ordering::Relaxed), 0);
        if before_map {
            assert_eq!(entries.load(Ordering::Relaxed), 0);
            assert!(
                outcome.is_err()
                    || matches!(outcome, Ok(Err(ReduceInputDenial::ScopeAdmission { .. })))
            );
            assert!(reports.is_empty(), "no tree operation ran");
        } else {
            let Ok(Err(ReduceInputDenial::ScopeAdmission { report, .. })) = outcome else {
                panic!("scheduled admission fails after the shape");
            };
            assert_eq!(entries.load(Ordering::Relaxed), PARTITIONS);
            assert_eq!(
                report.charged_work(),
                PARTITIONS as u64 * KERNEL_WORK + visits
            );
            let [TreeRun::Full(Cause::NoProducerPrior, metrics)] = reports.as_slice() else {
                panic!("shape construction has one full report: {reports:?}")
            };
            assert_eq!(metrics.structural_visits, u128::from(visits));
            assert_eq!(metrics.charged_work, u128::from(visits));
            assert_eq!(metrics.charged_span, u128::from(visits));
            assert_eq!(metrics.combine_calls, 0);
            assert_eq!(metrics.recombined_nodes, 0);
        }
    }
}
