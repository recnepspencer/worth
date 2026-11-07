//! Recorded branch coordinates cannot stand in for an admitted live basis.
use super::frame::{Reader, Writer};
use super::SignalDurableFrameDenial as D;
use worth_foundational::facade::{
    DurableSignalAdmissionIdentity, DurableSignalBranchIdentity, DurableSignalCellIncarnation,
    DurableSignalGeneration, DurableSignalOwnerIdentity, DurableSignalSnapshotIdentity,
    SignalDefinitionTransitionReference,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalDurableBranchDescriptor {
    generation: DurableSignalGeneration,
    snapshot: Option<DurableSignalSnapshotIdentity>,
    admission: DurableSignalAdmissionIdentity,
    definition: SignalDefinitionTransitionReference,
}
impl SignalDurableBranchDescriptor {
    pub fn from_recorded(
        generation: DurableSignalGeneration,
        snapshot: Option<DurableSignalSnapshotIdentity>,
        admission: DurableSignalAdmissionIdentity,
        definition: SignalDefinitionTransitionReference,
    ) -> Result<Self, D> {
        let owner = generation.cell().branch().owner();
        if snapshot.is_some_and(|snapshot| snapshot.owner() != owner)
            || admission.owner() != owner
            || definition.identity().owner() != owner
        {
            return Err(D::ForeignOwner);
        }
        Ok(Self {
            generation,
            snapshot,
            admission,
            definition,
        })
    }
    pub const fn owner(&self) -> DurableSignalOwnerIdentity {
        self.generation.cell().branch().owner()
    }
    pub const fn branch(&self) -> DurableSignalBranchIdentity {
        self.generation.cell().branch()
    }
    pub const fn cell(&self) -> DurableSignalCellIncarnation {
        self.generation.cell()
    }
    pub const fn generation(&self) -> DurableSignalGeneration {
        self.generation
    }
    pub const fn snapshot(&self) -> Option<DurableSignalSnapshotIdentity> {
        self.snapshot
    }
    pub const fn admission(&self) -> DurableSignalAdmissionIdentity {
        self.admission
    }
    pub const fn definition(&self) -> SignalDefinitionTransitionReference {
        self.definition
    }
    pub(super) fn write(&self, out: &mut Writer<'_>) -> Result<(), D> {
        out.branch(self.branch())?;
        out.number(self.cell().ordinal())?;
        out.number(self.generation.ordinal())?;
        out.byte(u8::from(self.snapshot.is_some()))?;
        if let Some(snapshot) = self.snapshot {
            out.number(snapshot.ordinal())?;
        }
        out.number(self.admission.ordinal())?;
        out.definition(self.definition)
    }
    pub(super) fn read(input: &mut Reader<'_>) -> Result<Self, D> {
        let branch = input.branch()?;
        let cell = DurableSignalCellIncarnation::from_recorded(branch, input.number()?);
        let generation = DurableSignalGeneration::from_recorded(cell, input.number()?);
        let snapshot = match input.byte()? {
            0 => None,
            1 => Some(DurableSignalSnapshotIdentity::from_recorded(
                branch.owner(),
                input.number()?,
            )),
            _ => return Err(D::Malformed),
        };
        let admission =
            DurableSignalAdmissionIdentity::from_recorded(branch.owner(), input.number()?);
        Self::from_recorded(generation, snapshot, admission, input.definition()?)
    }
}
