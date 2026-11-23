mod recovered_preparation;
use super::PublicationPreparationDenial;
use std::sync::Arc;

use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::branch::root_selection::SelectedRelationalBranchState;
use crate::history::commit::RelationalCommitArtifact;
use crate::history::data::{CanonicalCommitEnvelope, CommitId, RelationalCommitReceipt};

use super::validation::{
    validate_publication, PublicationRequest, PublicationSequence, ValidatedPublication,
};

pub(crate) struct RelationalPublicationAuthority<'runtime> {
    pub(crate) history: &'runtime crate::runtime::state::subsystems::history::HistorySubsystem,
    pub(crate) symbols:
        &'runtime crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
    pub(crate) schema: &'runtime crate::schema::data::RelationalSchemaRegistry,
}

pub(crate) struct PreparedRelationalPublication {
    publication: crate::runtime::state::subsystems::history::PreparedVersionedArtifactPublication,
}

pub(crate) struct PreparedRecoveredRelationalPublication {
    publication:
        crate::runtime::state::subsystems::history::PreparedRecoveredVersionedArtifactPublication,
}

pub(crate) struct PreparedRelationalPublicationAccelerators {
    publication: crate::runtime::state::subsystems::history::PreparedVersionedArtifactAccelerators,
}

impl PreparedRelationalPublication {
    pub(crate) fn caller_association(
        &self,
    ) -> Option<&crate::history::data::RelationalCallerAssociation> {
        self.publication.envelope.caller_association()
    }
    pub(crate) fn canonical_envelope(&self) -> &Arc<CanonicalCommitEnvelope> {
        &self.publication.envelope
    }
    pub(crate) fn envelope_residency(
        &self,
    ) -> Arc<crate::history::commit::RelationalCommitEnvelopeResidency> {
        self.publication.artifact.envelope_residency()
    }
    pub(crate) fn descriptive_touches(
        &self,
    ) -> &crate::history::data::RelationalDescriptiveTouchGraph {
        self.publication.envelope.descriptive_touches()
    }

    pub(crate) fn root(&self) -> &Arc<crate::branch::root::RelationalBranchRoot> {
        &self.publication.root
    }

    pub(crate) fn into_canonical_parts(
        self,
    ) -> (
        crate::branch::reference::RelationalBranchReferenceCell,
        Arc<crate::branch::root::RelationalBranchRoot>,
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

pub(crate) struct RecoveredPublicationInstall<'a> {
    pub(crate) history: &'a crate::runtime::state::subsystems::history::HistorySubsystem,
}

impl PreparedRecoveredRelationalPublication {
    pub(crate) fn install_recovered<'a, Owner: ?Sized>(
        self,
        install: &'a Owner,
        positioned: &crate::history::data::PositionedCanonicalCommit,
    ) -> Result<(), String>
    where
        &'a Owner: Into<RecoveredPublicationInstall<'a>>,
    {
        crate::durability::authority::install_recovered_versioned_artifact(
            install.into().history,
            self.publication,
            positioned,
        )
    }
}

impl PreparedRelationalPublicationAccelerators {
    pub(crate) fn install(
        self,
        history: &crate::runtime::state::subsystems::history::HistorySubsystem,
        position: crate::publication::patch::data::PatchStreamPosition,
    ) {
        self.publication.install(history, position);
    }
}

/// Complete initial-input and catalog custody after the selected root has been captured.
#[allow(clippy::too_many_arguments)]
pub(crate) fn assemble_versioned_publication(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    commit_reference: RelationalCommitReceipt,
    validated: ValidatedPublication,
    prepared_root: crate::branch::root::PreparedRelationalBranchRootCapture,
    envelope: Arc<CanonicalCommitEnvelope>,
    runtime_instance_id: u64,
    original_inputs: Option<VersionedPublicationOriginalInputs>,
    output_positions: Option<crate::history::data::RequiredOutputPublicationPositions>,
    recovered_original_artifact: Option<&RelationalCommitArtifact>,
) -> Result<
    crate::runtime::state::subsystems::history::PreparedVersionedArtifactPublication,
    PublicationPreparationDenial,
> {
    let commit_id = commit_reference.commit_id;
    let recovery_readmission = original_inputs.is_none();
    let prepared_root = match output_positions {
        Some(positions) => prepared_root.bind_output_publication_positions(positions),
        None => prepared_root,
    };
    let target = crate::branch::target::RelationalBranchTarget::from_commit_receipt(
        runtime_instance_id,
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
    let original_inputs = if let Some(inputs) = original_inputs {
        let original = crate::history::commit::OriginalCanonicalInputs {
            result_schema: root.retained_schema_authority(),
            prestate_schema: inputs.prestate_schema,
            mutation: inputs.mutation,
        };
        let original = original.capture(execution).map_err(|cause| match cause {
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
                PublicationPreparationDenial::Invalid(format!("original input capture: {cause:?}"))
            }
            crate::durability::data::RelationalRecoveryStop::AdmissionImage(cause) => {
                PublicationPreparationDenial::Invalid(format!("original input capture: {cause:?}"))
            }
        })?;
        envelope
            .retain_recorded_original_inputs(original.recorded())
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
    if recovery_readmission
        && let Some(previous) =
            recovered_original_artifact.filter(|artifact| artifact.original_inputs().is_some())
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
    Ok(
        crate::runtime::state::subsystems::history::PreparedVersionedArtifactPublication {
            commit_id,
            commit_reference,
            branch_id: validated.branch_id,
            next_cell: validated.next_cell,
            envelope,
            root,
            artifact,
            new_authoritative_bytes,
            recovery_readmission,
            sequence: validated.sequence_successors,
        },
    )
}

pub(crate) struct VersionedPublicationOriginalInputs {
    pub(crate) prestate_schema: Arc<crate::branch::root::RelationalBranchRootSchemaAuthority>,
    pub(crate) mutation: crate::config::data::MutationConfig,
}

impl PreparedRelationalPublication {
    pub(crate) fn from_versioned(
        publication: crate::runtime::state::subsystems::history::PreparedVersionedArtifactPublication,
    ) -> Self {
        Self { publication }
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
            self.history,
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
