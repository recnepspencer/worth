//! Original source coordinates are descriptions, never an ACK or live admission.
use super::SignalDurableFrameDenial as D;
use super::{frame, SignalBranchAdvanceArtifact, SignalDurableBranchDescriptor};
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest};
use worth_foundational::facade::{SignalBranchAdvanceIdentity, SignalBranchAdvanceReference};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalRecordedSourcePoint {
    reference: SignalBranchAdvanceReference,
    position: std::num::NonZeroU64,
    descriptor: SignalDurableBranchDescriptor,
}
impl SignalRecordedSourcePoint {
    pub fn from_recorded(
        reference: SignalBranchAdvanceReference,
        position: std::num::NonZeroU64,
        descriptor: SignalDurableBranchDescriptor,
    ) -> Result<Self, D> {
        if reference.identity().owner() != descriptor.owner() {
            return Err(D::ForeignOwner);
        }
        if reference.identity().ordinal() == 0 {
            return Err(D::Malformed);
        }
        Ok(Self {
            reference,
            position,
            descriptor,
        })
    }
    pub(crate) fn original(
        artifact: &SignalBranchAdvanceArtifact,
        position: std::num::NonZeroU64,
    ) -> Self {
        Self {
            reference: artifact.reference(),
            position,
            descriptor: artifact.descriptor().clone(),
        }
    }
    pub const fn reference(&self) -> SignalBranchAdvanceReference {
        self.reference
    }
    pub const fn position(&self) -> u64 {
        self.position.get()
    }
    pub fn descriptor(&self) -> &SignalDurableBranchDescriptor {
        &self.descriptor
    }
    pub fn encode_bytes(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<ExecutionImmutableBytes, D> {
        frame::encode(4, execution, &mut |out| self.write_fields(out))
    }
    /// Transport this exact versioned owner format without an intermediate copy.
    pub fn write_bytes(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        sink: &mut dyn FnMut(&[u8]) -> Result<(), D>,
    ) -> Result<(), D> {
        frame::write(4, execution, &mut |out| self.write_fields(out), sink)
    }
    fn write_fields(&self, out: &mut frame::Writer<'_>) -> Result<(), D> {
        out.owner(self.reference.identity().owner())?;
        out.number(self.reference.identity().ordinal())?;
        out.fixed(&self.reference.digest())?;
        out.number(self.position.get())?;
        self.descriptor.write(out)
    }
    pub fn readmit_bytes(execution: &ExecutionRequest<'_, '_>, bytes: &[u8]) -> Result<Self, D> {
        frame::admit_work(execution, bytes.len())?;
        let mut input = frame::payload(bytes, 4)?;
        let identity = SignalBranchAdvanceIdentity::from_recorded(input.owner()?, input.number()?);
        let reference = SignalBranchAdvanceReference::from_recorded(identity, input.fixed()?);
        let position = std::num::NonZeroU64::new(input.number()?).ok_or(D::Malformed)?;
        let descriptor = SignalDurableBranchDescriptor::read(&mut input)?;
        input.finish()?;
        Self::from_recorded(reference, position, descriptor)
    }
}
