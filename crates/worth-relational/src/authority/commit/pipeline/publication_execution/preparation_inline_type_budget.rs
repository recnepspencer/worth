//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<PreparedPublicationPhase>("crates/worth-relational/src/authority/commit/pipeline/publication_execution/preparation.rs", "PreparedPublicationPhase", 128 * 1024);
        owner.field::<crate::runtime::PendingRecordAllocations>("record_allocations");
        owner.field::<crate::runtime::PreparedSymbolPublication>("symbol_meanings");
        owner.field::<crate::runtime::RuntimeSymbolTable>("_candidate_symbols");
        owner.field::<crate::storage::PreparedMirrorPublication>("mirror_publication");
        owner.field::<crate::mvcc::PreparedRelationalPublication>("prepared_history");
        owner.field::<crate::runtime::ValidatedLineageEventBatch>("validated_lineage_events");
        owner.field::<Vec<crate::lineage::data::LineageNode>>("lineage_nodes");
        owner.field::<crate::storage::overlay::PublicationArtifacts>("artifacts");
        owner.field::<Vec<crate::transactions::data::RecordRef>>("changed_records");
        owner.field::<VersionId>("version_id");
        owner.field::<Option<VersionId>>("previous_branch_head_version");
        owner.field::<CommitId>("commit_id");
        owner.field::<RelationalCommitReceipt>("commit_reference");
        owner.field::<BranchId>("branch_id");
        owner.field::<Vec<CommitId>>("merge_base_commits");
        owner.field::<Vec<BranchId>>("merge_parent_branches");
        owner.field::<Vec<crate::diagnostics::data::RelationalDiagnosticArtifact>>(
            "deferred_diagnostic_artifacts",
        );
        owner.field::<crate::runtime::PreparedCanonicalPublicationRoute>(
            "canonical_publication_route",
        );
    }
    {
        let mut owner = census.owner::<PreparedPublicationCompletion>("crates/worth-relational/src/authority/commit/pipeline/publication_execution/preparation.rs", "PreparedPublicationCompletion", 128 * 1024);
        owner.field::<crate::storage::PreparedMirrorPublication>("mirror_publication");
        owner.field::<crate::mvcc::PreparedRelationalPublicationAccelerators>("prepared_history");
        owner.field::<crate::runtime::ValidatedLineageEventBatch>("validated_lineage_events");
        owner.field::<Vec<crate::lineage::data::LineageNode>>("lineage_nodes");
        owner.field::<crate::storage::overlay::PublicationArtifacts>("artifacts");
        owner.field::<Vec<crate::transactions::data::RecordRef>>("changed_records");
        owner.field::<VersionId>("version_id");
        owner.field::<Option<VersionId>>("previous_branch_head_version");
        owner.field::<CommitId>("commit_id");
        owner.field::<RelationalCommitReceipt>("commit_reference");
        owner.field::<BranchId>("branch_id");
        owner.field::<Vec<CommitId>>("merge_base_commits");
        owner.field::<Vec<BranchId>>("merge_parent_branches");
        owner.field::<Vec<crate::diagnostics::data::RelationalDiagnosticArtifact>>(
            "deferred_diagnostic_artifacts",
        );
    }
    {
        let mut owner = census.owner::<PreparePublicationPhaseInput<'static>>("crates/worth-relational/src/authority/commit/pipeline/publication_execution/preparation.rs", "PreparePublicationPhaseInput", 128 * 1024);
        owner.field::<&'static crate::transactions::data::CommitStructuralSummary>("structural");
        owner.field::<&'static [crate::validation::engine::InvariantExecutionResult]>("validation");
        owner.field::<Option<crate::mvcc::RelationalCallerAssociation>>("caller_association");
        owner.field::<&'static crate::validation::candidate_source::NativeManagedSourceInterpretation>("source_interpretation");
        owner.field::<Option<Arc<crate::durability::RecoveredCodecPublicationCustody>>>(
            "recovered_codec_publication_custody",
        );
        owner.field::<Option<crate::mvcc::RequiredOutputPublicationAdmission>>(
            "required_output_publication",
        );
        owner.field::<Option<Arc<crate::history::data::CanonicalRequiredOutputMaterialization>>>(
            "required_output_materialization",
        );
        owner.field::<&'static crate::transactions::data::CommitCreatedEntityBindings>(
            "created_entities",
        );
        owner.field::<&'static crate::transactions::data::CommitCreatedRelationBindings>(
            "created_relations",
        );
        owner.field::<&'static mut CommitLog>("commit_log");
        owner.field::<crate::storage::overlay::WorkingState>("working_state");
        owner
            .field::<crate::authority::commit::phases::applied_mutation::PendingMutationIdentities>(
                "mutation_identities",
            );
        owner.field::<crate::branch::SelectedRelationalBranchState>("selected_branch_state");
        owner.field::<CommitPhaseBacking<PublicationPreparation>>("publication");
        owner.field::<VersionId>("version_id");
        owner.field::<Option<VersionId>>("previous_branch_head_version");
        owner.field::<CommitId>("commit_id");
        owner.field::<RelationalCommitReceipt>("commit_reference");
        owner.field::<crate::branch::AdmittedRelationalBranchBasis>("branch_basis");
        owner.field::<BranchId>("branch_id");
        owner.field::<Vec<CommitId>>("merge_base_commits");
        owner.field::<Vec<BranchId>>("merge_parent_branches");
    }
}
