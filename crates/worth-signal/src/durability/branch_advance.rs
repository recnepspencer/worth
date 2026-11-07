//! A durable branch effect records coordinates and source references, never values.
use super::{frame, SignalDurableBranchDescriptor, SignalDurableFrameDenial as D};
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest};
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
    bytes: ExecutionImmutableBytes,
    identity: SignalBranchAdvanceIdentity,
    descriptor: SignalDurableBranchDescriptor,
    association: Option<CompositeAttemptAssociation>,
    change: SignalAuthoredBranchChange,
}
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
        Ok(Self {
            bytes,
            identity,
            descriptor,
            association,
            change,
        })
    }
    pub fn readmit_bytes(execution: &ExecutionRequest<'_, '_>, bytes: &[u8]) -> Result<Self, D> {
        frame::admit_work(execution, bytes.len())?;
        let (identity, descriptor, association, change) = inspect(bytes)?;
        Ok(Self {
            bytes: frame::copy_bytes(execution, bytes)?,
            identity,
            descriptor,
            association,
            change,
        })
    }
    pub fn as_bytes(&self) -> &[u8] {
        self.bytes.bytes()
    }
    pub const fn identity(&self) -> SignalBranchAdvanceIdentity {
        self.identity
    }
    pub fn descriptor(&self) -> &SignalDurableBranchDescriptor {
        &self.descriptor
    }
    pub fn association(&self) -> Option<&CompositeAttemptAssociation> {
        self.association.as_ref()
    }
    pub fn change(&self) -> &SignalAuthoredBranchChange {
        &self.change
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
fn inspect(
    bytes: &[u8],
) -> Result<
    (
        SignalBranchAdvanceIdentity,
        SignalDurableBranchDescriptor,
        Option<CompositeAttemptAssociation>,
        SignalAuthoredBranchChange,
    ),
    D,
> {
    let mut input = frame::payload(bytes, 2)?;
    let identity = SignalBranchAdvanceIdentity::from_recorded(input.owner()?, input.number()?);
    let descriptor = SignalDurableBranchDescriptor::read(&mut input)?;
    let association = input.association()?;
    let change = match input.byte()? {
        0 => {
            let name = input.blob()?;
            if name.len() > 256 {
                return Err(D::Malformed);
            }
            let name = std::str::from_utf8(name)
                .map_err(|_| D::Malformed)?
                .to_owned();
            let parent = match input.byte()? {
                0 => None,
                1 => Some(SignalDurableBranchDescriptor::read(&mut input)?),
                _ => return Err(D::Malformed),
            };
            SignalAuthoredBranchChange::Created { name, parent }
        }
        1 => SignalAuthoredBranchChange::Advanced,
        2 => SignalAuthoredBranchChange::SnapshotCaptured,
        3 => SignalAuthoredBranchChange::Restored,
        4 => SignalAuthoredBranchChange::Retired,
        _ => return Err(D::Malformed),
    };
    input.finish()?;
    validate(identity, &descriptor, association.as_ref(), &change)?;
    Ok((identity, descriptor, association, change))
}
impl worth_foundational::facade::DurableArtifactRetention for SignalBranchAdvanceArtifact {
    const RETENTION_CLASS: worth_foundational::facade::RetentionClass =
        worth_foundational::facade::RetentionClass::Authoritative;
}
