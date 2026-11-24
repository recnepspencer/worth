use std::sync::Arc;

use crate::history::data::{CanonicalCommitEnvelope, CommitId};
use crate::identity::data::VersionId;
use crate::publication::patch::data::PatchStreamPosition;

use super::{canonical_publication_routes::RelationalCanonicalPublicationRoutes, HistorySubsystem};
use std::sync::atomic::AtomicU64;
#[path = "history_canonical_routes/fork_snapshot.rs"]
mod fork_snapshot;

#[derive(Debug)]
pub(crate) struct RelationalPublicationHistory {
    pub(super) branch: HistorySubsystem,
    pub(super) canonical_publication_routes: Arc<RelationalCanonicalPublicationRoutes>,
}

impl RelationalPublicationHistory {
    pub(crate) fn into_parts(
        self,
    ) -> (HistorySubsystem, Arc<RelationalCanonicalPublicationRoutes>) {
        (self.branch, self.canonical_publication_routes)
    }
}

impl std::ops::Deref for RelationalPublicationHistory {
    type Target = HistorySubsystem;

    fn deref(&self) -> &Self::Target {
        &self.branch
    }
}

impl std::ops::DerefMut for RelationalPublicationHistory {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.branch
    }
}

impl RelationalPublicationHistory {
    pub(crate) fn commit_for_version_in_context(
        &self,
        version: VersionId,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<Option<CommitId>, crate::execution::RelationalExecutionDenialCause> {
        self.canonical_publication_routes
            .commit_for_version_in_context(version, context)
    }

    pub(crate) fn discard_unobserved_construction_schema_inputs_for_recovery(&self) {
        self.canonical_publication_routes
            .discard_unobserved_construction_schema_inputs_for_recovery();
    }

    pub(crate) fn capture_original_branch_schema_inputs(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        super::canonical_publication_routes::OriginalBranchSchemaInputs,
        super::canonical_publication_routes::OriginalBranchSchemaInputsDenial,
    > {
        self.canonical_publication_routes
            .capture_original_branch_schema_inputs(request)
    }

    pub(crate) fn performed_effect_residencies(
        &self,
    ) -> Vec<(
        CommitId,
        Arc<crate::history::commit::RelationalCommitEnvelopeResidency>,
    )> {
        self.canonical_publication_routes
            .performed_effect_residencies()
    }

    pub(crate) fn associated_effect_settlement(
        &self,
        entry: &crate::history::commit::RelationalCommitIndexEntry,
        residency: &Arc<crate::history::commit::RelationalCommitEnvelopeResidency>,
    ) -> Option<crate::durability::authority::RelationalAssociatedEffectSettlement> {
        self.canonical_publication_routes
            .associated_effect_settlement(entry, residency)
    }
    pub(crate) fn discard_authored_write_generation(
        &self,
        commit: CommitId,
    ) -> Result<(), crate::history::access::relevant_writes::RelationalAuthoredWriteDenial> {
        self.canonical_publication_routes
            .discard_authored_generation(commit)
    }
    pub(crate) fn rebuild_authored_write_generation(
        &self,
        lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        commit: CommitId,
    ) -> Result<usize, crate::history::access::relevant_writes::RelationalAuthoredWriteDenial> {
        self.canonical_publication_routes
            .rebuild_authored_generation(lane, execution, commit)
    }
    pub(crate) fn authored_write_generation(
        &self,
        commit: CommitId,
    ) -> Option<crate::history::access::relevant_writes::generation::AuthoredWriteGeneration> {
        self.canonical_publication_routes
            .authored_generation(commit)
    }
    pub(crate) fn performed_envelope_residency(
        &self,
        id: CommitId,
    ) -> Option<Arc<crate::history::commit::RelationalCommitEnvelopeResidency>> {
        self.canonical_publication_routes
            .performed_envelope_residency(id)
    }
    pub(crate) fn canonical_envelope(
        &self,
        commit_id: CommitId,
    ) -> Option<Arc<CanonicalCommitEnvelope>> {
        self.commit_artifact(commit_id)
            .and_then(|artifact| artifact.resident_envelope())
            .or_else(|| self.canonical_publication_routes.by_commit(commit_id))
    }

    pub(crate) fn canonical_envelope_for_version(
        &self,
        version_id: VersionId,
    ) -> Option<Arc<CanonicalCommitEnvelope>> {
        self.canonical_publication_routes.by_version(version_id)
    }

    pub(crate) fn canonical_envelope_at_patch(
        &self,
        position: PatchStreamPosition,
    ) -> Option<Arc<CanonicalCommitEnvelope>> {
        self.canonical_publication_routes.by_patch(position)
    }

    pub(crate) fn positioned_canonical_commit_at_patch(
        &self,
        position: PatchStreamPosition,
    ) -> Option<Arc<crate::history::data::PositionedCanonicalCommit>> {
        self.canonical_publication_routes
            .positioned_by_patch(position)
            .or_else(|| {
                self.recorded_commit_at_patch_position(position)
                    .and_then(|id| self.positioned_canonical_commit(id))
            })
    }

    /// Performed route inventory for cold history and inspection consumers.
    pub(crate) fn performed_canonical_envelopes(&self) -> Vec<Arc<CanonicalCommitEnvelope>> {
        self.canonical_publication_routes.visible_envelopes()
    }

    pub(crate) fn latest_canonical_patch_route(&self) -> Option<(PatchStreamPosition, CommitId)> {
        self.canonical_publication_routes.latest_visible_patch()
    }

    pub(crate) fn canonical_patch_routes_after(
        &self,
        after_position: Option<PatchStreamPosition>,
        max_commits: usize,
    ) -> Vec<(PatchStreamPosition, CommitId)> {
        self.canonical_publication_routes
            .visible_patches_after(after_position, max_commits)
    }

    pub(crate) fn latest_canonical_patch_references(
        &self,
        limit: usize,
    ) -> Vec<(PatchStreamPosition, CommitId)> {
        self.canonical_publication_routes
            .latest_patch_references(limit)
    }

    pub(crate) fn canonical_stream_position(
        &self,
        commit_id: CommitId,
    ) -> Option<PatchStreamPosition> {
        self.canonical_publication_routes.stream_position(commit_id)
    }

    pub(crate) fn positioned_canonical_commit(
        &self,
        commit_id: CommitId,
    ) -> Option<Arc<crate::history::data::PositionedCanonicalCommit>> {
        self.canonical_publication_routes
            .positioned_commit(commit_id)
    }

    pub(crate) fn positioned_canonical_commits_snapshot(
        &self,
    ) -> Vec<Arc<crate::history::data::PositionedCanonicalCommit>> {
        self.canonical_publication_routes.performed_snapshot()
    }

    pub(crate) fn canonical_checkpoint_gate(&self) -> Arc<RelationalCanonicalPublicationRoutes> {
        Arc::clone(&self.canonical_publication_routes)
    }

    pub(crate) fn advance_canonical_stream_floor(&self, floor: PatchStreamPosition) {
        self.canonical_publication_routes
            .advance_position_floor(floor);
    }

    pub(crate) fn install_recovered_canonical_route(
        &self,
        commit: Arc<crate::history::data::PositionedCanonicalCommit>,
    ) -> Result<(), &'static str> {
        let artifact = self
            .commit_artifact(commit.commit.commit_id)
            .ok_or("recovered canonical route has no admitted catalog entry")?;
        self.canonical_publication_routes.install_recovered(
            commit,
            artifact.envelope_residency(),
            artifact.roots().clone(),
        )
    }

    #[expect(
        clippy::arc_with_non_send_sync,
        reason = "Canonical route custody is shared inside its same-thread Relational runtime"
    )]
    pub(crate) fn rebuild_recovered_canonical_routes(
        &mut self,
        commits: impl IntoIterator<Item = Arc<crate::history::data::PositionedCanonicalCommit>>,
    ) -> Result<(), &'static str> {
        let routes = Arc::new(
            self.canonical_publication_routes
                .continuing_branch_transitions(),
        );
        routes.install_checkpoint_catalog(self.commit_artifacts());
        for commit in commits {
            let artifact = self
                .commit_artifact(commit.commit.commit_id)
                .ok_or("recovered canonical route has no admitted catalog entry")?;
            routes.install_recovered(
                commit,
                artifact.envelope_residency(),
                artifact.roots().clone(),
            )?;
        }
        self.canonical_publication_routes = routes;
        Ok(())
    }

    pub(crate) fn restore_branch_transition_coverage(
        &self,
        covered: crate::branch::transition::sequence::RelationalBranchTransitionSequence,
    ) {
        self.canonical_publication_routes
            .restore_branch_transition_coverage(covered);
    }

    pub(crate) fn compact_branch_transitions_through(
        &self,
        covered: crate::branch::transition::sequence::RelationalBranchTransitionSequence,
    ) {
        self.canonical_publication_routes
            .compact_branch_transitions_through(covered);
    }

    pub(crate) fn retained_branch_transitions(
        &self,
    ) -> (
        crate::branch::transition::sequence::RelationalBranchTransitionSequence,
        Vec<Arc<crate::branch::transition::PositionedBranchTransition>>,
    ) {
        self.canonical_publication_routes
            .retained_branch_transitions()
    }

    #[cfg(test)]
    pub(crate) fn remove_canonical_publication_route_for_test(&self, commit_id: CommitId) -> bool {
        self.canonical_publication_routes.remove_for_test(commit_id)
    }
}

