//! Immutable indexed facts remain available while envelope bodies stay cold.
use super::{visible_envelope, RelationalCanonicalPublicationRoutes};
use branch::history::data::canonical_commit_envelope::CanonicalCommitEnvelope;
use branch::history::data::positioned_canonical_commit::PositionedCanonicalCommit;
use contracts::history::data::commit_receipt::RelationalCommitReceipt;
use contracts::history::data::coordinates::CommitId;
use contracts::identity::data::primitives::VersionId;
use contracts::publication::patch::data::patch_position::PatchStreamPosition;
use std::sync::{atomic::Ordering, Arc};
use worth_relational_branch::internal_port as branch;
use worth_relational_contracts::internal_port as contracts;

type Artifact = Arc<branch::history::commit::artifact::RelationalCommitArtifact>;

/// The checkpoint catalog: resident artifacts (recovery overrides and heads)
/// over the history's cold base, which faults commit pages on demand (`G7`).
#[derive(Default)]
pub(super) struct IndexedPublicationCatalog {
    entries: std::collections::BTreeMap<CommitId, Artifact>,
    cold: Option<Arc<branch::history::commit::cold_catalog::ColdCommitCatalog>>,
    pub(super) head: Option<branch::history::commit::retained_index::RetainedCanonicalIndex>,
}
impl IndexedPublicationCatalog {
    pub(super) fn try_artifact(
        &self,
        id: CommitId,
    ) -> Result<
        Option<Artifact>,
        contracts::storage::data::storage_unavailable::RelationalStorageUnavailableCause,
    > {
        if let Some(artifact) = self.entries.get(&id) {
            return Ok(Some(artifact.clone()));
        }
        match &self.cold {
            Some(cold) => cold.artifact(id),
            None => Ok(None),
        }
    }
    pub(super) fn artifact(&self, id: CommitId) -> Option<Artifact> {
        self.try_artifact(id).ok().flatten()
    }
    fn receipt(&self, id: CommitId) -> Option<RelationalCommitReceipt> {
        if let Some(artifact) = self.entries.get(&id) {
            return Some(artifact.receipt());
        }
        Some(self.cold.as_ref()?.entry(id).ok()??.commit)
    }
    fn position(&self, id: CommitId, captured_only: bool) -> Option<PatchStreamPosition> {
        if let Some(artifact) = self.entries.get(&id) {
            let cell = artifact.envelope_residency();
            return if captured_only {
                cell.captured_position()
            } else {
                cell.journal_index().map(|entry| entry.position)
            };
        }
        Some(self.cold.as_ref()?.entry(id).ok()??.position)
    }
    pub(super) fn len_bound(&self) -> usize {
        let cold = self.cold.as_ref().map_or(0, |cold| cold.count());
        self.entries.len() + usize::try_from(cold).unwrap_or(usize::MAX)
    }
    fn try_commit_for_version(
        &self,
        version: VersionId,
    ) -> Result<
        Option<CommitId>,
        contracts::storage::data::storage_unavailable::RelationalStorageUnavailableCause,
    > {
        if let Some(artifact) = self
            .entries
            .values()
            .find(|artifact| artifact.version_id() == version)
        {
            return Ok(Some(artifact.commit_id()));
        }
        match &self.cold {
            Some(cold) => Ok(cold
                .index()?
                .by_version(version)?
                .map(|entry| entry.commit.commit_id)),
            None => Ok(None),
        }
    }
    /// Close walks resident artifacts and every faulted cold artifact still
    /// held; unfaulted cold commits own no cell and nothing to retire.
    pub(super) fn next_retirement(&self, after: Option<CommitId>) -> Option<(CommitId, Artifact)> {
        let beyond = |id: &CommitId| after.is_none_or(|after| *id > after);
        let resident = self
            .entries
            .iter()
            .find(|(id, _)| beyond(id))
            .map(|(id, artifact)| (*id, artifact.clone()));
        let cold = self.cold.as_ref().and_then(|cold| {
            cold.live_artifacts()
                .into_iter()
                .filter(|artifact| beyond(&artifact.commit_id()))
                .filter(|artifact| !self.entries.contains_key(&artifact.commit_id()))
                .min_by_key(|artifact| artifact.commit_id())
                .map(|artifact| (artifact.commit_id(), artifact))
        });
        match (resident, cold) {
            (Some(resident), Some(cold)) => Some(if cold.0 < resident.0 { cold } else { resident }),
            (resident, cold) => resident.or(cold),
        }
    }
}

