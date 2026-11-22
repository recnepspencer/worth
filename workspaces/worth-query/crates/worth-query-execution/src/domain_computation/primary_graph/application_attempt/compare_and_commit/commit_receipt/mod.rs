//! Closed application commit receipt.
pub(in crate::domain_computation::primary_graph) mod recording;

mod body;
mod committed_changes;
mod description;
mod recorded_settlement;
pub(super) use body::CommitReceiptBody;
pub use committed_changes::WorthQueryApplicationCommittedChanges;
#[cfg(doctest)]
mod disclosure_fences;
mod input_observation;
mod output_source_identity;
mod pending;
mod projection;
mod publication_source;
mod recorded_correspondence;

use worth_query_installation::facade::WorthQueryCanonicalWorkPhases;
use worth_relational::facade::history::RelationalCommitReceipt;

use crate::domain_computation::application_aftermath::{
    WorthQueryCommittedAftermathCausality, WorthQueryDispatchOutboxRecord,
    WorthQueryExternalEffectDispatch, WorthQueryRetainedPreImage,
};

use super::super::{
    WorthQueryApplicationCommitAuthorityBinding, WorthQueryApplicationCommitTerminalEvidence,
    WorthQueryApplicationOutputCorrespondence, WorthQueryApplicationOutputProjectionDenial,
    WorthQueryApplicationTypedOutputCorrespondence,
    WorthQueryMutationPreconditionComparisonEvidence,
};

pub(in crate::domain_computation::primary_graph) use projection::WorthQueryCommittedReceiptProjection;
pub use publication_source::WorthQueryApplicationCommitPublicationSource;

/// The record of one landed application commit: which branch and commit, what
/// changed, which records were touched, what must be dispatched, and under what
/// authority.
///
/// Carried by the `Committed` commit outcome. The fresh
/// receipt also holds the performed product-change witness; a clone drops it and
/// is descriptive history only.
#[derive(Debug, PartialEq)]
pub struct WorthQueryApplicationCommitReceipt {
    pub(super) changed_record_count: usize,
    pub(super) emitted_effect_count: usize,
    pub(super) canonical_work: WorthQueryCanonicalWorkPhases,
    pub(super) inner: Box<CommitReceiptBody>,
}

impl Eq for WorthQueryApplicationCommitReceipt {}

/// Receipt copies are descriptive historical evidence. The unique performed
/// product-change witness remains with the original fresh receipt.
impl Clone for WorthQueryApplicationCommitReceipt {
    fn clone(&self) -> Self {
        Self {
            changed_record_count: self.changed_record_count,
            emitted_effect_count: self.emitted_effect_count,
            canonical_work: self.canonical_work,
            inner: self.inner.clone_without_performed_change(),
        }
    }
}

impl WorthQueryApplicationCommitReceipt {
    pub fn product_branch(&self) -> crate::basis::WorthQueryProductBranch {
        crate::basis::WorthQueryProductBranch::from_occurrence(
            self.inner.committed_product_publication.product_incarnation(),
        )
    }

    pub fn outcome_identity(
        &self,
    ) -> Option<super::super::WorthQueryApplicationCommitOutcomeIdentity> {
        self.inner.outcome_identity
    }

    pub fn provider_runtime_instance_id(&self) -> u64 {
        self.inner.provider_runtime_instance_id
    }

    pub fn commit_reference(&self) -> &RelationalCommitReceipt {
        &self.inner.commit
    }

    /// Exact historical coordinates for selection after this receipt is released.
    pub fn correction_source(
        &self,
    ) -> crate::domain_computation::application_aftermath::WorthQueryRecordedCorrectionSource {
        crate::domain_computation::application_aftermath::WorthQueryRecordedCorrectionSource::from_untrusted_parts(
            self.inner.committed_product_publication.composite_commit().clone(),
            self.inner.commit.clone(),
        )
    }

