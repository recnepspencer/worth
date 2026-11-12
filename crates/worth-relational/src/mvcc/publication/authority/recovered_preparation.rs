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
        let selected_branch_state = self
            .runtime
            .selected_branch_state(binding)
            .map_err(|error| error.detail())?;
        let schema_registry = selected_branch_state
            .root()
            .map(|root| root.schema_authority().registry().clone())
            .unwrap_or_else(|| self.runtime.config.schema.registry.clone());
        let original_artifact = self.runtime.history.commit_artifact(commit_id);
        let content = envelope
            .content
            .readmit_unchanged_root(execution, &binding.inner.root)
            .map_err(|cause| {
                PublicationPreparationDenial::from(
                    crate::branch::RelationalBranchRootCaptureDenial::Content(cause),
                )
            })?;
        let prepared = prepare_versioned_publication(
            self.runtime,
            execution,
            commit_id,
            commit_reference,
            binding,
            &selected_branch_state,
            published_partition_delta,
            envelope,
            &schema_registry,
            PublicationSequence::RecoveryTruth,
            None,
            original_artifact.as_deref(),
            Some(&content),
        )?;
        Ok(PreparedRecoveredRelationalPublication {
            publication:
                crate::runtime::PreparedRecoveredVersionedArtifactPublication::from_recovery_readmission(
                    prepared,
                ).map_err(|detail| PublicationPreparationDenial::Invalid(detail.to_owned()))?,
        })
    }
}
