//! Operands from the assembled commit consumed by publication preparation.
use std::sync::Arc;
use worth_relational_contracts::internal_port as contracts;

use super::CommitPhaseBacking;
use crate::authority::commit::pipeline::artifact_execution::preparation::PublicationPreparation;
use contracts::history::data::commit_receipt::RelationalCommitReceipt;
use contracts::history::data::coordinates::BranchId;
use contracts::history::data::coordinates::CommitId;
use contracts::identity::data::primitives::VersionId;
use contracts::transactions::data::commit_log::CommitLog;

pub(in crate::internal_port::authority::commit::pipeline::publication_execution) struct PreparePublicationPhaseInput<'a> {
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) structural: &'a contracts::transactions::data::outcomes::commit_summaries::CommitStructuralSummary,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) validation: &'a [worth_relational_mutation::internal_port::validation::engine::result::InvariantExecutionResult],
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) caller_association: Option<worth_relational_branch::internal_port::history::data::caller_association::RelationalCallerAssociation>,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) lineage_deferred: bool,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) naming_plan: Option<&'a contracts::lineage::data::postures::CommitNamingPlan>,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) source_interpretation:
        &'a worth_relational_mutation::internal_port::validation::candidate_source::interpretation::NativeManagedSourceInterpretation,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) recovered_codec_publication_custody:
        Option<Arc<worth_relational_durability::internal_port::durability::authority::recovery::codec_transition::publication_custody::RecoveredCodecPublicationCustody>>,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) required_output_publication: Option<crate::mvcc::publication::application_publication::RequiredOutputPublicationAdmission>,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) required_output_materialization:
        Option<Arc<worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_materialization::CanonicalRequiredOutputMaterialization>>,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) created_entities: &'a contracts::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) created_relations: &'a contracts::transactions::data::outcomes::created_relation_bindings::CommitCreatedRelationBindings,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) commit_log: &'a mut CommitLog,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) working_state: worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) mutation_identities:
        worth_relational_mutation::internal_port::authority::mutation::pending_identities::PendingMutationIdentities,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) selected_branch_state: worth_relational_branch::internal_port::branch::root_selection::SelectedRelationalBranchState,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) publication: CommitPhaseBacking<PublicationPreparation>,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) version_id: VersionId,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) previous_branch_head_version: Option<VersionId>,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) commit_id: CommitId,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) commit_reference: RelationalCommitReceipt,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) branch_basis: worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) branch_id: BranchId,
}