impl RelationalCanonicalPublicationRoutes {
    pub fn install_checkpoint_catalog(
        &self,
        (entries, cold): (
            Vec<Artifact>,
            Option<Arc<branch::history::commit::cold_catalog::ColdCommitCatalog>>,
        ),
    ) {
        let resident_head = entries
            .iter()
            .filter_map(|artifact| {
                artifact
                    .envelope_residency()
                    .source_placement()
                    .map(|(entry, _)| entry)
            })
            .max_by_key(|entry| entry.position);
        // The cold tail is inline in the commit index: no page is read.
        let cold_head = cold
            .as_ref()
            .and_then(|cold| cold.index().ok())
            .and_then(|index| index.descriptor().tail.clone())
            .map(|tail| {
                branch::history::commit::retained_index::RetainedCanonicalIndex::copy_cold(
                    &tail,
                    worth_relational_durability::internal_port::durability::canonical_commit_artifact::journal_source_read::readmit_journal_commit,
                )
            });
        let head = match (resident_head, cold_head) {
            (Some(resident), Some(cold)) if cold.position > resident.position => Some(cold),
            (resident, cold) => resident.or(cold),
        };
        if let Some(head) = &head {
            self.performed_stream.advance_position_floor(head.position);
        }
        *self
            .checkpoint_catalog
            .write()
            .unwrap_or_else(|error| error.into_inner()) = Arc::new(IndexedPublicationCatalog {
            head,
            cold,
            entries: entries
                .into_iter()
                .map(|artifact| (artifact.commit_id(), artifact))
                .collect(),
        });
    }
    pub(crate) fn performed_envelope_residency(
        &self,
        id: CommitId,
    ) -> Option<Arc<branch::history::commit::envelope_residency::RelationalCommitEnvelopeResidency>>
    {
        self.by_commit
            .get(&id)
            .filter(|route| route.is_visible())
            .map(|route| route.envelope.clone())
    }
    pub(crate) fn demand_envelope(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: CommitId,
    ) -> Result<
        Option<Arc<CanonicalCommitEnvelope>>,
        worth_relational_storage::internal_port::storage::residency::demand_failure::RelationalResidencyDemandFailure,
    >{
        let cell = match self.performed_envelope_residency(id) {
            Some(cell) => Some(cell),
            None => self
                .checkpoint_catalog
                .read()
                .unwrap_or_else(|error| error.into_inner())
                .try_artifact(id)
                .map_err(|cause| {
                    worth_relational_storage::internal_port::storage::residency::demand_failure::RelationalResidencyDemandFailure::StorageUnavailable(
                        contracts::storage::data::storage_unavailable::RelationalStorageUnavailable {
                            subject: contracts::storage::data::storage_unavailable::RelationalStorageSubject::Envelope(
                                PatchStreamPosition(0),
                            ),
                            cause,
                        },
                    )
                })?
                .map(|artifact| artifact.envelope_residency()),
        };
        cell.map(|cell| cell.demand(execution, 1 << 30)).transpose()
    }
    pub(crate) fn indexed_receipt(&self, id: CommitId) -> Option<RelationalCommitReceipt> {
        if let Some(route) = self.by_commit.get(&id)
            && route.is_visible()
        {
            return Some(route.commit.clone());
        }
        self.checkpoint_catalog
            .read()
            .unwrap_or_else(|error| error.into_inner())
            .receipt(id)
    }
    pub(crate) fn by_commit(&self, commit_id: CommitId) -> Option<Arc<CanonicalCommitEnvelope>> {
        visible_envelope(self.by_commit.get(&commit_id).map(|entry| entry.clone())).or_else(|| {
            self.checkpoint_catalog
                .read()
                .unwrap_or_else(|error| error.into_inner())
                .artifact(commit_id)
                .and_then(|artifact| artifact.resident_envelope())
        })
    }

    /// Includes reserved routes; bounds traversal through the visible subset.
    pub(crate) fn catalog_len_bound(&self) -> usize {
        self.by_commit.len()
            + self
                .checkpoint_catalog
                .read()
                .unwrap_or_else(|error| error.into_inner())
                .len_bound()
    }

