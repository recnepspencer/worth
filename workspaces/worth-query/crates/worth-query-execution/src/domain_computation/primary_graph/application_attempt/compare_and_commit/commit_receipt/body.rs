//! Heap-resident payload for non-const fields of `WorthQueryApplicationCommitReceipt`.
use crate::domain_computation::application_aftermath::WorthQueryRetainedPreImage;

use super::super::super::WorthQueryApplicationOutputCorrespondence;

#[derive(Debug, PartialEq)]
pub(in crate::domain_computation::primary_graph::application_attempt::compare_and_commit) struct CommitReceiptExtras {
    pub(super) authoritative_provider_session:
        crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding,
    pub(super) mutation_work:
        Option<super::super::super::super::provider::WorthQueryPrimaryMutationWorkEvidence>,
    pub(super) committed_dispatch_outbox:
        Option<super::super::super::super::provider::WorthQueryCommittedDispatchOutboxBinding>,
    /// R8.2 — exact inverse pre-image slice retained from the decision read-set.
    pub(super) retained_preimage: Option<WorthQueryRetainedPreImage>,
    pub(super) performed_product_change: Option<
        crate::domain_computation::execution_runtime::product_world::WorthQueryPerformedRelationalProductChange,
    >,
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

impl CommitReceiptExtras {
    #[inline(never)]
    pub(super) fn clone_without_performed_change(&self) -> Box<Self> {
        Box::new(Self {
            authoritative_provider_session: self.authoritative_provider_session.clone(),
            mutation_work: self.mutation_work.clone(),
            committed_dispatch_outbox: self.committed_dispatch_outbox.clone(),
            retained_preimage: self.retained_preimage.clone(),
            performed_product_change: None,
            output_correspondence: self.output_correspondence.clone(),
            exact_output_settlement: self.exact_output_settlement.clone(),
            unrebased_own_effect: self.unrebased_own_effect,
            performed_input_facts: self.performed_input_facts.clone(),
            performed_output_fields: self.performed_output_fields.clone(),
        })
    }
}
