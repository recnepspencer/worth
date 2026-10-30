#[allow(unused_imports)]
use crate::fork_join_rounds::ordinary_context::*;
pub(crate) use {
    std::num::NonZeroUsize, std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering,
    std::sync::Mutex, std::sync::OnceLock, std::time::Duration, std::time::Instant,
};

pub(crate) use {
    worth_execution::CancellationSource, worth_execution::CancellationToken,
    worth_execution::ExecutionAuthority, worth_execution::ExecutionAuthorityConfig,
    worth_execution::ExecutionForkJoin, worth_execution::ExecutionResourceLease,
    worth_execution::ExecutionRounds, worth_execution::ForkChild, worth_execution::ForkJoinOutcome,
    worth_execution::LeaseRequest, worth_execution::MapDenial, worth_execution::MapKernelFailure,
    worth_execution::MapKernelStop, worth_execution::MapStop, worth_execution::RoundsOutcome,
};
pub(crate) use {
    worth_foundational::DeterminismContract, worth_foundational::ExecutionBudget,
    worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy,
    worth_foundational::PartitionIdentity,
};

#[test]
fn fork_children_must_prove_disjoint_access_before_dispatch() {
    let ids = [
        worth_foundational::PartitionIdentity::new(1),
        worth_foundational::PartitionIdentity::new(2),
    ];
    let child = |identity, read_keys, write_keys| ForkChild {
        identity,
        input: identity.value(),
        read_keys,
        write_keys,
        kernel_scratch_bytes: 0,
        max_result_bytes: 0,
    };
    let overlapping = worth_execution::ExecutionForkJoin::try_from_children(
        ids.to_vec(),
        vec![
            child(ids[0], vec![], vec![7]),
            child(ids[1], vec![], vec![7]),
        ],
    );
    assert!(matches!(
        overlapping,
        Err(worth_execution::MapDenial::WriteSetOverlap { .. })
    ));

    let read_after_write = worth_execution::ExecutionForkJoin::try_from_children(
        ids.to_vec(),
        vec![
            child(ids[0], vec![7], vec![]),
            child(ids[1], vec![], vec![7]),
        ],
    );
    assert!(matches!(
        read_after_write,
        Err(worth_execution::MapDenial::ReadWriteConflict { .. })
    ));
}
