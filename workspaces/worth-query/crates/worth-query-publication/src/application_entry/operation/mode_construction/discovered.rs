//! The declared discovered mode selects its genuine native owner before effects.
use super::*;
use worth_query_declaration::facade::application_program::{
    ApplicationAction, ApplicationDiscoveredActionDelivery,
    ApplicationDiscoveredDeliveryMode, ApplicationDiscoveredOutputRoot,
};
use worth_query_execution::facade::primary_graph::WorthQueryApplicationDiscoveredOutputConnection;
impl<S, I, P, A, R, Q> sealed::Factory<S, I, P, A>
    for ApplicationDiscoveredDeliveryMode<R, Q>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    R: ApplicationOutputGraphShape<S> + ApplicationDiscoveredOutputRoot,
    Connection<S, R>: WorthQueryApplicationDiscoveredOutputConnection<S, Source = I::Binding>,
    A::Delivery: ApplicationDiscoveredActionDelivery<S, I::Binding, Root = R, Projection = Q>,
    Q: WorthQueryOutputProjectionInstallation<S>,
{
}
impl<S, I, P, A, R, Q> WorthQueryOperationModeFactory<S, I, P, A>
    for ApplicationDiscoveredDeliveryMode<R, Q>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    R: ApplicationOutputGraphShape<S> + ApplicationDiscoveredOutputRoot,
    Connection<S, R>: WorthQueryApplicationDiscoveredOutputConnection<S, Source = I::Binding>,
    A::Delivery: ApplicationDiscoveredActionDelivery<S, I::Binding, Root = R, Projection = Q>,
    Q: WorthQueryOutputProjectionInstallation<S>,
{
    type Operation = WorthQueryDiscoveredOperation<S, I, P, R, A>;
    fn construct(
        description: WorthQueryOperationDescription<S, I>,
        runtime: &WorthQueryProgramApplicationRuntime<S, P>,
    ) -> Result<
        Self::Operation,
        (
            WorthQueryOperationDescription<S, I>,
            WorthQueryOperationConstructionDenial,
        ),
    > {
        WorthQueryDiscoveredOperation::construct(description, runtime)
    }
}
