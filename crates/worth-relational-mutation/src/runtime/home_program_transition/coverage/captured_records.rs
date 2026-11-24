//! The authoritative records and funded batch footprint of the admitted Home scope.
use crate::contracts::mvcc::transaction::staging::{
    RelationalTransactionStagingDenial, RelationalTransactionStagingDenialOwnerPort as _,
};
use crate::contracts::transactions::data::outcomes::transaction_commit_error::{
    TransactionCommitError, TransactionCommitErrorOwnerPort as _,
};
use crate::contracts::transactions::data::primitives::RecordRef;
use worth_execution::{
    ExecutionArray, ExecutionMemoryReservation, ExecutionOrderedMap, ExecutionOrderedMapDenial,
    ExecutionRequest, ExecutionRequestArrayBuilder, ExecutionRequestArrayDenial,
};

#[derive(Debug)]
pub struct RelationalHomeTargetCoverage {
    pub authored_batches: usize,
    pub rejudgment_body: worth_foundational::facade::HomeProgramTransitionBodyDigest,
    pub records: ExecutionArray<RecordRef>,
    pub _batch_memory: Option<ExecutionMemoryReservation>,
}

impl RelationalHomeTargetCoverage {
    pub fn include_changed(
        &self,
        request: &ExecutionRequest<'_, '_>,
        changed: &[RecordRef],
    ) -> Result<ExecutionArray<RecordRef>, TransactionCommitError> {
        let mut records = ExecutionOrderedMap::default();
        for record in self.records.iter().chain(changed) {
            records
                .insert_cloned_native_for_request(record, &(), *request)
                .map_err(ordered)?;
        }
        let mut captured =
            ExecutionRequestArrayBuilder::allocate(*request, records.len()).map_err(array)?;
        for (record, ()) in records.iter() {
            captured.push(record.clone()).map_err(array)?;
        }
        captured.seal().map_err(array)
    }
}

pub fn staging(cause: RelationalTransactionStagingDenial) -> TransactionCommitError {
    crate::contracts::execution::denial::allocation_commit_error(TransactionCommitError::conflict(
        cause.into_conflict(),
    ))
}
pub fn overflow() -> TransactionCommitError {
    staging(RelationalTransactionStagingDenial::CardinalityOverflow)
}
pub fn array(cause: ExecutionRequestArrayDenial) -> TransactionCommitError {
    match cause {
        ExecutionRequestArrayDenial::Request(cause) => {
            TransactionCommitError::execution(cause.into())
        }
        ExecutionRequestArrayDenial::Allocation(cause) => {
            staging(RelationalTransactionStagingDenial::AllocationDenied(cause))
        }
    }
}
pub fn ordered(cause: ExecutionOrderedMapDenial) -> TransactionCommitError {
    match cause {
        ExecutionOrderedMapDenial::Request(cause) => {
            TransactionCommitError::execution(cause.into())
        }
        ExecutionOrderedMapDenial::Allocation(cause) => {
            staging(RelationalTransactionStagingDenial::AllocationDenied(cause))
        }
        ExecutionOrderedMapDenial::CapacityOverflow => overflow(),
    }
}
