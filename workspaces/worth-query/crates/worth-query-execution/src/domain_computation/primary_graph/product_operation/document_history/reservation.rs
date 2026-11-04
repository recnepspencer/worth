//! Admission precedes attempt inspection and allocation precedes candidate effects.
use super::{
    journal::{AttemptLookup, ReservedHistoryRecord},
    WorthQueryDocumentHistory, WorthQueryDocumentHistoryDenial as Denial,
    WorthQueryDocumentHistoryOperation, WorthQueryHistoryStep,
};
use worth_query_declaration::facade::{
    application_history::{
        ApplicationHistoryAccessBinding, ApplicationHistoryBinding, ApplicationHistoryModel,
    },
    application_operation::{
        ApplicationMutationBinding, ApplicationMutationScopeBinding, NoApplicationMutationSource,
    },
    application_query::ApplicationQueryBinding,
    application_schema::{ApplicationSchema, ApplicationStructuredValueBinding},
};
use worth_query_installation::facade::WorthQueryInstalledDocumentHistory;
use {
    crate::basis::WorthQueryProductBranchReadIdentity,
    crate::domain_computation::primary_graph::request_execution::WorthQueryAdvancementPhase,
    crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding,
    crate::publication_boundary::WorthQueryProgramPublicationAccess,
};

type Access<S, H> = ApplicationHistoryAccessBinding<S, H>;
type Query<S, H> = <Access<S, H> as ApplicationQueryBinding<S>>::Query;
type Rows<S, H> = <<Access<S, H> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;

#[must_use = "consume the reservation through actual adoption or explicit no-effect disposal"]
pub struct WorthQueryDocumentHistoryReservation {
    pub(super) record: ReservedHistoryRecord,
    pub(super) expected: WorthQueryProductBranchReadIdentity,
    pub(super) meaning: WorthQueryApplicationIdempotencyBinding,
}

pub enum WorthQueryDocumentHistoryRecordPreparation {
    Reserved(WorthQueryDocumentHistoryReservation),
    AlreadyRecorded(WorthQueryHistoryStep),
}

impl<'runtime, S: ApplicationSchema, H: ApplicationHistoryBinding<S>>
    WorthQueryDocumentHistory<'runtime, S, H>
{
    /// Fresh ordinary READ must complete before entering this publication-only receiver.
    pub fn reserve_history_record<M>(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        installed: &WorthQueryInstalledDocumentHistory<S, H>,
        operation: WorthQueryDocumentHistoryOperation<'_, '_, S, M>,
        metadata: crate::publication_boundary::document_history::WorthQueryDocumentHistoryRead<
            '_,
            S,
            H,
        >,
    ) -> Result<WorthQueryDocumentHistoryRecordPreparation, Denial>
    where
        M: ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
        M::ScopeBinding: ApplicationMutationScopeBinding<S, Scope = ApplicationHistoryModel<S, H>>,
        Rows<S, H>: crate::domain_computation::primary_graph::WorthQueryApplicationProjection<
            S,
            Query<S, H>,
        >,
    {
        let runtime = self.application();
        // Every fresh attempt lookup is metadata disclosure, even when the result
        // will name a previously completed occurrence rather than execute again.
        runtime.read_document_history_step(phase, access, self, metadata)?;
        let WorthQueryDocumentHistoryOperation {
            selected,
            admitted,
            binding,
        } = operation;
        runtime.assess_document_history_operation::<H, M>(
            phase, access, self, installed, selected, admitted,
        )?;
        if M::WORKFLOW_CONTROL
            || M::REQUIRES_WORKFLOW_AUTHORITY
            || M::workflow_execution_requirements().is_applicable()
            || admitted
                .allowed_graph_contract()
                .external_effect()
                .is_declared()
        {
            return Err(Denial::UnsupportedOperation);
        }
        let meaning = binding.bind_admission(admitted);
        let expected =
            WorthQueryProductBranchReadIdentity::from_observation(selected.product().observation());
        let mut selected = self.selection.lock();
        if selected.expected != expected {
            return Err(Denial::HeadDiverged);
        }
        match selected.journal.lookup(meaning) {
            AttemptLookup::Drift => return Err(Denial::IntentDrift),
            AttemptLookup::Reserved => return Err(Denial::AttemptInFlight),
            AttemptLookup::Completed(node) => {
                return Ok(WorthQueryDocumentHistoryRecordPreparation::AlreadyRecorded(
                    selected
                        .journal
                        .step(node)
                        .expect("completed attempt binds a completed node")
                        .clone(),
                ))
            }
            AttemptLookup::Unseen => {}
        }
        let label = installed
            .operation_label::<M>()
            .expect("membership was admitted");
        let execution = phase
            .checked_execution_request_for(&runtime.runtime.product_runtime)
            .map_err(Denial::Stopped)?;
        let record = selected.journal.reserve(&execution, meaning, label)?;
        Ok(WorthQueryDocumentHistoryRecordPreparation::Reserved(
            WorthQueryDocumentHistoryReservation {
                record,
                expected: selected.expected.clone(),
                meaning,
            },
        ))
    }

    pub(super) fn validate_reservation(
        &self,
        reserved: &WorthQueryDocumentHistoryReservation,
    ) -> Result<(), Denial> {
        self.selection.lock().validate_reservation(reserved)
    }

    pub fn readmit_history_record<M>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        installed: &WorthQueryInstalledDocumentHistory<S, H>,
        operation: WorthQueryDocumentHistoryOperation<'_, '_, S, M>,
        reserved: &WorthQueryDocumentHistoryReservation,
    ) -> Result<(), Denial>
    where
        M: ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
        M::ScopeBinding: ApplicationMutationScopeBinding<S, Scope = ApplicationHistoryModel<S, H>>,
    {
        self.validate_reservation(reserved)?;
        self.application()
            .assess_document_history_operation::<H, M>(
                phase,
                access,
                self,
                installed,
                operation.selected,
                operation.admitted,
            )?;
        let meaning = operation.binding.bind_admission(operation.admitted);
        if meaning.canonical_key() != reserved.meaning.canonical_key()
            || !meaning.has_same_intent(reserved.meaning)
        {
            return Err(Denial::IntentDrift);
        }
        Ok(())
    }

    pub fn cancel_history_record(
        &mut self,
        _access: &WorthQueryProgramPublicationAccess,
        reserved: WorthQueryDocumentHistoryReservation,
    ) -> Result<(), (WorthQueryDocumentHistoryReservation, Denial)> {
        let mut selected = self.selection.lock();
        if let Err(denial) = selected.validate_reservation(&reserved) {
            return Err((reserved, denial));
        }
        selected.journal.cancel(reserved.record);
        Ok(())
    }

    pub fn current_history_step(
        &self,
        _access: &WorthQueryProgramPublicationAccess,
    ) -> WorthQueryHistoryStep {
        self.selection.lock().journal.current_step().clone()
    }
}

impl<S> super::selection::HistorySelection<'_, S> {
    pub(super) fn validate_reservation(
        &self,
        reserved: &WorthQueryDocumentHistoryReservation,
    ) -> Result<(), Denial> {
        if !self.journal.matches_reservation(&reserved.record) {
            return Err(Denial::ForeignRecording);
        }
        if self.expected != reserved.expected {
            return Err(Denial::HeadDiverged);
        }
        Ok(())
    }
}
