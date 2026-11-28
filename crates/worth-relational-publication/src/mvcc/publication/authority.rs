use contracts::branch::target::RelationalBranchTargetOwnerPort as _;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeOwnerPort as _;
use worth_relational_contracts::internal_port as contracts;
pub mod recovered_preparation;
use super::PublicationPreparationDenial;
use std::sync::Arc;

use contracts::history::data::commit_receipt::RelationalCommitReceipt;
use contracts::history::data::coordinates::CommitId;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis;
use worth_relational_branch::internal_port::branch::root_selection::SelectedRelationalBranchState;
use worth_relational_branch::internal_port::history::commit::artifact::RelationalCommitArtifact;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelope;

use super::validation::{
    validate_publication, PublicationRequest, PublicationSequence, ValidatedPublication,
};

pub struct RelationalPublicationAuthority<'runtime> {
    pub history: &'runtime worth_relational_branch::internal_port::runtime::state::subsystems::history::HistorySubsystem,
    pub symbols:
        &'runtime worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
    pub schema: &'runtime contracts::schema::data::RelationalSchemaRegistry,
}

pub struct PreparedRelationalPublication {
    publication: worth_relational_branch::internal_port::runtime::state::subsystems::history::history_publication::PreparedVersionedArtifactPublication,
}

pub struct PreparedRecoveredRelationalPublication {
    publication:
        worth_relational_branch::internal_port::runtime::state::subsystems::history::history_publication::PreparedRecoveredVersionedArtifactPublication,
}

pub(crate) struct PreparedRelationalPublicationAccelerators {
    publication: worth_relational_branch::internal_port::runtime::state::subsystems::history::history_publication::PreparedVersionedArtifactAccelerators,
}

impl PreparedRelationalPublication {
    pub(crate) fn caller_association(
        &self,
    ) -> Option<&worth_relational_branch::internal_port::history::data::caller_association::RelationalCallerAssociation>{
        self.publication.envelope.caller_association()
    }
    pub(crate) fn canonical_envelope(&self) -> &Arc<CanonicalCommitEnvelope> {
        &self.publication.envelope
    }
    pub(crate) fn envelope_residency(
        &self,
    ) -> Arc<worth_relational_branch::internal_port::history::commit::envelope_residency::RelationalCommitEnvelopeResidency>{
        self.publication.artifact.envelope_residency()
    }
    pub(crate) fn descriptive_touches(
        &self,
    ) -> &worth_relational_branch::internal_port::history::data::canonical_commit_envelope::descriptive_touches::RelationalDescriptiveTouchGraph{
        self.publication.envelope.descriptive_touches()
    }

    pub(crate) fn root(
        &self,
    ) -> &Arc<worth_relational_branch::internal_port::branch::root::RelationalBranchRoot> {
        &self.publication.root
    }

    pub(crate) fn into_canonical_parts(
        self,
    ) -> (
        worth_relational_branch::internal_port::branch::reference::RelationalBranchReferenceCell,
        Arc<worth_relational_branch::internal_port::branch::root::RelationalBranchRoot>,
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

    #[doc(hidden)]
    pub fn materialization_counts(&self) -> (u64, u64) {
        let cost = self.publication.root.publication_cost();
        (cost.touched_regions, cost.reused_regions)
    }
}

pub struct RecoveredPublicationInstall<'a> {
    pub history: &'a worth_relational_branch::internal_port::runtime::state::subsystems::history::HistorySubsystem,
}

impl PreparedRecoveredRelationalPublication {
    pub fn install_recovered<'a, Owner: ?Sized>(
        self,
        install: &'a Owner,
        positioned: &worth_relational_branch::internal_port::history::data::positioned_canonical_commit::PositionedCanonicalCommit,
    ) -> Result<(), String>
    where
        &'a Owner: Into<RecoveredPublicationInstall<'a>>,
    {
        worth_relational_durability::internal_port::durability::authority::recovery::versioned_artifact::install_recovered_versioned_artifact(
            install.into().history,
            self.publication,
            positioned,
        )
    }
}

impl PreparedRelationalPublicationAccelerators {
    pub(crate) fn install(
        self,
        history: &worth_relational_branch::internal_port::runtime::state::subsystems::history::HistorySubsystem,
        position: contracts::publication::patch::data::patch_position::PatchStreamPosition,
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
    prepared_root: worth_relational_branch::internal_port::branch::root::PreparedRelationalBranchRootCapture,
    envelope: Arc<CanonicalCommitEnvelope>,
    runtime_instance_id: u64,
    original_inputs: Option<VersionedPublicationOriginalInputs>,
    output_positions: Option<worth_relational_branch::internal_port::history::data::required_output_positions::RequiredOutputPublicationPositions>,
    recovered_original_artifact: Option<&RelationalCommitArtifact>,
) -> Result<
    worth_relational_branch::internal_port::runtime::state::subsystems::history::history_publication::PreparedVersionedArtifactPublication,
    PublicationPreparationDenial,
>{
    let commit_id = commit_reference.commit_id;
    let recovery_readmission = original_inputs.is_none();
    let prepared_root = match output_positions {
        Some(positions) => prepared_root.bind_output_publication_positions(positions),
        None => prepared_root,
    };
    let target = contracts::branch::target::RelationalBranchTarget::from_commit_receipt(
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
        let original = worth_relational_branch::internal_port::history::commit::artifact::original_inputs::OriginalCanonicalInputs {
            result_schema: root.retained_schema_authority(),
            prestate_schema: inputs.prestate_schema,
            mutation: inputs.mutation,
        };
        let original = original.capture(execution).map_err(|cause| match cause {
            contracts::durability::data::recovery_stop::RelationalRecoveryStop::Request(cause) => {
                PublicationPreparationDenial::Execution(cause)
            }
            contracts::durability::data::recovery_stop::RelationalRecoveryStop::Allocation(_) => {
                PublicationPreparationDenial::Execution(
                    contracts::execution::denial_cause::RelationalExecutionDenialCause::AllocationFailed,
                )
            }
            contracts::durability::data::recovery_stop::RelationalRecoveryStop::Content(cause) => {
                PublicationPreparationDenial::Execution(cause.into_execution_cause())
            }
            contracts::durability::data::recovery_stop::RelationalRecoveryStop::Durability(cause) => {
                PublicationPreparationDenial::Invalid(format!("original input capture: {cause:?}"))
            }
            contracts::durability::data::recovery_stop::RelationalRecoveryStop::AdmissionImage(cause) => {
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
        worth_relational_branch::internal_port::runtime::state::subsystems::history::history_publication::PreparedVersionedArtifactPublication {
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
    pub(crate) prestate_schema: Arc<worth_relational_branch::internal_port::branch::root::schema::RelationalBranchRootSchemaAuthority>,
    pub(crate) mutation: contracts::config::data::policies::MutationConfig,
}

impl PreparedRelationalPublication {
    pub(crate) fn from_versioned(
        publication: worth_relational_branch::internal_port::runtime::state::subsystems::history::history_publication::PreparedVersionedArtifactPublication,
    ) -> Self {
        Self { publication }
    }
}

impl<'runtime> RelationalPublicationAuthority<'runtime> {
    #[doc(hidden)]
    pub fn validate_versioned_publication_for_test(
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
