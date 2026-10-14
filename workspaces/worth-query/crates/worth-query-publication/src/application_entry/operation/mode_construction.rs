//! The declared delivery mode chooses a concrete owning lane before effects.
use super::*;
use worth_query_declaration::facade::application_program::{
    ApplicationAction, ApplicationActionDelivery, ApplicationFixedDeliveryMode,
    ApplicationRetainedCommit, ApplicationSourceCompletion,
};
mod sealed {
    pub trait Factory<'r, S, I, P, A> {}
}
/// Sealed construction dispatches installed delivery meaning to its concrete custody owner.
pub trait WorthQueryOperationModeFactory<'r, S, I, P, A>: sealed::Factory<'r, S, I, P, A>
where
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
{
    type Operation;
    fn construct(
        description: WorthQueryOperationDescription<S, I>,
        runtime: &'r WorthQueryProgramApplicationRuntime<S, P>,
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
    pub fn retain_operation<'r, P, A>(self, runtime: &'r WorthQueryProgramApplicationRuntime<S, P>) -> Result<<<A::Delivery as ApplicationActionDelivery<S, I::Binding>>::Mode as WorthQueryOperationModeFactory<'r, S, I, P, A>>::Operation, (Self, WorthQueryOperationConstructionDenial)>
    where P: ApplicationProgramDefinition<S>, A: ApplicationAction<S, I::Binding>, <A::Delivery as ApplicationActionDelivery<S, I::Binding>>::Mode: WorthQueryOperationModeFactory<'r, S, I, P, A>{
        <<A::Delivery as ApplicationActionDelivery<S, I::Binding>>::Mode as WorthQueryOperationModeFactory<'r, S, I, P, A>>::construct(self, runtime)
    }
}
impl<'r, S, I, P, A, R, Q> sealed::Factory<'r, S, I, P, A> for ApplicationFixedDeliveryMode<R, Q>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding> + ApplicationFixedAction<S, I::Binding>,
    R: ApplicationOutputGraphShape<S> + ApplicationRequiredOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
    I::Binding: WorthQueryApplicationRequiredOutputSource<S, Connection<S, R>>,
    <A as ApplicationFixedAction<S, I::Binding>>::Delivery:
        ApplicationFixedActionDelivery<S, I::Binding, Root = R, Read = Q>,
{
}
impl<'r, S, I, P, A, R, Q> WorthQueryOperationModeFactory<'r, S, I, P, A>
    for ApplicationFixedDeliveryMode<R, Q>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding> + ApplicationFixedAction<S, I::Binding>,
    R: ApplicationOutputGraphShape<S> + ApplicationRequiredOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
    I::Binding: WorthQueryApplicationRequiredOutputSource<S, Connection<S, R>>,
    <A as ApplicationFixedAction<S, I::Binding>>::Delivery:
        ApplicationFixedActionDelivery<S, I::Binding, Root = R, Read = Q>,
{
    type Operation = WorthQueryFixedOperation<'r, S, I, P, R, A>;
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
        description.retain_fixed_native::<P, A>(runtime)
    }
}
macro_rules! source_factory {
    ($mode:ty) => {
        impl<'r, S, I, P, A> sealed::Factory<'r, S, I, P, A> for $mode
        where
            S: ApplicationSchema + 'static,
            I: ApplicationMutationIntent<S>,
            P: ApplicationProgramDefinition<S>,
            A: ApplicationAction<S, I::Binding>,
            I::Binding: 'r,
        {
        }
        impl<'r, S, I, P, A> WorthQueryOperationModeFactory<'r, S, I, P, A> for $mode
        where
            S: ApplicationSchema + 'static,
            I: ApplicationMutationIntent<S>,
            P: ApplicationProgramDefinition<S>,
            A: ApplicationAction<S, I::Binding>,
            I::Binding: 'r,
        {
            type Operation =
                super::source_admission::WorthQuerySourceOperationAdmission<'r, S, I, P, A, $mode>;
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
                super::source_admission::WorthQuerySourceOperationAdmission::construct(
                    description,
                    runtime,
                )
            }
        }
    };
}
source_factory!(ApplicationSourceCompletion);
source_factory!(ApplicationRetainedCommit);
