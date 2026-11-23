//! Every envelope stage is freshly admitted on the document's own private candidate.
use super::super::{
    confirm_document_head, WorthQueryDocumentHistory, WorthQueryDocumentHistoryDenial as Denial,
    WorthQueryDocumentHistoryReservation,
};
use crate::{
    basis::{WorthQueryProductBranch, WorthQueryProductObservationLease},
    domain_computation::primary_graph::{
        WorthQueryAdmittedApplicationOperation, WorthQueryAdvancementPhase,
        WorthQueryExpectedProductBranch, WorthQueryPrimaryGraphApplicationRuntime,
    },
    publication_boundary::WorthQueryProgramPublicationAccess,
};
use std::sync::{Arc, Mutex};
use worth_query_declaration::facade::{
    application_history::ApplicationHistoryBinding, application_schema::ApplicationSchema,
};
use worth_query_execution_attempt::facade::ports::AdmittedOperationExecutionPort as _;
use worth_query_execution_query::facade::ports::ApplicationRuntimeCoreInternalPort as _;
use worth_query_execution_runtime::facade::ports::{
    ProductBranchInternalPort as _, ProductBranchLeaseInternalPort as _,
    ProductObservationLeaseInternalPort as _, ProductPublicationBindingInternalPort as _,
};

/// The restriction every stage of one recorded envelope is admitted under. It
/// also keeps the first typed stage refusal, because stage admission reports
/// its own failures as text.
#[must_use = "retain the original restriction for every stage admission"]
pub struct WorthQueryDocumentHistoryEnvelopeConstraint {
    source: WorthQueryProductObservationLease,
    candidate: WorthQueryProductBranch,
    refusal: Mutex<Option<(usize, Denial)>>,
}

impl WorthQueryDocumentHistoryEnvelopeConstraint {
    /// Descriptive targeting for the stages' consuming transition.
    pub fn candidate(&self) -> WorthQueryProductBranch {
        self.candidate
    }

    /// Keep the first refused stage and its typed reason.
    pub fn record_refusal(&self, ordinal: usize, denial: Denial) {
        self.refusal
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get_or_insert((ordinal, denial));
    }

    pub fn take_refusal(&self) -> Option<(usize, Denial)> {
        self.refusal
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
    }

    fn original_source(&self) -> WorthQueryProductBranch {
        WorthQueryProductBranch::from_occurrence(self.source.observation().lifecycle_incarnation())
    }
}

impl<'runtime, S: ApplicationSchema, H: ApplicationHistoryBinding<S>>
    WorthQueryDocumentHistory<'runtime, S, H>
{
    pub fn history_envelope_constraint(
        &self,
        _access: &WorthQueryProgramPublicationAccess,
        reserved: &WorthQueryDocumentHistoryReservation,
        candidate: &WorthQueryExpectedProductBranch<'runtime, S>,
    ) -> Result<Arc<WorthQueryDocumentHistoryEnvelopeConstraint>, Denial> {
        self.validate_reservation(reserved)?;
        if !std::ptr::eq(self.application(), candidate.application()) {
            return Err(Denial::ForeignRuntime);
        }
        if candidate.expected_source() != reserved.expected {
            return Err(Denial::HeadDiverged);
        }
        Ok(Arc::new(WorthQueryDocumentHistoryEnvelopeConstraint {
            source: candidate.retained_expected_source(),
            candidate: candidate.branch(),
            refusal: Mutex::new(None),
        }))
    }
}

impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
    /// A stage must be admitted on the envelope's candidate while the document
    /// head is still the source that candidate was forked from.
    pub fn validate_history_envelope_stage<SOp, Input, Model>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _access: &WorthQueryProgramPublicationAccess,
        restriction: &WorthQueryDocumentHistoryEnvelopeConstraint,
        admitted: &WorthQueryAdmittedApplicationOperation<S, SOp, Input, Model>,
    ) -> Result<(), Denial> {
        if !admitted.belongs_to(
            self.inner.runtime.authority_identity(),
            &self.inner.installed_schema.binding_identity(),
        ) {
            return Err(Denial::ForeignRuntime);
        }
        admitted
            .validate_installed_operation(&self.inner.runtime.installed_operations_ref())
            .map_err(Denial::OperationAdmission)?;
        admitted
            .validate_current_authority()
            .map_err(Denial::OperationAdmission)?;
        let actual = admitted
            .graph_work()
            .mutation_product()
            .ok_or(Denial::ForeignRecording)?
            .publication_binding();
        if actual.observation().lifecycle_incarnation() != restriction.candidate.occurrence() {
            return Err(Denial::ForeignRecording);
        }
        let current = self
            .inner
            .runtime
            .product_runtime_ref()
            .admit_product_occurrence(restriction.original_source().occurrence())
            .map_err(Denial::CandidateSelection)?;
        if current.observation() != restriction.source.observation() {
            return Err(Denial::HeadDiverged);
        }
        drop(current);
        confirm_document_head(self.inner(), phase, &restriction.source, ())
            .map_err(|(_, denial)| denial)
    }
}
