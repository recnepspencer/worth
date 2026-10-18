//! Whole original membership and typed read share the exact native observation/phase.
use super::*;
impl<S, P, R> WorthQuerySettledProgramOutputOperation<S, P, R>
where
    S: ApplicationSchema + 'static,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    pub fn deliver_current(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        policy: WorthQueryOperationDeliveryPolicy,
    ) -> Result<WorthQueryApplicationReadObservation, WorthQueryOperationDeliveryDenial> {
        self.delivery.deliver_current(request, policy)
    }
    pub fn read_current<I>(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        intent: I,
    ) -> Result<
        crate::domain_computation::WorthQueryPublishedApplicationResult<
            ReadQuery<S, I>,
            ReadValue<S, I>,
        >,
        WorthQueryOperationDeliveryDenial,
    >
    where
        I: ApplicationQueryIntent<S>,
        <ReadBinding<S, I> as ApplicationQueryBinding<S>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                S,
                <ReadBinding<S, I> as ApplicationQueryBinding<S>>::PrincipalIdentity,
            >,
        ReadValue<S, I>: WorthQueryApplicationProjection<S, ReadQuery<S, I>>,
    {
        self.delivery.read_current(request, intent)
    }
    pub fn read_current_with_policy<I>(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        intent: I,
        policy: WorthQueryOperationDeliveryPolicy,
    ) -> Result<
        crate::domain_computation::WorthQueryPublishedApplicationResult<
            ReadQuery<S, I>,
            ReadValue<S, I>,
        >,
        WorthQueryOperationDeliveryDenial,
    >
    where
        I: ApplicationQueryIntent<S>,
        <ReadBinding<S, I> as ApplicationQueryBinding<S>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                S,
                <ReadBinding<S, I> as ApplicationQueryBinding<S>>::PrincipalIdentity,
            >,
        ReadValue<S, I>: WorthQueryApplicationProjection<S, ReadQuery<S, I>>,
    {
        self.delivery
            .read_current_with_policy(request, intent, policy)
    }
    pub fn settlement(&self) -> &WorthQueryApplicationProgramOutputSettlement<Query<S, R>> {
        self.delivery.settlement()
    }
}
