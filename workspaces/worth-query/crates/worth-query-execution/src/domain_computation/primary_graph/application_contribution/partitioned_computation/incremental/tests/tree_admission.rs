//! Actual request exhaustion before mapping and after its last kernel.

use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicUsize, Ordering};

use worth_execution::{
    ExecutionAllocationDenialClass, ExecutionMap, LeaseDenial, MapKernelFailure, ReduceInputDenial,
    ReductionDenial, ReductionRunFailure, ReductionRunStop, RetainedMapDenial,
    RetainedReduceDenial, WorkCeilingDenial,
};

use super::*;
use crate::domain_computation::primary_graph::application_contribution::request_execution::{
    query_test_execution_authority_for_test, test_policy,
};
use crate::domain_computation::primary_graph::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;

const PARTITIONS: usize = 16;
const MEMORY: u64 = 1 << 20;
const KERNEL_WORK: u64 = 3;
static COMBINES: AtomicUsize = AtomicUsize::new(0);

fn sum(left: &u64, right: &u64) -> u64 {
    COMBINES.fetch_add(1, Ordering::Relaxed);
    left + right
}

fn fill_room(execution: &QueryRequestExecution<'_>) -> crate::domain_computation::primary_graph::application_contribution::request_execution::QueryMemoryReservation{
    let Err(Resource::MemoryLimit { admitted, .. }) = execution.reserve(2 * MEMORY) else {
        panic!("the finite policy names its remaining room")
    };
    execution.reserve(admitted).unwrap()
}

#[test]
fn native_request_exhaustion_refuses_before_map_or_after_the_last_kernel() {
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let keys: Vec<_> = (0..PARTITIONS).map(|n| Id::new(n as u64)).collect();
    let shape = shape::Shape::from_sorted(&keys);
    let visits = shape::Shape::shape_work(&shape, PARTITIONS);
    for before_map in [true, false] {
        let request = live_scope();
        let execution = QueryRequestExecution::open(
            RuntimeWorldExecutionPlacement::Leased {
                authority: query_test_execution_authority_for_test(),
                policy: test_policy(NonZeroUsize::new(2).unwrap(), Some(MEMORY)),
            },
            &request,
        );
        let dispatch = execution.dispatch().unwrap();
        let original_hold = dispatch.native_lease_for_test().charged_memory_bytes();
        let map =
            ExecutionMap::from_native_partitions(keys.iter().map(|id| (*id, 1_u64)).collect())
                .unwrap();
        let filled = Mutex::new(before_map.then(|| fill_room(&execution)));
        let entries = AtomicUsize::new(0);
        tree_runs();
        COMBINES.store(0, Ordering::Relaxed);
        let kernel = |value: &u64, context: &mut worth_execution::MapKernelContext<'_, '_>| {
            context.checkpoint(KERNEL_WORK)?;
            if entries.fetch_add(1, Ordering::Relaxed) + 1 == PARTITIONS {
                *filled.lock().unwrap() = Some(fill_room(&execution));
            }
            Ok::<u64, MapKernelFailure<u32>>(*value)
        };
        let outcome = dispatch.reduce_retained(
            Some(10_000),
            &map,
            (kernel, kernel),
            0,
            sum as fn(&u64, &u64) -> u64,
            Cause::NoProducerPrior,
        );
        match &outcome {
            Err(RetainedReduceDenial::Retention(RetainedMapDenial::Admission(
                LeaseDenial::MemoryExhausted(_),
            )))
            | Err(RetainedReduceDenial::Retention(RetainedMapDenial::Request(
                WorkCeilingDenial::Admission(LeaseDenial::MemoryExhausted(_)),
            )))
            | Err(RetainedReduceDenial::Reduction(ReduceInputDenial::ScopeAdmission {
                denial: LeaseDenial::MemoryExhausted(_),
                ..
            }))
            | Err(RetainedReduceDenial::Reduction(ReduceInputDenial::ReductionStopped {
                failure: ReductionRunFailure {
                    reason: ReductionRunStop::Denial(ReductionDenial::Admission(
                        LeaseDenial::MemoryExhausted(_),
                    )),
                    ..
                },
                ..
            })) => {}
            Err(RetainedReduceDenial::Retention(RetainedMapDenial::Allocation(cause))) => {
                assert!(matches!(
                    cause.classify(),
                    ExecutionAllocationDenialClass::Request(WorkCeilingDenial::Admission(
                        LeaseDenial::MemoryExhausted(_)
                    ))
                ));
            }
            _ => panic!(
                "request memory must refuse: before_map={before_map}, entries={}, combines={}, denial={:?}",
                entries.load(Ordering::Relaxed),
                COMBINES.load(Ordering::Relaxed),
                outcome.as_ref().err(),
            ),
        }
        let reports = tree_runs();
        assert_eq!(COMBINES.load(Ordering::Relaxed), 0);
        if before_map {
            assert_eq!(entries.load(Ordering::Relaxed), 0);
            assert!(
                outcome.is_err(),
                "the exhausted request cannot enter mapping"
            );
            assert!(reports.is_empty(), "no tree operation ran");
        } else {
            assert!(
                outcome.is_err(),
                "native custody cannot admit new cells after exhaustion"
            );
            assert_eq!(entries.load(Ordering::Relaxed), PARTITIONS);

            let [TreeRun::Full(Cause::NoProducerPrior, metrics)] = reports.as_slice() else {
                panic!("the refused tree reports its actual constructed shape: {reports:?}");
            };
            let metrics = metrics
                .accounted_metrics()
                .expect("explicit accounted fixture");
            assert_eq!(metrics.structural_visits, u128::from(visits));
            assert_eq!(metrics.charged_work, u128::from(visits));
            assert_eq!(metrics.charged_span, u128::from(visits));
            assert_eq!(metrics.combine_calls, 0);
            assert_eq!(metrics.recombined_nodes, 0);
        }
        drop(outcome);
        drop(filled);
        assert_eq!(
            dispatch.native_lease_for_test().charged_memory_bytes(),
            original_hold,
            "refused mapping and tree admission release their temporary grants",
        );
    }
}
