//! Retain aggregate and field-entry custody for one immutable applied image.
use super::*;
use worth_execution::ExecutionMemoryReservation;
use worth_relational_contracts::internal_port as contracts;

#[derive(Debug)]
pub struct AppliedReadChanges {
    pub(super) before_root: u64,
    pub(in crate::internal_port::visibility::materialization::read_records::reader) aggregate:
        Option<worth_relational_mutation::internal_port::aggregate_identity::candidate_read::AggregateCandidateRead>,
    pub(crate) fields: Arc<worth_relational_indexes::internal_port::indexes::authority::maintenance::applied_field::AppliedFieldImage>,
    _memory: ExecutionMemoryReservation,
}

pub(super) fn capture(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    before_root: u64,
    aggregate: Option<worth_relational_mutation::internal_port::aggregate_identity::candidate_read::AggregateCandidateRead>,
    previous: Option<&AppliedReadChanges>,
    fields: Arc<worth_relational_indexes::internal_port::indexes::authority::maintenance::applied_field::AppliedFieldImage>,
) -> Result<Arc<AppliedReadChanges>, TransactionCommitError> {
    if previous.is_some_and(|prior| prior.before_root != before_root) {
        return Err(
            contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                "applied frontier has a foreign before root",
            ),
        );
    }
    let memory = ExecutionMemoryReservation::reserve_in_scope(
        contracts::execution::request_custody::request_lease(execution),
        (std::mem::size_of::<AppliedReadChanges>() + 2 * std::mem::size_of::<usize>()) as u64,
    )
    .map_err(|cause| request_stop(contracts::execution::denial::lease_denial(cause)))?;
    Ok(Arc::new(AppliedReadChanges {
        before_root,
        fields,
        aggregate,
        _memory: memory,
    }))
}