pub(crate) fn mark_publication_settled(
    history: &RelationalPublicationHistory,
    commit_id: CommitId,
) -> bool {
    history.canonical_publication_routes.mark_settled(commit_id)
}

pub(crate) fn settlement_route_visit_count(history: &RelationalPublicationHistory) -> u64 {
    history
        .canonical_publication_routes
        .settlement_route_visit_count()
}

pub(crate) fn publication_requires_settlement(
    history: &RelationalPublicationHistory,
    commit_id: CommitId,
) -> bool {
    history
        .canonical_publication_routes
        .requires_settlement(commit_id)
}

pub(crate) fn canonical_publication_reservation_counters(
    history: &RelationalPublicationHistory,
) -> super::canonical_publication_routes::RelationalPatchPositionReservationCounters {
    history.canonical_publication_routes.reservation_counters()
}

pub(crate) fn pending_canonical_publication_route_count(
    history: &RelationalPublicationHistory,
) -> usize {
    history.canonical_publication_routes.pending_count()
}

pub(crate) fn retain_original_empty_birth(
    history: &RelationalPublicationHistory,
    schema: Arc<crate::branch::root::schema::RelationalBranchRootSchemaAuthority>,
) {
    history
        .canonical_publication_routes
        .retain_original_empty_birth(
            super::canonical_publication_routes::OriginalEmptySchemaObservation {
                generation: worth_foundational::FoundationalBranchReferenceGeneration::initial(),
                truth_version: crate::branch::version::RelationalBranchVersion::initial(),
                lifecycle: crate::branch::lifecycle::RelationalBranchLifecyclePosture::Live,
                schema,
            },
        );
}

