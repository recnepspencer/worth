use super::snapshot_validation::SnapshotValidatedCommitExecution;

mod phase;
pub(super) mod preparation;

use phase::{assemble_authoritative_publication_phase, ArtifactAssemblyInput};
use preparation::PublicationPreparation;

pub(super) struct AssembledCommitExecution {
    admitted: super::execution_admission::AdmittedCommitExecution,
    selected_branch_state: crate::branch::SelectedRelationalBranchState,
    public_structural_summary: crate::transactions::data::CommitStructuralSummary,
    working_state: crate::storage::overlay::WorkingState,
    invariant_executions: Vec<crate::validation::engine::InvariantExecutionResult>,
    version_id: crate::identity::data::VersionId,
    created_entities: crate::transactions::data::CommitCreatedEntityBindings,
    created_relations: crate::transactions::data::CommitCreatedRelationBindings,
    record_allocations: crate::runtime::PendingRecordAllocations,
    history: crate::authority::commit::phases::history::ResolvedCommitHistory,
    merge_parent_branches: Vec<crate::history::data::BranchId>,
    publication: PublicationPreparation,
    publication_snapshot: crate::snapshots::data::SnapshotHandle,
    aspect_evaluation_traces: Vec<crate::transactions::data::AspectEvaluationTrace>,
    aspect_emission_traces: Vec<preparation::PreparedAspectEmissionTrace>,
}

impl AssembledCommitExecution {
    #[allow(clippy::type_complexity)]
    pub(super) fn into_parts(
        self,
    ) -> (
        super::execution_admission::AdmittedCommitExecution,
        crate::branch::SelectedRelationalBranchState,
        crate::transactions::data::CommitStructuralSummary,
        crate::storage::overlay::WorkingState,
        Vec<crate::validation::engine::InvariantExecutionResult>,
        crate::identity::data::VersionId,
        crate::transactions::data::CommitCreatedEntityBindings,
        crate::transactions::data::CommitCreatedRelationBindings,
        crate::runtime::PendingRecordAllocations,
        crate::authority::commit::phases::history::ResolvedCommitHistory,
        Vec<crate::history::data::BranchId>,
        PublicationPreparation,
        crate::snapshots::data::SnapshotHandle,
        Vec<crate::transactions::data::AspectEvaluationTrace>,
        Vec<preparation::PreparedAspectEmissionTrace>,
    ) {
        (
            self.admitted,
            self.selected_branch_state,
            self.public_structural_summary,
            self.working_state,
            self.invariant_executions,
            self.version_id,
            self.created_entities,
            self.created_relations,
            self.record_allocations,
            self.history,
            self.merge_parent_branches,
            self.publication,
            self.publication_snapshot,
            self.aspect_evaluation_traces,
            self.aspect_emission_traces,
        )
    }
}

pub(super) fn assemble_commit_artifacts(
    runtime: &crate::runtime::RelationalPreparationRuntime,
    validated: SnapshotValidatedCommitExecution,
    lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
) -> Result<AssembledCommitExecution, crate::transactions::data::TransactionCommitError> {
    let (
        mut admitted,
        selected_branch_state,
        public_structural_summary,
        mut working_state,
        invariant_executions,
        version_id,
        effect,
        created_entities,
        created_relations,
        record_allocations,
        history,
        merge_parent_branches,
        additional_diagnostics_entries,
        merge_execution_authority,
        schema_continuity,
        deferred_diagnostic_artifacts,
    ) = validated.into_parts();
    let strategy_artifacts = admitted.strategy_artifacts().cloned();
    let (_, _, merged_plan, _, commit_log, phase_timing) = admitted.phase_view().into_parts();
    let publication = assemble_authoritative_publication_phase(
        runtime,
        commit_log,
        phase_timing,
        ArtifactAssemblyInput {
            working_state: &mut working_state,
            effect,
            commit_reference: &history.commit_reference,
            branch_id: &history.branch_id,
            version_id,
            merge_parent_branches: &merge_parent_branches,
            merge_base_commits: &history.merge_base_commits,
            merged_plan,
            record_allocations: record_allocations.canonical(),
            strategy_commit_artifacts: strategy_artifacts,
            merge_execution_authority,
            schema_continuity: &schema_continuity,
            additional_diagnostics_entries,
            deferred_diagnostic_artifacts,
        },
        lease,
    )?;
    let aspect_evaluation_traces = publication.aspect_evaluation_traces().to_vec();
    let aspect_emission_traces = publication.aspect_emission_traces().to_vec();
    let publication_snapshot = publication.snapshot().clone();
    Ok(AssembledCommitExecution {
        admitted,
        selected_branch_state,
        public_structural_summary,
        working_state,
        invariant_executions,
        version_id,
        created_entities,
        created_relations,
        record_allocations,
        history,
        merge_parent_branches,
        publication,
        publication_snapshot,
        aspect_evaluation_traces,
        aspect_emission_traces,
    })
}
