//! Fresh disclosure authorization borrows the original owner without advancing it.
use super::*;
use crate::application_entry::mutation::WorthQueryCurrentAuthorizationAssessment;

impl<S, I, P, A, M> WorthQuerySourceOnlyOperation<S, I, P, A, M>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    M: WorthQuerySourceDeliveryMode,
    I::Binding: WorthQuerySourceOperationBinding<S>,
{
    /// Observe current Query authorization before disclosing retained status.
    /// Uses the original captured input, preconditions, action and workflow context.
    /// Neither success nor denial advances publication, claims a key, consumes a
    /// workflow grant or releases this owner's custody. Success is not admission.
    pub fn assess_current_authorization(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryCurrentAuthorizationAssessment, WorthQueryApplicationRequestMutationDenial>
    {
        <I::Binding as binding::sealed::Driver<S>>::assess(self, request)
    }
}

impl<S, I, P, A, M> WorthQueryCompletedSourceOperation<S, I, P, A, M>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    M: WorthQuerySourceDeliveryMode,
    I::Binding: WorthQuerySourceOperationBinding<S>,
{
    /// Recheck current authorization without consuming retained completion evidence.
    pub fn assess_current_authorization(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryCurrentAuthorizationAssessment, WorthQueryApplicationRequestMutationDenial>
    {
        self.operation.assess_current_authorization(request)
    }
}

impl<S, I, P, A, M> WorthQueryTerminalSourceOperation<S, I, P, A, M>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    M: WorthQuerySourceDeliveryMode,
    I::Binding: WorthQuerySourceOperationBinding<S>,
{
    /// Recheck current authorization without replacing or closing terminal custody.
    pub fn assess_current_authorization(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryCurrentAuthorizationAssessment, WorthQueryApplicationRequestMutationDenial>
    {
        self.operation.assess_current_authorization(request)
    }
}
