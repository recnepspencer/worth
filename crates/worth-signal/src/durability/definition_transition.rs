//! Signal declarations and Bridge's opaque authored section move together.
use super::{
    frame, SignalAuthoredImage, SignalDurableFrameDenial as D, SignalRuntimeAuthoredImage,
};
use sha2::{Digest, Sha256};
use std::ops::Range;
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest};
use worth_foundational::facade::{
    DurableSignalBranchIdentity, DurableSignalConditionalDefinitionOccurrence,
    SignalDefinitionTransitionIdentity, SignalDefinitionTransitionReference,
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
    digest: [u8; 32],
    identity: SignalDefinitionTransitionIdentity,
    branch: DurableSignalBranchIdentity,
    predecessor: Option<SignalDefinitionTransitionReference>,
    image: Range<usize>,
    runtime: Range<usize>,
    conditional: Range<usize>,
    bridge: Range<usize>,
}
impl SignalDefinitionTransitionArtifact {
    pub fn capture(
        execution: &ExecutionRequest<'_, '_>,
        identity: SignalDefinitionTransitionIdentity,
        branch: DurableSignalBranchIdentity,
        predecessor: Option<SignalDefinitionTransitionReference>,
        image: &SignalAuthoredImage,
        runtime: &SignalRuntimeAuthoredImage,
        conditional: &[DurableSignalConditionalDefinitionOccurrence],
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
        if image.branch().owner() != identity.owner()
            || runtime.branch().owner() != identity.owner()
        {
            return Err(D::ForeignOwner);
        }
        if image.branch() != branch || runtime.branch() != branch {
            return Err(D::ForeignBranch);
        }
        let mut previous = None;
        for row in conditional {
            if row.node().branch().owner() != identity.owner() {
                return Err(D::ForeignOwner);
            }
            if row.node().branch() != branch {
                return Err(D::ForeignBranch);
            }
            if row.occurrence().ordinal() == 0 || previous.is_some_and(|node| node >= row.node()) {
                return Err(D::Malformed);
            }
            previous = Some(row.node());
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
            out.blob(runtime.as_bytes())?;
            out.number(conditional.len() as u64)?;
            for row in conditional {
                out.conditional_definition(*row)?;
            }
            out.blob(bridge.as_bytes())
        })?;
        frame::admit_work(execution, bytes.bytes().len().saturating_mul(2))?;
        Self::readmit_owned(bytes)
    }
    pub fn readmit_bytes(execution: &ExecutionRequest<'_, '_>, bytes: &[u8]) -> Result<Self, D> {
        frame::admit_work(execution, bytes.len().saturating_mul(3))?;
        let fields = Self::inspect(bytes)?;
        Ok(Self::from_fields(
            frame::copy_bytes(execution, bytes)?,
            fields,
        ))
    }
    pub(super) fn readmit_owned(bytes: ExecutionImmutableBytes) -> Result<Self, D> {
        let fields = Self::inspect(bytes.bytes())?;
        Ok(Self::from_fields(bytes, fields))
    }
    fn from_fields(bytes: ExecutionImmutableBytes, fields: DefinitionFields) -> Self {
        let DefinitionFields {
            identity,
            branch,
            predecessor,
            image,
            runtime,
            conditional,
            bridge,
        } = fields;
        Self {
            digest: Sha256::digest(bytes.bytes()).into(),
            bytes,
            identity,
            branch,
            predecessor,
            image,
            runtime,
            conditional,
            bridge,
        }
    }
    fn inspect(bytes: &[u8]) -> Result<DefinitionFields, D> {
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
        let image_branch = super::authored_image::validate_bytes(image_bytes).map_err(D::Image)?;
        let image = (input.position - image_bytes.len())..input.position;
        let runtime_bytes = input.blob()?;
        let runtime_branch =
            super::authored_image::validate_runtime_bytes(runtime_bytes).map_err(D::Image)?;
        let runtime = (input.position - runtime_bytes.len())..input.position;
        if image_branch.owner() != identity.owner() || runtime_branch.owner() != identity.owner() {
            return Err(D::ForeignOwner);
        }
        if image_branch != branch || runtime_branch != branch {
            return Err(D::ForeignBranch);
        }
        let count: usize = input.number()?.try_into().map_err(|_| D::SizeLimit)?;
        if count > input.bytes.len() / 80 {
            return Err(D::Malformed);
        }
        let start = input.position;
        let mut previous = None;
        for _ in 0..count {
            let row = input.conditional_definition()?;
            if row.node().branch().owner() != identity.owner() {
                return Err(D::ForeignOwner);
            }
            if row.node().branch() != branch {
                return Err(D::ForeignBranch);
            }
            if previous.is_some_and(|node| node >= row.node()) {
                return Err(D::Malformed);
            }
            previous = Some(row.node());
        }
        let conditional = start..input.position;
        let bridge_bytes = input.blob()?;
        if bridge_bytes.len() > 16 << 20 {
            return Err(D::SizeLimit);
        }
        let bridge = (input.position - bridge_bytes.len())..input.position;
        input.finish()?;
        Ok(DefinitionFields {
            identity,
            branch,
            predecessor,
            image,
            runtime,
            conditional,
            bridge,
        })
    }
    pub fn as_bytes(&self) -> &[u8] {
        self.bytes.bytes()
    }
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
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
    pub fn runtime_bytes(&self) -> &[u8] {
        &self.bytes.bytes()[self.runtime.clone()]
    }
    pub fn authored_image(&self) -> SignalAuthoredImage {
        SignalAuthoredImage::from_frame_bytes(self.bytes.clone(), self.image.clone(), self.branch)
    }
    pub fn authored_runtime_image(&self) -> SignalRuntimeAuthoredImage {
        SignalRuntimeAuthoredImage::from_frame_bytes(
            self.bytes.clone(),
            self.runtime.clone(),
            self.branch,
        )
    }
    pub fn conditional_occurrences(
        &self,
    ) -> impl ExactSizeIterator<Item = DurableSignalConditionalDefinitionOccurrence> + '_ {
        self.bytes.bytes()[self.conditional.clone()]
            .chunks_exact(80)
            .map(|bytes| {
                let mut input = frame::Reader { bytes, position: 0 };
                input
                    .conditional_definition()
                    .expect("validated immutable conditional occurrence")
            })
    }
    pub fn bridge_bytes(&self) -> &[u8] {
        &self.bytes.bytes()[self.bridge.clone()]
    }
    pub fn reference(&self) -> SignalDefinitionTransitionReference {
        SignalDefinitionTransitionReference::from_recorded(self.identity, self.digest)
    }
}
struct DefinitionFields {
    identity: SignalDefinitionTransitionIdentity,
    branch: DurableSignalBranchIdentity,
    predecessor: Option<SignalDefinitionTransitionReference>,
    image: Range<usize>,
    runtime: Range<usize>,
    conditional: Range<usize>,
    bridge: Range<usize>,
}
impl worth_foundational::facade::DurableArtifactRetention for SignalDefinitionTransitionArtifact {
    const RETENTION_CLASS: worth_foundational::facade::RetentionClass =
        worth_foundational::facade::RetentionClass::Authoritative;
}
