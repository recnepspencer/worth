//! Fresh edit eligibility is an observation, never a saved execution grant.
use super::*;
use crate::application_entry::{
    mutation::WorthQueryCurrentAuthorizationAssessment,
    operation::{
        SourceAssessmentDriver, WorthQuerySourceDeliveryMode, WorthQuerySourceOnlyOperation,
        WorthQuerySourceOperationBinding,
    },
};
use worth_query_declaration::facade::{
    application_history::ApplicationHistoryModel,
    application_operation::{
        ApplicationMutationBinding, ApplicationMutationIntent, ApplicationMutationScopeBinding,
    },
    application_program::{ApplicationAction, ApplicationProgramDefinition},
};

impl<
        S: ApplicationSchema + 'static,
        H: ApplicationHistoryBinding<S>,
        Payload: WorthQueryHistoryPayloadMode<S, H>,
    > WorthQueryApplicationHistoryDocument<'_, S, H, Payload>
where
    Scope<S, H>: ApplicationQueryScopeResolution<S, Principal<S, H>>,
    Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
{
    /// Observe whether this original no-output action currently belongs to this
    /// document. Uses fresh document READ and native action authorization.
    /// Neither owner is consumed; no key, candidate, handler or workflow slot advances.
    /// Success is not authority to execute: execution must authorize again.
    pub fn assess_edit<I, P, A, M>(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        operation: &WorthQuerySourceOnlyOperation<S, I, P, A, M>,
    ) -> Result<WorthQueryCurrentAuthorizationAssessment, WorthQueryApplicationHistoryDenial>
    where
        I: ApplicationMutationIntent<S>,
        P: ApplicationProgramDefinition<S>,
        A: ApplicationAction<S, I::Binding>,
        M: WorthQuerySourceDeliveryMode,
        I::Binding: WorthQuerySourceOperationBinding<S>,
        <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding:
            ApplicationMutationScopeBinding<S, Scope = ApplicationHistoryModel<S, H>>,
    {
        let runtime = self.owner.application();
        if !std::ptr::eq(runtime, request.application) {
            return Err(WorthQueryApplicationHistoryDenial::ForeignRuntime);
        }
        runtime
            .with_application_advancement(request.scope, |phase| {
                // Complete ordinary disclosure before inspecting operation membership.
                self.read_in_advancement(&phase, request)?;
                let current = WorthQueryApplicationRequest {
                    application: runtime,
                    principal: request.principal,
                    scope: request.scope,
                    branch: self.owner.branch(),
                };
                let (admitted, selected) =
                    <I::Binding as SourceAssessmentDriver<S>>::admit_for_assessment(
                        operation, &phase, &current,
                    )
                    .map_err(WorthQueryApplicationHistoryDenial::Mutation)?;
                let installed = runtime
                    .installed_schema()
                    .installed_history::<H>()
                    .map_err(WorthQueryApplicationHistoryDenial::Installation)?;
                runtime
                    .assess_document_history_operation::<H, I::Binding>(
                        &phase,
                        &program_publication_access(),
                        &self.owner,
                        &installed,
                        &selected,
                        &admitted,
                    )
                    .map_err(WorthQueryApplicationHistoryDenial::Document)?;
                Ok(WorthQueryCurrentAuthorizationAssessment::observed())
            })
            .map_err(WorthQueryApplicationHistoryDenial::ExecutionRequest)?
    }
}
