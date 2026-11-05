//! Exact receipt capture and head adoption complete already allocated journal slots.
use super::{
    journal::RecordedNode, WorthQueryDocumentHistory, WorthQueryDocumentHistoryDenial as Denial,
    WorthQueryDocumentHistoryOperation, WorthQueryDocumentHistoryReservation,
    WorthQueryHistoryStep,
};
use worth_query_declaration::facade::{
    application_history::{ApplicationHistoryBinding, ApplicationHistoryModel},
    application_operation::{
        ApplicationMutationBinding, ApplicationMutationScopeBinding, NoApplicationMutationSource,
    },
    application_schema::ApplicationSchema,
};
use worth_query_installation::facade::WorthQueryInstalledDocumentHistory;
use {
    crate::basis::WorthQueryProductBranchReadIdentity,
    crate::basis::WorthQueryProductObservationLease,
    crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    crate::domain_computation::primary_graph::WorthQueryCompletedRequiredOutputs,
    crate::domain_computation::primary_graph::WorthQueryExpectedProductBranch,
    crate::domain_computation::request_execution::WorthQueryAdvancementPhase,
    crate::publication_boundary::WorthQueryProgramPublicationAccess,
};

#[must_use = "retain the original reservation and actual candidate after refusal"]
pub struct WorthQueryDocumentHistoryAdoptionFailure<'runtime, S> {
    reservation: WorthQueryDocumentHistoryReservation,
    candidate: WorthQueryExpectedProductBranch<'runtime, S>,
    denial: Denial,
}

impl<'runtime, S> WorthQueryDocumentHistoryAdoptionFailure<'runtime, S> {
    pub fn into_parts(
        self,
    ) -> (
        WorthQueryDocumentHistoryReservation,
        WorthQueryExpectedProductBranch<'runtime, S>,
        Denial,
    ) {
        (self.reservation, self.candidate, self.denial)
    }
    pub fn denial(&self) -> &Denial {
        &self.denial
    }
}

impl<S> std::fmt::Debug for WorthQueryDocumentHistoryAdoptionFailure<'_, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryDocumentHistoryAdoptionFailure")
            .field("denial", &self.denial)
            .finish_non_exhaustive()
    }
}

pub(super) struct PreparedBinding {
    pub(super) node: RecordedNode,
    pub(super) source: std::sync::Arc<
        crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation,
    >,
    pub(super) endpoint: WorthQueryProductBranchReadIdentity,
    pub(super) exact: WorthQueryProductObservationLease,
    pub(super) returned: WorthQueryHistoryStep,
}

enum RecordingCompletion<'a> {
    Source(&'a WorthQueryApplicationCommitReceipt),
    RequiredOutputs {
        source: &'a WorthQueryApplicationCommitReceipt,
        completed: &'a WorthQueryCompletedRequiredOutputs,
    },
}
impl RecordingCompletion<'_> {
    fn receipt(&self) -> &WorthQueryApplicationCommitReceipt {
        match self {
            Self::Source(source) | Self::RequiredOutputs { source, .. } => source,
        }
    }
}

