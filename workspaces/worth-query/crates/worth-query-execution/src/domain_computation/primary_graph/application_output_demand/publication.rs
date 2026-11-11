//! Read-only evidence shared by genuine fresh and recovered producer phases.
use std::sync::Arc;
mod join;
use super::super::application_attempt::OutputFieldCaptureFailure;
use super::super::application_checkpoint::facts::FactPayload;
use super::super::output_lineage::RecordedSettlementIdentity;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitAuthorityBinding, WorthQueryApplicationCommitReceipt,
    WorthQueryApplicationCommittedChanges, WorthQueryApplicationIdempotencyBinding,
    WorthQueryApplicationOutputCorrespondence, WorthQueryCommittedProductPublication,
    WorthQueryRecoveredApplicationCommitReceipt,
};

/// Original in-process evidence, retained at the real session's seal. A keyed
/// description cannot construct this phase or recreate its settlement owner.
pub(in crate::domain_computation::primary_graph) struct RecoveredProducerOutputFacts {
    pub(in crate::domain_computation::primary_graph) publication:
        WorthQueryCommittedProductPublication,
    pub(in crate::domain_computation::primary_graph) outcome:
        super::super::WorthQueryApplicationCommitOutcomeIdentity,
    pub(in crate::domain_computation::primary_graph) changes: WorthQueryApplicationCommittedChanges,
    pub(in crate::domain_computation::primary_graph) correspondence:
        Arc<WorthQueryApplicationOutputCorrespondence>,
    pub(in crate::domain_computation::primary_graph) authority:
        Arc<WorthQueryApplicationCommitAuthorityBinding>,
    pub(in crate::domain_computation::primary_graph) inputs: Option<FactPayload>,
    pub(in crate::domain_computation::primary_graph) fields:
        Option<Result<FactPayload, OutputFieldCaptureFailure>>,
    pub(in crate::domain_computation::primary_graph) unrebased_own_effect:
        Option<super::super::provider::OwnEffectOnReads>,
    pub(in crate::domain_computation::primary_graph) settlement:
        Option<Arc<RecordedSettlementIdentity>>,
    pub(in crate::domain_computation::primary_graph) _memory:
        Option<worth_execution::ExecutionMemoryReservation>,
}

/// This phase joins the original session's immutable evidence to an authorized
/// original-owner description. Bridge delivery and readiness remain necessary.
///
/// ```compile_fail,E0308
/// use worth_query_execution::facade::primary_graph::{
///     WorthQueryApplicationCommitReceipt, WorthQueryRecoveredProducerPublication,
/// };
/// fn recovered_is_not_a_live_receipt(value: &WorthQueryRecoveredProducerPublication)
///     -> &WorthQueryApplicationCommitReceipt { value }
/// ```
///
/// ```compile_fail,E0451
/// use worth_query_execution::facade::primary_graph::{
///     WorthQueryRecoveredApplicationCommitReceipt, WorthQueryRecoveredProducerPublication,
/// };
/// fn data_cannot_mint_a_producer(receipt: WorthQueryRecoveredApplicationCommitReceipt)
///     -> WorthQueryRecoveredProducerPublication {
///     WorthQueryRecoveredProducerPublication { original: receipt, facts: unreachable!() }
/// }
/// ```
#[derive(Clone)]
pub struct WorthQueryRecoveredProducerPublication {
    pub(in crate::domain_computation::primary_graph) original:
        WorthQueryRecoveredApplicationCommitReceipt,
    pub(in crate::domain_computation::primary_graph) facts: Arc<RecoveredProducerOutputFacts>,
}
impl WorthQueryRecoveredProducerPublication {
    pub(in crate::domain_computation::primary_graph) fn view(&self) -> OutputPublicationView<'_> {
        OutputPublicationView::Recovered(self)
    }
    pub(in crate::domain_computation::primary_graph) fn same_publication(
        &self,
        other: &Self,
    ) -> bool {
        Arc::ptr_eq(&self.facts, &other.facts)
    }
}

