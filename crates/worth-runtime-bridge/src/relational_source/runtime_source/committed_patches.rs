use crate::facade::{
    BridgeCommittedPatchEnvelope, CommittedPatchSource, RelationalBridgeSourceError,
    RelationalCommittedPatchRequest, TruthBranchIdentity, TruthSnapshotIdentity,
};
use worth_proof::TransitionOutcome;

use super::{
    RelationalBridgeSelectedCommitObservation, RelationalBridgeSelectedObservation,
    RuntimeBridgeRelationalSource,
};
use crate::relational_source::change_publication::{
    lower_change_receipt_outcome, ChangeLoweringContext,
};
use crate::relational_source::identities::parse_bridge_commit_identity;
use crate::relational_source::{
    RelationalBridgePublicationOutcome, RelationalBridgePublicationRebindRequired,
    RelationalBridgePublicationStale, RelationalOpaqueAspectWideningAdmission,
    RelationalOpaqueAspectWideningAdmissionDenial,
};
use worth_relational::facade::history::CommitId;
use worth_relational::facade::identity::PartitionId;

impl RuntimeBridgeRelationalSource {
    /// Admit the one supported loss of precision, an opaque payload widened
    /// to its whole aspect, for this source's runtime and graph role.
    ///
    /// # Errors
    ///
    /// Returns [`RelationalOpaqueAspectWideningAdmissionDenial::InvalidGraphRole`]
    /// when the graph role contains whitespace.
    pub fn admit_opaque_aspect_widening(
        &self,
    ) -> Result<
        RelationalOpaqueAspectWideningAdmission,
        RelationalOpaqueAspectWideningAdmissionDenial,
    > {
        RelationalOpaqueAspectWideningAdmission::admit(
            self.runtime.runtime_instance_id(),
            self.graph_role.clone(),
        )
    }

    /// Publish one commit through an explicitly retained observation while
    /// consuming runtime-affine widening admission. A raw commit identity or
    /// copied snapshot identity cannot open this door.
    pub fn publish_commit_with_widening_at_snapshot(
        &self,
        commit_id: CommitId,
        snapshot_identity: &TruthSnapshotIdentity,
        admission: &RelationalOpaqueAspectWideningAdmission,
    ) -> Result<RelationalBridgePublicationOutcome, RelationalBridgeSourceError> {
        let observation = self.observation_bindings.resolve(snapshot_identity)?;
        let selected_commit = self.select_commit_for_observation(commit_id, observation)?;
        if admission.runtime_instance_id() != self.runtime.runtime_instance_id() {
            return Ok(TransitionOutcome::Stale(
                RelationalBridgePublicationStale::RuntimeAuthority,
            ));
        }
        if admission.graph_role() != &self.graph_role {
            return Ok(TransitionOutcome::RebindRequired(
                RelationalBridgePublicationRebindRequired::GraphRole,
            ));
        }
        let context = ChangeLoweringContext {
            graph_role: &self.graph_role,
            partition_role: None,
            widening: Some(admission.cause()),
        };
        Ok(self.publish_selected_commit(selected_commit, &context, None))
    }

    pub(super) fn publish_commit_for_selected_observation(
        &self,
        selected_commit: RelationalBridgeSelectedCommitObservation,
    ) -> RelationalBridgePublicationOutcome {
        let partition = self.partition.as_ref();
        let context = ChangeLoweringContext {
            graph_role: &self.graph_role,
            partition_role: partition.map(|partition| &partition.truth),
            widening: None,
        };
        self.publish_selected_commit(
            selected_commit,
            &context,
            partition.map(|partition| partition.relational),
        )
    }

    /// Mint the receipt under the runtime, then lower it outside the lock.
    fn publish_selected_commit(
        &self,
        selected_commit: RelationalBridgeSelectedCommitObservation,
        context: &ChangeLoweringContext<'_>,
        relational_partition: Option<PartitionId>,
    ) -> RelationalBridgePublicationOutcome {
        let RelationalBridgeSelectedCommitObservation {
            selected,
            snapshot_identity,
        } = selected_commit;
        let receipt = self
            .runtime
            .with_runtime(|runtime| runtime.mint_change_receipt(selected, relational_partition));
        lower_change_receipt_outcome(receipt, snapshot_identity, context)
    }

