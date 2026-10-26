use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::history::data::CommitId;
use crate::identity::data::VersionId;

use super::{RelationalCommitArtifact, RelationalCommitArtifactDenial, RelationalCommitIdentity};

/// Append-only immutable commit lookup. Branch references are intentionally
/// not stored here.
#[derive(Debug)]
pub(crate) struct RelationalCommitCatalog {
    entries: BTreeMap<CommitId, Arc<RelationalCommitArtifact>>,
    materializations: Arc<AtomicU64>,
    lookups: Arc<AtomicU64>,
}

impl Clone for RelationalCommitCatalog {
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
            // Catalog entries are immutable and may be shared across runtime
            // forks, but instrumentation is runtime-local. A fork inherits
            // the observed baseline without sharing future increments.
            materializations: Arc::new(AtomicU64::new(0)),
            lookups: Arc::new(AtomicU64::new(0)),
        }
    }
}

impl Default for RelationalCommitCatalog {
    fn default() -> Self {
        Self {
            entries: BTreeMap::new(),
            materializations: Arc::new(AtomicU64::new(0)),
            lookups: Arc::new(AtomicU64::new(0)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalCommitCatalogAppendDenial {
    DuplicateCommit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RelationalCommitCatalogEnvelopeAppendDenial {
    Artifact(RelationalCommitArtifactDenial),
    DuplicateCommit,
}

#[derive(Debug, Clone)]
pub struct RelationalCommitCatalogEntry {
    identity: RelationalCommitIdentity,
}

impl RelationalCommitCatalogEntry {
    pub fn identity(&self) -> &RelationalCommitIdentity {
        &self.identity
    }
}

impl RelationalCommitCatalog {
    pub(crate) fn install_prepared(&mut self, artifact: RelationalCommitArtifact) {
        let commit_id = artifact.commit_id();
        assert!(
            !self.entries.contains_key(&commit_id),
            "prepared catalog artifact was validated as unique"
        );
        self.materializations.fetch_add(1, Ordering::Relaxed);
        self.entries.insert(commit_id, Arc::new(artifact));
    }

    pub(crate) fn install_prepared_recovery(&mut self, artifact: RelationalCommitArtifact) {
        let commit_id = artifact.commit_id();
        debug_assert!(self.entries.get(&commit_id).is_none_or(|existing| existing
            .resident_envelope()
            .is_none_or(|envelope| artifact.resident_envelope() == Some(envelope))));
        self.materializations.fetch_add(1, Ordering::Relaxed);
        self.entries.insert(commit_id, Arc::new(artifact));
    }

    pub(crate) fn synchronize_recovered_envelope(
        &mut self,
        envelope: Arc<crate::history::data::CanonicalCommitEnvelope>,
    ) -> Result<(), RelationalCommitArtifactDenial> {
        let commit_id = envelope.commit.commit_id;
        if self
            .entries
            .get(&commit_id)
            .is_some_and(|existing| existing.resident_envelope() == Some(&envelope))
        {
            return Ok(());
        }
        let artifact = match self.entries.get(&commit_id) {
            Some(existing) => RelationalCommitArtifact::from_envelope_with_descriptor(
                envelope,
                existing.roots().clone(),
            )?,
            None => RelationalCommitArtifact::from_envelope(envelope)?,
        };
        self.materializations.fetch_add(1, Ordering::Relaxed);
        self.entries.insert(commit_id, Arc::new(artifact));
        Ok(())
    }

    fn append(
        &mut self,
        artifact: RelationalCommitArtifact,
    ) -> Result<RelationalCommitCatalogEntry, RelationalCommitCatalogAppendDenial> {
        let commit_id = artifact.commit_id();
        if self.entries.contains_key(&commit_id) {
            return Err(RelationalCommitCatalogAppendDenial::DuplicateCommit);
        }
        let identity = artifact.identity().clone();
        self.entries.insert(commit_id, Arc::new(artifact));
        Ok(RelationalCommitCatalogEntry { identity })
    }

    pub(crate) fn append_envelope(
        &mut self,
        envelope: Arc<crate::history::data::CanonicalCommitEnvelope>,
    ) -> Result<RelationalCommitCatalogEntry, RelationalCommitCatalogEnvelopeAppendDenial> {
        let artifact = RelationalCommitArtifact::from_envelope(envelope)
            .map_err(RelationalCommitCatalogEnvelopeAppendDenial::Artifact)?;
        self.materializations.fetch_add(1, Ordering::Relaxed);
        self.append(artifact)
            .map_err(|_| RelationalCommitCatalogEnvelopeAppendDenial::DuplicateCommit)
    }

    pub(crate) fn append_preencoded_recovery(
        &mut self,
        artifact: RelationalCommitArtifact,
    ) -> Result<RelationalCommitCatalogEntry, RelationalCommitCatalogAppendDenial> {
        self.materializations.fetch_add(1, Ordering::Relaxed);
        self.append(artifact)
    }

    pub(crate) fn append_indexed_recovery(
        &mut self,
        entry: super::RelationalCommitIndexEntry,
        source: Arc<dyn crate::durability::medium::RelationalDurableColdSource>,
        admitted: Option<Arc<crate::history::data::CanonicalCommitEnvelope>>,
    ) -> Result<(), RelationalCommitCatalogEnvelopeAppendDenial> {
        let artifact = RelationalCommitArtifact::from_index(entry, source, admitted)
            .map_err(RelationalCommitCatalogEnvelopeAppendDenial::Artifact)?;
        self.append(artifact)
            .map(|_| ())
            .map_err(|_| RelationalCommitCatalogEnvelopeAppendDenial::DuplicateCommit)
    }

    /// Preserve index-only facts while recovery relinks the resident heads.
    pub(crate) fn retained_cold_entries(&self) -> Self {
        Self {
            entries: self
                .entries
                .iter()
                .filter(|(_, artifact)| artifact.resident_envelope().is_none())
                .map(|(id, artifact)| (*id, Arc::clone(artifact)))
                .collect(),
            materializations: Arc::new(AtomicU64::new(0)),
            lookups: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Checkpoint recovery has already readmitted each live root from its
    /// durable image. Seal each canonical envelope once, with its exact root
    /// or retained fork descriptor, instead of building a provisional catalog.
    pub(crate) fn append_checkpoint_recovery(
        &mut self,
        envelope: Arc<crate::history::data::CanonicalCommitEnvelope>,
        root: Option<&Arc<crate::branch::RelationalBranchRoot>>,
        descriptor: Option<&crate::branch::RelationalBranchRootDescriptor>,
    ) -> Result<RelationalCommitCatalogEntry, RelationalCommitCatalogEnvelopeAppendDenial> {
        let artifact = match (root, descriptor) {
            (Some(root), _) => {
                RelationalCommitArtifact::from_envelope_with_root(envelope, Arc::clone(root))
            }
            (None, Some(descriptor)) => RelationalCommitArtifact::from_envelope_with_descriptor(
                envelope,
                descriptor.clone(),
            ),
            (None, None) => RelationalCommitArtifact::from_envelope(envelope),
        }
        .map_err(RelationalCommitCatalogEnvelopeAppendDenial::Artifact)?;
        self.materializations.fetch_add(1, Ordering::Relaxed);
        self.append(artifact)
            .map_err(|_| RelationalCommitCatalogEnvelopeAppendDenial::DuplicateCommit)
    }

    /// Validate an envelope without materializing or mutating the catalog.
    /// Publication uses this side-effect-free court before durable append so
    /// an invalid artifact cannot fail after storage or catalog effects.
    pub(crate) fn validate_envelope(
        &self,
        envelope: &crate::history::data::CanonicalCommitEnvelope,
    ) -> Result<(), RelationalCommitCatalogEnvelopeAppendDenial> {
        if let Some(existing) = self.entries.get(&envelope.commit.commit_id) {
            return match existing.resident_envelope() {
                Some(existing) if existing.as_ref() == envelope => Ok(()),
                _ => Err(RelationalCommitCatalogEnvelopeAppendDenial::DuplicateCommit),
            };
        }
        RelationalCommitArtifact::validate_envelope(envelope)
            .map_err(RelationalCommitCatalogEnvelopeAppendDenial::Artifact)
    }

    /// Validate an envelope readmitted outside any catalog: with no prior
    /// entry to compare, only its own commit-artifact invariants apply.
    pub(crate) fn validate_detached_envelope(
        envelope: &crate::history::data::CanonicalCommitEnvelope,
    ) -> Result<(), RelationalCommitArtifactDenial> {
        RelationalCommitArtifact::validate_envelope(envelope)
    }

    pub(crate) fn validate_new_envelope(
        &self,
        envelope: &crate::history::data::CanonicalCommitEnvelope,
    ) -> Result<(), RelationalCommitCatalogEnvelopeAppendDenial> {
        if self.entries.contains_key(&envelope.commit.commit_id) {
            return Err(RelationalCommitCatalogEnvelopeAppendDenial::DuplicateCommit);
        }
        RelationalCommitArtifact::validate_envelope(envelope)
            .map_err(RelationalCommitCatalogEnvelopeAppendDenial::Artifact)
    }

    pub(crate) fn validate_envelope_against(
        existing: Option<&crate::history::data::CanonicalCommitEnvelope>,
        envelope: &crate::history::data::CanonicalCommitEnvelope,
    ) -> Result<(), RelationalCommitCatalogEnvelopeAppendDenial> {
        if let Some(existing) = existing {
            return (existing == envelope)
                .then_some(())
                .ok_or(RelationalCommitCatalogEnvelopeAppendDenial::DuplicateCommit);
        }
        RelationalCommitArtifact::validate_envelope(envelope)
            .map_err(RelationalCommitCatalogEnvelopeAppendDenial::Artifact)
    }

    pub(crate) fn validate_new_envelope_against(
        existing: Option<&crate::history::data::CanonicalCommitEnvelope>,
        envelope: &crate::history::data::CanonicalCommitEnvelope,
    ) -> Result<(), RelationalCommitCatalogEnvelopeAppendDenial> {
        if existing.is_some() {
            return Err(RelationalCommitCatalogEnvelopeAppendDenial::DuplicateCommit);
        }
        RelationalCommitArtifact::validate_envelope(envelope)
            .map_err(RelationalCommitCatalogEnvelopeAppendDenial::Artifact)
    }

    pub(crate) fn materialization_count(&self) -> u64 {
        self.materializations.load(Ordering::Relaxed)
    }

    pub(crate) fn lookup_count(&self) -> u64 {
        self.lookups.load(Ordering::Relaxed)
    }

    pub(crate) fn get(&self, commit_id: CommitId) -> Option<&Arc<RelationalCommitArtifact>> {
        self.lookups.fetch_add(1, Ordering::Relaxed);
        self.entries.get(&commit_id)
    }

    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    pub(crate) fn latest_identity(&self) -> Option<&RelationalCommitIdentity> {
        self.entries
            .values()
            .max_by_key(|artifact| artifact.commit_id())
            .map(|artifact| artifact.identity())
    }

    pub(crate) fn latest_artifact(&self) -> Option<&Arc<RelationalCommitArtifact>> {
        self.entries
            .values()
            .max_by_key(|artifact| artifact.commit_id())
    }

    pub(crate) fn find_by_version(
        &self,
        version_id: VersionId,
    ) -> Option<&Arc<RelationalCommitArtifact>> {
        self.entries
            .values()
            .find(|artifact| artifact.version_id() == version_id)
    }

    pub(crate) fn snapshot(&self) -> Vec<Arc<RelationalCommitArtifact>> {
        self.entries.values().cloned().collect()
    }
}
