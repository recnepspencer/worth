pub(crate) use std::{
    num::NonZeroUsize,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex, OnceLock,
    },
    time::{Duration, Instant},
};

pub(crate) use worth_execution::{
    CancellationSource, CancellationToken, ExecutionAuthority, ExecutionAuthorityConfig,
    ExecutionForkJoin, ExecutionResourceLease, ExecutionRounds, ForkChild, ForkJoinOutcome,
    LeaseRequest, MapDenial, MapKernelFailure, MapKernelStop, MapStop, RoundsOutcome,
};
pub(crate) use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    PartitionIdentity,
};

pub(crate) static AUTHORITY: OnceLock<ExecutionAuthority> = OnceLock::new();
pub(crate) static TEST_LOCK: Mutex<()> = Mutex::new(());

pub(crate) fn authority() -> &'static ExecutionAuthority {
    AUTHORITY.get_or_init(|| {
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: NonZeroUsize::new(4).unwrap(),
            charged_memory_bytes: Some(65_536),
        })
        .unwrap()
    })
}

pub(crate) fn lease(
    posture: ExecutionPosture,
    work: u64,
    cancellation: CancellationToken,
) -> ExecutionResourceLease<'static> {
    authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                posture,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::new(4).unwrap(), None, Some(work)),
            ),
            deadline: None,
            cancellation,
        })
        .unwrap()
}

pub(crate) fn fork() -> ExecutionForkJoin<u64, u64> {
    let identities = [PartitionIdentity::new(1), PartitionIdentity::new(2)];
    ExecutionForkJoin::try_from_children(
        identities.to_vec(),
        identities
            .into_iter()
            .map(|identity| ForkChild {
                identity,
                input: identity.value(),
                read_keys: Vec::new(),
                write_keys: Vec::new(),
                kernel_scratch_bytes: 0,
                max_result_bytes: 0,
            })
            .collect(),
    )
    .unwrap()
}

#[test]
fn fork_children_must_prove_disjoint_access_before_dispatch() {
    let ids = [PartitionIdentity::new(1), PartitionIdentity::new(2)];
    let child = |identity, read_keys, write_keys| ForkChild {
        identity,
        input: identity.value(),
        read_keys,
        write_keys,
        kernel_scratch_bytes: 0,
        max_result_bytes: 0,
    };
    let overlapping = ExecutionForkJoin::try_from_children(
        ids.to_vec(),
        vec![
            child(ids[0], vec![], vec![7]),
            child(ids[1], vec![], vec![7]),
        ],
    );
    assert!(matches!(
        overlapping,
        Err(MapDenial::WriteSetOverlap { .. })
    ));

    let read_after_write = ExecutionForkJoin::try_from_children(
        ids.to_vec(),
        vec![
            child(ids[0], vec![7], vec![]),
            child(ids[1], vec![], vec![7]),
        ],
    );
    assert!(matches!(
        read_after_write,
        Err(MapDenial::ReadWriteConflict { .. })
    ));
}

pub(crate) fn recursive(
    lease: &ExecutionResourceLease<'_>,
    depth: u32,
) -> ForkJoinOutcome<u64, ()> {
    fork().run(Some(lease), |value, context| {
        context.checkpoint(1)?;
        if depth == 0 {
            return Ok::<_, MapKernelFailure<()>>(*value);
        }
        match recursive(lease, depth - 1) {
            ForkJoinOutcome::Complete { values, .. } => {
                Ok(*value + values.into_iter().sum::<u64>())
            }
            _ => Err(MapKernelFailure::Stop(MapKernelStop::NestedStopped)),
        }
    })
}
