mod original_inputs;
mod readmission;
mod reconstructed_inputs;
pub(crate) use original_inputs::OriginalCanonicalInputs;
mod retained_entity_selection;
#[path = "../../branch/root_artifact_linkage.rs"]
mod root_linkage;
pub use retained_entity_selection::RelationalRetainedSelectionArtifactDenial;

use std::sync::Arc;

use crate::history::data::{CanonicalCommitEnvelope, CommitId};
use crate::identity::data::VersionId;

use super::{RelationalCommitIdentity, RelationalCommitParentage};

/// Descriptive roots carried by a commit identity before the Phase-5 branch
/// root becomes the read/currentness authority. The branch target owns the
/// concrete vocabulary so commit and reference observations cannot drift.
pub type RelationalCommitRootDescriptor = crate::branch::target::RelationalBranchRootDescriptor;

/// Sealed immutable catalog artifact. The envelope is held behind an `Arc` and
/// is never exposed mutably after catalog insertion.
#[derive(Debug)]
pub(crate) struct RelationalCommitArtifact {
    identity: RelationalCommitIdentity,
    parentage: RelationalCommitParentage,
    roots: RelationalCommitRootDescriptor,
    schema_version: crate::schema::data::SchemaVersionId,
    envelope: Arc<super::RelationalCommitEnvelopeResidency>,
    original_inputs: Option<OriginalCanonicalInputs>,
    allocation_memory: Option<worth_execution::ExecutionMemoryReservation>,
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

pub use super::artifact_denial::RelationalCommitArtifactDenial;

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
        retained_entity_selection::validate(envelope)?;
        RelationalCommitParentage::from_ordered(envelope.commit.parents.clone())
            .map(|_| ())
            .map_err(|_| RelationalCommitArtifactDenial::Parentage)?;
        // Encoding belongs to root preparation, before effects. Artifact sealing
        // consumes that exact immutable payload instead of encoding twice.
        Ok(())
    }

    pub(crate) fn from_preencoded(
        original: Arc<crate::branch::RelationalOriginalRootCommitEvidence>,
    ) -> Result<Self, RelationalCommitArtifactDenial> {
        let envelope = original.envelope();
        retained_entity_selection::validate(envelope)?;
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
            roots: crate::branch::target::RelationalBranchRootDescriptor::new(
                *crate::branch::target::RelationalBranchTarget::roots_for_commit(&envelope.commit)
                    .truth_root(),
                crate::schema::data::schema_authority_snapshot_digest_bytes(
                    &envelope.schema_authority,
                ),
            ),
            schema_version: envelope.schema_version,
            original_inputs: None,
            allocation_memory: None,
            envelope: Arc::new(super::RelationalCommitEnvelopeResidency::resident(original)),
        })
    }

    pub(crate) fn from_preencoded_with_descriptor(
        original: Arc<crate::branch::RelationalOriginalRootCommitEvidence>,
        roots: RelationalCommitRootDescriptor,
    ) -> Result<Self, RelationalCommitArtifactDenial> {
        let envelope = original.envelope();
        if roots.schema_root()
            != &crate::schema::data::schema_authority_snapshot_digest_bytes(
                &envelope.schema_authority,
            )
        {
            return Err(RelationalCommitArtifactDenial::RootLinkage);
        }
        let mut artifact = Self::from_preencoded(original)?;
        artifact.roots = roots;
        Ok(artifact)
    }

    pub(crate) fn from_index(
        entry: super::RetainedCanonicalIndex,
        source: Arc<dyn crate::durability::medium::RelationalDurableColdSource>,
    ) -> Result<Self, RelationalCommitArtifactDenial> {
        let parentage = RelationalCommitParentage::from_ordered(entry.commit.parents.clone())
            .map_err(|_| RelationalCommitArtifactDenial::Parentage)?;
        Ok(Self {
            identity: RelationalCommitIdentity::new(
                entry.commit.commit_id,
                entry.commit.version_id,
                entry.commit.branch_id.clone(),
            ),
            parentage,
            roots: entry.root.clone(),
            schema_version: entry.schema_version,
            original_inputs: None,
            allocation_memory: None,
            envelope: Arc::new(super::RelationalCommitEnvelopeResidency::cold(
                entry, source, None, None,
            )),
        })
    }

    pub(crate) fn envelope_residency(&self) -> Arc<super::RelationalCommitEnvelopeResidency> {
        self.envelope.clone()
    }

    pub(crate) fn resident_envelope(&self) -> Option<Arc<CanonicalCommitEnvelope>> {
        self.envelope.get()
    }

    pub(crate) fn demand_envelope(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<
        Arc<CanonicalCommitEnvelope>,
        crate::storage::residency::RelationalResidencyDemandFailure,
    > {
        self.envelope.demand(execution, limit)
    }

    pub(crate) fn schema_version(&self) -> crate::schema::data::SchemaVersionId {
        self.schema_version
    }

    pub(crate) fn receipt(&self) -> crate::history::data::RelationalCommitReceipt {
        crate::history::data::RelationalCommitReceipt {
            commit_id: self.commit_id(),
            version_id: self.version_id(),
            branch_id: self.identity.authoring_branch().clone(),
            parents: self.parentage.as_slice().to_vec(),
        }
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

    #[cfg(test)]
    pub(crate) fn canonical_payload_bytes(&self) -> u64 {
        self.envelope.resident_payload_bytes().unwrap_or(0)
    }

    pub(crate) fn canonical_payload_digest(
        &self,
    ) -> Result<[u8; 32], RelationalCommitArtifactDenial> {
        self.envelope
            .payload_digest()
            .ok_or(RelationalCommitArtifactDenial::CanonicalPayloadEncoding)
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
                authoritative_bytes: self.envelope.payload_storage_bytes(),
            },
            RelationalCommitAuthoritativeAllocationObservation {
                kind: RelationalCommitAuthoritativeAllocationKind::EnvelopeObject,
                authoritative_bytes: self
                    .envelope
                    .get()
                    .map_or(0, |_| std::mem::size_of::<CanonicalCommitEnvelope>() as u64),
            },
            RelationalCommitAuthoritativeAllocationObservation {
                kind: RelationalCommitAuthoritativeAllocationKind::EnvelopeNestedOwnerStorage,
                authoritative_bytes: self.envelope.get().map_or(0, |envelope| {
                    envelope.allocation_inventory().authoritative_nested_bytes
                }),
            },
        ]
    }

    pub(crate) fn excluded_allocation_inventory(
        &self,
    ) -> RelationalCommitExcludedAllocationInventory {
        let Some(envelope) = self.envelope.get() else {
            return RelationalCommitExcludedAllocationInventory::default();
        };
        let inventory = envelope.allocation_inventory();
        RelationalCommitExcludedAllocationInventory {
            diagnostic_bytes: inventory.diagnostic_bytes,
            optional_cache_bytes: inventory.optional_cache_bytes,
        }
    }

    pub(crate) fn commit_id(&self) -> CommitId {
        self.identity.commit_id()
    }

    pub(crate) fn version_id(&self) -> VersionId {
        self.identity.version_id()
    }
}

#[cfg(test)]
#[path = "artifact_tests.rs"]
mod tests;
