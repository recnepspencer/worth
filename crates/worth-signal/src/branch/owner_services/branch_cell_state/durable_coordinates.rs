//! Durable source coordinates accompany the canonical cell, not a second head table.
use super::SignalBranchCellState;
use crate::durability::{
    SignalDurableBranchDescriptor, SignalDurableFrameDenial, SignalRecordedSourcePoint,
};
use worth_foundational::facade::{
    DurableSignalAdmissionIdentity, DurableSignalCellIncarnation, DurableSignalGeneration,
    DurableSignalSnapshotIdentity, SignalDefinitionTransitionReference,
};

pub(super) struct SignalDurableCellCoordinates {
    reference: worth_foundational::facade::SignalBranchAdvanceReference,
    position: std::num::NonZeroU64,
    cell: DurableSignalCellIncarnation,
    admission: DurableSignalAdmissionIdentity,
    definition: SignalDefinitionTransitionReference,
}
impl<D, I, T> SignalBranchCellState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn install_durable_coordinates(&mut self, point: SignalRecordedSourcePoint) {
        let descriptor = point.descriptor();
        assert_eq!(descriptor.branch().ordinal(), self.handle.id.0);
        assert_eq!(descriptor.generation().ordinal(), self.head_generation);
        assert_eq!(
            descriptor.snapshot().map(|id| id.ordinal()),
            self.handle.head_snapshot_id.map(|id| id.0)
        );
        self.durable_coordinates = Some(SignalDurableCellCoordinates {
            reference: point.reference(),
            position: std::num::NonZeroU64::new(point.position())
                .expect("source positions are nonzero"),
            cell: descriptor.cell(),
            admission: descriptor.admission(),
            definition: descriptor.definition(),
        });
    }
    pub(in crate::branch::owner_services) fn durable_source_point(
        &self,
    ) -> Result<Option<SignalRecordedSourcePoint>, SignalDurableFrameDenial> {
        let Some(coordinates) = &self.durable_coordinates else {
            return Ok(None);
        };
        let descriptor = self
            .durable_descriptor()?
            .expect("same installed coordinates");
        SignalRecordedSourcePoint::from_recorded(
            coordinates.reference,
            coordinates.position,
            descriptor,
        )
        .map(Some)
    }
    pub(in crate::branch::owner_services) fn recorded_source_observation(
        &self,
        descriptor: &SignalDurableBranchDescriptor,
        restored: bool,
    ) -> Result<crate::branch::SignalBranchObservation, crate::data::error::SignalError> {
        let snapshot = descriptor
            .snapshot()
            .map(|id| crate::state::SignalSnapshotId(id.ordinal()));
        self.observation_for(
            descriptor.generation().ordinal(),
            snapshot,
            restored.then_some(snapshot).flatten(),
        )
    }
    pub(in crate::branch::owner_services) fn durable_descriptor(
        &self,
    ) -> Result<Option<SignalDurableBranchDescriptor>, SignalDurableFrameDenial> {
        self.durable_coordinates
            .as_ref()
            .map(|coordinates| {
                let owner = coordinates.cell.branch().owner();
                SignalDurableBranchDescriptor::from_recorded(
                    DurableSignalGeneration::from_recorded(coordinates.cell, self.head_generation),
                    self.handle
                        .head_snapshot_id
                        .map(|id| DurableSignalSnapshotIdentity::from_recorded(owner, id.0)),
                    coordinates.admission,
                    coordinates.definition,
                )
            })
            .transpose()
    }
}
