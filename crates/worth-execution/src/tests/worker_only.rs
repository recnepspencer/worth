use super::*;
use crate::{
    ExecutionMap, ExecutionRequest, ExecutionResourceLease, ExecutionWorkCeiling, MapKernelFailure,
    MapOutcome, RetainedRunOutcome, WorkCeilingDenial,
};
use std::{collections::BTreeMap, sync::atomic::AtomicUsize};
use worth_proof::CanonicalUniqueVec;

fn workers(width: usize) -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::for_workers(
            ExecutionPosture::Automatic,
            DeterminismContract::CanonicalBitwise,
            NonZeroUsize::new(width).unwrap(),
        ),
        deadline: None,
        cancellation: CancellationToken::new(),
    }
}

fn entries(values: impl IntoIterator<Item = u64>) -> BTreeMap<PartitionIdentity, u64> {
    values
        .into_iter()
        .map(|value| (PartitionIdentity::new(value), value))
        .collect()
}

#[test]
fn worker_only_native_placement_has_no_meter_or_memory_charge() {
    let lease = ExecutionResourceLease::serial(workers(1)).unwrap();
    let map = ExecutionMap::<_, ()>::from_native_partitions(entries(1..=32)).unwrap();
    let outcome = ExecutionRequest::leased(&lease)
        .run_native(|active| {
            map.run_owned_native(active, |value, context| {
                assert!(!context.has_accounting());
                context.poll()?;
                Ok::<_, MapKernelFailure<()>>(value * 2)
            })
        })
        .unwrap();
    let MapOutcome::Complete { values, report } = outcome.0 else {
        panic!("native map refused")
    };
    assert_eq!(values, (1..=32).map(|value| value * 2).collect::<Vec<_>>());
    assert_eq!(report.accounted_cost(), None);
    assert!(!report.has_accounting());
    assert!(outcome.1.active_workers_high_watermark() >= 1);
    assert_eq!(lease.charged_memory_bytes(), 0);
    // The serial authority is shared with concurrent unit tests. Exact worker
    // release is proved by the isolated four-worker barrier/lifetime court.
}

#[test]
fn worker_only_retained_cells_reuse_without_a_cost_receipt() {
    let lease = ExecutionResourceLease::serial(workers(1)).unwrap();
    let request = ExecutionRequest::leased(&lease);
    let calls = AtomicUsize::new(0);
    let kernel = |value: &u64, context: &mut crate::MapKernelContext<'_, '_>| {
        calls.fetch_add(1, Ordering::SeqCst);
        assert!(!context.has_accounting());
        context.poll()?;
        Ok::<_, MapKernelFailure<()>>(*value * 10)
    };
    let map = ExecutionMap::<_, ()>::from_native_partitions(entries(1..=3)).unwrap();
    let RetainedRunOutcome::Complete { run, report, .. } = map
        .run_retained_run(request, ExecutionWorkCeiling::new(None), kernel)
        .unwrap()
    else {
        panic!("cold native run")
    };
    assert_eq!(report.accounted_cost(), None);
    assert_eq!(run.accounted_cost(), None);
    let prior = run.result(PartitionIdentity::new(1)).unwrap().clone();
    assert!(prior.native_reuse_available());
    let RetainedRunOutcome::Complete {
        run: next,
        report,
        metrics,
    } = run
        .advance_native(
            request,
            ExecutionWorkCeiling::new(None),
            BTreeMap::from([(PartitionIdentity::new(2), 20)]),
            CanonicalUniqueVec::try_from_sorted_unique(Vec::new()).unwrap(),
            kernel,
        )
        .unwrap()
    else {
        panic!("warm native run")
    };
    assert_eq!(calls.load(Ordering::SeqCst), 4);
    assert_eq!(report.accounted_cost(), None);
    assert_eq!(metrics.carried_slots, 2);
    assert!(prior.same_backing(next.result(PartitionIdentity::new(1)).unwrap()));
    assert_eq!(
        next.result(PartitionIdentity::new(2)).unwrap().value(),
        &200
    );
    assert_eq!(lease.charged_memory_bytes(), 0);
}