impl crate::history::envelope_demand::PerformedCanonicalHistory
    for RelationalCanonicalPublicationRoutes
{
    fn resident_envelope(&self, commit: CommitId) -> Option<Arc<CanonicalCommitEnvelope>> {
        self.by_commit(commit)
    }

    fn by_version(&self, version: VersionId) -> Option<Arc<CanonicalCommitEnvelope>> {
        RelationalCanonicalPublicationRoutes::by_version(self, version)
    }

    fn by_patch(&self, position: PatchStreamPosition) -> Option<Arc<CanonicalCommitEnvelope>> {
        RelationalCanonicalPublicationRoutes::by_patch(self, position)
    }

    fn latest_patch(&self) -> Option<PatchStreamPosition> {
        self.latest_visible_patch().map(|(position, _)| position)
    }

    fn positioned(
        &self,
        commit: CommitId,
    ) -> Option<Arc<crate::history::data::PositionedCanonicalCommit>> {
        self.positioned_commit(commit)
    }

    fn find_version(
        &self,
        version: VersionId,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<Option<CommitId>, crate::execution::RelationalExecutionDenialCause> {
        self.commit_for_version_in_context(version, context)
    }
}

impl RelationalPublicationHistory {
    pub(crate) fn canonical_read(
        &self,
    ) -> crate::history::envelope_demand::CanonicalHistoryRead<'_> {
        crate::history::envelope_demand::CanonicalHistoryRead {
            branch: &self.branch,
            performed: &*self.canonical_publication_routes,
        }
    }

    pub(crate) fn demand_commit_envelope(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        commit: CommitId,
    ) -> Result<
        Option<Arc<CanonicalCommitEnvelope>>,
        crate::execution::RelationalExecutionDenialCause,
    > {
        self.canonical_read()
            .demand_commit_envelope(execution, commit)
    }
}
