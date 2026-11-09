mod recovered_preparation;
mod runtime;
use super::PublicationPreparationDenial;
use runtime::PublicationPreparationRuntime;
use std::sync::Arc;

use crate::branch::{AdmittedRelationalBranchBasis, SelectedRelationalBranchState};
use crate::history::data::{CanonicalCommitEnvelope, CommitId, RelationalCommitReceipt};
use crate::history::RelationalCommitArtifact;
use crate::runtime::RelationalRuntime;

use super::validation::{
    validate_publication, PublicationRequest, PublicationSequence, ValidatedPublication,
};

pub(crate) struct RelationalPublicationAuthority<'runtime> {
    runtime: &'runtime RelationalRuntime,
}

pub(crate) struct RelationalPreparationPublicationAuthority<'runtime> {
    runtime: &'runtime crate::runtime::RelationalPreparationRuntime,
}

pub(crate) struct PreparedRelationalPublication {
    publication: crate::runtime::PreparedVersionedArtifactPublication,
}

pub(crate) struct PreparedRecoveredRelationalPublication {
    publication: crate::runtime::PreparedRecoveredVersionedArtifactPublication,
}

pub(crate) struct PreparedRelationalPublicationAccelerators {
    publication: crate::runtime::PreparedVersionedArtifactAccelerators,
}

impl PreparedRelationalPublication {
    pub(crate) fn caller_association(&self) -> Option<&crate::mvcc::RelationalCallerAssociation> {
        self.publication.envelope.caller_association()
    }
    pub(crate) fn canonical_envelope(&self) -> &Arc<CanonicalCommitEnvelope> {
        &self.publication.envelope
    }
    pub(crate) fn envelope_residency(
        &self,
    ) -> Arc<crate::history::RelationalCommitEnvelopeResidency> {
        self.publication.artifact.envelope_residency()
    }
    pub(crate) fn descriptive_touches(
        &self,
    ) -> &crate::history::data::RelationalDescriptiveTouchGraph {
        self.publication.envelope.descriptive_touches()
    }

    pub(crate) fn root(&self) -> &Arc<crate::branch::RelationalBranchRoot> {
        &self.publication.root
    }

    pub(crate) fn into_canonical_parts(
        self,
    ) -> (
        crate::branch::RelationalBranchReferenceCell,
        Arc<crate::branch::RelationalBranchRoot>,
        Arc<CanonicalCommitEnvelope>,
        PreparedRelationalPublicationAccelerators,
    ) {
        let (next_cell, root, envelope, publication) =
            self.publication.into_canonical_and_accelerators();
        (
            next_cell,
            root,
            envelope,
            PreparedRelationalPublicationAccelerators { publication },
        )
    }

    #[cfg(test)]
    pub(crate) fn materialization_counts(&self) -> (u64, u64) {
        let cost = self.publication.root.publication_cost();
        (cost.touched_regions, cost.reused_regions)
    }
}

impl PreparedRecoveredRelationalPublication {
    pub(crate) fn install_recovered(
        self,
        runtime: &RelationalRuntime,
        positioned: &crate::history::data::PositionedCanonicalCommit,
    ) -> Result<(), String> {
        runtime
            .history
            .install_prepared_recovered_versioned_artifact(self.publication, positioned)
    }
}

impl PreparedRelationalPublicationAccelerators {
    pub(crate) fn install(
        self,
        runtime: &RelationalRuntime,
        position: crate::publication::patch::data::PatchStreamPosition,
    ) {
        runtime
            .history
            .install_prepared_versioned_accelerators(self.publication, position);
    }
}

impl RelationalRuntime {
    pub(crate) fn mvcc_publication_authority(&self) -> RelationalPublicationAuthority<'_> {
        RelationalPublicationAuthority { runtime: self }
    }
}

impl crate::runtime::RelationalPreparationRuntime {
    pub(crate) fn mvcc_publication_authority(
        &self,
    ) -> RelationalPreparationPublicationAuthority<'_> {
        RelationalPreparationPublicationAuthority { runtime: self }
    }
}

