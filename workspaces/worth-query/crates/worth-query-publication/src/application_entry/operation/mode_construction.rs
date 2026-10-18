//! The declared delivery mode chooses a concrete owning lane before effects.
use super::*;
mod discovered;
use worth_query_declaration::facade::application_program::{
    ApplicationAction, ApplicationActionDelivery, ApplicationFixedDeliveryMode,
    ApplicationRetainedCommit, ApplicationSourceCompletion,
};
mod sealed {
    pub trait Factory<S, I, P, A> {}
}
/// Sealed construction dispatches installed delivery meaning to its concrete custody owner.
pub trait WorthQueryOperationModeFactory<S, I, P, A>: sealed::Factory<S, I, P, A>
where
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
{
    type Operation;
    fn construct(
        description: WorthQueryOperationDescription<S, I>,
        runtime: &WorthQueryProgramApplicationRuntime<S, P>,
    ) -> Result<
        Self::Operation,
        (
            WorthQueryOperationDescription<S, I>,
            WorthQueryOperationConstructionDenial,
        ),
    >;
}
impl<S: ApplicationSchema + 'static, I: ApplicationMutationIntent<S>>
    WorthQueryOperationDescription<S, I>
{
    pub fn retain_operation<P, A>(self, runtime: &WorthQueryProgramApplicationRuntime<S, P>) -> Result<<<A::Delivery as ApplicationActionDelivery<S, I::Binding>>::Mode as WorthQueryOperationModeFactory<S, I, P, A>>::Operation, (Self, WorthQueryOperationConstructionDenial)>
    where P: ApplicationProgramDefinition<S>, A: ApplicationAction<S, I::Binding>, <A::Delivery as ApplicationActionDelivery<S, I::Binding>>::Mode: WorthQueryOperationModeFactory<S, I, P, A>{
        <<A::Delivery as ApplicationActionDelivery<S, I::Binding>>::Mode as WorthQueryOperationModeFactory<S, I, P, A>>::construct(self, runtime)
    }
}
impl<S, I, P, A, R, Q> sealed::Factory<S, I, P, A> for ApplicationFixedDeliveryMode<R, Q>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    R: ApplicationOutputGraphShape<S> + ApplicationRequiredOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
    I::Binding: WorthQueryApplicationRequiredOutputSource<S, Connection<S, R>>,
    <A as ApplicationAction<S, I::Binding>>::Delivery:
        ApplicationFixedActionDelivery<S, I::Binding, Root = R, Projection = Q>,
    Q: WorthQueryOutputProjectionInstallation<S>,
{
}
impl<S, I, P, A, R, Q> WorthQueryOperationModeFactory<S, I, P, A>
    for ApplicationFixedDeliveryMode<R, Q>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    R: ApplicationOutputGraphShape<S> + ApplicationRequiredOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
    I::Binding: WorthQueryApplicationRequiredOutputSource<S, Connection<S, R>>,
    <A as ApplicationAction<S, I::Binding>>::Delivery:
        ApplicationFixedActionDelivery<S, I::Binding, Root = R, Projection = Q>,
    Q: WorthQueryOutputProjectionInstallation<S>,
{
    type Operation = WorthQueryFixedOperation<S, I, P, R, A>;
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
        description.retain_fixed_native::<P, A>(runtime)
    }
}
macro_rules! source_factory {
    ($mode:ty) => {
        impl<S, I, P, A> sealed::Factory<S, I, P, A> for $mode
        where
            S: ApplicationSchema + 'static,
            I: ApplicationMutationIntent<S>,
            P: ApplicationProgramDefinition<S>,
            A: ApplicationAction<S, I::Binding>,
        {
        }
        impl<S, I, P, A> WorthQueryOperationModeFactory<S, I, P, A> for $mode
        where
            S: ApplicationSchema + 'static,
            I: ApplicationMutationIntent<S>,
            P: ApplicationProgramDefinition<S>,
            A: ApplicationAction<S, I::Binding>,
        {
            type Operation = super::source_only::WorthQuerySourceOnlyOperation<S, I, P, A, $mode>;
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
                super::source_only::WorthQuerySourceOnlyOperation::construct(description, runtime)
            }
        }
    };
}
source_factory!(ApplicationSourceCompletion);
source_factory!(ApplicationRetainedCommit);
