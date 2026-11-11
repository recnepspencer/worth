//! Inspect borrowed names, fund their exact storage, then materialize once.
use super::*;
use crate::durability::frame::AssociationFields;
use sha2::{Digest, Sha256};

struct Fields<'a> {
    identity: SignalBranchAdvanceIdentity,
    descriptor: SignalDurableBranchDescriptor,
    association: Option<AssociationFields<'a>>,
    change: ChangeFields<'a>,
}
enum ChangeFields<'a> {
    Created {
        name: &'a str,
        parent: Option<SignalDurableBranchDescriptor>,
    },
    Advanced,
    SnapshotCaptured,
    Restored,
    Retired,
}
impl Fields<'_> {
    fn funding(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Option<ExecutionMemoryReservation>, D> {
        let layout = std::alloc::Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
            .extend(std::alloc::Layout::new::<BranchAdvanceMeaning>())
            .map_err(|_| D::SizeLimit)?
            .0
            .pad_to_align();
        let name = match self.change {
            ChangeFields::Created { name, .. } => name.len(),
            ChangeFields::Advanced
            | ChangeFields::SnapshotCaptured
            | ChangeFields::Restored
            | ChangeFields::Retired => 0,
        };
        let quote = layout
            .size()
            .checked_add(name)
            .and_then(|bytes| {
                bytes.checked_add(self.association.as_ref().map_or(0, |a| a.name.len()))
            })
            .ok_or(D::SizeLimit)?;
        execution
            .reserve_retained_memory(quote as u64)
            .map_err(D::Request)
    }
    fn complete(
        self,
        bytes: ExecutionImmutableBytes,
        funding: Option<ExecutionMemoryReservation>,
    ) -> Result<SignalBranchAdvanceArtifact, D> {
        let association = self
            .association
            .map(AssociationFields::materialize)
            .transpose()?;
        let change = match self.change {
            ChangeFields::Created { name, parent } => SignalAuthoredBranchChange::Created {
                name: name.to_owned(),
                parent,
            },
            ChangeFields::Advanced => SignalAuthoredBranchChange::Advanced,
            ChangeFields::SnapshotCaptured => SignalAuthoredBranchChange::SnapshotCaptured,
            ChangeFields::Restored => SignalAuthoredBranchChange::Restored,
            ChangeFields::Retired => SignalAuthoredBranchChange::Retired,
        };
        Ok(SignalBranchAdvanceArtifact {
            original: Arc::new(BranchAdvanceMeaning {
                digest: Sha256::digest(bytes.bytes()).into(),
                bytes,
                identity: self.identity,
                descriptor: self.descriptor,
                association,
                change,
                _funding: funding,
            }),
        })
    }
}
pub(super) fn from_owned(
    execution: &ExecutionRequest<'_, '_>,
    bytes: ExecutionImmutableBytes,
) -> Result<SignalBranchAdvanceArtifact, D> {
    frame::admit_work(execution, bytes.bytes().len().saturating_mul(2))?;
    let fields = inspect(bytes.bytes())?;
    let funding = fields.funding(execution)?;
    // The backing is shared; no input byte or parsed name is cloned by an artifact clone.
    fields.complete(bytes.clone(), funding)
}
pub(super) fn from_bytes(
    execution: &ExecutionRequest<'_, '_>,
    bytes: &[u8],
) -> Result<SignalBranchAdvanceArtifact, D> {
    frame::admit_work(execution, bytes.len().saturating_mul(3))?;
    let fields = inspect(bytes)?;
    let funding = fields.funding(execution)?;
    fields.complete(frame::copy_bytes(execution, bytes)?, funding)
}
fn inspect(bytes: &[u8]) -> Result<Fields<'_>, D> {
    let mut input = frame::payload(bytes, 2)?;
    let identity = SignalBranchAdvanceIdentity::from_recorded(input.owner()?, input.number()?);
    let descriptor = SignalDurableBranchDescriptor::read(&mut input)?;
    let association = input.association_fields()?;
    if identity.ordinal() == 0 {
        return Err(D::Malformed);
    }
    if identity.owner() != descriptor.owner()
        || association
            .as_ref()
            .is_some_and(|a| a.attempt.owner().home() != identity.owner().home())
    {
        return Err(D::ForeignOwner);
    }
    let change = match input.byte()? {
        0 => {
            let name = std::str::from_utf8(input.blob()?).map_err(|_| D::Malformed)?;
            if name.trim().is_empty() || name.len() > 256 {
                return Err(D::Malformed);
            }
            let parent = match input.byte()? {
                0 => None,
                1 => Some(SignalDurableBranchDescriptor::read(&mut input)?),
                _ => return Err(D::Malformed),
            };
            if parent
                .as_ref()
                .is_some_and(|p| p.owner() != identity.owner())
            {
                return Err(D::ForeignOwner);
            }
            ChangeFields::Created { name, parent }
        }
        1 => ChangeFields::Advanced,
        2 => {
            descriptor.require_snapshot()?;
            ChangeFields::SnapshotCaptured
        }
        3 => {
            descriptor.require_snapshot()?;
            ChangeFields::Restored
        }
        4 => ChangeFields::Retired,
        _ => return Err(D::Malformed),
    };
    input.finish()?;
    Ok(Fields {
        identity,
        descriptor,
        association,
        change,
    })
}
