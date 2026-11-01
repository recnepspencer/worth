//! Returned effect custody and the independently authoritative execution stop.
use std::{cell::Cell, num::NonZeroUsize, time::Instant};
use worth_execution::{
    CancellationSource, ExecutionRequest, ExecutionResourceLease, ExecutionWorkCeiling,
    LeaseRequest, MapKernelStop, SerialRequest, WorkCeilingDenial,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

mod accounted;
mod own_stop;

fn worker_grant() -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::for_workers(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            NonZeroUsize::MIN,
        ),
        cancellation: CancellationSource::new().token(),
        deadline: None,
    }
}

fn on_request(leased: bool, grant: LeaseRequest, operation: impl FnOnce(ExecutionRequest<'_, '_>)) {
    if leased {
        let lease = ExecutionResourceLease::serial(grant).unwrap();
        operation(ExecutionRequest::leased(&lease));
    } else {
        let serial = SerialRequest::from_policy(&grant.policy, grant.cancellation, grant.deadline);
        operation(ExecutionRequest::serial(&serial));
    }
}

fn complete<'authority, R>(
    request: ExecutionRequest<'_, 'authority>,
    accounted: bool,
    operation: impl for<'scope> FnOnce(Option<&'scope ExecutionResourceLease<'authority>>) -> R,
) -> Result<(R, worth_foundational::ExecutionPhysicalReport), WorkCeilingDenial> {
    if accounted {
        request
            .run_completion(ExecutionWorkCeiling::new(None), operation)
            .map(|(value, report)| (value, report.physical()))
    } else {
        request.run_native_completion(operation)
    }
}

#[test]
fn stopped_original_controls_refuse_completion_before_invoking_effects() {
    for leased in [false, true] {
        for cancelled in [false, true] {
            let cancellation = CancellationSource::new();
            let mut grant = worker_grant();
            grant.cancellation = cancellation.token();
            if cancelled {
                cancellation.cancel();
            } else {
                grant.deadline = Some(Instant::now());
            }
            on_request(leased, grant, |request| {
                let called = Cell::new(false);
                let result = request.run_native_completion(|_| called.set(true));
                assert!(!called.get());
                assert_eq!(
                    result,
                    Err(WorkCeilingDenial::Stopped(if cancelled {
                        MapKernelStop::Cancelled
                    } else {
                        MapKernelStop::DeadlineElapsed
                    }))
                );
            });
        }
    }
}

#[test]
fn returned_move_only_business_custody_survives_later_cancellation_or_deadline() {
    for leased in [false, true] {
        for cancelled in [false, true] {
            let cancellation = CancellationSource::new();
            let mut grant = worker_grant();
            grant.cancellation = cancellation.token();
            let deadline = Instant::now() + std::time::Duration::from_secs(1);
            if !cancelled {
                grant.deadline = Some(deadline);
            }
            on_request(leased, grant, |request| {
                let business_owner = Box::new(String::from("original accepted owner"));
                let address = std::ptr::from_ref(business_owner.as_ref());
                let (returned, physical) = request
                    .run_native_completion(|active| {
                        assert_eq!(active.is_some(), leased);
                        let business_result = Err::<(), _>(business_owner);
                        if cancelled {
                            cancellation.cancel();
                        } else {
                            std::thread::sleep(deadline.saturating_duration_since(Instant::now()));
                        }
                        assert_eq!(
                            request.consult(),
                            Err(WorkCeilingDenial::Stopped(if cancelled {
                                MapKernelStop::Cancelled
                            } else {
                                MapKernelStop::DeadlineElapsed
                            }))
                        );
                        business_result
                    })
                    .expect("a returned business refusal retains its original owner");
                let owner = returned.unwrap_err();
                assert_eq!(std::ptr::from_ref(owner.as_ref()), address);
                assert_eq!(*owner, "original accepted owner");
                assert_eq!(physical.active_workers_high_watermark(), 1);
                assert_eq!(physical.peak_charged_memory_bytes(), 0);
                assert!(!request.has_accounting());
            });
        }
    }
}

#[test]
fn completion_uses_accounted_parent_remainder_once_and_keeps_its_stop_latched() {
    let mut grant = worker_grant();
    grant.policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(NonZeroUsize::MIN, None, Some(7)),
    );
    for leased in [false, true] {
        on_request(leased, grant.clone(), |request| {
            let observed = Cell::new(false);
            let (_, report) = request
                .run(ExecutionWorkCeiling::new(None), |_| {
                    let outer = request.run_native_with_checkpoint(|parent| {
                        parent.checkpoint(2).unwrap();
                        let business_owner = Box::new(41);
                        let (returned, _) = request
                            .run_native_completion(|_| {
                                request
                                    .run_native_with_checkpoint(|context| context.checkpoint(3))
                                    .unwrap()
                                    .0
                                    .unwrap();
                                let maintenance = request
                                    .run_native_with_checkpoint(|context| context.checkpoint(3));
                                assert_eq!(
                                    maintenance,
                                    Err(WorkCeilingDenial::Stopped(MapKernelStop::WorkCeiling))
                                );
                                business_owner
                            })
                            .unwrap();
                        assert_eq!(*returned, 41);
                        // No poll or consult may turn the joined parent latch
                        // into a checkpoint stop before this effectful entry.
                        assert_eq!(
                            request.run_native_completion::<()>(|_| {
                                panic!("stopped original meter must refuse before effects")
                            }),
                            Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped))
                        );
                        assert_eq!(parent.charged_work(), 5);
                        assert_eq!(parent.take_nested_computation_stop(), None);
                        assert_eq!(parent.poll(), Err(MapKernelStop::NestedStopped));
                        assert_eq!(
                            request.consult(),
                            Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped))
                        );
                        observed.set(true);
                    });
                    assert!(matches!(outer, Err(WorkCeilingDenial::Stopped(_))));
                })
                .unwrap();
            assert!(observed.get(), "contained callback assertions must finish");
            assert_eq!(report.accounted_cost(), Some((5, 5)));
        });
    }
}

