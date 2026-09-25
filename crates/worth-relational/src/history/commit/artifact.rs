use std::sync::Arc;

use crate::history::data::{CanonicalCommitEnvelope, CommitId};
use crate::identity::data::VersionId;

use super::{RelationalCommitIdentity, RelationalCommitParentage};

/// Descriptive roots carried by a commit identity before the Phase-5 branch
/// root becomes the read/currentness authority. The branch target owns the
/// concrete vocabulary so commit and reference observations cannot drift.
pub type RelationalCommitRootDescriptor = crate::branch::RelationalBranchRootDescriptor;

/// Sealed immutable catalog artifact. The envelope is held behind an `Arc` and
/// is never exposed mutably after catalog insertion.
#[derive(Debug)]
pub(crate) struct RelationalCommitArtifact {
    identity: RelationalCommitIdentity,
    parentage: RelationalCommitParentage,
    roots: RelationalCommitRootDescriptor,
    envelope: Arc<CanonicalCommitEnvelope>,
    canonical_payload: Arc<[u8]>,
}

// The Phase-4 fork path may share an immutable catalog handle, but it may
// never deep-clone the sealed artifact.  This deliberately overlapping
// implementation is a compile-time tripwire: adding `Clone` to the artifact
// makes the two implementations conflict and fails the crate before a cost
// regression can hide behind a zero counter.
#[allow(dead_code)]
trait RelationalCommitArtifactCloneTripwire {}

#[allow(dead_code)]
impl RelationalCommitArtifactCloneTripwire for RelationalCommitArtifact {}