/// A borrowed view conveys no live delivery, session or mutation authority.
#[derive(Clone, Copy)]
pub(in crate::domain_computation::primary_graph) enum OutputPublicationView<'a> {
    Fresh(&'a WorthQueryApplicationCommitReceipt),
    Recovered(&'a WorthQueryRecoveredProducerPublication),
}
impl<'a> From<&'a WorthQueryApplicationCommitReceipt> for OutputPublicationView<'a> {
    fn from(value: &'a WorthQueryApplicationCommitReceipt) -> Self {
        Self::Fresh(value)
    }
}
impl<'a> OutputPublicationView<'a> {
    pub(in crate::domain_computation::primary_graph) fn committed_product_publication(
        self,
    ) -> &'a WorthQueryCommittedProductPublication {
        match self {
            Self::Fresh(v) => v.committed_product_publication(),
            Self::Recovered(v) => &v.facts.publication,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn product_branch(
        self,
    ) -> crate::basis::WorthQueryProductBranch {
        crate::basis::WorthQueryProductBranch::from_occurrence(
            self.committed_product_publication().product_incarnation(),
        )
    }
    pub(in crate::domain_computation::primary_graph) fn basis_descriptor(
        self,
    ) -> &'a worth_relational::facade::branch::RelationalBranchBasisDescriptor {
        match self {
            Self::Fresh(v) => v.basis_descriptor(),
            Self::Recovered(v) => v.original.basis_descriptor(),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn committed_changes(
        self,
    ) -> &'a WorthQueryApplicationCommittedChanges {
        match self {
            Self::Fresh(v) => v.committed_changes(),
            Self::Recovered(v) => &v.facts.changes,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn output_correspondence(
        self,
    ) -> &'a WorthQueryApplicationOutputCorrespondence {
        match self {
            Self::Fresh(v) => v.output_correspondence(),
            Self::Recovered(v) => &v.facts.correspondence,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn retain_output_correspondence(
        self,
    ) -> Arc<WorthQueryApplicationOutputCorrespondence> {
        match self {
            Self::Fresh(v) => v.retain_output_correspondence(),
            Self::Recovered(v) => Arc::clone(&v.facts.correspondence),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn performed_input_facts(
        self,
    ) -> Option<&'a FactPayload> {
        match self {
            Self::Fresh(v) => v.performed_input_facts(),
            Self::Recovered(v) => v.facts.inputs.as_ref(),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn performed_output_fields(
        self,
    ) -> Option<&'a Result<FactPayload, OutputFieldCaptureFailure>> {
        match self {
            Self::Fresh(v) => v.performed_output_fields(),
            Self::Recovered(v) => v.facts.fields.as_ref(),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn authority_binding(
        self,
    ) -> &'a WorthQueryApplicationCommitAuthorityBinding {
        match self {
            Self::Fresh(v) => v.authority_binding(),
            Self::Recovered(v) => &v.facts.authority,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn principal_scope(
        self,
    ) -> &'a crate::domain_computation::authorization::WorthQueryOperationScopeBinding {
        self.authority_binding().principal_scope()
    }
    pub(in crate::domain_computation::primary_graph) fn idempotency_binding(
        self,
    ) -> WorthQueryApplicationIdempotencyBinding {
        self.authority_binding().idempotency_binding()
    }
    pub(in crate::domain_computation::primary_graph) fn exact_output_settlement(
        self,
    ) -> Option<&'a Arc<RecordedSettlementIdentity>> {
        match self {
            Self::Fresh(v) => v.exact_output_settlement(),
            Self::Recovered(v) => v.facts.settlement.as_ref(),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn currentness_without_facts_at(
        self,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> Option<super::super::provider::FactlessCurrentness> {
        match self {
            Self::Fresh(v) => v.currentness_without_facts_at(observation),
            Self::Recovered(v) => Some(
                v.facts
                    .unrebased_own_effect?
                    .at_own_publication(v.facts.publication.is_selected_at(observation)),
            ),
        }
    }
}
