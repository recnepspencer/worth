//! Compiler bodies return genuine owners after destroying the constructing runtime handle.
//! Runtime behavior is proved by the external installed consumer courts.
use super::discovered_handoff_contract::NonCloneOriginal;
use super::*;
use worth_query_declaration::facade::application_program::{
    ApplicationDiscoveredActionDelivery, ApplicationDiscoveredOutputRoot,
};
use worth_query_execution::facade::primary_graph::WorthQueryApplicationDiscoveredOutputConnection;

fn fixed_escapes_construction<S, I, P, R, A>(
    description: WorthQueryOperationDescription<S, NonCloneOriginal<I>>,
    runtime: WorthQueryProgramApplicationRuntime<S, P>,
) -> Result<
    WorthQueryFixedOperation<S, NonCloneOriginal<I>, P, R, A>,
    (
        WorthQueryOperationDescription<S, NonCloneOriginal<I>>,
        WorthQueryOperationConstructionDenial,
    ),
>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationRequiredOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
    I::Binding: WorthQueryApplicationRequiredOutputSource<S, Connection<S, R>>,
    A: ApplicationAction<S, I::Binding>,
    A::Delivery: ApplicationFixedActionDelivery<S, I::Binding, Root = R>,
    <A::Delivery as ApplicationFixedActionDelivery<S, I::Binding>>::Projection:
        WorthQueryOutputProjectionInstallation<S>,
{
    let retained = description.retain_operation::<P, A>(&runtime);
    drop(runtime);
    retained
}

fn discovered_escapes_construction<S, I, P, R, A>(
    description: WorthQueryOperationDescription<S, NonCloneOriginal<I>>,
    runtime: WorthQueryProgramApplicationRuntime<S, P>,
) -> Result<
    WorthQueryDiscoveredOperation<S, NonCloneOriginal<I>, P, R, A>,
    (
        WorthQueryOperationDescription<S, NonCloneOriginal<I>>,
        WorthQueryOperationConstructionDenial,
    ),
>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationDiscoveredOutputRoot,
    Connection<S, R>: WorthQueryApplicationDiscoveredOutputConnection<S, Source = I::Binding>,
    A: ApplicationAction<S, I::Binding>,
    A::Delivery: ApplicationDiscoveredActionDelivery<S, I::Binding, Root = R>,
    <A::Delivery as ApplicationDiscoveredActionDelivery<S, I::Binding>>::Projection:
        WorthQueryOutputProjectionInstallation<S>,
{
    let retained = description.retain_operation::<P, A>(&runtime);
    drop(runtime);
    retained
}
