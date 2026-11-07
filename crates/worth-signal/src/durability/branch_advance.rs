//! A durable branch effect records coordinates and source references, never values.
use super::{frame, SignalDurableBranchDescriptor, SignalDurableFrameDenial as D};
use std::sync::Arc;
use worth_execution::{ExecutionImmutableBytes, ExecutionMemoryReservation, ExecutionRequest};
use worth_foundational::facade::{CompositeAttemptAssociation, SignalBranchAdvanceIdentity};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SignalAuthoredBranchChange {
    Created {
        name: String,
        parent: Option<SignalDurableBranchDescriptor>,
    },
    Advanced,
    SnapshotCaptured,
    Restored,
    Retired,
}
#[derive(Clone, Debug)]
pub struct SignalBranchAdvanceArtifact {
    original: Arc<BranchAdvanceMeaning>,
}
#[derive(Debug)]
struct BranchAdvanceMeaning {
    bytes: ExecutionImmutableBytes,
    digest: [u8; 32],
    identity: SignalBranchAdvanceIdentity,
    descriptor: SignalDurableBranchDescriptor,
    association: Option<CompositeAttemptAssociation>,
    change: SignalAuthoredBranchChange,
    _funding: Option<ExecutionMemoryReservation>,
}
mod readmission;
impl SignalBranchAdvanceArtifact {
    pub fn capture(
        execution: &ExecutionRequest<'_, '_>,
        identity: SignalBranchAdvanceIdentity,
        descriptor: SignalDurableBranchDescriptor,
        association: Option<CompositeAttemptAssociation>,
        change: SignalAuthoredBranchChange,
    ) -> Result<Self, D> {
        validate(identity, &descriptor, association.as_ref(), &change)?;
        let bytes = frame::encode(2, execution, &mut |out| {
            out.owner(identity.owner())?;
            out.number(identity.ordinal())?;
            descriptor.write(out)?;
            out.association(association.as_ref())?;
            match &change {
                SignalAuthoredBranchChange::Created { name, parent } => {
                    out.byte(0)?;
                    out.blob(name.as_bytes())?;
                    out.byte(u8::from(parent.is_some()))?;
                    if let Some(parent) = parent {
                        parent.write(out)?;
                    }
                }
                SignalAuthoredBranchChange::Advanced => out.byte(1)?,
                SignalAuthoredBranchChange::SnapshotCaptured => out.byte(2)?,
                SignalAuthoredBranchChange::Restored => out.byte(3)?,
                SignalAuthoredBranchChange::Retired => out.byte(4)?,
            }
            Ok(())
        })?;
        readmission::from_owned(execution, bytes)
    }
    pub fn readmit_bytes(execution: &ExecutionRequest<'_, '_>, bytes: &[u8]) -> Result<Self, D> {
        readmission::from_bytes(execution, bytes)
    }
    pub fn as_bytes(&self) -> &[u8] {
        self.original.bytes.bytes()
    }
    pub fn digest(&self) -> [u8; 32] {
        self.original.digest
    }
    pub fn identity(&self) -> SignalBranchAdvanceIdentity {
        self.original.identity
    }
    pub fn descriptor(&self) -> &SignalDurableBranchDescriptor {
        &self.original.descriptor
    }
    pub fn association(&self) -> Option<&CompositeAttemptAssociation> {
        self.original.association.as_ref()
    }
    pub fn change(&self) -> &SignalAuthoredBranchChange {
        &self.original.change
    }
}
fn validate(
    identity: SignalBranchAdvanceIdentity,
    descriptor: &SignalDurableBranchDescriptor,
    association: Option<&CompositeAttemptAssociation>,
    change: &SignalAuthoredBranchChange,
) -> Result<(), D> {
    if identity.ordinal() == 0 {
        return Err(D::Malformed);
    }
    if identity.owner() != descriptor.owner()
        || association.is_some_and(|a| a.attempt().owner().home() != identity.owner().home())
    {
        return Err(D::ForeignOwner);
    }
    if matches!(
        change,
        SignalAuthoredBranchChange::SnapshotCaptured | SignalAuthoredBranchChange::Restored
    ) && descriptor.snapshot().is_none()
    {
        return Err(D::SnapshotRequired);
    }
    match change {
        SignalAuthoredBranchChange::Created { name, parent } => {
            if name.trim().is_empty() || name.len() > 256 {
                return Err(D::Malformed);
            }
            if parent
                .as_ref()
                .is_some_and(|p| p.owner() != identity.owner())
            {
                return Err(D::ForeignOwner);
            }
        }
        SignalAuthoredBranchChange::Advanced
        | SignalAuthoredBranchChange::SnapshotCaptured
        | SignalAuthoredBranchChange::Restored
        | SignalAuthoredBranchChange::Retired => {}
    }
    Ok(())
}
impl worth_foundational::facade::DurableArtifactRetention for SignalBranchAdvanceArtifact {
    const RETENTION_CLASS: worth_foundational::facade::RetentionClass =
        worth_foundational::facade::RetentionClass::Authoritative;
}
