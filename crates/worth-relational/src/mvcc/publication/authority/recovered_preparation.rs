//! Prepare reconstructed publication through the same root owner.
use super::*;

impl RelationalPublicationAuthority<'_> {
    pub(crate) fn prepare_recovered_commit(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        commit_id: CommitId,
        commit_reference: RelationalCommitReceipt,
        binding: &AdmittedRelationalBranchBasis,
        published_partition_delta: crate::storage::RelationalPublishedPartitionDelta,
        envelope: Arc<CanonicalCommitEnvelope>,
    ) -> Result<PreparedRecoveredRelationalPublication, PublicationPreparationDenial> {
        let selected_branch_state = SelectedRelationalBranchState::from_owner_basis(
            self.history.runtime_instance_id,
            binding,
        )
        .map_err(|error| error.detail())?;
        let schema_registry = selected_branch_state
            .root()
            .map(|root| root.schema_authority().registry().clone())
            .unwrap_or_else(|| self.schema.clone());
        let original_artifact = self.history.commit_artifact(commit_id);
        let content = envelope
            .content
            .readmit_unchanged_root(execution, &binding.inner.root)
            .map_err(|cause| {
                PublicationPreparationDenial::from(
                    crate::branch::root::RelationalBranchRootCaptureDenial::Content(cause),
                )
            })?;
        let validated = validate_publication(
            self.history,
            PublicationRequest {
                commit_id,
                commit_reference: &commit_reference,
                binding,
                envelope: &envelope,
                sequence: PublicationSequence::RecoveryTruth,
            },
        )?;
        let root = self.history.prepare_branch_root_capture(
            execution,
            selected_branch_state.state(),
            &published_partition_delta,
            selected_branch_state.root(),
            Arc::clone(&envelope),
            &content,
            &schema_registry,
            self.symbols,
            None,
        )?;
        let prepared = assemble_versioned_publication(
            execution,
            commit_reference,
            validated,
            root,
            envelope,
            self.history.runtime_instance_id,
            None,
            None,
            original_artifact.as_deref(),
        )?;
        Ok(PreparedRecoveredRelationalPublication {
            publication:
                crate::runtime::state::subsystems::history::PreparedRecoveredVersionedArtifactPublication::from_recovery_readmission(
                    prepared,
                ).map_err(|detail| PublicationPreparationDenial::Invalid(detail.to_owned()))?,
        })
    }
}
