use super::phase_backing::CommitPhaseBacking;
use super::snapshot_validation::SnapshotValidatedCommitExecution;
use worth_relational_contracts::internal_port as contracts;

pub mod authored_index_writes;
pub mod descriptive_touches;
pub mod index_catalog;
pub mod phase;
pub mod preparation;

use phase::{assemble_authoritative_publication_phase, ArtifactAssemblyInput};
use preparation::{PublicationArtifactOwners, PublicationPreparation};

pub(super) struct AssembledCommitExecution {
    lineage_deferred: bool,
    source_interpretation:
        std::sync::Arc<worth_relational_mutation::internal_port::validation::candidate_source::interpretation::NativeManagedSourceInterpretation>,
    admitted: super::execution_admission::AdmittedCommitState,
    selected_branch_state: worth_relational_branch::internal_port::branch::root_selection::SelectedRelationalBranchState,
    public_structural_summary: contracts::transactions::data::outcomes::commit_summaries::CommitStructuralSummary,
    working_state: worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState,
    invariant_executions: Vec<worth_relational_mutation::internal_port::validation::engine::result::InvariantExecutionResult>,
    version_id: contracts::identity::data::primitives::VersionId,
    created_entities: contracts::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings,
    created_relations: contracts::transactions::data::outcomes::created_relation_bindings::CommitCreatedRelationBindings,
    mutation_identities:
        worth_relational_mutation::internal_port::authority::mutation::pending_identities::PendingMutationIdentities,
    history: crate::authority::commit::phases::history::ResolvedCommitHistory,
    publication: CommitPhaseBacking<PublicationPreparation>,
    publication_snapshot: worth_relational_branch::internal_port::snapshots::data::SnapshotHandle,
    aspect_evaluation_traces: Vec<contracts::transactions::data::aspect_traces::AspectEvaluationTrace>,
    aspect_emission_traces: Vec<preparation::PreparedAspectEmissionTrace>,
}

impl AssembledCommitExecution {
    #[allow(clippy::type_complexity)]
    pub(super) fn into_parts(
        self,
    ) -> (
        super::execution_admission::AdmittedCommitState,
        worth_relational_branch::internal_port::branch::root_selection::SelectedRelationalBranchState,
        contracts::transactions::data::outcomes::commit_summaries::CommitStructuralSummary,
        worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState,
        Vec<worth_relational_mutation::internal_port::validation::engine::result::InvariantExecutionResult>,
        contracts::identity::data::primitives::VersionId,
        contracts::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings,
        contracts::transactions::data::outcomes::created_relation_bindings::CommitCreatedRelationBindings,
        worth_relational_mutation::internal_port::authority::mutation::pending_identities::PendingMutationIdentities,
        crate::authority::commit::phases::history::ResolvedCommitHistory,
        CommitPhaseBacking<PublicationPreparation>,
        worth_relational_branch::internal_port::snapshots::data::SnapshotHandle,
        Vec<contracts::transactions::data::aspect_traces::AspectEvaluationTrace>,
        Vec<preparation::PreparedAspectEmissionTrace>,
        std::sync::Arc<worth_relational_mutation::internal_port::validation::candidate_source::interpretation::NativeManagedSourceInterpretation>,
        bool,
    ){
        (
            self.admitted,
            self.selected_branch_state,
            self.public_structural_summary,
            self.working_state,
            self.invariant_executions,
            self.version_id,
            self.created_entities,
            self.created_relations,
            self.mutation_identities,
            self.history,
            self.publication,
            self.publication_snapshot,
            self.aspect_evaluation_traces,
            self.aspect_emission_traces,
            self.source_interpretation,
            self.lineage_deferred,
        )
    }
}

pub(super) fn assemble_commit_artifacts(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    owners: PublicationArtifactOwners<'_>,
    max_patch_records: Option<usize>,
    validated: CommitPhaseBacking<SnapshotValidatedCommitExecution>,
    lineage_deferred: bool,
) -> Result<
    CommitPhaseBacking<AssembledCommitExecution>,
    contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
> {
    // Producer temporaries retire before funding the retained successor,
    // matching the former caller-side allocation boundary.
    let result = {
        let validated = validated.into_inner();
        super::consult_stage(execution, super::CommitStage::Artifact)?;
        let (
            mut admitted,
            selected_branch_state,
            public_structural_summary,
            mut working_state,
            content,
            invariant_executions,
            version_id,
            effect,
            created_entities,
            created_relations,
            mutation_identities,
            history,
            schema_continuity,
            deferred_diagnostic_artifacts,
            source_interpretation,
        ) = validated.into_parts();
        let (_, validation_input, merged_plan, commit_log, phase_timing) =
            admitted.phase_view().into_parts();
        let naming_plan = validation_input.naming_plan();
        let publication = assemble_authoritative_publication_phase(
            owners,
            max_patch_records,
            commit_log,
            phase_timing,
            ArtifactAssemblyInput {
                defer_lineage: lineage_deferred,
                working_state: &mut working_state,
                content,
                selected_branch_state: &selected_branch_state,
                effect,
                commit_reference: &history.commit_reference,
                branch_id: &history.branch_id,
                version_id,
                merged_plan,
                created_entities: &created_entities,
                created_relations: &created_relations,
                naming_plan,
                record_allocations: &mutation_identities.records,
                schema_continuity: &schema_continuity,
                deferred_diagnostic_artifacts,
            },
            execution,
        )?;
        let aspect_evaluation_traces = publication.value().aspect_evaluation_traces().to_vec();
        let aspect_emission_traces = publication.value().aspect_emission_traces().to_vec();
        let publication_snapshot = publication.value().snapshot().clone();
        AssembledCommitExecution {
            lineage_deferred,
            source_interpretation,
            admitted,
            selected_branch_state,
            public_structural_summary,
            working_state,
            invariant_executions,
            version_id,
            created_entities,
            created_relations,
            mutation_identities,
            history,
            publication,
            publication_snapshot,
            aspect_evaluation_traces,
            aspect_emission_traces,
        }
    };
    CommitPhaseBacking::new(execution, result)
}
#[cfg(test)]
#[path = "artifact_execution_inline_type_budget.rs"]
pub mod inline_type_budget;