    fn publish_commit(
        &self,
        commit_id: CommitId,
    ) -> Result<RelationalBridgePublicationOutcome, RelationalBridgeSourceError> {
        let snapshot_identity = match self
            .branch_head_bindings
            .unique_snapshot_for_commit(commit_id)?
        {
            Some(snapshot) => snapshot,
            None => self
                .observation_bindings
                .snapshot_identity_for_commit(commit_id)?,
        };
        let observation = self.observation_bindings.resolve(&snapshot_identity)?;
        let selected_commit = self.select_commit_for_observation(commit_id, observation)?;
        Ok(self.publish_commit_for_selected_observation(selected_commit))
    }

    fn publish_commit_on_branch(
        &self,
        commit_id: CommitId,
        branch_identity: &TruthBranchIdentity,
    ) -> Result<RelationalBridgePublicationOutcome, RelationalBridgeSourceError> {
        let selected_commit = self.select_commit_on_branch(commit_id, branch_identity)?;
        Ok(self.publish_commit_for_selected_observation(selected_commit))
    }

    /// Select `commit_id` at the branch's bound head: exactly when it is the
    /// head, through the head's ancestry otherwise.
    pub(in crate::relational_source) fn select_commit_on_branch(
        &self,
        commit_id: CommitId,
        branch_identity: &TruthBranchIdentity,
    ) -> Result<RelationalBridgeSelectedCommitObservation, RelationalBridgeSourceError> {
        let (head_commit_id, snapshot_identity) =
            self.branch_head_bindings.resolve(branch_identity)?;
        let observation = self.observation_bindings.resolve(&snapshot_identity)?;
        if head_commit_id == commit_id {
            self.select_exact_commit_for_observation(commit_id, observation)
        } else {
            self.select_commit_for_observation(commit_id, observation)
        }
    }

    /// Select `commit_id` through the ancestry of one retained snapshot.
    pub(in crate::relational_source) fn select_commit_at_snapshot(
        &self,
        commit_id: CommitId,
        snapshot_identity: &TruthSnapshotIdentity,
    ) -> Result<RelationalBridgeSelectedCommitObservation, RelationalBridgeSourceError> {
        let observation = self.observation_bindings.resolve(snapshot_identity)?;
        self.select_commit_for_observation(commit_id, observation)
    }

    fn select_commit_for_observation(
        &self,
        commit_id: CommitId,
        observation: RelationalBridgeSelectedObservation,
    ) -> Result<RelationalBridgeSelectedCommitObservation, RelationalBridgeSourceError> {
        self.runtime.with_runtime(|runtime| {
            observation.select_reachable_commit(runtime, commit_id, &self.selection_work)
        })
    }

    pub(super) fn select_exact_commit_for_observation(
        &self,
        commit_id: CommitId,
        observation: RelationalBridgeSelectedObservation,
    ) -> Result<RelationalBridgeSelectedCommitObservation, RelationalBridgeSourceError> {
        self.runtime.with_runtime(|runtime| {
            observation.select_exact_selected_commit(runtime, commit_id, &self.selection_work)
        })
    }
}

impl CommittedPatchSource for RuntimeBridgeRelationalSource {
    fn authoritative_source_profile(
        &self,
    ) -> Option<crate::facade::BridgeAuthoritativeSourceProfile> {
        Some(RuntimeBridgeRelationalSource::authoritative_source_profile(
            self,
        ))
    }

    fn load_committed_patch(
        &self,
        request: RelationalCommittedPatchRequest,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeCommittedPatchEnvelope, RelationalBridgeSourceError> {
        execution
            .run(worth_execution::ExecutionWorkCeiling::new(0), |_| ())
            .map_err(|denial| RelationalBridgeSourceError::execution_denied(denial.into()))?;

        let commit_id = parse_bridge_commit_identity(request.commit_identity())?;
        let publication = match request.snapshot_identity() {
            Some(snapshot) => {
                let selected_commit = self.select_commit_at_snapshot(commit_id, snapshot)?;
                self.publish_commit_for_selected_observation(selected_commit)
            }
            None => match request.branch_identity() {
                Some(branch) => self.publish_commit_on_branch(commit_id, branch)?,
                None => self.publish_commit(commit_id)?,
            },
        };
        super::publication_result::publication_envelope(publication)
    }
}