#[allow(dead_code)]
impl<T: Clone> RelationalCommitArtifactCloneTripwire for T {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalCommitArtifactDenial {
    Parentage,
    RootLinkage,
    CanonicalPayloadEncoding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum RelationalCommitAuthoritativeAllocationKind {
    ArtifactObject,
    CanonicalPayload,
    EnvelopeObject,
    EnvelopeNestedOwnerStorage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RelationalCommitAuthoritativeAllocationObservation {
    pub(crate) kind: RelationalCommitAuthoritativeAllocationKind,
    pub(crate) authoritative_bytes: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct RelationalCommitExcludedAllocationInventory {
    pub(crate) diagnostic_bytes: u64,
    pub(crate) optional_cache_bytes: u64,
}

impl RelationalCommitArtifact {
    pub(super) fn validate_envelope(
        envelope: &CanonicalCommitEnvelope,
    ) -> Result<(), RelationalCommitArtifactDenial> {
        RelationalCommitParentage::from_ordered(envelope.commit.parents.clone())
            .map(|_| ())
            .map_err(|_| RelationalCommitArtifactDenial::Parentage)?;
        encode_canonical_payload(envelope).map(|_| ())
    }

    pub(super) fn from_envelope(
        envelope: Arc<CanonicalCommitEnvelope>,
    ) -> Result<Self, RelationalCommitArtifactDenial> {
        let canonical_payload = encode_canonical_payload(&envelope)?;
        let identity = RelationalCommitIdentity::new(
            envelope.commit.commit_id,
            envelope.commit.version_id,
            envelope.branch_context.clone(),
        );
        let parentage = RelationalCommitParentage::from_ordered(envelope.commit.parents.clone())
            .map_err(|_| RelationalCommitArtifactDenial::Parentage)?;
        Ok(Self {
            identity,
            parentage,
            roots: crate::branch::RelationalBranchRootDescriptor::new(
                *crate::branch::RelationalBranchTarget::roots_for_commit(&envelope.commit)
                    .truth_root(),
                crate::schema::data::schema_authority_snapshot_digest_bytes(
                    &envelope.schema_authority,
                ),
            ),
            envelope,
            canonical_payload,
        })
    }

    pub(crate) fn from_envelope_with_root(
        envelope: Arc<CanonicalCommitEnvelope>,
        branch_root: Arc<crate::branch::RelationalBranchRoot>,
    ) -> Result<Self, RelationalCommitArtifactDenial> {
        if !branch_root.links_envelope(&envelope) {
            return Err(RelationalCommitArtifactDenial::RootLinkage);
        }
        let mut artifact = Self::from_envelope(envelope)?;
        artifact.roots = branch_root
            .descriptor()
            .cloned()
            .ok_or(RelationalCommitArtifactDenial::RootLinkage)?;
        Ok(artifact)
    }

    pub(super) fn from_envelope_with_descriptor(
        envelope: Arc<CanonicalCommitEnvelope>,
        roots: RelationalCommitRootDescriptor,
    ) -> Result<Self, RelationalCommitArtifactDenial> {
        if roots.schema_root()
            != &crate::schema::data::schema_authority_snapshot_digest_bytes(
                &envelope.schema_authority,
            )
        {
            return Err(RelationalCommitArtifactDenial::RootLinkage);
        }
        let mut artifact = Self::from_envelope(envelope)?;
        artifact.roots = roots;
        Ok(artifact)
    }

    /// Recovery first seals the canonical payload while admitting durable
    /// envelopes. Relinking checkpoint roots must not serialize it again.
    pub(crate) fn relink_preencoded_recovery(
        &self,
        envelope: &Arc<CanonicalCommitEnvelope>,
        root: Option<&Arc<crate::branch::RelationalBranchRoot>>,
        descriptor: Option<&RelationalCommitRootDescriptor>,
    ) -> Result<Self, RelationalCommitArtifactDenial> {
        if !Arc::ptr_eq(&self.envelope, envelope) {
            return Err(RelationalCommitArtifactDenial::RootLinkage);
        }
        let roots = if let Some(root) = root {
            if !root.links_envelope(envelope) {
                return Err(RelationalCommitArtifactDenial::RootLinkage);
            }
            root.descriptor()
                .cloned()
                .ok_or(RelationalCommitArtifactDenial::RootLinkage)?
        } else if let Some(descriptor) = descriptor {
            if descriptor.schema_root()
                != &crate::schema::data::schema_authority_snapshot_digest_bytes(
                    &envelope.schema_authority,
                )
            {
                return Err(RelationalCommitArtifactDenial::RootLinkage);
            }
            descriptor.clone()
        } else {
            self.roots.clone()
        };
        Ok(Self {
            identity: self.identity.clone(),
            parentage: self.parentage.clone(),
            roots,
            envelope: Arc::clone(envelope),
            canonical_payload: Arc::clone(&self.canonical_payload),
        })
    }

    pub fn identity(&self) -> &RelationalCommitIdentity {
        &self.identity
    }

    pub fn parentage(&self) -> &RelationalCommitParentage {
        &self.parentage
    }

    pub fn roots(&self) -> &RelationalCommitRootDescriptor {
        &self.roots
    }

    pub(crate) fn envelope(&self) -> &Arc<CanonicalCommitEnvelope> {
        &self.envelope
    }

    pub(crate) fn canonical_payload_bytes(&self) -> u64 {
        self.canonical_payload.len() as u64
    }

    pub(crate) fn canonical_payload_digest(&self) -> [u8; 32] {
        use sha2::{Digest, Sha256};

        Sha256::digest(self.canonical_payload.as_ref()).into()
    }

    pub(crate) fn authoritative_allocation_observations(
        &self,
    ) -> [RelationalCommitAuthoritativeAllocationObservation; 4] {
        [
            RelationalCommitAuthoritativeAllocationObservation {
                kind: RelationalCommitAuthoritativeAllocationKind::ArtifactObject,
                authoritative_bytes: std::mem::size_of::<Self>() as u64,
            },
            RelationalCommitAuthoritativeAllocationObservation {
                kind: RelationalCommitAuthoritativeAllocationKind::CanonicalPayload,
                authoritative_bytes: self.canonical_payload_bytes(),
            },
            RelationalCommitAuthoritativeAllocationObservation {
                kind: RelationalCommitAuthoritativeAllocationKind::EnvelopeObject,
                authoritative_bytes: std::mem::size_of::<CanonicalCommitEnvelope>() as u64,
            },
            RelationalCommitAuthoritativeAllocationObservation {
                kind: RelationalCommitAuthoritativeAllocationKind::EnvelopeNestedOwnerStorage,
                authoritative_bytes: self
                    .envelope
                    .allocation_inventory()
                    .authoritative_nested_bytes,
            },
        ]
    }

    pub(crate) fn excluded_allocation_inventory(
        &self,
    ) -> RelationalCommitExcludedAllocationInventory {
        let inventory = self.envelope.allocation_inventory();
        RelationalCommitExcludedAllocationInventory {
            diagnostic_bytes: inventory.diagnostic_bytes,
            optional_cache_bytes: inventory.optional_cache_bytes,
        }
    }

    pub(crate) fn links_root(&self, root: &Arc<crate::branch::RelationalBranchRoot>) -> bool {
        root.descriptor() == Some(&self.roots) && root.links_envelope(&self.envelope)
    }

    pub(crate) fn commit_id(&self) -> CommitId {
        self.identity.commit_id()
    }

    pub(crate) fn version_id(&self) -> VersionId {
        self.identity.version_id()
    }
}

fn encode_canonical_payload(
    envelope: &CanonicalCommitEnvelope,
) -> Result<Arc<[u8]>, RelationalCommitArtifactDenial> {
    envelope
        .encode_authoritative_payload()
        .map(Arc::from)
        .map_err(|_| RelationalCommitArtifactDenial::CanonicalPayloadEncoding)
}

#[cfg(test)]
#[path = "artifact_tests.rs"]
mod tests;
