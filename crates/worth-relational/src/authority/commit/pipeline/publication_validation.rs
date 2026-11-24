//! Selected draft authority becomes the smaller set needed for publication.
use crate::authority::commit::phases::schema_continuity::inputs::SchemaContinuityRequest;
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::branch::home_program_step::AdmittedRelationalHomeProgramStep;
use crate::branch::root::schema::RelationalBranchRootSchemaAuthority;
use crate::durability::authority::recovery::codec_transition::publication_custody::RecoveredCodecPublicationCustody;
use crate::history::data::{BranchId, CanonicalRequiredOutputOrigin};
use crate::history::retention::RelationalTransactionRetentionObligation;
use crate::lineage::data::CommitNamingPlan;
use crate::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity;
use crate::runtime::operation_control::RelationalOperationControl;
use crate::runtime::state::subsystems::indexing::PreparedRelationalIndexDefinitions;
use crate::schema::data::{ProposedSchemaTransition, SchemaReconciliationPolicy};
use crate::schema::SchemaContinuityAuthorityInput;
use std::sync::Arc;

pub(crate) trait DraftValidationInput {
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
pub(crate) struct CommitPublicationValidationInput {
    pub(crate) basis: AdmittedRelationalBranchBasis,
    pub(crate) merge_parent_bases: Vec<AdmittedRelationalBranchBasis>,
    pub(crate) schema_authority: Arc<RelationalBranchRootSchemaAuthority>,
    pub(crate) schema_authority_input: Option<SchemaContinuityAuthorityInput>,
    pub(crate) transition: Option<ProposedSchemaTransition>,
    pub(crate) reconciliation: Option<SchemaReconciliationPolicy>,
    pub(crate) appendix_adoption: Option<Arc<CanonicalRequiredOutputOrigin>>,
    pub(crate) recovered_codec_publication_custody: Option<Arc<RecoveredCodecPublicationCustody>>,
    pub(crate) prepared_home_indexes: Option<Arc<PreparedRelationalIndexDefinitions>>,
    pub(crate) home_step: Option<Arc<AdmittedRelationalHomeProgramStep>>,
    pub(crate) retention: Option<Arc<RelationalTransactionRetentionObligation>>,
    pub(crate) naming_plan: Option<Arc<CommitNamingPlan>>,
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
