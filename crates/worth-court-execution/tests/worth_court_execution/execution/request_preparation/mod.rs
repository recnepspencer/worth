//! Original request entry, active lineage and real remaining allowance.
use worth_execution::{
    CancellationSource, ExecutionRequest, ExecutionResourceLease, LeaseRequest, MapKernelStop,
    WorkCeilingDenial,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

mod stop_settlement;

fn grant(work: u64, memory: Option<u64>) -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(std::num::NonZeroUsize::MIN, memory, Some(work)),
        ),
        cancellation: CancellationSource::new().token(),
        deadline: None,
    }
}

fn step(request: ExecutionRequest<'_, '_>, units: u64) -> Result<(), WorkCeilingDenial> {
    let (result, _) = request.run_native_with_checkpoint(|context| context.checkpoint(units))?;
    result.map_err(WorkCeilingDenial::Stopped)
}

#[test]
fn standalone_preparation_bills_all_steps_to_the_original_finite_grant() {
    for (work, funded) in [(7, true), (6, false)] {
        let lease = ExecutionResourceLease::serial(grant(work, None)).unwrap();
        let request = ExecutionRequest::leased(&lease);
        let result = request.prepare_native(|active| {
            assert!(active.unwrap().descends_from(&lease));
            step(request, 3)?;
            step(request, 4)
        });
        assert_eq!(
            result,
            Ok(if funded {
                Ok(())
            } else {
                Err(WorkCeilingDenial::Stopped(MapKernelStop::WorkCeiling))
            })
        );
    }
}

#[test]
fn active_preparation_uses_prior_work_and_does_not_charge_a_second_report() {
    for (work, funded) in [(7, true), (6, false)] {
        let lease = ExecutionResourceLease::serial(grant(work, None)).unwrap();
        let request = ExecutionRequest::leased(&lease);
        let outer = request.run_native_with_checkpoint(|context| {
            context.checkpoint(2).unwrap();
            let result = request.prepare_native(|_| {
                step(request, 3)?;
                step(request, 2)
            });
            assert_eq!(
                result,
                Ok(if funded {
                    Ok(())
                } else {
                    Err(WorkCeilingDenial::Stopped(MapKernelStop::WorkCeiling))
                })
            );
            assert_eq!(context.charged_work(), if funded { 7 } else { 5 });
        });
        if funded {
            outer.unwrap();
        } else {
            assert!(matches!(
                outer,
                Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped))
            ));
        }
    }
}

#[test]
fn root_carriage_keeps_the_active_child_work_memory_and_cancellation() {
    let root = ExecutionResourceLease::serial(grant(100, None)).unwrap();
    let cancellation = CancellationSource::new();
    let mut child_grant = grant(5, Some(16 << 10));
    child_grant.cancellation = cancellation.token();
    let child = root.child(child_grant).unwrap();
    let request = ExecutionRequest::leased(&root);
    let outer = ExecutionRequest::leased(&child).run_native_with_checkpoint(|context| {
        context.checkpoint(2).unwrap();
        let result = request.prepare_native(|active| {
            let active = active.unwrap();
            assert!(active.descends_from(&child));
            assert_eq!(active.policy(), child.policy());
            assert_eq!(request.work_ceiling(), Some(100));
            step(request, 3)?;
            assert_eq!(context.charged_work(), 5);
            let denied = request.reserve_retained_memory(16 << 10);
            assert!(matches!(
                denied,
                Err(WorkCeilingDenial::Admission(
                    worth_execution::LeaseDenial::MemoryExhausted(_)
                ))
            ));
            cancellation.cancel();
            Ok::<_, WorkCeilingDenial>(())
        });
        assert_eq!(
            result,
            Err(WorkCeilingDenial::Stopped(MapKernelStop::Cancelled))
        );
    });
    assert_eq!(
        outer,
        Err(WorkCeilingDenial::Stopped(MapKernelStop::Cancelled))
    );
}

#[test]
fn supplied_child_below_an_active_ancestor_keeps_its_narrower_grant() {
    let root = ExecutionResourceLease::serial(grant(100, None)).unwrap();
    let child = root.child(grant(3, None)).unwrap();
    let request = ExecutionRequest::leased(&child);
    let outer = ExecutionRequest::leased(&root).run_native_with_checkpoint(|_| {
        assert_eq!(
            request.prepare_native(|active| {
                assert!(active.unwrap().descends_from(&child));
                step(request, 4)
            }),
            Ok(Err(WorkCeilingDenial::Stopped(MapKernelStop::WorkCeiling)))
        );
    });
    assert!(matches!(
        outer,
        Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped))
    ));
}

#[test]
fn foreign_and_sibling_preparation_refuse_lineage_before_ambient_cancellation_or_callback() {
    let root = ExecutionResourceLease::serial(grant(100, None)).unwrap();
    let cancellation = CancellationSource::new();
    let mut active_grant = grant(100, None);
    active_grant.cancellation = cancellation.token();
    let active = root.child(active_grant).unwrap();
    let sibling = root.child(grant(100, None)).unwrap();
    let foreign = ExecutionResourceLease::serial(grant(100, None)).unwrap();
    let outer = ExecutionRequest::leased(&active).run_native_with_checkpoint(|_| {
        cancellation.cancel();
        for refused in [&sibling, &foreign] {
            let called = std::cell::Cell::new(false);
            let result = ExecutionRequest::leased(refused).prepare_native(|_| {
                called.set(true);
                Ok::<_, ()>(())
            });
            assert_eq!(
                result,
                Err(WorkCeilingDenial::Admission(
                    worth_execution::LeaseDenial::UnrelatedNestedLease
                ))
            );
            assert!(!called.get());
        }
    });
    assert!(matches!(
        outer,
        Err(WorkCeilingDenial::Stopped(MapKernelStop::Cancelled))
    ));
}

#[test]
fn active_root_preparation_keeps_the_child_absolute_deadline() {
    let root = ExecutionResourceLease::serial(grant(100, None)).unwrap();
    let mut child_grant = grant(100, None);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
    child_grant.deadline = Some(deadline);
    let child = root.child(child_grant).unwrap();
    let outer = ExecutionRequest::leased(&child).run_native_with_checkpoint(|_| {
        let result = ExecutionRequest::leased(&root).prepare_native(|_| {
            std::thread::sleep(deadline.saturating_duration_since(std::time::Instant::now()));
            Ok::<_, ()>(())
        });
        assert_eq!(
            result,
            Err(WorkCeilingDenial::Stopped(MapKernelStop::DeadlineElapsed))
        );
    });
    assert!(matches!(
        outer,
        Err(WorkCeilingDenial::Stopped(MapKernelStop::DeadlineElapsed))
    ));
}
