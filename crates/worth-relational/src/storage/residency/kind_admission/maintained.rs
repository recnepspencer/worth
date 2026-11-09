//! The kind projection is the first adopter of the shared SDK contract.
use super::super::kind_incremental::{self, RegionKindIndexes};
use super::super::kind_projection::{ProjectionCheckpoint, ProjectionRetention};
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::overlay::{PartitionMutationJournal, PartitionState};
use crate::storage::substrate::maintenance::{
    ChangeWork, CommitChange, Maintained, MaintenanceConsistency, MaintenanceRefusal,
    MaintenanceRoute, ReconstructionLane, ReconstructionWork,
};
use std::collections::BTreeSet;
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
    routes: BTreeSet<MaintenanceRoute>,
}
impl<'a, 'request, 'lease> KindMaintainer<'a, 'request, 'lease> {
    pub(super) fn new(
        execution: &'a worth_execution::ExecutionRequest<'request, 'lease>,
        partition: &'a mut PartitionState,
        previous: &'a PartitionState,
    ) -> Self {
        Self {
            execution,
            routes: BTreeSet::from([MaintenanceRoute::Partition(partition.partition_id)]),
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
impl Maintained for KindMaintainer<'_, '_, '_> {
    type State = RegionKindIndexes;
    // Kind history is committed meaning, never silently compacted by a cache.
    type Retirement = std::convert::Infallible;
    type Error = KindMaintenanceError;
    fn routes(&self) -> &BTreeSet<MaintenanceRoute> {
        &self.routes
    }
    fn consistency(&self) -> MaintenanceConsistency {
        MaintenanceConsistency::Synchronous
    }
    fn apply(
        &mut self,
        previous: &Arc<Self::State>,
        change: &CommitChange,
        work: &mut ChangeWork<'_>,
    ) -> Result<Arc<Self::State>, Self::Error> {
        let Some(change) = change.partition(self.partition.partition_id) else {
            return Ok(Arc::clone(previous));
        };
        self.apply_journal(previous, &change.journal, work)
    }
    fn cold(
        &mut self,
        lane: &ReconstructionLane<'_>,
        work: &mut ReconstructionWork<'_, '_>,
    ) -> Result<Arc<Self::State>, Self::Error> {
        // cost-lane: reconstruction. Local history comes from the arena;
        // authenticated peer projections remain root-local claims.
        controlled(
            self.execution,
            &mut |units, bytes| work.prepare(units, bytes),
            |checkpoint, retain| {
                super::normalize_value(lane, self.partition, checkpoint, retain)?;
                RegionKindIndexes::capture_controlled(lane, self.partition, checkpoint)
            },
        )
        .map(Arc::new)
    }
    fn retire(
        &mut self,
        _: &Arc<Self::State>,
        horizon: &Self::Retirement,
        _: &mut ChangeWork<'_>,
    ) -> Result<Arc<Self::State>, Self::Error> {
        match *horizon {}
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
    if let Some(cause) = first_stop.into_inner() {
        return Err(cause.into());
    }
    result.map_err(KindMaintenanceError::Storage)
}
