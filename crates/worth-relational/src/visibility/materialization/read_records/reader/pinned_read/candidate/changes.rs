//! Retain only the actual applied entity delta, never record payloads.
use super::*;
use crate::{identity::data::EntityId, transactions::data::RecordRef};
use worth_execution::ExecutionMemoryReservation;

#[derive(Debug)]
pub(in crate::visibility::materialization::read_records::reader) struct AppliedReadChanges {
    pub(super) before_root: u64,
    pub(in crate::visibility::materialization::read_records::reader) aggregate:
        Option<crate::aggregate_identity::AggregateCandidateRead>,
    pub(super) entities: Vec<EntityId>,
    _memory: ExecutionMemoryReservation,
}

pub(super) fn capture(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    before_root: u64,
    records: &[RecordRef],
    aggregate: Option<crate::aggregate_identity::AggregateCandidateRead>,
) -> Result<Arc<AppliedReadChanges>, TransactionCommitError> {
    let result = execution
        .run_native_with_checkpoint(|context| {
            context
                .checkpoint(records.len() as u64)
                .map_err(request_stop)?;
            let count = records
                .iter()
                .filter(|r| matches!(r, RecordRef::Entity(_)))
                .count();
            let bytes = count
                .checked_mul(std::mem::size_of::<EntityId>())
                .and_then(|n| {
                    n.checked_add(
                        std::mem::size_of::<AppliedReadChanges>()
                            + 2 * std::mem::size_of::<usize>(),
                    )
                })
                .ok_or_else(|| {
                    request_stop(
                        crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow,
                    )
                })?;
            let memory = ExecutionMemoryReservation::reserve_in_scope(
                crate::authority::commit::pipeline::request_lease(execution),
                bytes as u64,
            )
            .map_err(|cause| request_stop(crate::execution::denial::lease_denial(cause)))?;
            let mut entities = Vec::new();
            entities.try_reserve_exact(count).map_err(|_| {
                request_stop(crate::execution::RelationalExecutionDenialCause::AllocationFailed)
            })?;
            entities.extend(records.iter().filter_map(|r| match r {
                RecordRef::Entity(id) => Some(*id),
                _ => None,
            }));
            let comparisons =
                count.saturating_mul((usize::BITS - count.max(1).leading_zeros()) as usize);
            context
                .checkpoint(comparisons as u64)
                .map_err(request_stop)?;
            entities.sort_unstable();
            entities.dedup();
            Ok(Arc::new(AppliedReadChanges {
                before_root,
                aggregate,
                entities,
                _memory: memory,
            }))
        })
        .map_err(request_stop)?;
    result.0
}
