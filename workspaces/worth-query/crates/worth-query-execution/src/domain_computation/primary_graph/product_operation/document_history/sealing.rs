//! Exact receipt capture and head adoption complete already allocated journal slots.
use super::source_evidence::WorthQueryHistorySourceEvidence;
use super::{
    journal::RecordedNode, WorthQueryDocumentHistory, WorthQueryDocumentHistoryDenial as Denial,
    WorthQueryDocumentHistoryOperation, WorthQueryDocumentHistoryReservation,
    WorthQueryHistoryStep,
};
use crate::{
    basis::{WorthQueryProductBranchReadIdentity, WorthQueryProductObservationLease},
    domain_computation::primary_graph::{
        WorthQueryAdvancementPhase, WorthQueryApplicationCommitReceipt,
        WorthQueryCompletedRequiredOutputs, WorthQueryExpectedProductBranch,
        WorthQueryRecoveredApplicationCommitReceipt,
    },
    publication_boundary::WorthQueryProgramPublicationAccess,
};
use worth_query_declaration::facade::{
    application_history::ApplicationHistoryBinding,
    application_operation::{ApplicationMutationBinding, NoApplicationMutationSource},
    application_schema::ApplicationSchema,
};
use worth_query_installation::facade::WorthQueryInstalledDocumentHistory;

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
    pub(super) discovery_recipes:
        Option<std::sync::Arc<super::discovery_recipes::HistoryDiscoveryRecipes>>,
    pub(super) node: RecordedNode,
    pub(super) source: std::sync::Arc<
        crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation,
    >,
    pub(super) endpoint: WorthQueryProductBranchReadIdentity,
    pub(super) exact: WorthQueryProductObservationLease,
    pub(super) returned: WorthQueryHistoryStep,
}

mod completion;
pub(super) use completion::RecordingCompletion;

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
                crate::domain_computation::primary_graph::application_contribution::advancement_denial(
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

    /// Adopt an authenticated original source through fresh history admission.
    /// The recorded receipt carries evidence, never current execution authority.
    pub fn adopt_original_history<M>(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        installed: &WorthQueryInstalledDocumentHistory<S, H>,
        operation: WorthQueryDocumentHistoryOperation<'_, '_, S, M>,
        reserved: WorthQueryDocumentHistoryReservation,
        candidate: WorthQueryExpectedProductBranch<'runtime, S>,
        receipt: &WorthQueryRecoveredApplicationCommitReceipt,
    ) -> Result<WorthQueryHistoryStep, WorthQueryDocumentHistoryAdoptionFailure<'runtime, S>>
    where
        M: ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
    {
        self.adopt_recording::<M>(
            phase,
            access,
            installed,
            operation,
            reserved,
            candidate,
            RecordingCompletion::RecordedSource(receipt),
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

    pub fn adopt_recording_output_forest<M>(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        installed: &WorthQueryInstalledDocumentHistory<S, H>,
        operation: WorthQueryDocumentHistoryOperation<'_, '_, S, M>,
        reserved: WorthQueryDocumentHistoryReservation,
        candidate: WorthQueryExpectedProductBranch<'runtime, S>,
        source: WorthQueryHistorySourceEvidence<'_>,
        required: Option<&WorthQueryCompletedRequiredOutputs>,
        completed: &super::WorthQueryCompletedRecordingOutputs,
    ) -> Result<WorthQueryHistoryStep, WorthQueryDocumentHistoryAdoptionFailure<'runtime, S>>
    where
        M: ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
    {
        self.adopt_recording::<M>(
            phase,
            access,
            installed,
            operation,
            reserved,
            candidate,
            RecordingCompletion::Forest {
                source,
                required,
                completed,
            },
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
        let excluded = selected.journal.complete(
            reserved.record,
            prepared.node,
            prepared.source,
            prepared.discovery_recipes,
            previous,
        );
        drop(selected);
        drop(excluded);
        drop(previous_source);
        drop(release);
        Ok(prepared.returned)
    }

    pub(super) fn prepare_binding(
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
        if !completion
            .source()
            .matches(reserved.operation_scope()?, reserved.meaning)
        {
            return Err(Denial::ReceiptMismatch);
        }
        let (selected, retained, root_posture) = match &completion {
            RecordingCompletion::Forest {
                source,
                required,
                completed,
            } => {
                completed.require_source(
                    reserved,
                    candidate,
                    reserved.operation_scope()?,
                    *source,
                    *required,
                )?;
                let retained = completed.endpoint();
                let selected = runtime
                    .select_application_read_observation(phase, retained)
                    .map_err(Denial::CandidateSelection)?;
                (
                    selected,
                    Some(std::sync::Arc::clone(retained)),
                    completed.root_posture(),
                )
            }

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
            RecordingCompletion::RecordedSource(receipt) => {
                let selected = runtime
                    .on_branch(candidate.branch())
                    .select(phase)
                    .map_err(Denial::CandidateSelection)?;
                if !WorthQueryHistorySourceEvidence::Recorded(receipt)
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
        let discovery_recipes = match &completion {
            RecordingCompletion::Forest { completed, .. } => {
                Some(std::sync::Arc::clone(&completed.discovery_recipes))
            }
            _ => None,
        };
        let node = RecordedNode { step, root_posture };
        let returned = node.step.clone();
        Ok(PreparedBinding {
            discovery_recipes,
            node,
            source: retained.unwrap_or_else(|| selected.retain_application_read()),
            endpoint,
            exact,
            returned,
        })
    }
}