impl<'runtime, S: ApplicationSchema, H: ApplicationHistoryBinding<S>>
    WorthQueryDocumentHistory<'runtime, S, H>
{
    /// Delivery can stop after selection. The caller keeps the already selected
    /// document and its original completion instead of claiming no effect.
    pub fn validate_record_delivery(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _access: &WorthQueryProgramPublicationAccess,
    ) -> Result<(), Denial> {
        let execution = phase
            .checked_execution_request_for(&self.application().runtime.product_runtime)
            .map_err(Denial::Stopped)?;
        execution.check_live().map_err(|stop| {
            Denial::Stopped(
                crate::domain_computation::request_execution::advancement_denial(
                    worth_execution::WorkCeilingDenial::Stopped(stop),
                ),
            )
        })
    }

    pub fn adopt_recorded_history<M>(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        installed: &WorthQueryInstalledDocumentHistory<S, H>,
        operation: WorthQueryDocumentHistoryOperation<'_, '_, S, M>,
        reserved: WorthQueryDocumentHistoryReservation,
        candidate: WorthQueryExpectedProductBranch<'runtime, S>,
        receipt: &WorthQueryApplicationCommitReceipt,
    ) -> Result<WorthQueryHistoryStep, WorthQueryDocumentHistoryAdoptionFailure<'runtime, S>>
    where
        M: ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
        M::ScopeBinding: ApplicationMutationScopeBinding<S, Scope = ApplicationHistoryModel<S, H>>,
    {
        self.adopt_recording::<M>(
            phase,
            access,
            installed,
            operation,
            reserved,
            candidate,
            RecordingCompletion::Source(receipt),
        )
    }

    /// Record one exact basis qualified against this action's complete original
    /// output set. A later candidate head is never substituted for that basis.
    pub fn adopt_completed_history<M>(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        installed: &WorthQueryInstalledDocumentHistory<S, H>,
        operation: WorthQueryDocumentHistoryOperation<'_, '_, S, M>,
        reserved: WorthQueryDocumentHistoryReservation,
        candidate: WorthQueryExpectedProductBranch<'runtime, S>,
        source: &WorthQueryApplicationCommitReceipt,
        completed: &WorthQueryCompletedRequiredOutputs,
    ) -> Result<WorthQueryHistoryStep, WorthQueryDocumentHistoryAdoptionFailure<'runtime, S>>
    where
        M: ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
        M::ScopeBinding: ApplicationMutationScopeBinding<S, Scope = ApplicationHistoryModel<S, H>>,
    {
        self.adopt_recording::<M>(
            phase,
            access,
            installed,
            operation,
            reserved,
            candidate,
            RecordingCompletion::RequiredOutputs { source, completed },
        )
    }

    fn adopt_recording<M>(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        installed: &WorthQueryInstalledDocumentHistory<S, H>,
        operation: WorthQueryDocumentHistoryOperation<'_, '_, S, M>,
        reserved: WorthQueryDocumentHistoryReservation,
        candidate: WorthQueryExpectedProductBranch<'runtime, S>,
        completion: RecordingCompletion<'_>,
    ) -> Result<WorthQueryHistoryStep, WorthQueryDocumentHistoryAdoptionFailure<'runtime, S>>
    where
        M: ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
        M::ScopeBinding: ApplicationMutationScopeBinding<S, Scope = ApplicationHistoryModel<S, H>>,
    {
        let current =
            self.readmit_history_record::<M>(phase, access, installed, operation, &reserved);
        if let Err(denial) = current {
            return Err(WorthQueryDocumentHistoryAdoptionFailure {
                reservation: reserved,
                candidate,
                denial,
            });
        }
        let prepared = match self.prepare_binding(phase, &reserved, &candidate, completion) {
            Ok(prepared) => prepared,
            Err(denial) => {
                return Err(WorthQueryDocumentHistoryAdoptionFailure {
                    reservation: reserved,
                    candidate,
                    denial,
                })
            }
        };
        self.adopt_prepared_history(phase, reserved, candidate, prepared)
    }

    pub(super) fn adopt_prepared_history(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        reserved: WorthQueryDocumentHistoryReservation,
        candidate: WorthQueryExpectedProductBranch<'runtime, S>,
        prepared: PreparedBinding,
    ) -> Result<WorthQueryHistoryStep, WorthQueryDocumentHistoryAdoptionFailure<'runtime, S>> {
        if let Some(capture) = &self.capture_pause {
            capture.pause();
        }
        let mut selected = self.selection.lock();
        if let Err(denial) = selected.validate_reservation(&reserved) {
            drop(selected);
            return Err(WorthQueryDocumentHistoryAdoptionFailure {
                reservation: reserved,
                candidate,
                denial,
            });
        }
        selected.invalidate_output_authority();
        let (previous, release) =
            match selected
                .head
                .adopt_at_deferred(candidate, phase, &prepared.exact)
            {
                Ok(adopted) => adopted,
                Err(failure) => {
                    drop(selected);
                    let (candidate, denial) = failure.into_parts();
                    return Err(WorthQueryDocumentHistoryAdoptionFailure {
                        reservation: reserved,
                        candidate,
                        denial: Denial::Adoption(denial),
                    });
                }
            };
        // Every destination slot and returned value exists before adoption.
        // Only infallible metadata moves remain while the selection is guarded.
        selected.expected = prepared.endpoint;
        let previous_source = std::mem::replace(&mut selected.physical_source, prepared.exact);
        let excluded =
            selected
                .journal
                .complete(reserved.record, prepared.node, prepared.source, previous);
        drop(selected);
        drop(excluded);
        drop(previous_source);
        drop(release);
        Ok(prepared.returned)
    }

    fn prepare_binding(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        reserved: &WorthQueryDocumentHistoryReservation,
        candidate: &WorthQueryExpectedProductBranch<'runtime, S>,
        completion: RecordingCompletion<'_>,
    ) -> Result<PreparedBinding, Denial> {
        self.validate_reservation(reserved)?;
        let runtime = self.application();
        if !std::ptr::eq(runtime, candidate.application()) {
            return Err(Denial::ForeignRuntime);
        }
        if candidate.expected_source() != reserved.expected {
            return Err(Denial::HeadDiverged);
        }
        let receipt = completion.receipt();
        if !self.model.matches_receipt(receipt)
            || receipt.idempotency_binding().canonical_key() != reserved.meaning.canonical_key()
            || !receipt
                .idempotency_binding()
                .has_same_intent(reserved.meaning)
        {
            return Err(Denial::ReceiptMismatch);
        }
        let (selected, retained, root_posture) = match &completion {
            RecordingCompletion::Source(receipt) => {
                let selected = runtime
                    .on_branch(candidate.branch())
                    .select(phase)
                    .map_err(Denial::CandidateSelection)?;
                if !receipt
                    .committed_product_publication()
                    .is_selected_at(selected.product().observation())
                {
                    return Err(Denial::ReceiptMismatch);
                }
                (selected, None, None)
            }
            RecordingCompletion::RequiredOutputs { source, completed } => {
                let retained = completed.retained_output();
                if !completed.matches_source(source)
                    || retained.branch_incarnation() != candidate.branch().occurrence()
                {
                    return Err(Denial::ReceiptMismatch);
                }
                let selected = runtime
                    .select_application_read_observation(phase, retained)
                    .map_err(Denial::CandidateSelection)?;
                (
                    selected,
                    Some(std::sync::Arc::clone(retained)),
                    Some(completed.root_posture()),
                )
            }
        };
        let endpoint =
            WorthQueryProductBranchReadIdentity::from_observation(selected.product().observation());
        let exact = selected.product().read_lease();
        let step = self
            .selection
            .lock()
            .journal
            .prepared_step(&reserved.record, endpoint.clone());
        let node = RecordedNode { step, root_posture };
        let returned = node.step.clone();
        Ok(PreparedBinding {
            node,
            source: retained.unwrap_or_else(|| selected.retain_application_read()),
            endpoint,
            exact,
            returned,
        })
    }
}
