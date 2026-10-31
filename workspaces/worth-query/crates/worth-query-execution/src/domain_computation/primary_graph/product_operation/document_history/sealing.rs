//! Exact receipt capture and head adoption complete already allocated journal slots.
use super::{
    journal::RecordedNode, WorthQueryDocumentHistory, WorthQueryDocumentHistoryDenial as Denial,
    WorthQueryDocumentHistoryOperation, WorthQueryDocumentHistoryReservation,
    WorthQueryHistoryStep,
};
use crate::{
    basis::{WorthQueryProductBranchReadIdentity, WorthQueryProductObservationLease},
    domain_computation::primary_graph::{
        WorthQueryAdvancementPhase, WorthQueryApplicationCommitReceipt,
        WorthQueryExpectedProductBranch, WorthQueryFulfilledRequiredRoot,
    },
    publication_boundary::WorthQueryProgramPublicationAccess,
};
use worth_query_declaration::facade::{
    application_history::{ApplicationHistoryBinding, ApplicationHistoryModel},
    application_operation::{
        ApplicationMutationBinding, ApplicationMutationScopeBinding, NoApplicationMutationSource,
    },
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

struct PreparedBinding {
    node: RecordedNode,
    endpoint: WorthQueryProductBranchReadIdentity,
    exact: WorthQueryProductObservationLease,
    returned: WorthQueryHistoryStep,
}

enum RecordingCompletion<'a> {
    Source(&'a WorthQueryApplicationCommitReceipt),
    RequiredRoot {
        source: &'a WorthQueryApplicationCommitReceipt,
        root: &'a WorthQueryFulfilledRequiredRoot,
    },
}
impl RecordingCompletion<'_> {
    fn receipt(&self) -> &WorthQueryApplicationCommitReceipt {
        match self {
            Self::Source(source) | Self::RequiredRoot { source, .. } => source,
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
            .checked_execution_request_for(&self.application().product_runtime)
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

    /// Record the exact root that fulfilled this action, retaining its actual
    /// producer seed. A later candidate head is never substituted for that root.
    pub fn adopt_fulfilled_history<M>(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        installed: &WorthQueryInstalledDocumentHistory<S, H>,
        operation: WorthQueryDocumentHistoryOperation<'_, '_, S, M>,
        reserved: WorthQueryDocumentHistoryReservation,
        candidate: WorthQueryExpectedProductBranch<'runtime, S>,
        source: &WorthQueryApplicationCommitReceipt,
        root: &WorthQueryFulfilledRequiredRoot,
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
            RecordingCompletion::RequiredRoot { source, root },
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
        if let Some(capture) = &self.capture_pause {
            capture.pause();
        }
        let previous = match self.head.adopt_at(candidate, phase, &prepared.exact) {
            Ok(previous) => previous,
            Err(failure) => {
                let (candidate, denial) = failure.into_parts();
                return Err(WorthQueryDocumentHistoryAdoptionFailure {
                    reservation: reserved,
                    candidate,
                    denial: Denial::Adoption(denial),
                });
            }
        };
        // The exclusive document borrow is the document guard. Every destination
        // slot and every returned value exists before the actual head changes.
        self.expected = prepared.endpoint;
        self.physical_source = prepared.exact;
        self.journal
            .complete(reserved.record, prepared.node, previous);
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
            RecordingCompletion::RequiredRoot { source, root } => {
                let retained = root.retained_output();
                if !root.matches_source(source)
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
                    Some(root.posture()),
                )
            }
        };
        let endpoint =
            WorthQueryProductBranchReadIdentity::from_observation(selected.product().observation());
        let exact = selected.product().read_lease();
        let node = RecordedNode {
            step: self
                .journal
                .prepared_step(&reserved.record, endpoint.clone()),
            _source: retained.unwrap_or_else(|| selected.retain_application_read()),
            root_posture,
        };
        let returned = node.step.clone();
        Ok(PreparedBinding {
            node,
            endpoint,
            exact,
            returned,
        })
    }
}
