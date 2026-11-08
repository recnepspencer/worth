//! The actual current Signal source and original composite association bind together.
use super::{frame, SignalDurableFrameDenial};
use crate::branch::owner_services::SignalDurableBranchHandle;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::CompositeAttemptAssociation;

/// Descriptive binding only; execution separately requires the installed issuer's grant.
#[derive(Debug)]
pub struct SignalCompositeAdvanceBinding {
    expected: SignalDurableBranchHandle,
    association: CompositeAttemptAssociation,
    commitment: [u8; 32],
}
impl SignalCompositeAdvanceBinding {
    pub(crate) fn capture(
        execution: &ExecutionRequest<'_, '_>,
        expected: SignalDurableBranchHandle,
        association: CompositeAttemptAssociation,
    ) -> Result<Self, SignalDurableFrameDenial> {
        if expected.descriptor().owner().home() != association.attempt().owner().home() {
            return Err(SignalDurableFrameDenial::ForeignOwner);
        }
        let commitment = frame::commitment(
            execution,
            b"worth.signal.composite-advance-binding.v1",
            &mut |out| {
                expected.descriptor().write(out)?;
                out.association(Some(&association))
            },
        )?;
        Ok(Self {
            expected,
            association,
            commitment,
        })
    }

    pub fn commitment(&self) -> [u8; 32] {
        self.commitment
    }
    pub fn source(&self) -> &SignalDurableBranchHandle {
        &self.expected
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        SignalDurableBranchHandle,
        CompositeAttemptAssociation,
        [u8; 32],
    ) {
        (self.expected, self.association, self.commitment)
    }
}

pub(crate) fn fork_commitment(
    execution: &ExecutionRequest<'_, '_>,
    source: &SignalDurableBranchHandle,
    cell: worth_foundational::facade::DurableSignalCellIncarnation,
    name: &str,
    association: &CompositeAttemptAssociation,
) -> Result<[u8; 32], SignalDurableFrameDenial> {
    if source.descriptor().owner().home() != association.attempt().owner().home()
        || source.descriptor().owner() != cell.branch().owner()
    {
        return Err(SignalDurableFrameDenial::ForeignOwner);
    }
    frame::commitment(
        execution,
        b"worth.signal.composite-fork-binding.v1",
        &mut |out| {
            source.descriptor().write(out)?;
            out.branch(cell.branch())?;
            out.number(cell.ordinal())?;
            out.blob(name.as_bytes())?;
            out.association(Some(association))
        },
    )
}