    pub fn committed_product_publication(
        &self,
    ) -> &super::WorthQueryCommittedProductPublication {
        &self.inner.committed_product_publication
    }

    /// What a commit that kept no fact answers at `observation`, and `None`
    /// for a commit that rebased: its facts answer. One whose effect moved a
    /// read is superseded at its own publication, as it is where its rebase
    /// succeeds.
    pub(in crate::domain_computation::primary_graph) fn currentness_without_facts_at(
        &self,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> Option<crate::domain_computation::primary_graph::provider::FactlessCurrentness> {
        let own_effect = self.inner.unrebased_own_effect?;
        Some(
            own_effect.at_own_publication(
                self.inner
                    .committed_product_publication
                    .is_selected_at(observation),
            ),
        )
    }

    pub fn basis_descriptor(
        &self,
    ) -> &worth_relational::facade::branch::RelationalBranchBasisDescriptor {
        &self.inner.basis_descriptor
    }

    pub const fn changed_record_count(&self) -> usize {
        self.changed_record_count
    }

    pub const fn emitted_effect_count(&self) -> usize {
        self.emitted_effect_count
    }

    pub fn committed_changes(&self) -> &WorthQueryApplicationCommittedChanges {
        &self.inner.committed_changes
    }

    /// The committed output roles, read as the output contract `Contract`.
    /// Refused as `ForeignContract` when this commit was made under another
    /// contract; every read on the view is then checked at compile time.
    pub fn outputs_of<Contract: 'static>(
        &self,
    ) -> Result<
        WorthQueryApplicationTypedOutputCorrespondence<'_, Contract>,
        WorthQueryApplicationOutputProjectionDenial,
    > {
        self.inner.output_correspondence.outputs_of()
    }

    pub fn output_correspondence(&self) -> &WorthQueryApplicationOutputCorrespondence {
        self.inner.output_correspondence.as_ref()
    }

    pub(in crate::domain_computation::primary_graph) fn retain_output_correspondence(
        &self,
    ) -> std::sync::Arc<WorthQueryApplicationOutputCorrespondence> {
        std::sync::Arc::clone(&self.inner.output_correspondence)
    }

    pub(in crate::domain_computation::primary_graph) fn exact_output_settlement(
        &self,
    ) -> Option<
        &std::sync::Arc<
            crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity,
        >,
    > {
        self.inner.exact_output_settlement.as_ref()
    }

    pub(crate) fn publication_source(&self) -> WorthQueryApplicationCommitPublicationSource {
        WorthQueryApplicationCommitPublicationSource::from_receipt(self)
    }

    pub fn mutation_work(
        &self,
    ) -> Option<&super::super::super::provider::WorthQueryPrimaryMutationWorkEvidence> {
        self.inner.mutation_work.as_ref()
    }

    pub fn precondition_comparison(
        &self,
    ) -> &WorthQueryMutationPreconditionComparisonEvidence {
        &self.inner.precondition_comparison
    }

    pub const fn canonical_work(&self) -> WorthQueryCanonicalWorkPhases {
        self.canonical_work
    }

    pub fn terminal(&self) -> &WorthQueryApplicationCommitTerminalEvidence {
        &self.inner.terminal
    }

    pub fn dispatch_outbox(&self) -> Option<&WorthQueryDispatchOutboxRecord> {
        self.inner
            .committed_dispatch_outbox
            .as_ref()
            .map(super::super::super::provider::WorthQueryCommittedDispatchOutboxBinding::record)
    }

    pub(in crate::domain_computation) fn committed_dispatch_outbox(
        &self,
    ) -> Option<&super::super::super::provider::WorthQueryCommittedDispatchOutboxBinding> {
        self.inner.committed_dispatch_outbox.as_ref()
    }

    pub fn external_dispatch(&self) -> Option<&WorthQueryExternalEffectDispatch> {
        self.inner.external_dispatch.as_ref()
    }

