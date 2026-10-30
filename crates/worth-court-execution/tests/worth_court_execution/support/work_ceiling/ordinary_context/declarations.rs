#[allow(unused_imports)]
pub(crate) use crate::work_ceiling::isolated_context::retained_run;

pub(crate) use {std::num::NonZeroUsize, std::panic::catch_unwind, std::panic::AssertUnwindSafe};

pub(crate) use {
    worth_execution::CancellationSource, worth_execution::CancellationToken,
    worth_execution::ExecutionAuthority, worth_execution::ExecutionAuthorityConfig,
    worth_execution::ExecutionMap, worth_execution::ExecutionResourceLease,
    worth_execution::ExecutionWorkCeiling, worth_execution::LeaseRequest,
    worth_execution::MapKernelFailure, worth_execution::MapKernelStop,
    worth_execution::MapPartition, worth_execution::MapStop, worth_execution::ReduceInputDenial,
    worth_execution::ReductionRunStop, worth_execution::SerialRequest,
    worth_execution::WorkCeilingDenial,
};
pub(crate) use {
    worth_foundational::DeterminismContract, worth_foundational::ExecutionBudget,
    worth_foundational::ExecutionPosture, worth_foundational::ExecutionReport,
    worth_foundational::ExecutionRequestPolicy, worth_foundational::PartitionIdentity,
};

pub(crate) const PARTITION_WORK: u64 = 3;

pub(crate) fn map(count: u64) -> ExecutionMap<u64, u64> {
    let ids: Vec<_> = (1..=count)
        .map(worth_foundational::PartitionIdentity::new)
        .collect();
    let partitions = ids
        .iter()
        .map(|identity| MapPartition {
            identity: *identity,
            value: identity.value(),
            read_keys: Vec::new(),
            write_keys: Vec::new(),
            kernel_scratch_bytes: 0,
            max_result_bytes: 0,
        })
        .collect();
    worth_execution::ExecutionMap::try_from_declared_partitions(ids, partitions).unwrap()
}

pub(crate) type Reduced = Result<(u64, ExecutionReport), ReduceInputDenial<()>>;

pub(crate) fn reduce(
    lease: Option<&ExecutionResourceLease<'_>>,
    ceiling: u64,
    count: u64,
) -> (Reduced, ExecutionReport) {
    let input = map(count);
    let computation = || {
        input
            .run_reduce(
                lease,
                |value, context| {
                    context.checkpoint(PARTITION_WORK)?;
                    Ok::<_, MapKernelFailure<()>>(*value)
                },
                0_u64,
                |left: &u64, right: &u64| left + right,
                8,
                0,
            )
            .map(|(tree, report, _)| (*tree.result(), report))
    };
    let ceiling = worth_execution::ExecutionWorkCeiling::new(Some(ceiling));
    match lease {
        Some(lease) => ceiling.run(lease, computation),
        None => ceiling.run_serial(&funded_serial_request(), computation),
    }
    .expect("the computation ran")
}

pub(crate) fn funded_serial_request() -> SerialRequest {
    worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::from_policy(
            &worth_foundational::ExecutionRequestPolicy::new(
                worth_foundational::ExecutionPosture::Serial,
                worth_foundational::DeterminismContract::CanonicalBitwise,
                worth_foundational::ExecutionBudget::new(
                    std::num::NonZeroUsize::MIN,
                    None,
                    Some(10_000),
                ),
            ),
        ),
        worth_execution::CancellationToken::new(),
        None,
    )
}

pub(crate) fn exhausted_at(reduced: &Reduced) -> Option<u64> {
    match reduced {
        Err(worth_execution::ReduceInputDenial::MapStopped {
            reason: worth_execution::MapStop::WorkExhausted { identity },
            ..
        }) => Some(identity.value()),
        _ => None,
    }
}

pub(crate) fn authority() -> &'static ExecutionAuthority {
    static OWNER: std::sync::OnceLock<ExecutionAuthority> = std::sync::OnceLock::new();
    OWNER.get_or_init(|| {
        worth_execution::ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: std::num::NonZeroUsize::new(2).unwrap(),
            charged_memory_bytes: Some(1 << 20),
        })
        .unwrap()
    })
}
