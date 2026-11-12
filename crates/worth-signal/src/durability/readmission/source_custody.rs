//! Original source-location storage is admitted before physical publication.
use super::source_locations::SignalDefinitionSourceRecord;
use super::{source_catalog::RecordedAdvanceLocation, SignalOwnerReadmissionDenial as D};
use crate::data::retained_storage::{arc_allocation_charge, btree_structure_charge};
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_foundational::facade::{SignalBranchAdvanceIdentity, SignalDefinitionTransitionIdentity};

pub(crate) struct SignalDefinitionLocationReservation {
    _memory: Option<ExecutionMemoryReservation>,
}
pub(crate) struct SignalAdvanceLocationReservation {
    _memory: Option<ExecutionMemoryReservation>,
}
impl SignalDefinitionLocationReservation {
    pub(crate) fn reserve(execution: &ExecutionRequest<'_, '_>) -> Result<Self, D> {
        let charge = btree_structure_charge::<
            SignalDefinitionTransitionIdentity,
            Arc<SignalDefinitionSourceRecord>,
        >(1)
        .and_then(|tree| tree.checked_add(arc_allocation_charge::<SignalDefinitionSourceRecord>()?))
        .map_err(|_| D::MetadataCapacity)?;
        execution
            .reserve_retained_memory(charge.bytes())
            .map(|_memory| Self { _memory })
            .map_err(D::Request)
    }
}
pub(crate) struct SignalFundedAdvanceLocation {
    pub(super) location: RecordedAdvanceLocation,
    pub(super) _custody: SignalAdvanceLocationReservation,
}
impl SignalAdvanceLocationReservation {
    pub(crate) fn reserve(execution: &ExecutionRequest<'_, '_>) -> Result<Self, D> {
        let charge =
            btree_structure_charge::<SignalBranchAdvanceIdentity, SignalFundedAdvanceLocation>(1)
                .map_err(|_| D::MetadataCapacity)?;
        execution
            .reserve_retained_memory(charge.bytes())
            .map(|_memory| Self { _memory })
            .map_err(D::Request)
    }
}
