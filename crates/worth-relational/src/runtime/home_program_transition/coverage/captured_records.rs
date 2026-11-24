//! The authoritative records and funded batch footprint of the admitted Home scope.
use worth_relational_contracts::internal_port::mvcc::transaction::staging::RelationalTransactionStagingDenialOwnerPort as _;
use worth_relational_contracts::internal_port::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;
use crate::mvcc::transaction::staging::RelationalTransactionStagingDenial;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::transactions::data::primitives::RecordRef;
use worth_execution::{
    ExecutionArray, ExecutionMemoryReservation, ExecutionOrderedMap, ExecutionOrderedMapDenial,
    ExecutionRequest, ExecutionRequestArrayBuilder, ExecutionRequestArrayDenial,
};

#[derive(Debug)]
pub(crate) struct RelationalHomeTargetCoverage {
    pub(crate) authored_batches: usize,
    pub(crate) rejudgment_body: worth_foundational::facade::HomeProgramTransitionBodyDigest,
    pub(crate) records: ExecutionArray<RecordRef>,
    pub(super) _batch_memory: Option<ExecutionMemoryReservation>,
}

impl RelationalHomeTargetCoverage {
    pub(crate) fn include_changed(
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

pub(super) fn staging(cause: RelationalTransactionStagingDenial) -> TransactionCommitError {
    crate::execution::allocation_commit_error(TransactionCommitError::conflict(
        cause.into_conflict(),
    ))
}
pub(super) fn overflow() -> TransactionCommitError {
    staging(RelationalTransactionStagingDenial::CardinalityOverflow)
}
pub(super) fn array(cause: ExecutionRequestArrayDenial) -> TransactionCommitError {
    match cause {
        ExecutionRequestArrayDenial::Request(cause) => {
            TransactionCommitError::execution(cause.into())
        }
        ExecutionRequestArrayDenial::Allocation(cause) => {
            staging(RelationalTransactionStagingDenial::AllocationDenied(cause))
        }
    }
}
pub(super) fn ordered(cause: ExecutionOrderedMapDenial) -> TransactionCommitError {
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
