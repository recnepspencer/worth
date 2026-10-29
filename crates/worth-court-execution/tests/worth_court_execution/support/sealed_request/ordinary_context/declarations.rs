pub(crate) use {std::num::NonZeroUsize, std::sync::atomic::AtomicBool, std::sync::atomic::Ordering};
pub(crate) use {worth_execution::CancellationToken, worth_execution::ExecutionAuthority, worth_execution::ExecutionAuthorityConfig, worth_execution::ExecutionRequest, worth_execution::ExecutionWorkCeiling, worth_execution::LeaseDenial, worth_execution::LeaseRequest, worth_execution::MemoryLimitLevel, worth_execution::SerialMemoryBudget, worth_execution::SerialRequest, worth_execution::WorkCeilingDenial};
pub(crate) use {worth_foundational::DeterminismContract, worth_foundational::ExecutionBudget, worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy};

pub(crate) fn policy(memory: u64) -> ExecutionRequestPolicy {
    worth_foundational::ExecutionRequestPolicy::new(
        worth_foundational::ExecutionPosture::Serial,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        worth_foundational::ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(memory), Some(1_000)),
    )
}

pub(crate) fn one_unit_policy() -> ExecutionRequestPolicy {
    worth_foundational::ExecutionRequestPolicy::new(
        worth_foundational::ExecutionPosture::Serial,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        worth_foundational::ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(1 << 20), Some(1)),
    )
}

pub(crate) fn unbounded_work_policy() -> ExecutionRequestPolicy {
    worth_foundational::ExecutionRequestPolicy::new(
        worth_foundational::ExecutionPosture::Serial,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        worth_foundational::ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(1 << 20), None),
    )
}

pub(crate) fn charged_scan(
    lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
) -> (bool, u64) {
    use {worth_execution::ExecutionScan, worth_execution::MapKernelFailure, worth_execution::MapKernelStop, worth_execution::MapStop, worth_execution::ScanOutcome};
    use {worth_foundational::PartitionIdentity};
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

pub(crate) fn authority() -> &'static ExecutionAuthority {
    static OWNER: std::sync::OnceLock<ExecutionAuthority> = std::sync::OnceLock::new();
    OWNER.get_or_init(|| {
        worth_execution::ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: std::num::NonZeroUsize::MIN,
            charged_memory_bytes: Some(1 << 20),
        })
        .unwrap()
    })
}