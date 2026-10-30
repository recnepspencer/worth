#[allow(unused_imports)]
pub(crate) use crate::leased_preparation::isolated_context::leased_preparation_denial_causes;
pub(crate) use worth_execution::ExecutionAllocationPolicy as AllocationPolicy;

#[allow(unused_imports)]
pub(crate) use {};

#[allow(unused_imports)]
pub(crate) use crate::leased_preparation::isolated_context::preimage;

pub(crate) use {std::num::NonZeroUsize, std::sync::Mutex, std::sync::OnceLock};

pub(crate) use worth_foundational::facade::AspectValue;
pub(crate) use {
    worth_execution::CancellationToken, worth_execution::ExecutionAuthority,
    worth_execution::ExecutionAuthorityConfig, worth_execution::LeaseRequest,
};
pub(crate) use {
    worth_foundational::DeterminismContract, worth_foundational::ExecutionBudget,
    worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy,
};
pub(crate) use {
    worth_relational::facade::history::BranchId, worth_relational::facade::identity::KindId,
    worth_relational::facade::identity::PartitionId,
    worth_relational::facade::indexes::DerivedIndexBuildRequest,
    worth_relational::facade::indexes::DerivedIndexDefinition,
    worth_relational::facade::indexes::DerivedIndexExecutionDenialKind,
    worth_relational::facade::indexes::DerivedIndexId,
    worth_relational::facade::indexes::DerivedIndexKind,
    worth_relational::facade::mvcc::RelationalTransactionIntent,
    worth_relational::facade::runtime::RelationalRuntimeApi,
    worth_relational::facade::symbols::ClientKey,
    worth_relational::facade::transactions::AspectFieldPatch,
    worth_relational::facade::transactions::BulkEntityCreateIntent,
    worth_relational::facade::transactions::CommitExecutionDenialKind,
    worth_relational::facade::transactions::CreateIntent,
    worth_relational::facade::transactions::EntitySpec,
    worth_relational::facade::transactions::MutationIntent,
    worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause,
    worth_relational::facade::transactions::TransactionCommitError,
    worth_relational::facade::transactions::WorkerIntentBatch,
};

pub(crate) static AUTHORITY: OnceLock<ExecutionAuthority> = std::sync::OnceLock::new();
pub(crate) static TEST_SERIAL: Mutex<()> = std::sync::Mutex::new(());

pub(crate) fn serial() -> std::sync::MutexGuard<'static, ()> {
    TEST_SERIAL
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub(crate) fn authority() -> &'static ExecutionAuthority {
    AUTHORITY.get_or_init(|| {
        worth_execution::ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: std::num::NonZeroUsize::new(4).unwrap(),
            charged_memory_bytes: Some(64 * 1024 * 1024),
        })
        .expect("one authority for this integration binary")
    })
}

pub(crate) fn lease_request(memory: u64, cancellation: CancellationToken) -> LeaseRequest {
    lease_request_with_work(memory, cancellation, 1_000_000)
}

pub(crate) fn lease_request_with_work(
    memory: u64,
    cancellation: CancellationToken,
    work_ceiling: u64,
) -> LeaseRequest {
    LeaseRequest {
        policy: worth_foundational::ExecutionRequestPolicy::new(
            worth_foundational::ExecutionPosture::Automatic,
            worth_foundational::DeterminismContract::CanonicalBitwise,
            worth_foundational::ExecutionBudget::new(
                std::num::NonZeroUsize::new(4).unwrap(),
                Some(memory),
                Some(work_ceiling),
            ),
        ),
        deadline: None,
        cancellation,
    }
}
#[allow(unused_imports)]
pub(crate) use {};

#[allow(unused_imports)]
pub(crate) use worth_testkit_relational::shared_relational_certification_test_execution_policy as test_execution_policy;
