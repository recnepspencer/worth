//! Durable source coordinates accompany the canonical cell, not a second head table.
use super::SignalBranchCellState;
use crate::durability::{SignalDurableBranchDescriptor, SignalDurableFrameDenial};
use worth_foundational::facade::{
    DurableSignalAdmissionIdentity, DurableSignalCellIncarnation, DurableSignalGeneration,
    DurableSignalSnapshotIdentity, SignalDefinitionTransitionReference,
};

pub(super) struct SignalDurableCellCoordinates {
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
    pub(crate) fn install_durable_coordinates(
        &mut self,
        descriptor: SignalDurableBranchDescriptor,
    ) {
        assert_eq!(descriptor.branch().ordinal(), self.handle.id.0);
        assert_eq!(descriptor.generation().ordinal(), self.head_generation);
        assert_eq!(
            descriptor.snapshot().map(|id| id.ordinal()),
            self.handle.head_snapshot_id.map(|id| id.0)
        );
        self.durable_coordinates = Some(SignalDurableCellCoordinates {
            cell: descriptor.cell(),
            admission: descriptor.admission(),
            definition: descriptor.definition(),
        });
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