    pub(crate) fn commit_for_version_in_context(
        &self,
        version: VersionId,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<Option<CommitId>, contracts::execution::denial_cause::RelationalExecutionDenialCause>
    {
        if context.has_accounting() {
            context.checkpoint(1)
        } else {
            context.poll()
        }
        .map_err(contracts::execution::denial_cause::RelationalExecutionDenialCause::from)?;
        if let Some(route) = self
            .by_version
            .get(&version)
            .filter(|route| route.is_visible())
        {
            return Ok(Some(route.commit.commit_id));
        }
        let catalog = self
            .checkpoint_catalog
            .read()
            .unwrap_or_else(|error| error.into_inner());
        let work = u64::try_from(catalog.len_bound()).map_err(|_| {
            contracts::execution::denial_cause::RelationalExecutionDenialCause::WorkCounterOverflow
        })?;
        if context.has_accounting() {
            context.checkpoint(work)
        } else {
            context.poll()
        }
        .map_err(contracts::execution::denial_cause::RelationalExecutionDenialCause::from)?;
        catalog.try_commit_for_version(version).map_err(|cause| {
            contracts::execution::denial_cause::RelationalExecutionDenialCause::StorageUnavailable(
                contracts::storage::data::storage_unavailable::RelationalStorageUnavailable {
                    subject: contracts::storage::data::storage_unavailable::RelationalStorageSubject::Envelope(
                        PatchStreamPosition(0),
                    ),
                    cause,
                },
            )
        })
    }

    pub(crate) fn by_version(&self, version_id: VersionId) -> Option<Arc<CanonicalCommitEnvelope>> {
        visible_envelope(self.by_version.get(&version_id).map(|entry| entry.clone()))
    }

    pub(crate) fn by_patch(
        &self,
        position: PatchStreamPosition,
    ) -> Option<Arc<CanonicalCommitEnvelope>> {
        self.performed_stream.envelope_at(position)
    }

    pub(crate) fn positioned_by_patch(
        &self,
        position: PatchStreamPosition,
    ) -> Option<Arc<PositionedCanonicalCommit>> {
        self.performed_stream.positioned_at(position)
    }

    pub(crate) fn visible_envelopes(&self) -> Vec<Arc<CanonicalCommitEnvelope>> {
        self.performed_stream.visible_envelopes()
    }

    pub(crate) fn latest_visible_patch(&self) -> Option<(PatchStreamPosition, CommitId)> {
        self.performed_stream.latest()
    }

    pub(crate) fn visible_patches_after(
        &self,
        after_position: Option<PatchStreamPosition>,
        max_commits: usize,
    ) -> Vec<(PatchStreamPosition, CommitId)> {
        self.performed_stream.after(after_position, max_commits)
    }

    pub(crate) fn latest_patch_references(
        &self,
        limit: usize,
    ) -> Vec<(PatchStreamPosition, CommitId)> {
        self.performed_stream.latest_references(limit)
    }

    pub(crate) fn stream_position(&self, commit_id: CommitId) -> Option<PatchStreamPosition> {
        if let Some(route) = self.by_commit.get(&commit_id)
            && route.is_visible()
        {
            return route.position();
        }
        self.checkpoint_catalog
            .read()
            .unwrap_or_else(|error| error.into_inner())
            .position(commit_id, false)
    }

    /// The publication-order position of a visible performed or checkpointed
    /// commit, including a Memory-backed checkpoint artifact. A parent is
    /// always published before its child, so its position is strictly lower.
    pub(crate) fn ordered_position(&self, commit_id: CommitId) -> Option<PatchStreamPosition> {
        if let Some(route) = self.by_commit.get(&commit_id)
            && route.is_visible()
        {
            return route.position();
        }
        self.checkpoint_catalog
            .read()
            .unwrap_or_else(|error| error.into_inner())
            .position(commit_id, true)
    }

    pub(crate) fn positioned_commit(
        &self,
        commit_id: CommitId,
    ) -> Option<Arc<PositionedCanonicalCommit>> {
        self.performed_stream.positioned(commit_id).or_else(|| {
            self.checkpoint_catalog
                .read()
                .unwrap_or_else(|error| error.into_inner())
                .artifact(commit_id)
                .and_then(|artifact| artifact.envelope_residency().positioned_if_resident())
                .map(Arc::new)
        })
    }

    pub(crate) fn requires_settlement(&self, commit_id: CommitId) -> bool {
        self.by_commit
            .get(&commit_id)
            .is_some_and(|route| !route.settled.load(Ordering::Acquire))
    }

    pub(crate) fn performed_snapshot(&self) -> Vec<Arc<PositionedCanonicalCommit>> {
        self.performed_stream.positioned_snapshot()
    }
}
