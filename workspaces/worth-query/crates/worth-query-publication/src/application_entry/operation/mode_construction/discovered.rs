//! The declared discovered mode selects its genuine native owner before effects.
use super::*;
use worth_query_declaration::facade::application_program::{
    ApplicationDiscoveredAction, ApplicationDiscoveredActionDelivery,
    ApplicationDiscoveredDeliveryMode, ApplicationDiscoveredOutputRoot,
};
use worth_query_execution::facade::primary_graph::WorthQueryApplicationDiscoveredOutputConnection;
impl<'r, S, I, P, A, R, Q> sealed::Factory<'r, S, I, P, A>
    for ApplicationDiscoveredDeliveryMode<R, Q>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationDiscoveredAction<S, I::Binding>,
    R: ApplicationOutputGraphShape<S> + ApplicationDiscoveredOutputRoot,
    Connection<S, R>: WorthQueryApplicationDiscoveredOutputConnection<S, Source = I::Binding>,
    A::Delivery: ApplicationDiscoveredActionDelivery<S, I::Binding, Root = R, Projection = Q>,
    Q: WorthQueryOutputProjectionInstallation<S>,
{
}
impl<'r, S, I, P, A, R, Q> WorthQueryOperationModeFactory<'r, S, I, P, A>
    for ApplicationDiscoveredDeliveryMode<R, Q>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationDiscoveredAction<S, I::Binding>,
    R: ApplicationOutputGraphShape<S> + ApplicationDiscoveredOutputRoot,
    Connection<S, R>: WorthQueryApplicationDiscoveredOutputConnection<S, Source = I::Binding>,
    A::Delivery: ApplicationDiscoveredActionDelivery<S, I::Binding, Root = R, Projection = Q>,
    Q: WorthQueryOutputProjectionInstallation<S>,
{
    type Operation = WorthQueryDiscoveredOperation<'r, S, I, P, R, A>;
    fn construct(
        description: WorthQueryOperationDescription<S, I>,
        runtime: &'r WorthQueryProgramApplicationRuntime<S, P>,
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
