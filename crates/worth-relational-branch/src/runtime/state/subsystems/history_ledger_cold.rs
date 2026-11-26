//! Ledger answers over the resident catalog and its cold checkpoint base.
//!
//! Resident entries win wherever both answer: they are commits published or
//! recovered since reopen, or recovery overrides of selected heads. A later
//! checkpoint may cover resident commits, so positions are deduplicated by
//! counting resident entries only above the cold count.
use std::sync::Arc;

use worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;

use super::HistoryLedger;
use crate::history::commit::identity::RelationalCommitIdentityOwnerPort as _;
use crate::history::commit::{
    RelationalCommitArtifact, RelationalCommitCatalogAppendDenial,
    RelationalCommitCatalogEnvelopeAppendDenial, RelationalCommitIdentity,
};
use crate::history::data::{CanonicalCommitEnvelope, CommitId, RelationalCommitReceipt};
use crate::identity::data::VersionId;
use crate::publication::patch::data::PatchStreamPosition;

type Artifact = Arc<RelationalCommitArtifact>;

impl HistoryLedger {
    fn cold_count(&self) -> u64 {
        self.cold_catalog.as_ref().map_or(0, |cold| cold.count())
    }

    /// The catalog artifact for `commit`, faulting its cold page if needed.
    pub fn try_catalog_artifact(&self, commit: CommitId) -> Result<Option<Artifact>, Cause> {
        if let Some(artifact) = self.commit_catalog.get(commit) {
            return Ok(Some(artifact.clone()));
        }
        match &self.cold_catalog {
            Some(cold) => cold.artifact(commit),
            None => Ok(None),
        }
    }

    /// Read-only receipt/identity/version lookups: cold entries answer from
    /// the commit index without materializing into `ColdCommitCatalog::live`.
    pub fn try_catalog_receipt(
        &self,
        commit: CommitId,
    ) -> Result<Option<RelationalCommitReceipt>, Cause> {
        if let Some(artifact) = self.commit_catalog.get(commit) {
            return Ok(Some(artifact.receipt()));
        }
        Ok(match &self.cold_catalog {
            Some(cold) => cold.entry(commit)?.map(|entry| entry.commit),
            None => None,
        })
    }

    pub fn try_catalog_identity(
        &self,
        commit: CommitId,
    ) -> Result<Option<RelationalCommitIdentity>, Cause> {
        Ok(self.try_catalog_receipt(commit)?.map(|receipt| {
            RelationalCommitIdentity::new(receipt.commit_id, receipt.version_id, receipt.branch_id)
        }))
    }

    pub fn try_catalog_receipt_for_version(
        &self,
        version: VersionId,
    ) -> Result<Option<RelationalCommitReceipt>, Cause> {
        if let Some(artifact) = self.commit_catalog.find_by_version(version) {
            return Ok(Some(artifact.receipt()));
        }
        Ok(match &self.cold_catalog {
            Some(cold) => cold.index()?.by_version(version)?.map(|entry| entry.commit),
            None => None,
        })
    }

    pub fn try_latest_catalog_receipt(&self) -> Result<Option<RelationalCommitReceipt>, Cause> {
        let resident = self
            .commit_catalog
            .latest_artifact()
            .map(|artifact| artifact.receipt());
        let cold = match &self.cold_catalog {
            Some(cold) => {
                let index = cold.index()?;
                match index.max_commit() {
                    Some(commit) => index.by_commit(commit)?.map(|entry| entry.commit),
                    None => None,
                }
            }
            None => None,
        };
        Ok(match (resident, cold) {
            (Some(resident), Some(cold)) if cold.commit_id > resident.commit_id => Some(cold),
            (resident, cold) => resident.or(cold),
        })
    }

    /// As [`Self::try_catalog_artifact`]; an unreadable cold page answers as
    /// absent, which every caller already refuses as a missing commit.
    pub fn catalog_artifact(&self, commit: CommitId) -> Option<Artifact> {
        self.try_catalog_artifact(commit).ok().flatten()
    }

    pub fn latest_catalog_artifact(&self) -> Option<Artifact> {
        let resident = self.commit_catalog.latest_artifact().cloned();
        let cold = self
            .cold_catalog
            .as_ref()
            .and_then(|cold| cold.latest().ok().flatten());
        match (resident, cold) {
            (Some(resident), Some(cold)) if cold.commit_id() > resident.commit_id() => Some(cold),
            (resident, cold) => resident.or(cold),
        }
    }

    pub fn try_catalog_artifact_for_version(
        &self,
        version: VersionId,
    ) -> Result<Option<Artifact>, Cause> {
        if let Some(artifact) = self.commit_catalog.find_by_version(version) {
            return Ok(Some(artifact.clone()));
        }
        match &self.cold_catalog {
            Some(cold) => cold.artifact_for_version(version),
            None => Ok(None),
        }
    }

