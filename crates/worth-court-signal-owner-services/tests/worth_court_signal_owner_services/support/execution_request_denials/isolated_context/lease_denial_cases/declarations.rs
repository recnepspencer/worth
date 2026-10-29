// Authority-owned admission cases shared by downstream conversion tests.
//! Authority-owned admission cases shared by downstream conversion tests.
pub(crate) use {std::num::NonZeroUsize, std::sync::OnceLock};
pub(crate) use {worth_execution::CancellationToken, worth_execution::ExecutionAuthority, worth_execution::ExecutionAuthorityConfig, worth_execution::ExecutionResourceLease, worth_execution::ExecutionScan, worth_execution::ExecutionWorkCeiling, worth_execution::LeaseDenial, worth_execution::LeaseRequest, worth_execution::MapKernelFailure, worth_execution::MapStop, worth_execution::ScanOutcome};
pub(crate) use {worth_foundational::DeterminismContract, worth_foundational::EquivalenceContractId, worth_foundational::ExecutionBudget, worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy, worth_foundational::PartitionIdentity};

pub(crate) fn authority() -> &'static ExecutionAuthority {
    static OWNER: OnceLock<ExecutionAuthority> = std::sync::OnceLock::new();
    OWNER.get_or_init(|| {
        worth_execution::ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: std::num::NonZeroUsize::new(4).unwrap(),
            charged_memory_bytes: Some(64 << 20),
        })
        .unwrap()
    })
}
pub(crate) fn request(workers: usize, memory: u64, work: u64) -> LeaseRequest {
    LeaseRequest {
        policy: worth_foundational::ExecutionRequestPolicy::new(
            worth_foundational::ExecutionPosture::Automatic,
            worth_foundational::DeterminismContract::CanonicalBitwise,
            worth_foundational::ExecutionBudget::new(
                std::num::NonZeroUsize::new(workers).unwrap(),
                Some(memory),
                Some(work),
            ),
        ),
        cancellation: worth_execution::CancellationToken::new(),
        deadline: None,
    }
}
pub(crate) fn lease() -> ExecutionResourceLease<'static> {
    authority()
        .request_lease(request(1, 2 << 20, 1_000))
        .unwrap()
}
pub(crate) fn scan() -> ExecutionScan<()> {
    let key = worth_foundational::PartitionIdentity::new(1);
    worth_execution::ExecutionScan::try_from_ordered(vec![key], vec![(key, ())]).unwrap()
}
pub(crate) fn refused(lease: Option<&ExecutionResourceLease<'_>>, state: u64, scratch: u64) -> LeaseDenial {
    let result = scan().run(lease, (), state, 0, 0, scratch, |_, _, _| {
        Ok::<_, MapKernelFailure<()>>(((), ()))
    });
    match result {
        worth_execution::ScanOutcome::Stopped {
            reason: worth_execution::MapStop::Admission(denial),
            ..
        } => denial,
        _ => panic!("operation must be refused at execution admission"),
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Cause {
    Workers,
    MemoryLimit,
    Work,
    Memory,
    Overflow,
    Nested,
    Equivalence,
    MissingScope,
}
pub(crate) const CASES: [Cause; 8] = [
    crate::execution_request_denials::isolated_context::lease_denial_cases::Cause::Workers,
    crate::execution_request_denials::isolated_context::lease_denial_cases::Cause::MemoryLimit,
    crate::execution_request_denials::isolated_context::lease_denial_cases::Cause::Work,
    crate::execution_request_denials::isolated_context::lease_denial_cases::Cause::Memory,
    crate::execution_request_denials::isolated_context::lease_denial_cases::Cause::Overflow,
    crate::execution_request_denials::isolated_context::lease_denial_cases::Cause::Nested,
    crate::execution_request_denials::isolated_context::lease_denial_cases::Cause::Equivalence,
    crate::execution_request_denials::isolated_context::lease_denial_cases::Cause::MissingScope,
];

pub(crate) fn denial(cause: Cause) -> LeaseDenial {
    let parent = lease();
    match cause {
        crate::execution_request_denials::isolated_context::lease_denial_cases::Cause::Workers => parent.child(request(2, 2 << 20, 1000)).unwrap_err(),
        crate::execution_request_denials::isolated_context::lease_denial_cases::Cause::MemoryLimit => parent.child(request(1, 4 << 20, 1000)).unwrap_err(),
        crate::execution_request_denials::isolated_context::lease_denial_cases::Cause::Work => parent.child(request(1, 2 << 20, 1001)).unwrap_err(),
        crate::execution_request_denials::isolated_context::lease_denial_cases::Cause::Memory => refused(Some(&parent), 0, 4 << 20),
        crate::execution_request_denials::isolated_context::lease_denial_cases::Cause::Overflow => refused(Some(&parent), u64::MAX, 0),
        crate::execution_request_denials::isolated_context::lease_denial_cases::Cause::Nested => {
            worth_execution::ExecutionWorkCeiling::new(Some(1000))
                .run(&parent, || refused(None, 0, 0))
                .unwrap()
                .0
        }
        crate::execution_request_denials::isolated_context::lease_denial_cases::Cause::MissingScope => {
            worth_execution::ExecutionMemoryReservation::reserve_in_scope(None, 1).unwrap_err()
        }
        crate::execution_request_denials::isolated_context::lease_denial_cases::Cause::Equivalence => {
            let mut child = request(1, 2 << 20, 1000);
            child.policy = worth_foundational::ExecutionRequestPolicy::new(
                worth_foundational::ExecutionPosture::Automatic,
                worth_foundational::DeterminismContract::ContractEquivalent(worth_foundational::EquivalenceContractId::new(91)),
                child
                    .policy
                    .accounting()
                    .expect("explicitly accounted fixture"),
            );
            parent.child(child).unwrap_err()
        }
    }
}