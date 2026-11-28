//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;
use worth_relational_contracts::internal_port as contracts;

pub(crate) fn collect(census: &mut contracts::inline_type_sizes::census::Census) {
    {
        let mut owner = census.owner::<AssembledCommitExecution>(
            "crates/worth-relational/src/authority/commit/pipeline/artifact_execution.rs",
            "AssembledCommitExecution",
            128 * 1024,
        );
        owner.field::<std::sync::Arc<worth_relational_mutation::internal_port::validation::candidate_source::interpretation::NativeManagedSourceInterpretation>>("source_interpretation");
        owner.field::<super::super::execution_admission::AdmittedCommitState>("admitted");
        owner.field::<worth_relational_branch::internal_port::branch::root_selection::SelectedRelationalBranchState>(
            "selected_branch_state",
        );
        owner.field::<contracts::transactions::data::outcomes::commit_summaries::CommitStructuralSummary>(
            "public_structural_summary",
        );
        owner.field::<worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState>("working_state");
        owner.field::<Vec<worth_relational_mutation::internal_port::validation::engine::result::InvariantExecutionResult>>(
            "invariant_executions",
        );
        owner.field::<contracts::identity::data::primitives::VersionId>("version_id");
        owner.field::<contracts::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings>("created_entities");
        owner
            .field::<contracts::transactions::data::outcomes::created_relation_bindings::CommitCreatedRelationBindings>("created_relations");
        owner
            .field::<worth_relational_mutation::internal_port::authority::mutation::pending_identities::PendingMutationIdentities>(
                "mutation_identities",
            );
        owner.field::<crate::authority::commit::phases::history::ResolvedCommitHistory>("history");
        owner.field::<CommitPhaseBacking<PublicationPreparation>>("publication");
        owner.field::<worth_relational_branch::internal_port::snapshots::data::SnapshotHandle>(
            "publication_snapshot",
        );
        owner.field::<Vec<contracts::transactions::data::aspect_traces::AspectEvaluationTrace>>(
            "aspect_evaluation_traces",
        );
        owner.field::<Vec<preparation::PreparedAspectEmissionTrace>>("aspect_emission_traces");
    }
}