#[test]
fn worker_only_first_stop_survives_ignored_billing_and_callback_handoff() {
    let lease = ExecutionResourceLease::serial(workers(1)).unwrap();
    let result = ExecutionRequest::leased(&lease).run_native_with_checkpoint(|context| {
        assert_eq!(
            context.checkpoint(7),
            Err(KernelStop::AccountingUnavailable)
        );
        assert_eq!(context.poll(), Err(KernelStop::AccountingUnavailable));
        99
    });
    assert_eq!(
        result.unwrap_err(),
        WorkCeilingDenial::Stopped(KernelStop::AccountingUnavailable)
    );
}

#[test]
fn worker_only_request_handoff_observes_control_after_last_kernel() {
    let source = CancellationSource::new();
    let mut declared = workers(1);
    declared.cancellation = source.token();
    let lease = ExecutionResourceLease::serial(declared).unwrap();
    let map = ExecutionMap::<_, ()>::from_native_partitions(entries([1])).unwrap();
    let completed_kernel = std::cell::Cell::new(false);
    let result = ExecutionRequest::leased(&lease).run_native(|active| {
        let outcome = map.run_owned_native(active, |value, context| {
            assert!(!context.has_accounting());
            source.cancel();
            Ok::<_, MapKernelFailure<()>>(value)
        });
        // Cancellation after the last kernel has no earlier checkpoint at
        // which to reject that completed canonical value. The request's
        // handoff remains the next control boundary.
        let MapOutcome::Complete { values, report } = outcome else {
            panic!("completed kernel was retroactively rejected")
        };
        assert_eq!(values, vec![1]);
        assert_eq!(report.accounted_cost(), None);
        completed_kernel.set(true);
    });
    assert!(completed_kernel.get());
    assert_eq!(
        result.unwrap_err(),
        WorkCeilingDenial::Stopped(KernelStop::Cancelled)
    );
}

#[test]
fn worker_only_native_pattern_preserves_typed_failure_without_accepting_swallowed_success() {
    for propagate_failure in [true, false] {
        let source = CancellationSource::new();
        let lease = ExecutionResourceLease::serial(workers(1)).unwrap();
        let request = ExecutionRequest::leased(&lease);
        let map = ExecutionMap::<_, ()>::from_native_partitions(entries([1])).unwrap();
        source.cancel();
        let outer = request.run_native(|_| {
            let result = request.run_native_report(ExecutionWorkCeiling::new(None), |active| {
                let mut declared = workers(1);
                declared.cancellation = source.token();
                let child = active.unwrap().child(declared).unwrap();
                let outcome = map.run_owned_native(Some(&child), |_, _| {
                    panic!("the canceled child must refuse before its kernel");
                    #[allow(unreachable_code)]
                    Ok::<_, MapKernelFailure<()>>(())
                });
                let MapOutcome::Stopped { reason, .. } = outcome else {
                    panic!("the canceled child completed")
                };
                if propagate_failure {
                    Err(reason)
                } else {
                    Ok(())
                }
            });
            if propagate_failure {
                let (failed, report) = result.unwrap();
                assert_eq!(report.accounted_cost(), None);
                assert!(matches!(
                    failed,
                    Err(crate::MapStop::Failure {
                        identity,
                        cause: MapKernelFailure::Stop(KernelStop::Cancelled),
                    }) if identity == PartitionIdentity::new(1)
                ));
            } else {
                assert!(matches!(
                    result,
                    Err(WorkCeilingDenial::Stopped(KernelStop::NestedStopped))
                ));
            }
            // Even returning the actual typed cause does not clear its
            // request ownership: an enclosing successful handoff still stops.
        });
        assert_eq!(
            outer.unwrap_err(),
            WorkCeilingDenial::Stopped(KernelStop::NestedStopped)
        );
    }
}

#[test]
fn worker_only_cancellation_and_deadline_refuse_before_callback() {
    for expired in [false, true] {
        let source = CancellationSource::new();
        let mut declared = workers(1);
        declared.cancellation = source.token();
        if expired {
            declared.deadline = Some(Instant::now() - Duration::from_secs(1));
        } else {
            source.cancel();
        }
        let lease = ExecutionResourceLease::serial(declared).unwrap();
        let result =
            ExecutionRequest::leased(&lease).run_native(|_| panic!("callback must not enter"));
        let expected = if expired {
            KernelStop::DeadlineElapsed
        } else {
            KernelStop::Cancelled
        };
        assert_eq!(result.unwrap_err(), WorkCeilingDenial::Stopped(expected));
    }
}

