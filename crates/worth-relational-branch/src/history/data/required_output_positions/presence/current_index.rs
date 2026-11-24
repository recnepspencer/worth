//! Warm source-keyed position image; cold history's unresolved locator is unchanged.
use worth_relational_contracts::internal_port::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaningOwnerPort as _;
use worth_relational_contracts::internal_port::transactions::data::outcomes::commit_conflict::CommitConflictOwnerPort as _;
use super::*;
use crate::transactions::data::outcomes::{
    commit_conflict::CommitConflict, conflict_class::ConflictClass,
    transaction_commit_error::TransactionCommitError,
};
use std::sync::Arc;
use worth_execution::{
    ExecutionMemoryReservation, ExecutionOrderedMap, ExecutionOrderedMapDenial, ExecutionRequest,
    MapKernelContext,
};
#[derive(Clone, Debug)]
pub struct SourcePositions {
    pub routes: Vec<RoutePresence>,
    _funding: Arc<Option<ExecutionMemoryReservation>>,
}
#[derive(Clone, Debug, Default)]
pub struct CurrentPositions(ExecutionOrderedMap<EntityId, Arc<SourcePositions>>);
impl CurrentPositions {
    pub fn at(&self, source: EntityId) -> &[RoutePresence] {
        self.0
            .get(&source)
            .map_or(&[], |value| value.routes.as_slice())
    }
    pub fn replace(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        meaning: &RequiredOutputPublicationMeaning,
        next: Option<RoutePresence>,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<(), TransactionCommitError> {
        let prior = self.at(meaning.source_scope);
        crate::publication::ownership::read_request::checkpoint(context, prior.len() as u64 + 1)?;
        let bytes = (prior.len() + 1)
            .checked_mul(std::mem::size_of::<RoutePresence>() * 4)
            .and_then(|bytes| {
                prior
                    .iter()
                    .try_fold(bytes, |sum, p| sum.checked_add(p.meaning.family.len()))
            })
            .and_then(|sum| sum.checked_add(meaning.family.len()))
            .ok_or_else(|| {
                crate::publication::ownership::read_request::request(
                    crate::execution::denial_cause::RelationalExecutionDenialCause::ChargedBytesOverflow,
                )
            })?;
        let funding = execution
            .reserve_retained_memory(bytes as u64)
            .map_err(crate::publication::ownership::read_request::request)?;
        let mut routes = prior
            .iter()
            .filter(|p| !p.meaning.same_route(meaning))
            .cloned()
            .collect::<Vec<_>>();
        if let Some(next) = next {
            routes.push(next);
        }
        if routes.is_empty() {
            self.0
                .remove_for_request(&meaning.source_scope, *execution)
                .map_err(denial)?;
        } else {
            let value = Arc::new(SourcePositions {
                routes,
                _funding: Arc::new(funding),
            });
            self.0
                .insert_cloned_native_for_request(&meaning.source_scope, &value, *execution)
                .map_err(denial)?;
        }
        Ok(())
    }
}
fn denial(cause: ExecutionOrderedMapDenial) -> TransactionCommitError {
    match cause {
        ExecutionOrderedMapDenial::Request(stop) => crate::publication::ownership::read_request::request(stop),
        ExecutionOrderedMapDenial::CapacityOverflow => crate::publication::ownership::read_request::request(
            crate::execution::denial_cause::RelationalExecutionDenialCause::ChargedBytesOverflow,
        ),
        ExecutionOrderedMapDenial::Allocation(denial) => {
            crate::execution::denial::allocation_commit_error(TransactionCommitError::conflict(
                CommitConflict::new(ConflictClass::ExecutionAllocationDenied { denial }),
            ))
        }
    }
}
