//! Closed application commit receipt.
use worth_query_execution_runtime::facade::ports::ProductBranchInternalPort as _;
pub mod recording;

mod body;
pub mod committed_changes;
pub mod description;
pub mod recorded_settlement;
pub(crate) use body::CommitReceiptExtras;
pub use committed_changes::WorthQueryApplicationCommittedChanges;
#[cfg(doctest)]
pub mod disclosure_fences;
pub mod input_observation;
pub mod output_source_identity;
pub mod pending;
pub mod projection;
pub mod publication_source;
pub mod recorded_correspondence;

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

pub use projection::WorthQueryCommittedReceiptProjection;
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
    pub(crate) outcome_identity:
        Option<super::super::WorthQueryApplicationCommitOutcomeIdentity>,
    pub(crate) provider_runtime_instance_id: u64,
    pub(crate) commit: RelationalCommitReceipt,
    pub(crate) committed_product_publication: super::WorthQueryCommittedProductPublication,
    pub(crate) basis_descriptor:
        worth_relational::facade::branch::RelationalBranchBasisDescriptor,
    pub(crate) changed_record_count: usize,
    pub(crate) emitted_effect_count: usize,
    pub(crate) precondition_comparison: WorthQueryMutationPreconditionComparisonEvidence,
    pub(crate) canonical_work: WorthQueryCanonicalWorkPhases,
    pub(crate) terminal: WorthQueryApplicationCommitTerminalEvidence,
    pub(crate) external_dispatch: Option<WorthQueryExternalEffectDispatch>,
    pub(crate) external_dispatch_preparation_denial:
        Option<super::super::WorthQueryExternalDispatchPreparationDenial>,
    /// R8.62 / C1 — derived from admission at construction; never caller-supplied.
    pub(crate) authority_binding: WorthQueryApplicationCommitAuthorityBinding,
    pub(crate) has_retained_preimage: bool,
    pub(crate) aftermath_causality: Option<WorthQueryCommittedAftermathCausality>,
    pub(crate) committed_changes: WorthQueryApplicationCommittedChanges,
    pub(crate) inner: Box<CommitReceiptExtras>,
}

impl Eq for WorthQueryApplicationCommitReceipt {}

/// Receipt copies are descriptive historical evidence. The unique performed
/// product-change witness remains with the original fresh receipt.
impl Clone for WorthQueryApplicationCommitReceipt {
    #[inline(never)]
    fn clone(&self) -> Self {
        Self {
            outcome_identity: self.outcome_identity,
            provider_runtime_instance_id: self.provider_runtime_instance_id,
            commit: self.commit.clone(),
            committed_product_publication: self.committed_product_publication.clone(),
            basis_descriptor: self.basis_descriptor.clone(),
            changed_record_count: self.changed_record_count,
            emitted_effect_count: self.emitted_effect_count,
            precondition_comparison: self.precondition_comparison.clone(),
            canonical_work: self.canonical_work,
            terminal: self.terminal.clone(),
            external_dispatch: self.external_dispatch.clone(),
            external_dispatch_preparation_denial: self.external_dispatch_preparation_denial,
            authority_binding: self.authority_binding.clone(),
            has_retained_preimage: self.has_retained_preimage,
            aftermath_causality: self.aftermath_causality.clone(),
            committed_changes: self.committed_changes.clone(),
            inner: self.inner.clone_without_performed_change(),
        }
    }
}

impl WorthQueryApplicationCommitReceipt {
    pub fn product_branch(&self) -> crate::basis::WorthQueryProductBranch {
        crate::basis::WorthQueryProductBranch::from_occurrence(
            self.committed_product_publication.product_incarnation(),
        )
    }

    pub const fn outcome_identity(
        &self,
    ) -> Option<super::super::WorthQueryApplicationCommitOutcomeIdentity> {
        self.outcome_identity
    }

    pub const fn provider_runtime_instance_id(&self) -> u64 {
        self.provider_runtime_instance_id
    }

    pub const fn commit_reference(&self) -> &RelationalCommitReceipt {
        &self.commit
    }

    /// Exact historical coordinates for selection after this receipt is released.
    pub fn correction_source(
        &self,
    ) -> crate::domain_computation::application_aftermath::WorthQueryRecordedCorrectionSource {
        crate::domain_computation::application_aftermath::WorthQueryRecordedCorrectionSource::from_untrusted_parts(
            self.committed_product_publication.composite_commit().clone(),
            self.commit.clone(),
        )
    }

    pub const fn committed_product_publication(
        &self,
    ) -> &super::WorthQueryCommittedProductPublication {
        &self.committed_product_publication
    }

    pub(crate) fn settle_for_ready_cache(&mut self) {
        self.committed_product_publication.settle_for_ready_cache();
    }

