//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;
use crate::authority::commit::pipeline::artifact_execution::preparation::PublicationPreparation;
use crate::publication::live_publication_inputs::BoundLivePublicationInputs;
use contracts::history::data::commit_receipt::RelationalCommitReceipt;
use contracts::history::data::coordinates::BranchId;
use contracts::history::data::coordinates::CommitId;
use contracts::identity::data::primitives::VersionId;
use contracts::transactions::data::commit_log::CommitLog;
use worth_relational_contracts::internal_port as contracts;

pub(crate) fn collect(census: &mut contracts::inline_type_sizes::census::Census) {
    {
        let mut owner = census.owner::<PreparedPublicationPhase>("crates/worth-relational/src/authority/commit/pipeline/publication_execution/preparation.rs", "PreparedPublicationPhase", 128 * 1024);
        owner.field::<Option<BoundLivePublicationInputs>>("live_inputs");
        owner
            .field::<worth_relational_storage::internal_port::runtime::state::subsystems::record_identity::pending_allocations::PendingRecordAllocations>(
                "record_allocations",
            );
        owner.field::<worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::reservations::PreparedSymbolPublication>("symbol_meanings");
        owner
            .field::<worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable>(
                "_candidate_symbols",
            );
        owner.field::<worth_relational_storage::internal_port::storage::mirror_plan::PreparedMirrorPublication>("mirror_publication");
        owner.field::<crate::mvcc::publication::authority::PreparedRelationalPublication>(
            "prepared_history",
        );
        owner.field::<worth_relational_lineage::internal_port::runtime::state::subsystems::lineage::state::ValidatedLineageEventBatch>(
            "validated_lineage_events",
        );
        owner.field::<Vec<contracts::lineage::data::graph::LineageNode>>("lineage_nodes");
        owner.field::<Vec<(
            contracts::identity::data::record_ids::RelationId,
            contracts::identity::data::primitives::LineageId,
        )>>("relation_lineages");
        owner.field::<crate::publication::authority::artifacts::PublicationArtifacts>("artifacts");
        owner.field::<Vec<contracts::transactions::data::primitives::RecordRef>>("changed_records");
        owner.field::<VersionId>("version_id");
        owner.field::<Option<VersionId>>("previous_branch_head_version");
        owner.field::<CommitId>("commit_id");
        owner.field::<RelationalCommitReceipt>("commit_reference");
        owner.field::<BranchId>("branch_id");
        owner.field::<Vec<CommitId>>("merge_base_commits");
        owner.field::<Vec<BranchId>>("merge_parent_branches");
        owner.field::<Vec<contracts::diagnostics::data::artifacts::RelationalDiagnosticArtifact>>(
            "deferred_diagnostic_artifacts",
        );
        owner.field::<crate::runtime::state::subsystems::history::canonical_publication_routes::PreparedCanonicalPublicationRoute>(
            "canonical_publication_route",
        );
    }
    {
        let mut owner = census.owner::<PreparedPublicationCompletion>("crates/worth-relational/src/authority/commit/pipeline/publication_execution/preparation.rs", "PreparedPublicationCompletion", 128 * 1024);
        owner.field::<worth_relational_storage::internal_port::storage::mirror_plan::PreparedMirrorPublication>("mirror_publication");
        owner.field::<crate::mvcc::publication::authority::PreparedRelationalPublicationAccelerators>("prepared_history");
        owner.field::<worth_relational_lineage::internal_port::runtime::state::subsystems::lineage::state::ValidatedLineageEventBatch>(
            "validated_lineage_events",
        );
        owner.field::<Vec<contracts::lineage::data::graph::LineageNode>>("lineage_nodes");
        owner.field::<Vec<(
            contracts::identity::data::record_ids::RelationId,
            contracts::identity::data::primitives::LineageId,
        )>>("relation_lineages");
        owner.field::<crate::publication::authority::artifacts::PublicationArtifacts>("artifacts");
        owner.field::<Vec<contracts::transactions::data::primitives::RecordRef>>("changed_records");
        owner.field::<VersionId>("version_id");
        owner.field::<Option<VersionId>>("previous_branch_head_version");
        owner.field::<CommitId>("commit_id");
        owner.field::<RelationalCommitReceipt>("commit_reference");
        owner.field::<BranchId>("branch_id");
        owner.field::<Vec<CommitId>>("merge_base_commits");
        owner.field::<Vec<BranchId>>("merge_parent_branches");
        owner.field::<Vec<contracts::diagnostics::data::artifacts::RelationalDiagnosticArtifact>>(
            "deferred_diagnostic_artifacts",
        );
    }
    {
        let mut owner = census.owner::<PreparePublicationPhaseInput<'static>>("crates/worth-relational/src/authority/commit/pipeline/publication_execution/preparation/input.rs", "PreparePublicationPhaseInput", 128 * 1024);
        owner.field::<&'static contracts::transactions::data::outcomes::commit_summaries::CommitStructuralSummary>("structural");
        owner.field::<&'static [worth_relational_mutation::internal_port::validation::engine::result::InvariantExecutionResult]>("validation");
        owner.field::<Option<worth_relational_branch::internal_port::history::data::caller_association::RelationalCallerAssociation>>(
            "caller_association",
        );
        owner.field::<Option<&'static contracts::lineage::data::postures::CommitNamingPlan>>(
            "naming_plan",
        );
        owner.field::<&'static worth_relational_mutation::internal_port::validation::candidate_source::interpretation::NativeManagedSourceInterpretation>("source_interpretation");
        owner.field::<Option<Arc<worth_relational_durability::internal_port::durability::authority::recovery::codec_transition::publication_custody::RecoveredCodecPublicationCustody>>>(
            "recovered_codec_publication_custody",
        );
        owner.field::<Option<
            crate::mvcc::publication::application_publication::RequiredOutputPublicationAdmission,
        >>("required_output_publication");
        owner.field::<Option<Arc<worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_materialization::CanonicalRequiredOutputMaterialization>>>(
            "required_output_materialization",
        );
        owner.field::<&'static contracts::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings>(
            "created_entities",
        );
        owner.field::<&'static contracts::transactions::data::outcomes::created_relation_bindings::CommitCreatedRelationBindings>(
            "created_relations",
        );
        owner.field::<&'static mut CommitLog>("commit_log");
        owner.field::<worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState>("working_state");
        owner
            .field::<worth_relational_mutation::internal_port::authority::mutation::pending_identities::PendingMutationIdentities>(
                "mutation_identities",
            );
        owner.field::<worth_relational_branch::internal_port::branch::root_selection::SelectedRelationalBranchState>(
            "selected_branch_state",
        );
        owner.field::<CommitPhaseBacking<PublicationPreparation>>("publication");
        owner.field::<VersionId>("version_id");
        owner.field::<Option<VersionId>>("previous_branch_head_version");
        owner.field::<CommitId>("commit_id");
        owner.field::<RelationalCommitReceipt>("commit_reference");
        owner.field::<worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis>("branch_basis");
        owner.field::<BranchId>("branch_id");
        owner.field::<Vec<CommitId>>("merge_base_commits");
        owner.field::<Vec<BranchId>>("merge_parent_branches");
    }
}
