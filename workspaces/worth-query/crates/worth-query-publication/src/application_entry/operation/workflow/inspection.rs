//! Freshly authorized observation of performed source evidence, independent of dispatch progress.
use super::*;
use crate::application_entry::{
    WorthQueryApplicationRecoveryRequestDenial, WorthQueryWorkflowOperationOwnerPosture,
};
use worth_query_declaration::facade::application_operation::ApplicationMutationAdmissionBinding;
use worth_query_execution::publication_boundary::workflow_advance::{
    WorthQueryGuardedWorkflowOperationCustody, WorthQueryWorkflowAdvanceAdapter,
};

/// Native custody of an established source completion, not whole-workflow completion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryWorkflowSourcePosture {
    /// The original source is committed with no outstanding guarded dispatch.
    Committed,
    /// The source committed, but its external effect has not settled.
    DispatchPending,
    /// Native custody records settlement of the original external effect.
    ExternallySettled,
}
/// Borrowed original source evidence. Neither this view nor its posture settles a workflow.
pub struct WorthQueryWorkflowSourceInspection<'owner, Result> {
    source: &'owner WorthQuerySourceCompletion<Result>,
    posture: WorthQueryWorkflowSourcePosture,
}
impl<Result> WorthQueryWorkflowSourceInspection<'_, Result> {
    /// Borrows retained original completion; does not transfer a performed witness.
    pub const fn source(&self) -> &WorthQuerySourceCompletion<Result> {
        self.source
    }
    /// Reports current native custody without changing the owner's lifecycle.
    pub const fn posture(&self) -> WorthQueryWorkflowSourcePosture {
        self.posture
    }
}
/// Fresh admission, evidence availability or native custody refused observation.
#[derive(Debug)]
pub enum WorthQueryWorkflowSourceInspectionDenial {
    /// The request could not freshly open the original installed binding.
    Opening(WorthQueryApplicationRecoveryRequestDenial),
    /// Native inspection of the exact original key and transition failed.
    Inspection(worth_query_execution::facade::primary_graph::WorthQueryApplicationIdempotencyResolutionDenial),
    /// The owner has no established source completion. It may still retain uncertainty.
    SourceEvidenceUnavailable,
    /// Native custody does not currently establish an observable committed posture.
    Owner(WorthQueryWorkflowOperationOwnerPosture),
}
impl<S, I, P, A> WorthQueryWorkflowOperation<S, I, P, A>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    I::Binding: ApplicationMutationAdmissionBinding<S>,
    <I::Binding as ApplicationMutationAdmissionBinding<S>>::Admission:
        super::super::super::mutation::authorization::WorthQueryBindingAdmission<S, I::Binding>,
    <I::Binding as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync + 'static,
    <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding: ApplicationMutationScopeResolution<
        S,
        <I::Binding as ApplicationMutationBinding<S>>::PrincipalIdentity,
    >,
{
    /// Observes the original source under fresh admission and native custody inspection.
    /// Does not execute, retry, mint recovery, release cleanup, or settle the workflow.
    pub fn inspect_source(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        WorthQueryWorkflowSourceInspection<
            '_,
            <I::Binding as ApplicationMutationBinding<S>>::Result,
        >,
        WorthQueryWorkflowSourceInspectionDenial,
    > {
        use WorthQueryWorkflowSourceInspectionDenial as Denial;
        let source = &self.source.admission;
        let posture = source
            .runtime
            .runtime()
            .with_application_advancement(request.scope, |phase| {
                let mut native = source
                    .description
                    .request_view(request, &source.installed_action);
                let prepared = source
                    .authorize_recovery_view(&phase, request, &mut native)
                    .map_err(Denial::Opening)?;
                if !self.source_completed {
                    return Err(Denial::SourceEvidenceUnavailable);
                }
                let receipt = match self.source.completion() {
                    WorthQuerySourceCompletion::Fresh { receipt, .. }
                    | WorthQuerySourceCompletion::Recovered { receipt, .. } => receipt,
                };
                let custody = WorthQueryWorkflowAdvanceAdapter::resolve_guarded_operation_custody(
                    &phase,
                    source.runtime.runtime(),
                    &prepared.admission,
                    prepared.idempotency,
                    self.required.transition_identity_bytes(),
                    Some(receipt),
                )
                .map_err(Denial::Inspection)?;
                match custody {
                    WorthQueryGuardedWorkflowOperationCustody::Committed(_) => {
                        Ok(WorthQueryWorkflowSourcePosture::Committed)
                    }
                    WorthQueryGuardedWorkflowOperationCustody::DispatchPending(_) => {
                        Ok(WorthQueryWorkflowSourcePosture::DispatchPending)
                    }
                    WorthQueryGuardedWorkflowOperationCustody::ExternallySettled(_) => {
                        Ok(WorthQueryWorkflowSourcePosture::ExternallySettled)
                    }
                    other => Err(Denial::Owner(
                        crate::application_entry::workflow::operation::owner::other_custody(other),
                    )),
                }
            })
            .map_err(|cause| {
                Denial::Opening(WorthQueryApplicationRecoveryRequestDenial::advancement(
                    cause,
                ))
            })??;
        Ok(WorthQueryWorkflowSourceInspection {
            source: self.source.completion(),
            posture,
        })
    }
}
