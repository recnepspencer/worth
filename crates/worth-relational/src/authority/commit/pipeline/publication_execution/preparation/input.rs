//! Operands from the assembled commit consumed by publication preparation.
use std::sync::Arc;

use super::CommitPhaseBacking;
use crate::authority::commit::pipeline::artifact_execution::preparation::PublicationPreparation;
use crate::history::data::{BranchId, CommitId, RelationalCommitReceipt};
use crate::identity::data::VersionId;
use crate::transactions::data::commit_log::CommitLog;

pub(in crate::authority::commit::pipeline::publication_execution) struct PreparePublicationPhaseInput<'a> {
    pub(in crate::authority::commit::pipeline::publication_execution) structural: &'a crate::transactions::data::outcomes::commit_summaries::CommitStructuralSummary,
    pub(in crate::authority::commit::pipeline::publication_execution) validation: &'a [crate::validation::engine::InvariantExecutionResult],
    pub(in crate::authority::commit::pipeline::publication_execution) caller_association: Option<crate::history::data::RelationalCallerAssociation>,
    pub(in crate::authority::commit::pipeline::publication_execution) lineage_deferred: bool,
    pub(in crate::authority::commit::pipeline::publication_execution) naming_plan: Option<&'a crate::lineage::data::CommitNamingPlan>,
    pub(in crate::authority::commit::pipeline::publication_execution) source_interpretation:
        &'a crate::validation::candidate_source::NativeManagedSourceInterpretation,
    pub(in crate::authority::commit::pipeline::publication_execution) recovered_codec_publication_custody:
        Option<Arc<crate::durability::authority::RecoveredCodecPublicationCustody>>,
    pub(in crate::authority::commit::pipeline::publication_execution) required_output_publication: Option<crate::mvcc::publication::application_publication::RequiredOutputPublicationAdmission>,
    pub(in crate::authority::commit::pipeline::publication_execution) required_output_materialization:
        Option<Arc<crate::history::data::CanonicalRequiredOutputMaterialization>>,
    pub(in crate::authority::commit::pipeline::publication_execution) created_entities: &'a crate::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings,
    pub(in crate::authority::commit::pipeline::publication_execution) created_relations: &'a crate::transactions::data::outcomes::created_relation_bindings::CommitCreatedRelationBindings,
    pub(in crate::authority::commit::pipeline::publication_execution) commit_log: &'a mut CommitLog,
    pub(in crate::authority::commit::pipeline::publication_execution) working_state: crate::storage::overlay::WorkingState,
    pub(in crate::authority::commit::pipeline::publication_execution) mutation_identities:
        crate::authority::commit::phases::applied_mutation::PendingMutationIdentities,
    pub(in crate::authority::commit::pipeline::publication_execution) selected_branch_state: crate::branch::root_selection::SelectedRelationalBranchState,
    pub(in crate::authority::commit::pipeline::publication_execution) publication: CommitPhaseBacking<PublicationPreparation>,
    pub(in crate::authority::commit::pipeline::publication_execution) version_id: VersionId,
    pub(in crate::authority::commit::pipeline::publication_execution) previous_branch_head_version: Option<VersionId>,
    pub(in crate::authority::commit::pipeline::publication_execution) commit_id: CommitId,
    pub(in crate::authority::commit::pipeline::publication_execution) commit_reference: RelationalCommitReceipt,
    pub(in crate::authority::commit::pipeline::publication_execution) branch_basis: crate::branch::basis::AdmittedRelationalBranchBasis,
    pub(in crate::authority::commit::pipeline::publication_execution) branch_id: BranchId,
    pub(in crate::authority::commit::pipeline::publication_execution) merge_base_commits: Vec<CommitId>,
    pub(in crate::authority::commit::pipeline::publication_execution) merge_parent_branches: Vec<BranchId>,
}