    pub fn catalog_artifact_for_version(&self, version: VersionId) -> Option<Artifact> {
        self.try_catalog_artifact_for_version(version)
            .ok()
            .flatten()
    }

    pub fn catalog_len(&self) -> usize {
        match &self.cold_catalog {
            None => self.commit_catalog.len(),
            Some(_) => usize::try_from(self.patch_position_count()).unwrap_or(usize::MAX),
        }
    }

    /// The resident catalog and every faulted cold artifact still held.
    pub fn resident_catalog_artifacts(&self) -> Vec<Artifact> {
        let mut artifacts = self.commit_catalog.snapshot();
        if let Some(cold) = &self.cold_catalog {
            artifacts.extend(
                cold.live_artifacts()
                    .into_iter()
                    .filter(|artifact| !self.commit_catalog.contains(artifact.commit_id())),
            );
            artifacts.sort_by_key(|artifact| artifact.commit_id());
        }
        artifacts
    }

    /// Every catalog artifact; whole-history consumers fault every cold page.
    pub fn try_catalog_artifacts(&self) -> Result<Vec<Artifact>, Cause> {
        let Some(cold) = &self.cold_catalog else {
            return Ok(self.commit_catalog.snapshot());
        };
        let mut artifacts = std::collections::BTreeMap::new();
        cold.for_each_artifact(|artifact| {
            artifacts.insert(artifact.commit_id(), artifact);
        })?;
        for artifact in self.commit_catalog.snapshot() {
            artifacts.insert(artifact.commit_id(), artifact);
        }
        Ok(artifacts.into_values().collect())
    }

    pub fn validate_envelope(
        &self,
        envelope: &CanonicalCommitEnvelope,
    ) -> Result<(), RelationalCommitCatalogEnvelopeAppendDenial> {
        let id = envelope.commit.commit_id;
        if self.commit_catalog.contains(id) {
            return self.commit_catalog.validate_envelope(envelope);
        }
        match self.cold_catalog.as_ref().map(|cold| cold.artifact(id)) {
            Some(Ok(Some(existing))) => match existing.resident_envelope() {
                Some(existing) if existing.as_ref() == envelope => Ok(()),
                _ => Err(RelationalCommitCatalogEnvelopeAppendDenial::DuplicateCommit),
            },
            Some(Err(_)) => Err(RelationalCommitCatalogEnvelopeAppendDenial::DuplicateCommit),
            Some(Ok(None)) | None => self.commit_catalog.validate_envelope(envelope),
        }
    }

    /// A new commit id must be absent from both tiers; an unreadable cold page
    /// cannot prove absence, so it refuses.
    pub fn validate_new_envelope(
        &self,
        envelope: &CanonicalCommitEnvelope,
    ) -> Result<(), RelationalCommitCatalogEnvelopeAppendDenial> {
        let id = envelope.commit.commit_id;
        if let Some(cold) = &self.cold_catalog
            && !matches!(cold.entry(id), Ok(None))
        {
            return Err(RelationalCommitCatalogEnvelopeAppendDenial::DuplicateCommit);
        }
        self.commit_catalog.validate_new_envelope(envelope)
    }

    pub fn try_source_replacement_matches(
        &self,
        id: CommitId,
        expected: Option<&Artifact>,
    ) -> Result<bool, Cause> {
        Ok(match (self.try_catalog_artifact(id)?, expected) {
            (None, None) => true,
            (Some(actual), Some(expected)) => Arc::ptr_eq(&actual, expected),
            _ => false,
        })
    }

    /// Fail-closed: an unreadable cold page never confirms absence.
    pub fn source_replacement_matches(&self, id: CommitId, expected: Option<&Artifact>) -> bool {
        self.try_source_replacement_matches(id, expected)
            .unwrap_or(false)
    }

    /// Installs a recovered source artifact; over a cold commit it becomes a
    /// resident override.
    pub fn install_source_recovery(
        &mut self,
        expected: Option<&Artifact>,
        artifact: Artifact,
    ) -> Result<(), RelationalCommitCatalogAppendDenial> {
        if !self.source_replacement_matches(artifact.commit_id(), expected) {
            return Err(RelationalCommitCatalogAppendDenial::ChangedDuringPreparation);
        }
        if expected.is_some_and(|expected| Arc::ptr_eq(expected, &artifact)) {
            return Ok(());
        }
        self.commit_catalog.install_prepared_recovery(artifact);
        Ok(())
    }

    pub fn latest_patch_position(&self) -> Option<PatchStreamPosition> {
        let resident = self.patch_stream_index.last_key_value().map(|(p, _)| *p);
        let cold = Some(PatchStreamPosition(self.cold_count())).filter(|p| p.0 > 0);
        resident.max(cold)
    }

