//! Verify exact immutable root custody at the history catalog boundary.
use std::sync::Arc;

use super::{
    RelationalCommitArtifact, RelationalCommitArtifactDenial, RelationalCommitRootDescriptor,
};
use crate::branch::root::{RelationalBranchRoot, RelationalRootCommitEvidence};
use crate::history::data::canonical_commit_envelope::CanonicalCommitEnvelope;

impl RelationalCommitArtifact {
    pub fn from_envelope_with_root(
        envelope: Arc<CanonicalCommitEnvelope>,
        branch_root: Arc<RelationalBranchRoot>,
    ) -> Result<Self, RelationalCommitArtifactDenial> {
        if !branch_root.links_envelope(&envelope) {
            return Err(RelationalCommitArtifactDenial::RootLinkage);
        }
        let canonical_payload = branch_root
            .canonical_payload()
            .ok_or(RelationalCommitArtifactDenial::RootLinkage)?;
        let mut artifact = Self::from_preencoded(canonical_payload)?;
        artifact.roots = branch_root
            .descriptor()
            .cloned()
            .ok_or(RelationalCommitArtifactDenial::RootLinkage)?;
        Ok(artifact)
    }

    /// Recovery first seals the canonical payload while admitting durable
    /// envelopes. Relinking checkpoint roots must not serialize it again.
    pub fn relink_preencoded_recovery(
        &self,
        envelope: &Arc<CanonicalCommitEnvelope>,
        root: Option<&Arc<RelationalBranchRoot>>,
        descriptor: Option<&RelationalCommitRootDescriptor>,
    ) -> Result<Self, RelationalCommitArtifactDenial> {
        if self
            .resident_envelope()
            .is_none_or(|resident| !Arc::ptr_eq(&resident, envelope))
        {
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
            schema_version: self.schema_version,
            original_inputs: self.original_inputs.clone(),
            allocation_memory: None,
            envelope: Arc::clone(&self.envelope),
        })
    }

    pub fn links_root(&self, root: &Arc<RelationalBranchRoot>) -> bool {
        root.descriptor() == Some(&self.roots)
            && match root.commit_evidence() {
                Some(RelationalRootCommitEvidence::Original(_)) => self
                    .envelope
                    .get()
                    .is_some_and(|envelope| root.links_envelope(&envelope)),
                Some(RelationalRootCommitEvidence::Checkpoint(commit)) => {
                    commit.links_artifact(self)
                }
                None => false,
            }
    }
}
