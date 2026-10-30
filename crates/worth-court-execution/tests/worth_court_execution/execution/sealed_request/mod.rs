#[allow(unused_imports)]
use crate::sealed_request::ordinary_context::*;

#[test]
fn sealed_serial_request_refuses_memory_before_running_the_operation() {
    let serial = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::from_policy(&policy(1)),
        worth_execution::CancellationToken::new(),
        None,
    );
    let called = std::sync::atomic::AtomicBool::new(false);
    let denial = worth_execution::ExecutionRequest::serial(&serial)
        .run(
            worth_execution::ExecutionWorkCeiling::new(Some(1_000)),
            |_| {
                called.store(true, std::sync::atomic::Ordering::Relaxed);
            },
        )
        .unwrap_err();
    let worth_execution::WorkCeilingDenial::Admission(
        worth_execution::LeaseDenial::MemoryExhausted(memory),
    ) = denial
    else {
        panic!("required serial memory must produce its own resource cause")
    };
    let worth_execution::MemoryLimitDenial::Capacity {
        requested,
        admitted,
        level,
    } = memory
    else {
        panic!("finite memory policy must refuse capacity, not counter overflow")
    };
    assert_eq!(
        level,
        worth_execution::MemoryLimitLevel::Policy { ancestor: 0 }
    );
    assert!(requested > admitted);
    assert_eq!(admitted, 1);
    assert!(!called.load(std::sync::atomic::Ordering::Relaxed));
}

#[test]
fn serial_meter_preserves_counter_overflow() {
    use worth_foundational::PartitionIdentity;
    use {
        worth_execution::ExecutionScan, worth_execution::MapKernelFailure,
        worth_execution::MapKernelStop, worth_execution::MapStop, worth_execution::ScanOutcome,
    };
    let policy = worth_foundational::ExecutionRequestPolicy::new(
        worth_foundational::ExecutionPosture::Serial,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        worth_foundational::ExecutionBudget::new(
            std::num::NonZeroUsize::MIN,
            Some(1 << 20),
            Some(u64::MAX),
        ),
    );
    let serial = worth_execution::SerialRequest::from_policy(
        &policy,
        worth_execution::CancellationToken::new(),
        None,
    );
    let identity = PartitionIdentity::new(1);
    let scan = ExecutionScan::try_from_ordered(vec![identity], vec![(identity, ())]).unwrap();
    worth_execution::ExecutionRequest::serial(&serial)
        .in_scope(|lease| {
            let outcome = scan.run(lease, (), 0, 0, 0, 0, |_, _, work| {
                work.checkpoint(u64::MAX)
                    .map_err(MapKernelFailure::<()>::Stop)?;
                work.checkpoint(1).map_err(MapKernelFailure::<()>::Stop)?;
                Ok(((), ()))
            });
            assert!(matches!(
                outcome,
                ScanOutcome::Stopped {
                    reason: MapStop::Failure {
                        cause: MapKernelFailure::Stop(MapKernelStop::WorkCounterOverflow),
                        ..
                    },
                    ..
                }
            ));
        })
        .unwrap();
}

/// docs/how-it-works.md, "One work ceiling": the policy's work ceiling bounds
/// the whole advancement in both placements.
#[test]
fn serial_policy_work_ceiling_bounds_the_request_like_its_lease() {
    let request = worth_execution::SerialRequest::from_policy(
        &one_unit_policy(),
        worth_execution::CancellationToken::new(),
        None,
    );
    let result = worth_execution::ExecutionRequest::serial(&request)
        .in_scope(charged_scan)
        .unwrap();
    assert_eq!(
        result,
        (false, 0),
        "the one-unit policy refuses the first 64-unit checkpoint"
    );
}

#[test]
fn serial_policy_without_a_work_ceiling_meters_every_checkpoint() {
    let request = worth_execution::SerialRequest::from_policy(
        &unbounded_work_policy(),
        worth_execution::CancellationToken::new(),
        None,
    );
    let result = worth_execution::ExecutionRequest::serial(&request)
        .in_scope(charged_scan)
        .unwrap();
    assert_eq!(result, (true, 32 * 64));
}

#[test]
fn serial_computation_declared_work_ceiling_still_bounds_the_same_scan() {
    let absent_work = worth_foundational::ExecutionRequestPolicy::new(
        worth_foundational::ExecutionPosture::Serial,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        worth_foundational::ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(1 << 20), None),
    );
    let request = worth_execution::SerialRequest::from_policy(
        &absent_work,
        worth_execution::CancellationToken::new(),
        None,
    );
    let (result, report) = worth_execution::ExecutionRequest::serial(&request)
        .run(
            worth_execution::ExecutionWorkCeiling::new(Some(100)),
            charged_scan,
        )
        .unwrap();
    assert_eq!(result, (false, 64));
    assert_eq!(report.charged_work(), 64);
    // A declaration never widens the request: the narrower policy ceiling binds.
    let request = worth_execution::SerialRequest::from_policy(
        &one_unit_policy(),
        worth_execution::CancellationToken::new(),
        None,
    );
    let (result, report) = worth_execution::ExecutionRequest::serial(&request)
        .run(
            worth_execution::ExecutionWorkCeiling::new(Some(100)),
            charged_scan,
        )
        .unwrap();
    assert_eq!(result, (false, 0));
    assert_eq!(report.charged_work(), 0);
}
