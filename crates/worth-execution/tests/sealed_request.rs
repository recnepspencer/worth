use std::{
    num::NonZeroUsize,
    sync::atomic::{AtomicBool, Ordering},
};
use worth_execution::{
    CancellationToken, ExecutionAuthority, ExecutionAuthorityConfig, ExecutionRequest,
    ExecutionWorkCeiling, LeaseDenial, LeaseRequest, MemoryLimitLevel, SerialMemoryBudget,
    SerialRequest, WorkCeilingDenial,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

fn policy(memory: u64) -> ExecutionRequestPolicy {
    ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(NonZeroUsize::MIN, memory, 1_000),
    )
}

#[test]
fn sealed_serial_request_refuses_memory_before_running_the_operation() {
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::from_policy(&policy(1)),
        CancellationToken::new(),
        None,
    );
    let called = AtomicBool::new(false);
    let denial = ExecutionRequest::serial(&serial)
        .run(ExecutionWorkCeiling::new(1_000), |_| {
            called.store(true, Ordering::Relaxed);
        })
        .unwrap_err();
    let WorkCeilingDenial::Admission(LeaseDenial::MemoryExhausted(memory)) = denial else {
        panic!("required serial memory must produce its own resource cause")
    };
    assert_eq!(memory.level, MemoryLimitLevel::Policy { ancestor: 0 });
    assert!(memory.requested > memory.admitted);
    assert_eq!(memory.admitted, 1);
    assert!(!called.load(Ordering::Relaxed));
}

#[test]
fn sealed_leased_request_projects_the_exact_callers_lease() {
    let authority = authority();
    let lease = authority
        .request_lease(LeaseRequest {
            policy: policy(1 << 20),
            cancellation: CancellationToken::new(),
            deadline: None,
        })
        .unwrap();
    let (same, _) = ExecutionRequest::leased(&lease)
        .run(ExecutionWorkCeiling::new(1_000), |carried| {
            std::ptr::eq(
                carried.expect("leased carriage cannot erase its proof"),
                &lease,
            )
        })
        .unwrap();
    assert!(same);
}

#[test]
fn serial_meter_preserves_counter_overflow() {
    use worth_execution::{ExecutionScan, MapKernelFailure, MapKernelStop, MapStop, ScanOutcome};
    use worth_foundational::PartitionIdentity;
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(NonZeroUsize::MIN, 1 << 20, u64::MAX),
    );
    let serial = SerialRequest::from_policy(&policy, CancellationToken::new(), None);
    let identity = PartitionIdentity::new(1);
    let scan = ExecutionScan::try_from_ordered(vec![identity], vec![(identity, ())]).unwrap();
    ExecutionRequest::serial(&serial)
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

fn one_unit_policy() -> ExecutionRequestPolicy {
    ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(NonZeroUsize::MIN, 1 << 20, 1),
    )
}

fn charged_scan(lease: Option<&worth_execution::ExecutionResourceLease<'_>>) -> (bool, u64) {
    use worth_execution::{ExecutionScan, MapKernelFailure, MapKernelStop, MapStop, ScanOutcome};
    use worth_foundational::PartitionIdentity;
    let ids: Vec<_> = (1..=32).map(PartitionIdentity::new).collect();
    let scan =
        ExecutionScan::try_from_ordered(ids.clone(), ids.into_iter().map(|id| (id, ())).collect())
            .unwrap();
    match scan.run(lease, (), 0, 0, 0, 0, |_, _, work| {
        work.checkpoint(64).map_err(MapKernelFailure::<()>::Stop)?;
        Ok(((), ()))
    }) {
        ScanOutcome::Complete { report, .. } => (true, report.charged_work()),
        ScanOutcome::Stopped { reason, report, .. } => {
            assert!(
                matches!(
                    reason,
                    MapStop::WorkExhausted { .. }
                        | MapStop::Failure {
                            cause: MapKernelFailure::Stop(MapKernelStop::WorkCeiling),
                            ..
                        }
                ),
                "only the work ceiling may refuse this funded scan: {reason:?}"
            );
            (false, report.charged_work())
        }
    }
}

#[test]
fn serial_policy_work_ceiling_does_not_bound_the_request() {
    let request = SerialRequest::from_policy(&one_unit_policy(), CancellationToken::new(), None);
    let result = ExecutionRequest::serial(&request)
        .in_scope(charged_scan)
        .unwrap();
    assert_eq!(
        result,
        (true, 32 * 64),
        "all declared checkpoints are metered beyond one unit"
    );
}

#[test]
fn leased_policy_work_ceiling_still_bounds_the_same_scan() {
    let authority = authority();
    let lease = authority
        .request_lease(LeaseRequest {
            policy: one_unit_policy(),
            cancellation: CancellationToken::new(),
            deadline: None,
        })
        .unwrap();
    assert_eq!(
        ExecutionRequest::leased(&lease)
            .in_scope(charged_scan)
            .unwrap(),
        (false, 0)
    );
}

#[test]
fn serial_computation_declared_work_ceiling_still_bounds_the_same_scan() {
    let request = SerialRequest::from_policy(&one_unit_policy(), CancellationToken::new(), None);
    let (result, report) = ExecutionRequest::serial(&request)
        .run(ExecutionWorkCeiling::new(100), charged_scan)
        .unwrap();
    assert_eq!(result, (false, 64));
    assert_eq!(report.charged_work(), 64);
}

fn authority() -> &'static ExecutionAuthority {
    static OWNER: std::sync::OnceLock<ExecutionAuthority> = std::sync::OnceLock::new();
    OWNER.get_or_init(|| {
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: NonZeroUsize::MIN,
            charged_memory_bytes: Some(1 << 20),
        })
        .unwrap()
    })
}