    pub fn try_commit_at_patch_position(
        &self,
        position: PatchStreamPosition,
    ) -> Result<Option<CommitId>, Cause> {
        if let Some(commit) = self.patch_stream_index.get(&position) {
            return Ok(Some(*commit));
        }
        let Some(cold) = &self.cold_catalog else {
            return Ok(None);
        };
        Ok(cold
            .index()?
            .at_position(position)?
            .map(|entry| entry.commit.commit_id))
    }

    pub fn commit_at_patch_position(&self, position: PatchStreamPosition) -> Option<CommitId> {
        self.try_commit_at_patch_position(position).ok().flatten()
    }

    pub fn try_patch_positions_after(
        &self,
        after: Option<PatchStreamPosition>,
        limit: usize,
    ) -> Result<Vec<(PatchStreamPosition, CommitId)>, Cause> {
        let after = after.map_or(0, |position| position.0);
        let count = self.cold_count();
        let mut rows = Vec::new();
        if after < count
            && let Some(cold) = &self.cold_catalog
        {
            rows.extend(
                cold.index()?
                    .after(after, limit)?
                    .into_iter()
                    .map(|entry| (entry.position, entry.commit.commit_id)),
            );
        }
        let floor = PatchStreamPosition(after.max(count));
        rows.extend(
            self.patch_stream_index
                .range((std::ops::Bound::Excluded(floor), std::ops::Bound::Unbounded))
                .take(limit.saturating_sub(rows.len()))
                .map(|(position, commit)| (*position, *commit)),
        );
        Ok(rows)
    }

    pub fn patch_positions_after(
        &self,
        after: Option<PatchStreamPosition>,
        limit: usize,
    ) -> Vec<(PatchStreamPosition, CommitId)> {
        self.try_patch_positions_after(after, limit)
            .unwrap_or_default()
    }

    pub fn try_latest_patch_references(
        &self,
        limit: usize,
    ) -> Result<Vec<(PatchStreamPosition, CommitId)>, Cause> {
        let count = self.cold_count();
        let mut rows = self
            .patch_stream_index
            .range((
                std::ops::Bound::Excluded(PatchStreamPosition(count)),
                std::ops::Bound::Unbounded,
            ))
            .rev()
            .take(limit)
            .map(|(position, commit)| (*position, *commit))
            .collect::<Vec<_>>();
        let wanted = (limit - rows.len()) as u64;
        if wanted > 0
            && let Some(cold) = &self.cold_catalog
        {
            let start = count.saturating_sub(wanted);
            let tail = cold.index()?.after(start, wanted as usize)?;
            rows.extend(
                tail.into_iter()
                    .rev()
                    .map(|entry| (entry.position, entry.commit.commit_id)),
            );
        }
        Ok(rows)
    }

    pub fn latest_patch_references(&self, limit: usize) -> Vec<(PatchStreamPosition, CommitId)> {
        self.try_latest_patch_references(limit).unwrap_or_default()
    }

    pub fn patch_position_count(&self) -> u64 {
        let count = self.cold_count();
        let resident = self
            .patch_stream_index
            .range((
                std::ops::Bound::Excluded(PatchStreamPosition(count)),
                std::ops::Bound::Unbounded,
            ))
            .count();
        count + resident as u64
    }

    /// Largest commit and version ids over both tiers, from descriptors.
    pub fn try_max_commit_and_version(&self) -> Result<(Option<u64>, Option<u64>), Cause> {
        let resident_commit = self
            .commit_catalog
            .latest_artifact()
            .map(|a| a.commit_id().0);
        let resident_version = self.commit_catalog.max_version().map(|version| version.0);
        let index = self
            .cold_catalog
            .as_ref()
            .map(|cold| cold.index())
            .transpose()?;
        let cold_commit = index
            .as_ref()
            .and_then(|index| index.max_commit())
            .map(|c| c.0);
        let cold_version = index
            .as_ref()
            .and_then(|index| index.max_version())
            .map(|v| v.0);
        Ok((
            resident_commit.max(cold_commit),
            resident_version.max(cold_version),
        ))
    }

    pub fn max_commit_and_version(&self) -> (Option<u64>, Option<u64>) {
        self.try_max_commit_and_version().unwrap_or((None, None))
    }

    pub fn try_has_commit(&self, commit: CommitId) -> Result<bool, Cause> {
        if self.commit_catalog.contains(commit) {
            return Ok(true);
        }
        match &self.cold_catalog {
            Some(cold) => Ok(cold.entry(commit)?.is_some()),
            None => Ok(false),
        }
    }

    pub fn has_commit(&self, commit: CommitId) -> bool {
        self.try_has_commit(commit).unwrap_or(false)
    }
}
