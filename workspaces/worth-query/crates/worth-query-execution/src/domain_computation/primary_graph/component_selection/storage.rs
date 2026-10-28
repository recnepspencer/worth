//! Backing owned by complete material selections, on their original request.
use super::*;
use crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation;
use worth_execution::ExecutionRequest;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

pub(in crate::domain_computation::primary_graph) fn vector<T>(
    execution: &ExecutionRequest<'_, '_>,
    allocation: &SourceFactsAllocation,
    count: usize,
) -> Result<Vec<T>, Cause> {
    allocation.claim_slice::<T>(execution, count)?;
    let mut result = Vec::new();
    result
        .try_reserve_exact(count)
        .map_err(|_| Cause::AllocationFailed)?;
    Ok(result)
}

pub(in crate::domain_computation::primary_graph) fn aggregate<Member>(
    execution: &ExecutionRequest<'_, '_>,
    allocation: &SourceFactsAllocation,
    subject: &str,
) -> Result<(), Cause> {
    // Query wrapper and its copied subject Arc; Native owns the actual body.
    let bytes = size_of::<AggregateObservation<Member>>()
        .checked_add(4 * size_of::<usize>())
        .and_then(|bytes| bytes.checked_add(subject.len()))
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or(Cause::ChargedBytesOverflow)?;
    allocation.claim(execution, bytes)
}

pub(in crate::domain_computation::primary_graph) fn body(
    execution: &ExecutionRequest<'_, '_>,
    allocation: &SourceFactsAllocation,
) -> Result<(), Cause> {
    allocation.claim(
        execution,
        (size_of::<ComponentSelection>() + 2 * size_of::<usize>()) as u64,
    )
}
