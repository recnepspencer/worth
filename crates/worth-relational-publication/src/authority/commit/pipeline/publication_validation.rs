//! Selected draft authority becomes the smaller set needed for publication.
use worth_relational_contracts::internal_port as contracts;
use crate::authority::commit::phases::schema_continuity::inputs::SchemaContinuityRequest;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis;
use worth_relational_branch::internal_port::branch::home_program_step::AdmittedRelationalHomeProgramStep;
use worth_relational_branch::internal_port::branch::root::schema::RelationalBranchRootSchemaAuthority;
use worth_relational_durability::internal_port::durability::authority::recovery::codec_transition::publication_custody::RecoveredCodecPublicationCustody;
use contracts::history::data::coordinates::BranchId;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::CanonicalRequiredOutputOrigin;
use worth_relational_branch::internal_port::history::retention::obligation::RelationalTransactionRetentionObligation;
use contracts::lineage::data::postures::CommitNamingPlan;
use worth_relational_mutation::internal_port::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity;
use contracts::runtime::operation_control::RelationalOperationControl;
use worth_relational_indexes::internal_port::runtime::state::subsystems::indexing::definition_preparation::PreparedRelationalIndexDefinitions;
use contracts::schema::data::continuity::transition_plans::ProposedSchemaTransition;
use contracts::schema::data::continuity::transition_classification::SchemaReconciliationPolicy;
use worth_relational_branch::internal_port::schema::continuity::authority_input::SchemaContinuityAuthorityInput;
use std::sync::Arc;

pub trait DraftValidationInput {
    fn basis(&self) -> &AdmittedRelationalBranchBasis;
    fn merge_parent_bases(&self) -> &[AdmittedRelationalBranchBasis];
    fn control(&self) -> &RelationalOperationControl;
    fn coupled_proposal(&self) -> Option<&RelationalMutationProposalIdentity>;
    fn into_publication(self) -> CommitPublicationValidationInput;

    fn target_branch(&self) -> &BranchId {
        self.basis().identity().branch_id()
    }

    fn merge_parent_branch_ids(&self) -> Vec<BranchId> {
        self.merge_parent_bases()
            .iter()
            .map(|basis| basis.identity().branch_id().clone())
            .collect()
    }
}

#[derive(Debug)]
pub struct CommitPublicationValidationInput {
    pub basis: AdmittedRelationalBranchBasis,
    pub merge_parent_bases: Vec<AdmittedRelationalBranchBasis>,
    pub schema_authority: Arc<RelationalBranchRootSchemaAuthority>,
    pub schema_authority_input: Option<SchemaContinuityAuthorityInput>,
    pub transition: Option<ProposedSchemaTransition>,
    pub reconciliation: Option<SchemaReconciliationPolicy>,
    pub appendix_adoption: Option<Arc<CanonicalRequiredOutputOrigin>>,
    pub recovered_codec_publication_custody: Option<Arc<RecoveredCodecPublicationCustody>>,
    pub prepared_home_indexes: Option<Arc<PreparedRelationalIndexDefinitions>>,
    pub home_step: Option<Arc<AdmittedRelationalHomeProgramStep>>,
    pub retention: Option<Arc<RelationalTransactionRetentionObligation>>,
    pub naming_plan: Option<Arc<CommitNamingPlan>>,
}

impl CommitPublicationValidationInput {
    pub(super) fn basis(&self) -> &AdmittedRelationalBranchBasis {
        &self.basis
    }

    pub(super) fn merge_parent_bases(&self) -> &[AdmittedRelationalBranchBasis] {
        &self.merge_parent_bases
    }

    pub(super) fn merge_parent_branch_ids(&self) -> Vec<BranchId> {
        self.merge_parent_bases
            .iter()
            .map(|basis| basis.identity().branch_id().clone())
            .collect()
    }

    pub(super) fn appendix_adoption(&self) -> Option<&CanonicalRequiredOutputOrigin> {
        self.appendix_adoption.as_deref()
    }

    pub(super) fn recovered_codec_publication_custody(
        &self,
    ) -> Option<&Arc<RecoveredCodecPublicationCustody>> {
        self.recovered_codec_publication_custody.as_ref()
    }

    pub(super) fn home_step(&self) -> Option<&Arc<AdmittedRelationalHomeProgramStep>> {
        self.home_step.as_ref()
    }

    pub(super) fn naming_plan(&self) -> Option<&CommitNamingPlan> {
        self.naming_plan.as_deref()
    }
}

impl<'a> From<&'a CommitPublicationValidationInput> for SchemaContinuityRequest<'a> {
    fn from(input: &'a CommitPublicationValidationInput) -> Self {
        Self {
            basis: &input.basis,
            authority: &input.schema_authority,
            copied_authority: input.schema_authority_input.as_ref(),
            transition: input.transition.as_ref(),
            reconciliation: input.reconciliation.as_ref(),
        }
    }
}
