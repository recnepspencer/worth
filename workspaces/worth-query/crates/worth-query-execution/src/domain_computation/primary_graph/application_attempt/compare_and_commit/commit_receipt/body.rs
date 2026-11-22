//! Heap-resident payload for `WorthQueryApplicationCommitReceipt`.
use worth_relational::facade::history::RelationalCommitReceipt;

use crate::domain_computation::application_aftermath::{
    WorthQueryCommittedAftermathCausality, WorthQueryExternalEffectDispatch,
    WorthQueryRetainedPreImage,
};

use super::super::super::{
    WorthQueryApplicationCommitAuthorityBinding, WorthQueryApplicationCommitTerminalEvidence,
    WorthQueryApplicationOutputCorrespondence, WorthQueryMutationPreconditionComparisonEvidence,
};
use super::WorthQueryApplicationCommittedChanges;

#[derive(Debug, PartialEq)]
pub(in crate::domain_computation::primary_graph::application_attempt::compare_and_commit) struct CommitReceiptBody {
    pub(super) authoritative_provider_session:
        crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding,
    pub(super) outcome_identity:
        Option<super::super::super::WorthQueryApplicationCommitOutcomeIdentity>,
    pub(super) provider_runtime_instance_id: u64,
    pub(super) commit: RelationalCommitReceipt,
    pub(super) committed_product_publication: super::super::WorthQueryCommittedProductPublication,
    pub(super) basis_descriptor:
        worth_relational::facade::branch::RelationalBranchBasisDescriptor,
    pub(super) mutation_work:
        Option<super::super::super::super::provider::WorthQueryPrimaryMutationWorkEvidence>,
    pub(super) precondition_comparison: WorthQueryMutationPreconditionComparisonEvidence,
    pub(super) terminal: WorthQueryApplicationCommitTerminalEvidence,
    pub(super) committed_dispatch_outbox:
        Option<super::super::super::super::provider::WorthQueryCommittedDispatchOutboxBinding>,
    pub(super) external_dispatch: Option<WorthQueryExternalEffectDispatch>,
    pub(super) external_dispatch_preparation_denial:
        Option<super::super::super::WorthQueryExternalDispatchPreparationDenial>,
    /// R8.62 / C1 — derived from admission at construction; never caller-supplied.
    pub(super) authority_binding: WorthQueryApplicationCommitAuthorityBinding,
    /// R8.2 — exact inverse pre-image slice retained from the decision read-set.
    pub(super) retained_preimage: Option<WorthQueryRetainedPreImage>,
    pub(super) aftermath_causality: Option<WorthQueryCommittedAftermathCausality>,
    pub(super) performed_product_change: Option<
        crate::domain_computation::execution_runtime::product_world::WorthQueryPerformedRelationalProductChange,
    >,
    pub(super) committed_changes: WorthQueryApplicationCommittedChanges,
    pub(super) performed_input_facts: Option<
        crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload,
    >,
    pub(super) performed_output_fields: Option<
        Result<
            crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload,
            crate::domain_computation::primary_graph::application_attempt::OutputFieldCaptureFailure,
        >,
    >,
    pub(super) output_correspondence: std::sync::Arc<WorthQueryApplicationOutputCorrespondence>,
    pub(super) exact_output_settlement: Option<
        std::sync::Arc<
            crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity,
        >,
    >,
    /// Set where the rebase of this commit failed, so it retains no fact.
    pub(super) unrebased_own_effect:
        Option<crate::domain_computation::primary_graph::provider::OwnEffectOnReads>,
}

impl CommitReceiptBody {
    #[inline(never)]
    pub(super) fn clone_without_performed_change(&self) -> Box<Self> {
        Box::new(Self {
            authoritative_provider_session: self.authoritative_provider_session.clone(),
            outcome_identity: self.outcome_identity,
            provider_runtime_instance_id: self.provider_runtime_instance_id,
            commit: self.commit.clone(),
            committed_product_publication: self.committed_product_publication.clone(),
            basis_descriptor: self.basis_descriptor.clone(),
            mutation_work: self.mutation_work.clone(),
            precondition_comparison: self.precondition_comparison.clone(),
            terminal: self.terminal.clone(),
            committed_dispatch_outbox: self.committed_dispatch_outbox.clone(),
            external_dispatch: self.external_dispatch.clone(),
            external_dispatch_preparation_denial: self.external_dispatch_preparation_denial,
            authority_binding: self.authority_binding.clone(),
            retained_preimage: self.retained_preimage.clone(),
            aftermath_causality: self.aftermath_causality.clone(),
            performed_product_change: None,
            output_correspondence: self.output_correspondence.clone(),
            exact_output_settlement: self.exact_output_settlement.clone(),
            unrebased_own_effect: self.unrebased_own_effect,
            committed_changes: self.committed_changes.clone(),
            performed_input_facts: self.performed_input_facts.clone(),
            performed_output_fields: self.performed_output_fields.clone(),
        })
    }
}
