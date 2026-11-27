//! Exact receipt capture and head adoption complete already allocated journal slots.
use super::{
    journal::RecordedNode, source_evidence::WorthQueryHistorySourceEvidence,
    WorthQueryDocumentHistory, WorthQueryDocumentHistoryDenial as Denial,
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
use worth_query_execution_query::facade::ports::ApplicationRuntimeCoreInternalPort as _;
use worth_query_installation::facade::WorthQueryInstalledDocumentHistory;

mod adoption_failure;
pub use adoption_failure::WorthQueryDocumentHistoryAdoptionFailure;

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

mod binding;
mod completion;
pub(super) use binding::BoundSource;
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
            .checked_execution_request_for(self.application().inner.runtime.product_runtime_ref())
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
        let mut current_head = selected.take_head().rebind(self.custody());
        let (previous, release) =
            match current_head.adopt_at_deferred(candidate, phase, &prepared.exact) {
                Ok(adopted) => adopted,
                Err(failure) => {
                    selected.put_head(current_head.into_detached());
                    drop(selected);
                    let (candidate, denial) = failure.into_parts();
                    return Err(WorthQueryDocumentHistoryAdoptionFailure {
                        reservation: reserved,
                        candidate,
                        denial: Denial::Adoption(denial),
                    });
                }
            };
        selected.put_head(current_head.into_detached());
        // Every destination slot and returned value exists before adoption.
        // Only infallible metadata moves remain while the selection is guarded.
        selected.expected = prepared.endpoint;
        let previous_source = std::mem::replace(&mut selected.physical_source, prepared.exact);
        let excluded = selected.journal.complete(
            reserved.record,
            prepared.node,
            prepared.source,
            prepared.discovery_recipes,
            previous.into_detached(),
        );
        drop(selected);
        drop(excluded);
        drop(previous_source);
        drop(release);
        Ok(prepared.returned)
    }
}
