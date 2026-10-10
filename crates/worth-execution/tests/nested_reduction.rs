//! Nested reducer ownership through the public request and map doors.
use std::{
    num::NonZeroUsize,
    sync::{Mutex, OnceLock},
};
use worth_execution::*;
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    PartitionIdentity,
};

const MEMORY: u64 = 1 << 20;
static TEST_LOCK: Mutex<()> = Mutex::new(());
static AUTHORITY: OnceLock<ExecutionAuthority> = OnceLock::new();
fn request(width: usize) -> ExecutionResourceLease<'static> {
    AUTHORITY
        .get_or_init(|| {
            ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
                max_workers: NonZeroUsize::new(4).unwrap(),
                charged_memory_bytes: Some(MEMORY),
            })
            .unwrap()
        })
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::new(width).unwrap(), MEMORY, 100),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .unwrap()
}
fn singleton() -> ExecutionMap<u64, u64> {
    ExecutionMap::from_keyless_partitions(std::collections::BTreeMap::from([(
        PartitionIdentity::new(1),
        KeylessPartition {
            value: 1_u64,
            kernel_scratch_bytes: 0,
            max_result_bytes: 0,
        },
    )]))
    .unwrap()
}

#[test]
fn reducer_panic_is_taken_as_data_at_every_request_backing() {
    let _guard = TEST_LOCK.lock().unwrap();
    for width in [0, 1, 2, 4] {
        let serial = SerialRequest::from_memory(
            SerialMemoryBudget::new(MEMORY),
            CancellationToken::new(),
            None,
        );
        let leased = (width != 0).then(|| request(width));
        let backing = leased
            .as_ref()
            .map_or(ExecutionRequest::serial(&serial), ExecutionRequest::leased);
        let outer = backing
            .in_scope(|lease| {
                singleton().run(lease, |_, context| {
                    let child = singleton().run_reduce(
                        lease,
                        |value, context| {
                            context.checkpoint(1)?;
                            Ok::<_, MapKernelFailure<()>>(*value)
                        },
                        0_u64,
                        |_, _| -> u64 { panic!("declared reducer panic") },
                        8,
                        0,
                    );
                    assert!(
                        matches!(
                            child,
                            Err(ReduceInputDenial::ReductionStopped {
                                failure: ReductionRunFailure {
                                    reason: ReductionRunStop::Panic,
                                    ..
                                },
                                ..
                            })
                        ),
                        "width {width}, child: {:?}",
                        child.as_ref().err()
                    );
                    assert_eq!(
                        context.take_nested_computation_stop(),
                        Some(NestedComputationStop::Panic)
                    );
                    Ok::<_, MapKernelFailure<()>>(1_u64)
                })
            })
            .unwrap();
        assert!(matches!(outer, MapOutcome::Complete { ref values, .. } if *values == vec![1]));
        assert!(outer.report().charged_work() > 0);
    }
}

#[test]
fn reducer_admission_overflow_is_request_owned_at_every_backing() {
    let _guard = TEST_LOCK.lock().unwrap();
    for width in [0, 1, 2, 4] {
        let serial = SerialRequest::from_memory(
            SerialMemoryBudget::new(MEMORY),
            CancellationToken::new(),
            None,
        );
        let leased = (width != 0).then(|| request(width));
        let backing = leased
            .as_ref()
            .map_or(ExecutionRequest::serial(&serial), ExecutionRequest::leased);
        let outer = backing
            .in_scope(|lease| {
                singleton().run(lease, |_, context| {
                    context.checkpoint(1)?;
                    // One leaf cannot declare u64::MAX bytes plus its owned node and
                    // temporary values: the checked memory declaration must overflow.
                    let child = singleton().run_reduce(
                        lease,
                        |value, _| Ok::<_, MapKernelFailure<()>>(*value),
                        0_u64,
                        |left, right| left + right,
                        u64::MAX,
                        0,
                    );
                    assert!(matches!(
                        child,
                        Err(ReduceInputDenial::ScopeAdmission {
                            denial: LeaseDenial::ChargedBytesOverflow,
                            ..
                        })
                    ));
                    assert_eq!(context.take_nested_computation_stop(), None);
                    Ok::<_, MapKernelFailure<()>>(1_u64)
                })
            })
            .unwrap();
        assert!(matches!(
            outer,
            MapOutcome::Stopped {
                reason: MapStop::Failure {
                    cause: MapKernelFailure::Stop(MapKernelStop::NestedStopped),
                    ..
                },
                ..
            }
        ));
        assert_eq!(outer.report().charged_work(), 1);
        assert_eq!(outer.report().charged_span(), 1);
    }
}