    /// What a commit that kept no fact answers at `observation`, and `None`
    /// for a commit that rebased: its facts answer. One whose effect moved a
    /// read is superseded at its own publication, as it is where its rebase
    /// succeeds.
    pub(crate) fn currentness_without_facts_at(
        &self,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> Option<crate::domain_computation::primary_graph::provider::FactlessCurrentness> {
        let own_effect = self.inner.unrebased_own_effect?;
        Some(
            own_effect.at_own_publication(
                self.committed_product_publication
                    .is_selected_at(observation),
            ),
        )
    }

    pub const fn basis_descriptor(
        &self,
    ) -> &worth_relational::facade::branch::RelationalBranchBasisDescriptor {
        &self.basis_descriptor
    }

    pub const fn changed_record_count(&self) -> usize {
        self.changed_record_count
    }

    pub const fn emitted_effect_count(&self) -> usize {
        self.emitted_effect_count
    }

    pub const fn committed_changes(&self) -> &WorthQueryApplicationCommittedChanges {
        &self.committed_changes
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

    pub(crate) fn retain_output_correspondence(
        &self,
    ) -> std::sync::Arc<WorthQueryApplicationOutputCorrespondence> {
        std::sync::Arc::clone(&self.inner.output_correspondence)
    }

    pub(crate) fn exact_output_settlement(
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

    pub const fn precondition_comparison(
        &self,
    ) -> &WorthQueryMutationPreconditionComparisonEvidence {
        &self.precondition_comparison
    }

    pub const fn canonical_work(&self) -> WorthQueryCanonicalWorkPhases {
        self.canonical_work
    }

    pub const fn terminal(&self) -> &WorthQueryApplicationCommitTerminalEvidence {
        &self.terminal
    }

    pub fn dispatch_outbox(&self) -> Option<&WorthQueryDispatchOutboxRecord> {
        self.inner
            .committed_dispatch_outbox
            .as_ref()
            .map(super::super::super::provider::WorthQueryCommittedDispatchOutboxBinding::record)
    }

    pub(crate) fn committed_dispatch_outbox(
        &self,
    ) -> Option<&super::super::super::provider::WorthQueryCommittedDispatchOutboxBinding> {
        self.inner.committed_dispatch_outbox.as_ref()
    }

    pub const fn external_dispatch(&self) -> Option<&WorthQueryExternalEffectDispatch> {
        self.external_dispatch.as_ref()
    }

    pub const fn external_dispatch_preparation_denial(
        &self,
    ) -> Option<super::super::WorthQueryExternalDispatchPreparationDenial> {
        self.external_dispatch_preparation_denial
    }

    pub const fn authority_binding(&self) -> &WorthQueryApplicationCommitAuthorityBinding {
        &self.authority_binding
    }

    pub const fn installed_operation(&self) -> &[u8; 32] {
        self.authority_binding.installed_operation()
    }

    pub(crate) const fn runtime_authority(
        &self,
    ) -> crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity {
        self.authority_binding.runtime_authority()
    }

    pub(crate) fn installed_aftermath(
        &self,
    ) -> Option<&worth_query_installation::facade::WorthQueryInstalledAftermathContract> {
        self.authority_binding.installed_aftermath()
    }

    /// Installed recovery meaning from this original commit, not recovery authority.
    pub const fn installed_recovery_contract(
        &self,
    ) -> Option<worth_query_installation::facade::InstalledAftermathRecoveryContract> {
        self.authority_binding.installed_recovery_contract()
    }

    pub const fn published_aftermath_posture(
        &self,
    ) -> Option<worth_query_installation::facade::PublishedAftermathPosture> {
        self.authority_binding.published_aftermath_posture()
    }

    pub const fn principal_scope(
        &self,
    ) -> &crate::domain_computation::authorization::WorthQueryOperationScopeBinding {
        self.authority_binding.principal_scope()
    }

    pub const fn idempotency_binding(
        &self,
    ) -> super::super::WorthQueryApplicationIdempotencyBinding {
        self.authority_binding.idempotency_binding()
    }

    pub(crate) fn retained_preimage(&self) -> Option<&WorthQueryRetainedPreImage> {
        self.inner.retained_preimage.as_ref()
    }

    /// Whether the performed occurrence retained values for a declared correction.
    pub const fn has_retained_preimage(&self) -> bool {
        self.has_retained_preimage
    }

    pub const fn aftermath_causality(&self) -> Option<&WorthQueryCommittedAftermathCausality> {
        self.aftermath_causality.as_ref()
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

    pub(crate) fn with_external_dispatch(
        mut self,
        dispatch: WorthQueryExternalEffectDispatch,
    ) -> Self {
        self.canonical_work = self
            .canonical_work
            .with_external_dispatch_work(dispatch.canonical_work());
        self.external_dispatch = Some(dispatch);
        self
    }

    pub(crate) fn with_external_dispatch_preparation_denial(
        mut self,
        denial: super::super::WorthQueryExternalDispatchPreparationDenial,
    ) -> Self {
        self.external_dispatch_preparation_denial = Some(denial);
        self
    }

    pub fn is_same_authoritative_commit(&self, other: &Self) -> bool {
        self.provider_runtime_instance_id == other.provider_runtime_instance_id
            && self.terminal.branch() == other.terminal.branch()
            && self.commit == other.commit
            && self.committed_product_publication == other.committed_product_publication
    }
}

pub use committed_changes::MaterializedCommittedField;

mod court_doors_for_test;

pub use pending::WorthQueryPendingApplicationCommitReceipt;

impl crate::ports::CommitReceiptPreimagePort for WorthQueryApplicationCommitReceipt {
    fn retained_preimage(&self) -> Option<&WorthQueryRetainedPreImage> {
        Self::retained_preimage(self)
    }
}

#[cfg(test)]
#[path = "mod_inline_type_budget.rs"]
pub mod inline_type_budget;
