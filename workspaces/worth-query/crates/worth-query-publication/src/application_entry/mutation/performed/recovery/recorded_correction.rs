//! Fresh admission from the recorded correction's retained, encoded inverse.
use super::super::*;
use crate::application_entry::mutation::{
    authorization::WorthQueryBindingAdmission, captured_input::CapturedMutationInput,
    request::WorthQueryApplicationMutationRequest, WorthQueryApplicationRecoveryRequestDenial,
};
use crate::application_entry::WorthQueryApplicationRequest;
use std::sync::Arc;
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationAdmissionBinding, ApplicationMutationInputScope,
    ApplicationMutationScopeResolution, ApplicationOwnedMutationIdentities,
};
use worth_query_declaration::facade::application_program::{
    ApplicationProgramOutputsShape, ApplicationRequiredOutputRoot,
};
use worth_query_execution::facade::primary_graph::WorthQueryManagedApplicationRecoveryDenial;

impl<S, B, P, R> WorthQueryApplicationPerformedMutationOutcome<S, B, P, R>
where
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S>
        + WorthQueryApplicationRequiredOutputSource<S, RootConnection<S, R>>,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationRequiredOutputRoot,
    RootConnection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    pub(in crate::application_entry) fn retain_recorded_input(
        self,
        input: ApplicationOwnedMutationIdentities<S, B>,
    ) -> Self {
        match self {
            Self::ProductUnpublished(mut owner) => {
                owner.recorded_input = Some(Arc::new(input));
                Self::ProductUnpublished(owner)
            }
            Self::RequiredOutputDenied(mut owner) => {
                owner.recorded_input = Some(Arc::new(input));
                Self::RequiredOutputDenied(owner)
            }
            Self::CleanupRequired { cleanup, outcome } => Self::CleanupRequired {
                cleanup,
                outcome: Box::new(outcome.retain_recorded_input(input)),
            },
            other => other,
        }
    }
}

impl<S, B, P, R> WorthQueryUnpublishedRequiredApplicationMutation<S, B, P, R>
where
    S: ApplicationSchema + 'static,
    B: ApplicationMutationAdmissionBinding<S>
        + WorthQueryApplicationRequiredOutputSource<S, RootConnection<S, R>>,
    B::Admission: WorthQueryBindingAdmission<S, B>,
    B::Input: Clone + Send + Sync + ApplicationMutationInputScope<S, B>,
    B::ScopeBinding: ApplicationMutationScopeResolution<S, B::PrincipalIdentity>,
    P: ApplicationProgramDefinition<S>,
    P::Outputs: ApplicationProgramOutputsShape<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationRequiredOutputRoot,
    RootConnection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    /// Continues the original inverse under this fresh request's current authority.
    /// Refusal preserves this owner and its captured input and key.
    pub fn recover_recorded_correction(
        &mut self,
        application: &WorthQueryProgramApplicationRuntime<S, P>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        crate::application_entry::WorthQueryProgramSourceRecoveryProgress,
        WorthQueryApplicationRecoveryRequestDenial,
    > {
        let input = self.recorded_input.as_ref().cloned().ok_or_else(mismatch)?;
        let encoded = input.borrow().encoded_input();
        let mut mutation = WorthQueryApplicationMutationRequest::new(
            request.application,
            request.principal,
            request.scope,
            request.branch,
            CapturedMutationInput::<S, B>::new(encoded.clone()),
        )
        .idempotency(input.key());
        let encoded = Ok(encoded);
        mutation.operation_input = Some(&encoded);
        mutation.recover_unpublished_required_in_program(self, application)
    }

    /// Transfers original recovered source custody after a fresh keyed admission.
    /// Every refusal returns this complete owner, including its captured inverse.
    pub fn promote_recorded_correction(
        self,
        application: &WorthQueryProgramApplicationRuntime<S, P>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        WorthQueryRecoveredRequiredOutputs<S, P, R, B>,
        (WorthQueryApplicationRecoveryRequestDenial, Self),
    > {
        let Some(input) = self.recorded_input.as_ref().cloned() else {
            return Err((mismatch(), self));
        };
        let encoded = input.borrow().encoded_input();
        let mut mutation = WorthQueryApplicationMutationRequest::new(
            request.application,
            request.principal,
            request.scope,
            request.branch,
            CapturedMutationInput::<S, B>::new(encoded.clone()),
        )
        .idempotency(input.key());
        let encoded = Ok(encoded);
        mutation.operation_input = Some(&encoded);
        mutation.promote_recovered_required_outputs(self, application)
    }
}

impl<S, B, P, R> WorthQueryRequiredOutputRetentionFailure<S, B, P, R>
where
    S: ApplicationSchema + 'static,
    B: ApplicationMutationAdmissionBinding<S>
        + WorthQueryApplicationRequiredOutputSource<S, RootConnection<S, R>>,
    B::Admission: WorthQueryBindingAdmission<S, B>,
    B::Input: Clone + Send + Sync + ApplicationMutationInputScope<S, B>,
    B::ScopeBinding: ApplicationMutationScopeResolution<S, B::PrincipalIdentity>,
    P: ApplicationProgramDefinition<S>,
    P::Outputs: ApplicationProgramOutputsShape<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationRequiredOutputRoot,
    RootConnection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    /// Retries retention of the original correction through current admission.
    pub fn retry_recorded_correction_retention(
        self,
        application: &WorthQueryProgramApplicationRuntime<S, P>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        WorthQueryPerformedApplicationMutation<S, B, P, R>,
        (WorthQueryApplicationRecoveryRequestDenial, Self),
    > {
        let Some(input) = self.recorded_input.as_ref().cloned() else {
            return Err((mismatch(), self));
        };
        let encoded = input.borrow().encoded_input();
        let mut mutation = WorthQueryApplicationMutationRequest::new(
            request.application,
            request.principal,
            request.scope,
            request.branch,
            CapturedMutationInput::<S, B>::new(encoded.clone()),
        )
        .idempotency(input.key());
        let encoded = Ok(encoded);
        mutation.operation_input = Some(&encoded);
        mutation.retry_required_output_retention(self, application)
    }
}

fn mismatch() -> WorthQueryApplicationRecoveryRequestDenial {
    WorthQueryApplicationRecoveryRequestDenial::Recovery(
        WorthQueryManagedApplicationRecoveryDenial::BindingMismatch,
    )
}
