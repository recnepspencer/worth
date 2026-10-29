#[path = "leased_preparation/companion_read.rs"]
pub(crate) mod companion_read;

#[allow(unused_imports)]
pub(crate) use crate::serial::leased_preparation::leased_preparation_denial_causes;
pub(crate) use worth_execution::ExecutionAllocationPolicy as AllocationPolicy;

#[path = "../fixtures/leased_preparation/index_budget.rs"]
pub(crate) mod leased_preparation_index_budget;

#[allow(unused_imports)]
pub(crate) use crate::serial::leased_preparation::leased_preparation_index_request_memory;

#[allow(unused_imports)]
pub(crate) use crate::serial::leased_preparation::preimage;

#[path = "../../../examples/support.rs"]
pub(crate) mod support;

pub(crate) use std::{
    num::NonZeroUsize,
    sync::{Mutex, OnceLock},
};

pub(crate) use worth_execution::{
    CancellationToken, ExecutionAuthority, ExecutionAuthorityConfig, LeaseRequest,
};
pub(crate) use worth_foundational::facade::AspectValue;
pub(crate) use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
pub(crate) use worth_relational::facade::{
    history::BranchId,
    identity::{KindId, PartitionId},
    indexes::{
        DerivedIndexBuildRequest, DerivedIndexDefinition, DerivedIndexExecutionDenialKind,
        DerivedIndexId, DerivedIndexKind,
    },
    mvcc::RelationalTransactionIntent,
    runtime::RelationalRuntimeApi,
    symbols::ClientKey,
    transactions::{
        AspectFieldPatch, BulkEntityCreateIntent, CommitExecutionDenialKind, CreateIntent,
        EntitySpec, MutationIntent, RelationalExecutionDenialCause as Cause,
        TransactionCommitError, WorkerIntentBatch,
    },
};

pub(crate) static AUTHORITY: OnceLock<ExecutionAuthority> = OnceLock::new();
pub(crate) static TEST_SERIAL: Mutex<()> = Mutex::new(());

pub(crate) fn serial() -> std::sync::MutexGuard<'static, ()> {
    TEST_SERIAL
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub(crate) fn authority() -> &'static ExecutionAuthority {
    AUTHORITY.get_or_init(|| {
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: NonZeroUsize::new(4).unwrap(),
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
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Automatic,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(
                NonZeroUsize::new(4).unwrap(),
                Some(memory),
                Some(work_ceiling),
            ),
        ),
        deadline: None,
        cancellation,
    }
}
#[allow(unused_imports)]
pub(crate) use crate::serial::leased_preparation::bulk_creation;

#[allow(unused_imports)]
pub(crate) use crate::shared_relational_certification_test_execution_policy as test_execution_policy;