    pub fn external_dispatch_preparation_denial(
        &self,
    ) -> Option<super::super::WorthQueryExternalDispatchPreparationDenial> {
        self.inner.external_dispatch_preparation_denial
    }

    pub fn authority_binding(&self) -> &WorthQueryApplicationCommitAuthorityBinding {
        &self.inner.authority_binding
    }

    pub fn installed_operation(&self) -> &[u8; 32] {
        self.inner.authority_binding.installed_operation()
    }

    pub(crate) fn runtime_authority(
        &self,
    ) -> crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity {
        self.inner.authority_binding.runtime_authority()
    }

    pub(crate) fn installed_aftermath(
        &self,
    ) -> Option<&worth_query_installation::facade::WorthQueryInstalledAftermathContract> {
        self.inner.authority_binding.installed_aftermath()
    }

    /// Installed recovery meaning from this original commit, not recovery authority.
    pub fn installed_recovery_contract(
        &self,
    ) -> Option<worth_query_installation::facade::InstalledAftermathRecoveryContract> {
        match self.inner.authority_binding.installed_aftermath() {
            Some(aftermath) => Some(aftermath.recovery()),
            None => None,
        }
    }

    pub fn published_aftermath_posture(
        &self,
    ) -> Option<worth_query_installation::facade::PublishedAftermathPosture> {
        match self.inner.authority_binding.installed_aftermath() {
            Some(aftermath) => Some(aftermath.published_posture()),
            None => None,
        }
    }

    pub fn principal_scope(
        &self,
    ) -> &crate::domain_computation::authorization::WorthQueryOperationScopeBinding {
        self.inner.authority_binding.principal_scope()
    }

    pub fn idempotency_binding(
        &self,
    ) -> super::super::WorthQueryApplicationIdempotencyBinding {
        self.inner.authority_binding.idempotency_binding()
    }

    pub(crate) fn retained_preimage(&self) -> Option<&WorthQueryRetainedPreImage> {
        self.inner.retained_preimage.as_ref()
    }

    /// Whether the performed occurrence retained values for a declared correction.
    pub fn has_retained_preimage(&self) -> bool {
        self.inner.retained_preimage.is_some()
    }

    pub fn aftermath_causality(&self) -> Option<&WorthQueryCommittedAftermathCausality> {
        self.inner.aftermath_causality.as_ref()
    }

    /// Takes the one conditional-delivery witness from a fresh performed
    /// application publication. Recovered and already-committed receipts
    /// return `None`.
    pub fn take_performed_relational_product_change(
        &mut self,
    ) -> Option<
        crate::domain_computation::execution_runtime::product_world::WorthQueryPerformedRelationalProductChange,
    >{
        self.inner.performed_product_change.take()
    }

    pub(in crate::domain_computation::primary_graph) fn with_external_dispatch(
        mut self,
        dispatch: WorthQueryExternalEffectDispatch,
    ) -> Self {
        self.canonical_work = self
            .canonical_work
            .with_external_dispatch_work(dispatch.canonical_work());
        self.inner.external_dispatch = Some(dispatch);
        self
    }

    pub(in crate::domain_computation::primary_graph) fn with_external_dispatch_preparation_denial(
        mut self,
        denial: super::super::WorthQueryExternalDispatchPreparationDenial,
    ) -> Self {
        self.inner.external_dispatch_preparation_denial = Some(denial);
        self
    }

    pub fn is_same_authoritative_commit(&self, other: &Self) -> bool {
        self.inner.provider_runtime_instance_id == other.inner.provider_runtime_instance_id
            && self.inner.terminal.branch() == other.inner.terminal.branch()
            && self.inner.commit == other.inner.commit
            && self.inner.committed_product_publication == other.inner.committed_product_publication
    }
}

pub(in crate::domain_computation::primary_graph) use committed_changes::MaterializedCommittedField;

mod court_doors_for_test;

pub use pending::WorthQueryPendingApplicationCommitReceipt;
#[cfg(test)]
#[path = "mod_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
