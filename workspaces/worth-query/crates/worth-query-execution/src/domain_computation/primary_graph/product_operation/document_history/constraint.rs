//! A history restriction never replaces the ordinary operation's fresh native admission.
use super::{
    confirm_document_head, HistoryModelIdentity, WorthQueryDocumentHistory,
    WorthQueryDocumentHistoryDenial as Denial, WorthQueryDocumentHistoryReservation,
};
use crate::{
    basis::{WorthQueryProductBranch, WorthQueryProductObservationLease},
    domain_computation::primary_graph::{
        WorthQueryAdmittedApplicationOperation, WorthQueryAdvancementPhase,
        WorthQueryApplicationIdempotencyBinding, WorthQueryExpectedProductBranch,
        WorthQueryPrimaryGraphApplicationRuntime,
    },
    publication_boundary::WorthQueryProgramPublicationAccess,
};
use worth_query_declaration::facade::{
    application_history::ApplicationHistoryBinding, application_schema::ApplicationSchema,
};


#[must_use = "retain the original restriction for every native attempt preparation"]
pub struct WorthQueryDocumentHistoryOperationConstraint {
    model: HistoryModelIdentity,
    source: WorthQueryProductObservationLease,
    candidate: WorthQueryProductBranch,
    meaning: WorthQueryApplicationIdempotencyBinding,
    candidate_predecessor: Option<crate::basis::WorthQueryProductBranchReadIdentity>,
}

impl WorthQueryDocumentHistoryOperationConstraint {
    pub(super) fn for_group(
        model: HistoryModelIdentity,
        source: WorthQueryProductObservationLease,
        candidate: WorthQueryProductBranch,
        meaning: WorthQueryApplicationIdempotencyBinding,
        candidate_predecessor: crate::basis::WorthQueryProductBranchReadIdentity,
    ) -> Self {
        Self {
            model,
            source,
            candidate,
            meaning,
            candidate_predecessor: Some(candidate_predecessor),
        }
    }
    /// Descriptive targeting for the one initial admission's consuming transition.
    pub fn candidate(&self) -> WorthQueryProductBranch {
        self.candidate
    }
    pub fn original_source(&self) -> WorthQueryProductBranch {
        WorthQueryProductBranch::from_occurrence(self.source.observation().lifecycle_incarnation())
    }
}

impl<'runtime, S: ApplicationSchema, H: ApplicationHistoryBinding<S>>
    WorthQueryDocumentHistory<'runtime, S, H>
{
    pub fn history_operation_constraint(
        &self,
        _access: &WorthQueryProgramPublicationAccess,
        reserved: &WorthQueryDocumentHistoryReservation,
        candidate: &WorthQueryExpectedProductBranch<'runtime, S>,
    ) -> Result<WorthQueryDocumentHistoryOperationConstraint, Denial> {
        self.validate_reservation(reserved)?;
        if !std::ptr::eq(self.application(), candidate.application()) {
            return Err(Denial::ForeignRuntime);
        }
        if candidate.expected_source() != reserved.expected {
            return Err(Denial::HeadDiverged);
        }
        Ok(WorthQueryDocumentHistoryOperationConstraint {
            model: self.model.clone(),
            source: candidate.retained_expected_source(),
            candidate: candidate.branch(),
            meaning: reserved.meaning,
            candidate_predecessor: None,
        })
    }
}

impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
    /// Only fresh business execution needs the group predecessor. Original
    /// outcome lookup/recovery must survive that member's own publication.
    pub fn validate_history_execution_predecessor<SOp, Input, Model>(
        &self,
        _access: &WorthQueryProgramPublicationAccess,
        restriction: &WorthQueryDocumentHistoryOperationConstraint,
        admitted: &WorthQueryAdmittedApplicationOperation<S, SOp, Input, Model>,
    ) -> Result<(), Denial> {
        if let Some(expected) = &restriction.candidate_predecessor {
            let actual = admitted
                .graph_work()
                .mutation_product()
                .ok_or(Denial::ForeignRecording)?
                .publication_binding();
            if !expected.matches_observation(actual.observation()) {
                return Err(Denial::HeadDiverged);
            }
        }
        Ok(())
    }

    pub fn validate_history_operation<SOp, Input, Model>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _access: &WorthQueryProgramPublicationAccess,
        restriction: &WorthQueryDocumentHistoryOperationConstraint,
        admitted: &WorthQueryAdmittedApplicationOperation<S, SOp, Input, Model>,
        binding: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<(), Denial> {
        if !admitted.belongs_to(
            self.runtime.authority_identity(),
            &self.installed_schema.binding_identity(),
        ) {
            return Err(Denial::ForeignRuntime);
        }
        if !restriction.model.matches_operation(admitted) {
            return Err(Denial::ForeignModel);
        }
        admitted
            .validate_installed_operation(&self.runtime.installed_operations)
            .map_err(Denial::OperationAdmission)?;
        admitted
            .validate_current_authority()
            .map_err(Denial::OperationAdmission)?;
        let meaning = binding.bind_admission(admitted);
        if meaning.canonical_key() != restriction.meaning.canonical_key()
            || !meaning.has_same_intent(restriction.meaning)
        {
            return Err(Denial::IntentDrift);
        }
        let actual = admitted
            .graph_work()
            .mutation_product()
            .ok_or(Denial::ForeignRecording)?
            .publication_binding();
        if actual.observation().lifecycle_incarnation() != restriction.candidate.occurrence() {
            return Err(Denial::ForeignRecording);
        }
        let current = self
            .runtime
            .product_runtime
            .admit_product_occurrence(restriction.original_source().occurrence())
            .map_err(Denial::CandidateSelection)?;
        if current.observation() != restriction.source.observation() {
            return Err(Denial::HeadDiverged);
        }
        drop(current);
        confirm_document_head(self, phase, &restriction.source, ()).map_err(|(_, denial)| denial)
    }
}
