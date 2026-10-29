pub(crate) use {std::num::NonZeroUsize, std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering};

pub(crate) use {worth_execution::compare_canonical_values, worth_execution::CancellationSource, worth_execution::CancellationToken, worth_execution::ConstructionDenial, worth_execution::EquivalencePredicate, worth_execution::ExecutionAuthority, worth_execution::ExecutionAuthorityConfig, worth_execution::ExecutionMap, worth_execution::LeaseDenial, worth_execution::LeaseRequest, worth_execution::MapKernelFailure, worth_execution::MapOutcome, worth_execution::MapPartition, worth_execution::OracleMismatch};
pub(crate) use {worth_foundational::DeterminismContract, worth_foundational::EquivalenceContractId, worth_foundational::ExecutionBudget, worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy, worth_foundational::PartitionIdentity};

pub(crate) fn request(determinism: DeterminismContract) -> LeaseRequest {
    LeaseRequest {
        policy: worth_foundational::ExecutionRequestPolicy::new(
            worth_foundational::ExecutionPosture::Serial,
            determinism,
            worth_foundational::ExecutionBudget::new(std::num::NonZeroUsize::new(1).unwrap(), Some(4096), Some(10)),
        ),
        deadline: None,
        cancellation: worth_execution::CancellationToken::new(),
    }
}

pub(crate) fn predicate(id: u64, identity: u8) -> EquivalencePredicate {
    worth_execution::EquivalencePredicate::new(
        worth_foundational::EquivalenceContractId::new(id),
        [identity; 32],
        |left, right| left == right,
    )
}