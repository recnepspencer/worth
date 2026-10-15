//! No-projection delivery validates native settlement without query authority.
use super::*;
impl<'runtime, Schema, Intent, Program, Root, Action>
    WorthQuerySettledFixedOperation<'runtime, Schema, Intent, Program, Root, Action>
where
    Schema: ApplicationSchema + 'static,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema> + ApplicationRequiredOutputRoot,
    Connection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Binding<Schema, Intent>:
        WorthQueryApplicationRequiredOutputSource<Schema, Connection<Schema, Root>>,
    Action: ApplicationFixedAction<Schema, Binding<Schema, Intent>>,
    Delivery<Schema, Intent, Action>: ApplicationFixedActionDelivery<
        Schema,
        Binding<Schema, Intent>,
        Root = Root,
        Projection = worth_query_declaration::facade::application_program::ApplicationNoProjection,
    >,
{
    /// Validates all required settlements at one authentic current observation without a query.
    /// Refusal borrows this owner and preserves its descriptor, history, and cleanup.
    ///
    /// No-projection delivery is usable without query binding or selector bounds.
    /// ```
    /// use worth_query_publication::facade::application_entry::*;
    /// use worth_query_declaration::facade::{application_operation::*, application_program::*, application_schema::ApplicationSchema};
    /// use worth_query_execution::facade::primary_graph::{WorthQueryApplicationRequiredOutputConnection, WorthQueryApplicationRequiredOutputSource};
    /// fn deliver<S, I, P, R, A>(settled: &WorthQuerySettledFixedOperation<'_, S, I, P, R, A>, request: &WorthQueryApplicationRequest<'_, '_, '_, S>, policy: WorthQueryOperationDeliveryPolicy)
    /// where S: ApplicationSchema + 'static, I: ApplicationMutationIntent<S>, P: ApplicationProgramDefinition<S>,
    /// R: ApplicationOutputGraphShape<S> + ApplicationRequiredOutputRoot,
    /// <R::RootConnection as ApplicationConnectionShape<S>>::Binding: WorthQueryApplicationRequiredOutputConnection<S>,
    /// I::Binding: WorthQueryApplicationRequiredOutputSource<S, <R::RootConnection as ApplicationConnectionShape<S>>::Binding>,
    /// A: ApplicationFixedAction<S, I::Binding>,
    /// A::Delivery: ApplicationFixedActionDelivery<S, I::Binding, Root = R, Projection = ApplicationNoProjection> {
    ///     let _ = settled.deliver_current(request, policy);
    /// }
    /// ```
    /// A no-projection owner cannot issue a declared projected read.
    /// ```compile_fail,E0599
    /// use worth_query_publication::facade::application_entry::*;
    /// use worth_query_declaration::facade::{application_operation::*, application_program::*, application_schema::ApplicationSchema};
    /// use worth_query_execution::facade::primary_graph::{WorthQueryApplicationRequiredOutputConnection, WorthQueryApplicationRequiredOutputSource};
    /// fn deliver<S, I, P, R, A>(settled: &WorthQuerySettledFixedOperation<'_, S, I, P, R, A>, request: &WorthQueryApplicationRequest<'_, '_, '_, S>, policy: WorthQueryOperationDeliveryPolicy)
    /// where S: ApplicationSchema + 'static, I: ApplicationMutationIntent<S>, P: ApplicationProgramDefinition<S>,
    /// R: ApplicationOutputGraphShape<S> + ApplicationRequiredOutputRoot,
    /// <R::RootConnection as ApplicationConnectionShape<S>>::Binding: WorthQueryApplicationRequiredOutputConnection<S>,
    /// I::Binding: WorthQueryApplicationRequiredOutputSource<S, <R::RootConnection as ApplicationConnectionShape<S>>::Binding>,
    /// A: ApplicationFixedAction<S, I::Binding>,
    /// A::Delivery: ApplicationFixedActionDelivery<S, I::Binding, Root = R, Projection = ApplicationNoProjection> {
    ///     let _ = settled.read_current(request);
    /// }
    /// ```
    pub fn deliver_current(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
        policy: WorthQueryOperationDeliveryPolicy,
    ) -> Result<WorthQueryApplicationReadObservation, WorthQueryOperationDeliveryDenial> {
        self.runtime
            .runtime()
            .with_application_advancement(request.scope, |phase| {
                self.current_observation(request, &phase, policy)
            })
            .map_err(WorthQueryOperationDeliveryDenial::Execution)?
    }
}
