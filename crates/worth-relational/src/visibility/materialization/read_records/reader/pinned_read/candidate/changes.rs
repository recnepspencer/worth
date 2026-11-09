//! Retain aggregate and field-entry custody for one immutable applied image.
use super::*;
use worth_execution::ExecutionMemoryReservation;

#[derive(Debug)]
pub(in crate::visibility::materialization::read_records::reader) struct AppliedReadChanges {
    pub(super) before_root: u64,
    pub(in crate::visibility::materialization::read_records::reader) aggregate:
        Option<crate::aggregate_identity::AggregateCandidateRead>,
    pub(crate) fields: Arc<crate::indexes::AppliedFieldImage>,
    _memory: ExecutionMemoryReservation,
}

pub(super) fn capture(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    before_root: u64,
    aggregate: Option<crate::aggregate_identity::AggregateCandidateRead>,
    previous: Option<&AppliedReadChanges>,
    fields: Arc<crate::indexes::AppliedFieldImage>,
) -> Result<Arc<AppliedReadChanges>, TransactionCommitError> {
    if previous.is_some_and(|prior| prior.before_root != before_root) {
        return Err(crate::mvcc::origin_denial(
            "applied frontier has a foreign before root",
        ));
    }
    let memory = ExecutionMemoryReservation::reserve_in_scope(
        crate::authority::commit::pipeline::request_lease(execution),
        (std::mem::size_of::<AppliedReadChanges>() + 2 * std::mem::size_of::<usize>()) as u64,
    )
    .map_err(|cause| request_stop(crate::execution::denial::lease_denial(cause)))?;
    Ok(Arc::new(AppliedReadChanges {
        before_root,
        fields,
        aggregate,
        _memory: memory,
    }))
}
