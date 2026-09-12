//! Owner-private inputs carried from exact transaction admission through
//! schema, invariant, footprint, and publication validation.

#[path = "validation/custom_invariant_execution_receipt.rs"]
mod custom_invariant_execution_receipt;
#[path = "validation/invariant_plan.rs"]
mod invariant_plan;
#[path = "validation/proposal_footprint/mod.rs"]
mod proposal_footprint;
#[path = "validation/proposal_identity.rs"]
pub(crate) mod proposal_identity;
#[path = "validation/proposal_invariants.rs"]
mod proposal_invariants;
#[path = "validation/proposal_revalidation.rs"]
mod proposal_revalidation;
#[path = "validation/proposal_touches.rs"]
mod proposal_touches;
#[path = "validation/proposal_validation.rs"]
mod proposal_validation;
#[path = "validation/validated_proposal.rs"]
mod validated_proposal;

pub use custom_invariant_execution_receipt::CustomInvariantExecutionReceipt;
pub use proposal_footprint::{
    ValidatedMutationFootprint, ValidatedMutationFootprintNotRequested,
    ValidatedMutationFootprintProjection, ValidatedMutationFootprintWork,
};
pub use proposal_identity::RelationalMutationProposalIdentity;
pub use proposal_touches::{
    ValidatedMutationTouch, ValidatedMutationTouchProjectionError,
    ValidatedMutationTouchProjectionWork, ValidatedMutationTouches,
};
pub(crate) use validated_proposal::StrategyProposalDecoration;
pub use validated_proposal::{RelationalMutationInvariantEvidence, ValidatedRelationalProposal};

use crate::branch::{AdmittedRelationalBranchBasis, RelationalBranchRootSchemaAuthority};
use crate::history::data::BranchId;
use crate::schema::data::{ProposedSchemaTransition, SchemaReconciliationPolicy};

#[derive(Debug)]
pub(crate) struct RelationalTransactionValidationInput {
    basis: AdmittedRelationalBranchBasis,
    intent: crate::mvcc::RelationalTransactionIntent,
    merge_parent_bases: Vec<AdmittedRelationalBranchBasis>,
    schema_authority_input: Option<crate::schema::SchemaContinuityAuthorityInput>,
    schema_authority: std::sync::Arc<RelationalBranchRootSchemaAuthority>,
    footprint: crate::mvcc::RelationalTransactionFootprint,
    retention:
        Option<std::sync::Arc<crate::history::retention::RelationalTransactionRetentionObligation>>,
    control: crate::mvcc::RelationalOperationControl,
}

impl PartialEq for RelationalTransactionValidationInput {
    fn eq(&self, other: &Self) -> bool {
        self.basis == other.basis
            && self.intent == other.intent
            && self.merge_parent_bases == other.merge_parent_bases
            && self.schema_authority_input == other.schema_authority_input
            && self.schema_authority.allocation_id() == other.schema_authority.allocation_id()
            && self.footprint == other.footprint
    }
}

impl Eq for RelationalTransactionValidationInput {}

impl RelationalTransactionValidationInput {
    pub(crate) fn from_transaction(
        transaction: &crate::mvcc::BranchBoundRelationalTransaction,
    ) -> Self {
        Self {
            basis: transaction.basis.clone(),
            intent: transaction.intent.clone(),
            merge_parent_bases: transaction.merge_parent_bases.clone(),
            schema_authority_input: transaction.schema_authority_input.clone(),
            schema_authority: std::sync::Arc::clone(&transaction.schema_authority),
            footprint: transaction.footprint.clone(),
            retention: Some(std::sync::Arc::clone(&transaction.retention)),
            control: transaction.control.clone(),
        }
    }

    pub(crate) fn for_owner_basis(basis: &AdmittedRelationalBranchBasis) -> Self {
        Self {
            basis: basis.clone(),
            intent: crate::mvcc::RelationalTransactionIntent::ordinary(),
            merge_parent_bases: Vec::new(),
            schema_authority_input: None,
            schema_authority: basis.inner.root.retained_schema_authority(),
            footprint: crate::mvcc::RelationalTransactionFootprint::for_basis(basis),
            retention: None,
            control: crate::mvcc::RelationalOperationControl::uninterrupted(),
        }
    }

    pub(crate) fn basis(&self) -> &AdmittedRelationalBranchBasis {
        &self.basis
    }

    pub(crate) fn intent(&self) -> &crate::mvcc::RelationalTransactionIntent {
        &self.intent
    }

    pub(crate) fn target_branch(&self) -> &BranchId {
        self.basis.identity().branch_id()
    }

    pub(crate) fn merge_parent_bases(&self) -> &[AdmittedRelationalBranchBasis] {
        &self.merge_parent_bases
    }

    pub(crate) fn merge_parent_branch_ids(&self) -> Vec<BranchId> {
        self.merge_parent_bases
            .iter()
            .map(|basis| basis.identity().branch_id().clone())
            .collect()
    }

    pub(crate) fn with_merge_parent_bases(
        mut self,
        bases: Vec<AdmittedRelationalBranchBasis>,
    ) -> Self {
        self.merge_parent_bases = bases;
        self
    }

    pub(crate) fn proposed_schema_transition(&self) -> Option<&ProposedSchemaTransition> {
        self.intent.proposed_schema_transition()
    }

    pub(crate) fn schema_reconciliation_policy(&self) -> Option<&SchemaReconciliationPolicy> {
        self.intent.schema_reconciliation_policy()
    }

    pub(crate) fn schema_authority_input(
        &self,
    ) -> Option<&crate::schema::SchemaContinuityAuthorityInput> {
        self.schema_authority_input.as_ref()
    }

    pub(crate) fn with_schema_authority_input(
        mut self,
        input: crate::schema::SchemaContinuityAuthorityInput,
    ) -> Self {
        self.schema_authority_input = Some(input);
        self
    }

    pub(crate) fn with_schema_transition(
        mut self,
        proposed_schema_transition: ProposedSchemaTransition,
        schema_reconciliation_policy: Option<SchemaReconciliationPolicy>,
    ) -> Self {
        self.intent = self
            .intent
            .with_schema_transition(proposed_schema_transition, schema_reconciliation_policy);
        self
    }

    pub(crate) fn schema_authority(&self) -> &RelationalBranchRootSchemaAuthority {
        &self.schema_authority
    }

    pub(crate) fn footprint(&self) -> &crate::mvcc::RelationalTransactionFootprint {
        &self.footprint
    }

    pub(crate) fn apply_owner_inputs_to(
        self,
        transaction: &mut crate::mvcc::BranchBoundRelationalTransaction,
    ) {
        debug_assert_eq!(transaction.basis, self.basis);
        debug_assert_eq!(transaction.intent, self.intent);
        transaction.merge_parent_bases = self.merge_parent_bases;
        transaction.schema_authority_input = self.schema_authority_input;
        if let Some(retention) = self.retention {
            transaction.retention = retention;
        }
        transaction.control = self.control;
    }

    pub(crate) fn release_transaction_retention(&mut self) {
        self.retention = None;
    }

    pub(crate) fn control(&self) -> &crate::mvcc::RelationalOperationControl {
        &self.control
    }
}