#[allow(clippy::too_many_arguments)]
fn prepare_versioned_publication(
    runtime: &impl PublicationPreparationRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    commit_id: CommitId,
    commit_reference: RelationalCommitReceipt,
    binding: &AdmittedRelationalBranchBasis,
    selected_branch_state: &SelectedRelationalBranchState,
    published_partition_delta: crate::storage::RelationalPublishedPartitionDelta,
    envelope: Arc<CanonicalCommitEnvelope>,
    schema_registry: &crate::schema::data::RelationalSchemaRegistry,
    sequence: PublicationSequence,
    output_positions: Option<crate::history::data::RequiredOutputPublicationPositions>,
    original_artifact: Option<&RelationalCommitArtifact>,
    recovered_content: Option<&crate::content::ContentPublication>,
) -> Result<crate::runtime::PreparedVersionedArtifactPublication, PublicationPreparationDenial> {
    let validated = runtime.validate_publication_request(PublicationRequest {
        commit_id,
        commit_reference: &commit_reference,
        binding,
        envelope: envelope.as_ref(),
        sequence,
    })?;
    let sequence_successors = crate::runtime::PreparedCommitSequence::new(&commit_reference)?;
    let prepared_root = runtime.prepare_publication_root(
        execution,
        selected_branch_state,
        &published_partition_delta,
        &envelope,
        recovered_content.unwrap_or(&envelope.content),
        schema_registry,
    )?;
    let prepared_root = match output_positions {
        Some(positions) => prepared_root.bind_output_publication_positions(positions),
        None => prepared_root,
    };
    let target = crate::branch::RelationalBranchTarget::from_commit_receipt(
        runtime.publication_runtime_instance_id(),
        &commit_reference,
        prepared_root
            .root()
            .descriptor()
            .cloned()
            .ok_or_else(|| "prepared branch root has no descriptor".to_owned())?,
    );
    validated
        .next_cell
        .replace_truth_target(worth_foundational::FoundationalBranchTarget::basis(target))
        .map_err(|denial| format!("prepared target binding denied: {denial:?}"))?;
    let root = prepared_root.into_root();
    let original_inputs = if matches!(sequence, PublicationSequence::Truth) {
        let original = crate::history::OriginalCanonicalInputs {
            result_schema: root.retained_schema_authority(),
            prestate_schema: selected_branch_state
                .state()
                .root()
                .retained_schema_authority(),
            mutation: runtime.publication_mutation_config(),
        };
        let recorded =
            crate::durability::source_initial_inputs::recorded::RecordedCanonicalInputs::capture(
                &original, execution,
            )
            .map_err(|cause| match cause {
                crate::durability::data::RelationalRecoveryStop::Request(cause) => {
                    PublicationPreparationDenial::Execution(cause)
                }
                crate::durability::data::RelationalRecoveryStop::Allocation(_) => {
                    PublicationPreparationDenial::Execution(
                        crate::execution::RelationalExecutionDenialCause::AllocationFailed,
                    )
                }
                crate::durability::data::RelationalRecoveryStop::Content(cause) => {
                    PublicationPreparationDenial::Execution(cause.into_execution_cause())
                }
                crate::durability::data::RelationalRecoveryStop::Durability(cause) => {
                    PublicationPreparationDenial::Invalid(format!(
                        "original input capture: {cause:?}"
                    ))
                }
                crate::durability::data::RelationalRecoveryStop::AdmissionImage(cause) => {
                    PublicationPreparationDenial::Invalid(format!(
                        "original input capture: {cause:?}"
                    ))
                }
            })?;
        envelope
            .retain_recorded_original_inputs(recorded)
            .map_err(|cause| PublicationPreparationDenial::Invalid(cause.into()))?;
        Some(original)
    } else {
        None
    };
    let mut artifact =
        RelationalCommitArtifact::from_envelope_with_root(Arc::clone(&envelope), Arc::clone(&root))
            .map_err(|denial| format!("prepared catalog artifact denied: {denial:?}"))?;
    if let Some(original) = original_inputs {
        artifact.set_original_inputs(original);
    }
    if matches!(sequence, PublicationSequence::RecoveryTruth)
        && let Some(previous) =
            original_artifact.filter(|artifact| artifact.original_inputs().is_some())
    {
        artifact
            .preserve_original_inputs(previous)
            .map_err(|cause| {
                PublicationPreparationDenial::Invalid(format!(
                    "original catalog input linkage: {cause:?}"
                ))
            })?;
    }
    let artifact = artifact
        .prepare_catalog_owner(execution)
        .map_err(PublicationPreparationDenial::Execution)?;
    let new_authoritative_bytes = root
        .publication_cost()
        .new_authoritative_bytes
        .saturating_add(
            artifact
                .authoritative_allocation_observations()
                .iter()
                .map(|allocation| allocation.authoritative_bytes)
                .sum(),
        );
    Ok(crate::runtime::PreparedVersionedArtifactPublication {
        commit_id,
        commit_reference,
        branch_id: validated.branch_id,
        next_cell: validated.next_cell,
        envelope,
        root,
        artifact,
        new_authoritative_bytes,
        recovery_readmission: matches!(sequence, PublicationSequence::RecoveryTruth),
        sequence: sequence_successors,
    })
}

impl RelationalPreparationPublicationAuthority<'_> {
    pub(crate) fn prepare_commit(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        commit_id: CommitId,
        commit_reference: RelationalCommitReceipt,
        binding: &AdmittedRelationalBranchBasis,
        selected_branch_state: &SelectedRelationalBranchState,
        published_partition_delta: crate::storage::RelationalPublishedPartitionDelta,
        envelope: Arc<CanonicalCommitEnvelope>,
        schema_registry: &crate::schema::data::RelationalSchemaRegistry,
        output_positions: Option<crate::history::data::RequiredOutputPublicationPositions>,
    ) -> Result<PreparedRelationalPublication, PublicationPreparationDenial> {
        let publication = prepare_versioned_publication(
            self.runtime,
            execution,
            commit_id,
            commit_reference,
            binding,
            selected_branch_state,
            published_partition_delta,
            envelope,
            schema_registry,
            PublicationSequence::Truth,
            output_positions,
            None,
            None,
        )?;
        Ok(PreparedRelationalPublication { publication })
    }
}

impl<'runtime> RelationalPublicationAuthority<'runtime> {
    #[cfg(test)]
    pub(crate) fn validate_versioned_publication(
        &self,
        commit_id: CommitId,
        commit_reference: &RelationalCommitReceipt,
        binding: &AdmittedRelationalBranchBasis,
        envelope: &CanonicalCommitEnvelope,
    ) -> Result<(), String> {
        validate_publication(
            self.runtime,
            PublicationRequest {
                commit_id,
                commit_reference,
                binding,
                envelope,
                sequence: PublicationSequence::Truth,
            },
        )
        .map(|_| ())
    }
}
