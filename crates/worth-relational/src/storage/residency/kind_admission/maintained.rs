//! The kind projection is the first adopter of the shared SDK contract.
use super::super::kind_incremental::{self, RegionKindIndexes};
use super::super::kind_projection::{ProjectionCheckpoint, ProjectionRetention};
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::overlay::{PartitionMutationJournal, PartitionState};
use crate::storage::substrate::maintenance::{ChangeWork, MaintenanceRefusal};
use std::sync::Arc;

#[derive(Debug)]
pub(in crate::storage::residency) enum KindMaintenanceError {
    Admission(MaintenanceRefusal),
    Storage(Cause),
}
impl From<MaintenanceRefusal> for KindMaintenanceError {
    fn from(cause: MaintenanceRefusal) -> Self {
        Self::Admission(cause)
    }
}

pub(super) struct KindMaintainer<'a, 'request, 'lease> {
    execution: &'a worth_execution::ExecutionRequest<'request, 'lease>,
    partition: &'a mut PartitionState,
    previous: &'a PartitionState,
}
impl<'a, 'request, 'lease> KindMaintainer<'a, 'request, 'lease> {
    pub(super) fn new(
        execution: &'a worth_execution::ExecutionRequest<'request, 'lease>,
        partition: &'a mut PartitionState,
        previous: &'a PartitionState,
    ) -> Self {
        Self {
            execution,
            partition,
            previous,
        }
    }
    pub(super) fn apply_journal(
        &mut self,
        previous: &Arc<RegionKindIndexes>,
        journal: &PartitionMutationJournal,
        work: &mut ChangeWork<'_>,
    ) -> Result<Arc<RegionKindIndexes>, KindMaintenanceError> {
        if journal.adjacency_slots.is_empty()
            && journal.reverse_adjacency_slots.is_empty()
            && journal.relation_slots.is_empty()
        {
            return Ok(Arc::clone(previous));
        }
        controlled(
            self.execution,
            &mut |units, bytes| work.prepare(units, bytes),
            |checkpoint, retain| {
                kind_incremental::update_value(
                    self.partition,
                    self.previous,
                    previous,
                    journal,
                    checkpoint,
                    retain,
                )
            },
        )
        .map(Arc::new)
    }
}
pub(super) fn controlled<T>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    prepare: &mut dyn FnMut(u64, u64) -> Result<(), MaintenanceRefusal>,
    operation: impl FnOnce(
        &mut ProjectionCheckpoint<'_>,
        &mut ProjectionRetention<'_>,
    ) -> Result<T, Cause>,
) -> Result<T, KindMaintenanceError> {
    let first_stop = std::cell::RefCell::new(None);
    let mut checkpoint = |units, bytes| {
        if first_stop.borrow().is_some() {
            return Err(Cause::SourceUnavailable);
        }
        prepare(units, bytes).map_err(|cause| {
            *first_stop.borrow_mut() = Some(cause);
            Cause::SourceUnavailable
        })
    };
    let mut retain = |prior: Option<&crate::storage::partition::HistoricalKindCustody>, bytes| {
        if first_stop.borrow().is_some() {
            return Err(Cause::SourceUnavailable);
        }
        super::retained_admission::admit(prior, bytes, &mut |bytes| {
            execution.reserve_retained_memory(bytes).map_err(Into::into)
        })
        .map_err(|cause| {
            *first_stop.borrow_mut() = Some(MaintenanceRefusal::Execution(cause));
            Cause::SourceUnavailable
        })
    };
    let result = operation(&mut checkpoint, &mut retain);
    preserve_original_stop(result, first_stop.into_inner())
}

pub(super) fn preserve_original_stop<T>(
    result: Result<T, Cause>,
    first: Option<MaintenanceRefusal>,
) -> Result<T, KindMaintenanceError> {
    match first {
        Some(cause) => Err(cause.into()),
        None => result.map_err(KindMaintenanceError::Storage),
    }
}