#[test]
fn worker_only_width_lineage_and_inactive_scope_remain_authoritative() {
    let lease = ExecutionResourceLease::serial(workers(1)).unwrap();
    assert!(matches!(
        lease.child(workers(2)),
        Err(LeaseDenial::WorkerLimitExceedsParent)
    ));
    assert_eq!(
        crate::backend::RunLimits::check_native_scope(None),
        Err(LeaseDenial::NoActiveExecutionScope)
    );
    let unrelated = ExecutionResourceLease::serial(workers(1)).unwrap();
    let outer = ExecutionRequest::leased(&lease).run_native(|_| {
        assert!(matches!(
            ExecutionRequest::leased(&unrelated).run_native(|_| ()),
            Err(WorkCeilingDenial::Admission(
                LeaseDenial::UnrelatedNestedLease
            ))
        ));
    });
    assert_eq!(
        outer.unwrap_err(),
        WorkCeilingDenial::Stopped(KernelStop::NestedStopped)
    );
}

#[test]
fn worker_only_child_cannot_evade_a_finite_accounted_ancestor() {
    let mut parent = request(1, 100_000, 1);
    parent.policy = ExecutionRequestPolicy::new(
        parent.policy.posture(),
        parent.policy.determinism(),
        ExecutionBudget::new(NonZeroUsize::MIN, None, Some(1)),
    );
    let lease = ExecutionResourceLease::serial(parent).unwrap();
    assert!(matches!(
        lease.child(workers(1)),
        Err(LeaseDenial::WorkLimitExceedsParent)
    ));
    let result = ExecutionRequest::leased(&lease).run_native_with_checkpoint(|context| {
        assert!(context.has_accounting());
        context.checkpoint(2)
    });
    assert_eq!(
        result.unwrap_err(),
        WorkCeilingDenial::Stopped(KernelStop::WorkCeiling)
    );
}

#[test]
fn worker_only_native_allocation_preserves_physical_layout_denial() {
    let lease = ExecutionResourceLease::serial(workers(1)).unwrap();
    ExecutionRequest::leased(&lease)
        .run_native(|_| {
            let denial = crate::ExecutionArrayBuilder::<u64>::allocate(
                usize::MAX,
                crate::ExecutionAllocationPolicy::SystemAllocation,
            )
            .err()
            .expect("layout overflow");
            assert!(matches!(
                denial.classify(),
                crate::ExecutionAllocationDenialClass::Physical(
                    crate::ExecutionPhysicalAllocationDenialKind::Layout
                )
            ));
            let mut builder = crate::ExecutionArrayBuilder::<u64>::allocate(
                2,
                crate::ExecutionAllocationPolicy::Execution(&lease),
            )
            .unwrap();
            builder.push(10).unwrap();
            builder.push(20).unwrap();
            let array = builder.seal().unwrap();
            assert_eq!(array.elements(), &[10, 20]);
            assert_eq!(array.charged_payload_bytes(), None);
            assert_eq!(lease.charged_memory_bytes(), 0);
        })
        .unwrap();
}

#[test]
fn original_worker_only_lease_observes_only_its_authentic_accounted_descendant() {
    let parent = ExecutionResourceLease::serial(workers(1)).unwrap();
    let unrelated = ExecutionResourceLease::serial(workers(1)).unwrap();
    let child = parent
        .child(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::MIN, None, None),
            ),
            cancellation: CancellationToken::new(),
            deadline: None,
        })
        .unwrap();
    assert!(!parent.has_accounting());
    ExecutionRequest::leased(&child)
        .run_native_with_checkpoint(|context| {
            assert!(context.has_accounting());
            assert!(parent.has_accounting());
            assert!(
                !unrelated.has_accounting(),
                "ambient accounting is not another lease's authority"
            );
            ExecutionRequest::leased(&parent)
                .run_native_with_checkpoint(|nested| {
                    assert!(nested.has_accounting());
                    nested.checkpoint(1).unwrap();
                    assert_eq!(nested.charged_work(), 1);
                })
                .unwrap();
        })
        .unwrap();
    assert!(!parent.has_accounting());
}
