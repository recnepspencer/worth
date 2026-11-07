//! Signal declarations and Bridge's opaque authored section move together.
use super::{frame, SignalAuthoredImage, SignalDurableFrameDenial as D};
use sha2::{Digest, Sha256};
use std::ops::Range;
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest};
use worth_foundational::facade::{
    DurableSignalBranchIdentity, SignalDefinitionTransitionIdentity,
    SignalDefinitionTransitionReference,
};

/// The contents and version are defined and readmitted by Bridge, never Store.
/// This descriptive carrier grants no installed-definition authority.
#[derive(Clone, Debug)]
pub struct SignalBridgeAuthoredSection {
    bytes: ExecutionImmutableBytes,
}
impl SignalBridgeAuthoredSection {
    pub fn from_bridge_bytes(bytes: ExecutionImmutableBytes) -> Result<Self, D> {
        if bytes.bytes().len() > 16 << 20 {
            return Err(D::SizeLimit);
        }
        Ok(Self { bytes })
    }
    pub fn as_bytes(&self) -> &[u8] {
        self.bytes.bytes()
    }
}
#[derive(Clone, Debug)]
pub struct SignalDefinitionTransitionArtifact {
    bytes: ExecutionImmutableBytes,
    identity: SignalDefinitionTransitionIdentity,
    branch: DurableSignalBranchIdentity,
    predecessor: Option<SignalDefinitionTransitionReference>,
    image: Range<usize>,
    bridge: Range<usize>,
}
impl SignalDefinitionTransitionArtifact {
    pub fn capture(
        execution: &ExecutionRequest<'_, '_>,
        identity: SignalDefinitionTransitionIdentity,
        branch: DurableSignalBranchIdentity,
        predecessor: Option<SignalDefinitionTransitionReference>,
        image: &SignalAuthoredImage,
        bridge: &SignalBridgeAuthoredSection,
    ) -> Result<Self, D> {
        if identity.ordinal() == 0 {
            return Err(D::Malformed);
        }
        if identity.owner() != branch.owner()
            || predecessor.is_some_and(|p| p.identity().owner() != identity.owner())
        {
            return Err(D::ForeignOwner);
        }
        let bytes = frame::encode(1, execution, &mut |out| {
            out.owner(identity.owner())?;
            out.number(identity.ordinal())?;
            out.branch(branch)?;
            out.byte(u8::from(predecessor.is_some()))?;
            if let Some(previous) = predecessor {
                out.definition(previous)?;
            }
            out.blob(image.as_bytes())?;
            out.blob(bridge.as_bytes())
        })?;
        Self::readmit_owned(bytes)
    }
    pub fn readmit_bytes(execution: &ExecutionRequest<'_, '_>, bytes: &[u8]) -> Result<Self, D> {
        frame::admit_work(execution, bytes.len().saturating_mul(2))?;
        Self::inspect(bytes)?;
        Self::readmit_owned(frame::copy_bytes(execution, bytes)?)
    }
    pub(super) fn readmit_owned(bytes: ExecutionImmutableBytes) -> Result<Self, D> {
        let (identity, branch, predecessor, image, bridge) = Self::inspect(bytes.bytes())?;
        Ok(Self {
            bytes,
            identity,
            branch,
            predecessor,
            image,
            bridge,
        })
    }
    fn inspect(
        bytes: &[u8],
    ) -> Result<
        (
            SignalDefinitionTransitionIdentity,
            DurableSignalBranchIdentity,
            Option<SignalDefinitionTransitionReference>,
            Range<usize>,
            Range<usize>,
        ),
        D,
    > {
        let mut input = frame::payload(bytes, 1)?;
        let identity =
            SignalDefinitionTransitionIdentity::from_recorded(input.owner()?, input.number()?);
        let branch = input.branch()?;
        let predecessor = match input.byte()? {
            0 => None,
            1 => Some(input.definition()?),
            _ => return Err(D::Malformed),
        };
        if identity.ordinal() == 0 {
            return Err(D::Malformed);
        }
        if identity.owner() != branch.owner()
            || predecessor.is_some_and(|p| p.identity().owner() != identity.owner())
        {
            return Err(D::ForeignOwner);
        }
        let image_bytes = input.blob()?;
        super::authored_image::validate_bytes(image_bytes).map_err(D::Image)?;
        let image = (input.position - image_bytes.len())..input.position;
        let bridge_bytes = input.blob()?;
        if bridge_bytes.len() > 16 << 20 {
            return Err(D::SizeLimit);
        }
        let bridge = (input.position - bridge_bytes.len())..input.position;
        input.finish()?;
        Ok((identity, branch, predecessor, image, bridge))
    }
    pub fn as_bytes(&self) -> &[u8] {
        self.bytes.bytes()
    }
    pub const fn identity(&self) -> SignalDefinitionTransitionIdentity {
        self.identity
    }
    pub const fn branch(&self) -> DurableSignalBranchIdentity {
        self.branch
    }
    pub const fn predecessor(&self) -> Option<SignalDefinitionTransitionReference> {
        self.predecessor
    }
    pub fn image_bytes(&self) -> &[u8] {
        &self.bytes.bytes()[self.image.clone()]
    }
    pub fn bridge_bytes(&self) -> &[u8] {
        &self.bytes.bytes()[self.bridge.clone()]
    }
    pub fn reference(&self) -> SignalDefinitionTransitionReference {
        SignalDefinitionTransitionReference::from_recorded(
            self.identity,
            Sha256::digest(self.as_bytes()).into(),
        )
    }
}
impl worth_foundational::facade::DurableArtifactRetention for SignalDefinitionTransitionArtifact {
    const RETENTION_CLASS: worth_foundational::facade::RetentionClass =
        worth_foundational::facade::RetentionClass::Authoritative;
}
