//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<AssembledCommitExecution>(
            "crates/worth-relational/src/authority/commit/pipeline/artifact_execution.rs",
            "AssembledCommitExecution",
            128 * 1024,
        );
        owner.field::<std::sync::Arc<crate::validation::candidate_source::NativeManagedSourceInterpretation>>("source_interpretation");
        owner.field::<super::super::execution_admission::AdmittedCommitState>("admitted");
        owner.field::<crate::branch::SelectedRelationalBranchState>("selected_branch_state");
        owner.field::<crate::transactions::data::CommitStructuralSummary>(
            "public_structural_summary",
        );
        owner.field::<crate::storage::overlay::WorkingState>("working_state");
        owner.field::<Vec<crate::validation::engine::InvariantExecutionResult>>(
            "invariant_executions",
        );
        owner.field::<crate::identity::data::VersionId>("version_id");
        owner.field::<crate::transactions::data::CommitCreatedEntityBindings>("created_entities");
        owner
            .field::<crate::transactions::data::CommitCreatedRelationBindings>("created_relations");
        owner
            .field::<crate::authority::commit::phases::applied_mutation::PendingMutationIdentities>(
                "mutation_identities",
            );
        owner.field::<crate::authority::commit::phases::history::ResolvedCommitHistory>("history");
        owner.field::<Vec<crate::history::data::BranchId>>("merge_parent_branches");
        owner.field::<CommitPhaseBacking<PublicationPreparation>>("publication");
        owner.field::<crate::snapshots::data::SnapshotHandle>("publication_snapshot");
        owner.field::<Vec<crate::transactions::data::AspectEvaluationTrace>>(
            "aspect_evaluation_traces",
        );
        owner.field::<Vec<preparation::PreparedAspectEmissionTrace>>("aspect_emission_traces");
    }
}
