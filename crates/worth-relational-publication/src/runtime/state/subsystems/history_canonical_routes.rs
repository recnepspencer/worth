use std::sync::Arc;
use worth_relational_contracts::internal_port as contracts;

use contracts::history::data::coordinates::CommitId;
use contracts::identity::data::primitives::VersionId;
use contracts::publication::patch::data::patch_position::PatchStreamPosition;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelope;

use super::canonical_publication_routes::RelationalCanonicalPublicationRoutes;
use std::sync::atomic::AtomicU64;
use worth_relational_branch::internal_port::runtime::state::subsystems::history::HistorySubsystem;
#[path = "history_canonical_routes/fork_snapshot.rs"]
pub mod fork_snapshot;

#[derive(Debug)]
pub struct RelationalPublicationHistory {
    pub(super) branch: HistorySubsystem,
    pub canonical_publication_routes: Arc<RelationalCanonicalPublicationRoutes>,
}

impl RelationalPublicationHistory {
    pub fn into_parts(self) -> (HistorySubsystem, Arc<RelationalCanonicalPublicationRoutes>) {
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
    pub fn discard_unobserved_construction_schema_inputs_for_recovery(&self) {
        self.canonical_publication_routes
            .discard_unobserved_construction_schema_inputs_for_recovery();
    }

    pub fn capture_original_branch_schema_inputs(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        worth_relational_durability::internal_port::durability::source_initial_inputs::producer_roster::OriginalBranchSchemaInputs,
        worth_relational_durability::internal_port::durability::source_initial_inputs::producer_roster::OriginalBranchSchemaInputsDenial,
    >{
        self.canonical_publication_routes
            .capture_original_branch_schema_inputs(request)
    }

    pub fn performed_effect_residencies(
        &self,
    ) -> Vec<(
        CommitId,
        Arc<worth_relational_branch::internal_port::history::commit::envelope_residency::RelationalCommitEnvelopeResidency>,
    )>{
        self.canonical_publication_routes
            .performed_effect_residencies()
    }

    pub fn performed_effect_residency(
        &self,
        commit_id: CommitId,
    ) -> Option<Arc<worth_relational_branch::internal_port::history::commit::envelope_residency::RelationalCommitEnvelopeResidency>> {
        self.canonical_publication_routes
            .performed_effect_residency(commit_id)
    }

    pub fn associated_effect_settlement(
        &self,
        entry: &worth_relational_branch::internal_port::history::commit::index_entry::RelationalCommitIndexEntry,
        residency: &Arc<worth_relational_branch::internal_port::history::commit::envelope_residency::RelationalCommitEnvelopeResidency>,
    ) -> Option<worth_relational_durability::internal_port::durability::authority::associated_effects::RelationalAssociatedEffectSettlement>{
        self.canonical_publication_routes
            .associated_effect_settlement(entry, residency)
    }
    pub fn discard_authored_write_generation(
        &self,
        commit: CommitId,
    ) -> Result<(), worth_relational_branch::internal_port::history::access::relevant_writes::RelationalAuthoredWriteDenial>{
        self.canonical_publication_routes
            .discard_authored_generation(commit)
    }
    pub fn rebuild_authored_write_generation(
        &self,
        lane: &contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane<'_>,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        commit: CommitId,
    ) -> Result<usize, worth_relational_branch::internal_port::history::access::relevant_writes::RelationalAuthoredWriteDenial>{
        self.canonical_publication_routes
            .rebuild_authored_generation(lane, execution, commit)
    }
    pub fn seed_authored_write_span(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        original: CommitId,
        commit: CommitId,
        maximum_links: Option<std::num::NonZeroUsize>,
    ) -> Result<worth_relational_branch::internal_port::history::access::relevant_writes::RelationalAuthoredWriteSpan, worth_relational_branch::internal_port::history::access::relevant_writes::RelationalAuthoredWriteDenial>{
        self.canonical_publication_routes
            .seed_authored_span(execution, original, commit, maximum_links)
    }
    pub fn authored_write_generation(
        &self,
        commit: CommitId,
    ) -> Option<worth_relational_branch::internal_port::history::access::relevant_writes::generation::AuthoredWriteGeneration>{
        self.canonical_publication_routes
            .authored_generation(commit)
    }
    pub fn performed_envelope_residency(
        &self,
        id: CommitId,
    ) -> Option<Arc<worth_relational_branch::internal_port::history::commit::envelope_residency::RelationalCommitEnvelopeResidency>>{
        self.canonical_publication_routes
            .performed_envelope_residency(id)
    }
    pub fn canonical_envelope(&self, commit_id: CommitId) -> Option<Arc<CanonicalCommitEnvelope>> {
        self.commit_artifact(commit_id)
            .and_then(|artifact| artifact.resident_envelope())
            .or_else(|| self.canonical_publication_routes.by_commit(commit_id))
    }

    pub fn canonical_envelope_at_patch(
        &self,
        position: PatchStreamPosition,
    ) -> Option<Arc<CanonicalCommitEnvelope>> {
        self.canonical_publication_routes.by_patch(position)
    }

    pub(crate) fn positioned_canonical_commit_at_patch(
        &self,
        position: PatchStreamPosition,
    ) -> Option<Arc<worth_relational_branch::internal_port::history::data::positioned_canonical_commit::PositionedCanonicalCommit>>{
        self.canonical_publication_routes
            .positioned_by_patch(position)
            .or_else(|| {
                self.recorded_commit_at_patch_position(position)
                    .and_then(|id| self.positioned_canonical_commit(id))
            })
    }

    /// Performed route inventory for cold history and inspection consumers.
    pub fn performed_canonical_envelopes(&self) -> Vec<Arc<CanonicalCommitEnvelope>> {
        self.canonical_publication_routes.visible_envelopes()
    }

    pub fn latest_canonical_patch_route(&self) -> Option<(PatchStreamPosition, CommitId)> {
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

    pub fn latest_canonical_patch_references(
        &self,
        limit: usize,
    ) -> Vec<(PatchStreamPosition, CommitId)> {
        self.canonical_publication_routes
            .latest_patch_references(limit)
    }

    pub fn canonical_stream_position(&self, commit_id: CommitId) -> Option<PatchStreamPosition> {
        self.canonical_publication_routes.stream_position(commit_id)
    }

    pub fn positioned_canonical_commit(
        &self,
        commit_id: CommitId,
    ) -> Option<Arc<worth_relational_branch::internal_port::history::data::positioned_canonical_commit::PositionedCanonicalCommit>>{
        self.canonical_publication_routes
            .positioned_commit(commit_id)
    }

    pub fn positioned_canonical_commits_snapshot(
        &self,
    ) -> Vec<Arc<worth_relational_branch::internal_port::history::data::positioned_canonical_commit::PositionedCanonicalCommit>>{
        self.canonical_publication_routes.performed_snapshot()
    }

    pub fn canonical_checkpoint_gate(&self) -> Arc<RelationalCanonicalPublicationRoutes> {
        Arc::clone(&self.canonical_publication_routes)
    }

    pub fn advance_canonical_stream_floor(&self, floor: PatchStreamPosition) {
        self.canonical_publication_routes
            .advance_position_floor(floor);
    }

    pub fn install_recovered_canonical_route(
        &self,
        commit: Arc<worth_relational_branch::internal_port::history::data::positioned_canonical_commit::PositionedCanonicalCommit>,
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

    pub fn rebuild_recovered_canonical_routes(
        &mut self,
        commits: impl IntoIterator<Item = Arc<worth_relational_branch::internal_port::history::data::positioned_canonical_commit::PositionedCanonicalCommit>>,
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

    pub fn restore_branch_transition_coverage(
        &self,
        covered: contracts::branch::transition::sequence::RelationalBranchTransitionSequence,
    ) {
        self.canonical_publication_routes
            .restore_branch_transition_coverage(covered);
    }

    pub fn compact_branch_transitions_through(
        &self,
        covered: contracts::branch::transition::sequence::RelationalBranchTransitionSequence,
    ) {
        self.canonical_publication_routes
            .compact_branch_transitions_through(covered);
    }

    pub fn retained_branch_transitions(
        &self,
    ) -> (
        contracts::branch::transition::sequence::RelationalBranchTransitionSequence,
        Vec<Arc<worth_relational_branch::internal_port::branch::transition::positioned::PositionedBranchTransition>>,
    ){
        self.canonical_publication_routes
            .retained_branch_transitions()
    }

    #[doc(hidden)]
    pub fn remove_canonical_publication_route_for_test(&self, commit_id: CommitId) -> bool {
        self.canonical_publication_routes.remove_for_test(commit_id)
    }
}

pub(crate) fn mark_publication_settled(
    history: &RelationalPublicationHistory,
    commit_id: CommitId,
) -> bool {
    history.canonical_publication_routes.mark_settled(commit_id)
}

pub fn settlement_route_visit_count(history: &RelationalPublicationHistory) -> u64 {
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

pub fn canonical_publication_reservation_counters(
    history: &RelationalPublicationHistory,
) -> super::canonical_publication_routes::RelationalPatchPositionReservationCounters {
    history.canonical_publication_routes.reservation_counters()
}

pub fn pending_canonical_publication_route_count(history: &RelationalPublicationHistory) -> usize {
    history.canonical_publication_routes.pending_count()
}

pub fn retain_original_empty_birth(
    history: &RelationalPublicationHistory,
    schema: Arc<worth_relational_branch::internal_port::branch::root::schema::RelationalBranchRootSchemaAuthority>,
) {
    history
        .canonical_publication_routes
        .retain_original_empty_birth(
            worth_relational_branch::internal_port::branch::transition::original_inputs::OriginalEmptySchemaObservation {
                generation: worth_foundational::FoundationalBranchReferenceGeneration::initial(),
                truth_version: contracts::branch::version::RelationalBranchVersion::initial(),
                lifecycle: worth_relational_branch::internal_port::branch::lifecycle::RelationalBranchLifecyclePosture::Live,
                schema,
            },
        );
}

impl worth_relational_branch::internal_port::history::envelope_demand::PerformedCanonicalHistory
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
    ) -> Option<Arc<worth_relational_branch::internal_port::history::data::positioned_canonical_commit::PositionedCanonicalCommit>>{
        self.positioned_commit(commit)
    }

    fn find_version(
        &self,
        version: VersionId,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<Option<CommitId>, contracts::execution::denial_cause::RelationalExecutionDenialCause>
    {
        self.commit_for_version_in_context(version, context)
    }
}

impl RelationalPublicationHistory {
    pub fn canonical_read(
        &self,
    ) -> worth_relational_branch::internal_port::history::envelope_demand::CanonicalHistoryRead<'_>
    {
        worth_relational_branch::internal_port::history::envelope_demand::CanonicalHistoryRead {
            branch: &self.branch,
            performed: &*self.canonical_publication_routes,
        }
    }

    pub fn demand_commit_envelope(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        commit: CommitId,
    ) -> Result<
        Option<Arc<CanonicalCommitEnvelope>>,
        contracts::execution::denial_cause::RelationalExecutionDenialCause,
    > {
        self.canonical_read()
            .demand_commit_envelope(execution, commit)
    }
}