#[test]
fn root_completion_keeps_active_child_affinity_and_rejects_sibling_or_foreign_entry() {
    let root = ExecutionResourceLease::serial(worker_grant()).unwrap();
    let child = root.child(worker_grant()).unwrap();
    let sibling = root.child(worker_grant()).unwrap();
    let foreign = ExecutionResourceLease::serial(worker_grant()).unwrap();
    let cancellation = CancellationSource::new();
    let mut child_grant = worker_grant();
    child_grant.cancellation = cancellation.token();
    let active = child.child(child_grant).unwrap();
    let observed = Cell::new(false);
    let outer = ExecutionRequest::leased(&active).run_native(|_| {
        let (returned, _) = ExecutionRequest::leased(&root)
            .run_native_completion(|carried| {
                assert!(carried.unwrap().descends_from(&active));
                cancellation.cancel();
                for refused in [&sibling, &foreign] {
                    let called = Cell::new(false);
                    assert_eq!(
                        ExecutionRequest::leased(refused)
                            .run_native_completion(|_| called.set(true)),
                        Err(WorkCeilingDenial::Admission(
                            worth_execution::LeaseDenial::UnrelatedNestedLease
                        ))
                    );
                    assert!(!called.get());
                }
                Box::new(73)
            })
            .unwrap();
        assert_eq!(*returned, 73);
        observed.set(true);
    });
    assert!(observed.get());
    assert_eq!(
        outer,
        Err(WorkCeilingDenial::Stopped(MapKernelStop::Cancelled))
    );
}

#[test]
fn completion_contains_a_callback_panic_before_a_business_value_returns() {
    for leased in [false, true] {
        on_request(leased, worker_grant(), |request| {
            let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                request.run_native_completion::<()>(|_| panic!("original completion panic"))
            }))
            .expect("Execution retains its ordinary panic boundary");
            assert_eq!(panic, Err(WorkCeilingDenial::Panicked));
            request.run_native_completion(|_| ()).unwrap();
        });
    }
}

#[test]
fn completion_preserves_an_untaken_computational_panic_and_allows_recovery_after_take() {
    for leased in [false, true] {
        for accounted in [false, true] {
            on_request(leased, worker_grant(), |request| {
                request
                    .run_native_with_checkpoint(|parent| {
                        assert_eq!(
                            request.run_native::<()>(|_| panic!("original nested panic")),
                            Err(WorkCeilingDenial::Panicked)
                        );
                        // The unpolled cause remains computation-owned through
                        // refusal; callers can still explicitly take and recover it.
                        assert_eq!(
                            complete::<()>(request, accounted, |_| {
                                panic!("untaken computation stop must refuse effects")
                            }),
                            Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped))
                        );
                        assert_eq!(
                            parent.take_nested_computation_stop(),
                            Some(worth_execution::NestedComputationStop::Panic)
                        );
                        let (recovered, _) =
                            complete(request, accounted, |_| Box::new("recovered business owner"))
                                .unwrap();
                        assert_eq!(*recovered, "recovered business owner");
                    })
                    .unwrap();
            });
        }
    }
}

#[test]
fn native_completion_uses_existing_workers_without_inventing_an_accounted_bill() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }
    let authority = worth_execution::ExecutionAuthority::try_construct(
        worth_execution::ExecutionAuthorityConfig::for_workers(NonZeroUsize::new(2).unwrap()),
    )
    .unwrap();
    let mut grant = worker_grant();
    grant.policy = ExecutionRequestPolicy::for_workers(
        ExecutionPosture::Automatic,
        DeterminismContract::CanonicalBitwise,
        NonZeroUsize::new(2).unwrap(),
    );
    let lease = authority.request_lease(grant).unwrap();
    let request = ExecutionRequest::leased(&lease);
    let map = worth_execution::ExecutionMap::<_, ()>::from_native_partitions(
        (1..=8)
            .map(|id| (worth_foundational::PartitionIdentity::new(id), id))
            .collect(),
    )
    .unwrap();
    let (result, physical) = request
        .run_native_completion(|active| {
            assert!(active.unwrap().descends_from(&lease));
            map.run_owned_native(active, |value, context| {
                assert!(!context.has_accounting());
                context.poll()?;
                Ok::<_, worth_execution::MapKernelFailure<()>>(value)
            })
        })
        .unwrap();
    let worth_execution::MapOutcome::Complete { values, report } = result else {
        panic!("the native map must actually complete");
    };
    assert_eq!(values, (1..=8).collect::<Vec<_>>());
    assert_eq!(report.accounted_cost(), None);
    assert_eq!(physical.peak_charged_memory_bytes(), 0);
    assert!(physical.active_workers_high_watermark() > 0);
    assert!(physical.active_workers_high_watermark() <= 2);
}
