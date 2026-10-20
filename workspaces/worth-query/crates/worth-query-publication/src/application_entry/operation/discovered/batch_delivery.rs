//! Declared homogeneous batches reuse native complete-only execution.
use super::*;
use worth_query_declaration::facade::application_program::ApplicationDiscoveredBatchDelivery;
type BatchRead<S, I, A, const N: usize> =
    <ActionDelivery<S, I, A> as ApplicationDiscoveredBatchDelivery<S, Binding<S, I>, N>>::Read;
impl<Schema, Intent, Program, Root, Action>
    WorthQuerySettledDiscoveredOperation<Schema, Intent, Program, Root, Action>
where
    Schema: ApplicationSchema + 'static,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema> + ApplicationDiscoveredOutputRoot,
    Connection<Schema, Root>:
        WorthQueryApplicationDiscoveredOutputConnection<Schema, Source = Intent::Binding>,
    Action: ApplicationAction<Schema, Binding<Schema, Intent>>,
    ActionDelivery<Schema, Intent, Action>:
        ApplicationDiscoveredActionDelivery<Schema, Binding<Schema, Intent>, Root = Root>,
{
    /// Reads the installed ordered batch at one checked current observation.
    /// A refusal publishes no partial collection and retains this owner.
    ///
    /// A different cardinality cannot replace the installed ordered selector batch.
    /// ```compile_fail,E0277
    /// use worth_query_publication::facade::application_entry::*;
    /// use worth_query_declaration::facade::{application_program::*,application_operation::ApplicationMutationIntent,application_query::*,application_schema::{ApplicationSchema,ApplicationStructuredValueBinding}};
    /// use worth_query_execution::facade::primary_graph::{WorthQueryApplicationDiscoveredOutputConnection,WorthQueryApplicationProjection};
    /// fn exact<S,I,P,R,A,Q>(owner:&WorthQuerySettledDiscoveredOperation<S,I,P,R,A>,request:&WorthQueryApplicationRequest<'_, '_, '_, S>)
    /// where S:ApplicationSchema+'static,I:ApplicationMutationIntent<S>,P:ApplicationProgramDefinition<S>,
    /// R:ApplicationOutputGraphShape<S>+ApplicationDiscoveredOutputRoot,
    /// <R::RootConnection as ApplicationConnectionShape<S>>::Binding:WorthQueryApplicationDiscoveredOutputConnection<S,Source=I::Binding>,
    /// A:ApplicationAction<S,I::Binding>,A::Delivery:ApplicationDiscoveredBatchDelivery<S,I::Binding,2,Root=R,Read=Q>,
    /// Q:ApplicationQueryBinding<S>,Q::Input:ApplicationQueryIntent<S,Binding=Q>,
    /// Q::ScopeBinding:ApplicationQueryScopeResolution<S,Q::PrincipalIdentity>,
    /// <Q::ResultBinding as ApplicationStructuredValueBinding>::Value:WorthQueryApplicationProjection<S,Q::Query> {
    ///     let _ = owner.read_batch_current::<3>(request);
    /// }
    /// ```
    /// The installed cardinality has a usable delivery entrance under the same bounds.
    /// ```
    /// use worth_query_publication::facade::application_entry::*;
    /// use worth_query_declaration::facade::{application_program::*,application_operation::ApplicationMutationIntent,application_query::*,application_schema::{ApplicationSchema,ApplicationStructuredValueBinding}};
    /// use worth_query_execution::facade::primary_graph::{WorthQueryApplicationDiscoveredOutputConnection,WorthQueryApplicationProjection};
    /// fn exact<S,I,P,R,A,Q>(owner:&WorthQuerySettledDiscoveredOperation<S,I,P,R,A>,request:&WorthQueryApplicationRequest<'_, '_, '_, S>)
    /// where S:ApplicationSchema+'static,I:ApplicationMutationIntent<S>,P:ApplicationProgramDefinition<S>,
    /// R:ApplicationOutputGraphShape<S>+ApplicationDiscoveredOutputRoot,
    /// <R::RootConnection as ApplicationConnectionShape<S>>::Binding:WorthQueryApplicationDiscoveredOutputConnection<S,Source=I::Binding>,
    /// A:ApplicationAction<S,I::Binding>,A::Delivery:ApplicationDiscoveredBatchDelivery<S,I::Binding,2,Root=R,Read=Q>,
    /// Q:ApplicationQueryBinding<S>,Q::Input:ApplicationQueryIntent<S,Binding=Q>,
    /// Q::ScopeBinding:ApplicationQueryScopeResolution<S,Q::PrincipalIdentity>,
    /// <Q::ResultBinding as ApplicationStructuredValueBinding>::Value:WorthQueryApplicationProjection<S,Q::Query> {
    ///     let _ = owner.read_batch_current::<2>(request);
    /// }
    /// ```
    pub fn read_batch_current<const N: usize>(&self, request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>) -> Result<DeliveredProjectionBatch<Schema, BatchRead<Schema, Intent, Action, N>>, WorthQueryOperationDeliveryDenial>
    where
    ActionDelivery<Schema, Intent, Action>: ApplicationDiscoveredBatchDelivery<Schema, Binding<Schema, Intent>, N, Root = Root>,
    <BatchRead<Schema, Intent, Action, N> as ApplicationQueryBinding<Schema>>::Input: ApplicationQueryIntent<Schema, Binding = BatchRead<Schema, Intent, Action, N>>,
    <BatchRead<Schema, Intent, Action, N> as ApplicationQueryBinding<Schema>>::ScopeBinding: ApplicationQueryScopeResolution<Schema, <BatchRead<Schema, Intent, Action, N> as ApplicationQueryBinding<Schema>>::PrincipalIdentity>,
    <<BatchRead<Schema, Intent, Action, N> as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<Schema, <BatchRead<Schema, Intent, Action, N> as ApplicationQueryBinding<Schema>>::Query>,
    {
        self.read_batch_current_with_policy::<N>(
            request,
            WorthQueryOperationDeliveryPolicy::default(),
        )
    }
    /// Narrows native batch ceilings for a fresh delivery attempt.
    pub fn read_batch_current_with_policy<const N: usize>(&self, request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>, policy: WorthQueryOperationDeliveryPolicy) -> Result<DeliveredProjectionBatch<Schema, BatchRead<Schema, Intent, Action, N>>, WorthQueryOperationDeliveryDenial>
    where
    ActionDelivery<Schema, Intent, Action>: ApplicationDiscoveredBatchDelivery<Schema, Binding<Schema, Intent>, N, Root = Root>,
    <BatchRead<Schema, Intent, Action, N> as ApplicationQueryBinding<Schema>>::Input: ApplicationQueryIntent<Schema, Binding = BatchRead<Schema, Intent, Action, N>>,
    <BatchRead<Schema, Intent, Action, N> as ApplicationQueryBinding<Schema>>::ScopeBinding: ApplicationQueryScopeResolution<Schema, <BatchRead<Schema, Intent, Action, N> as ApplicationQueryBinding<Schema>>::PrincipalIdentity>,
    <<BatchRead<Schema, Intent, Action, N> as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<Schema, <BatchRead<Schema, Intent, Action, N> as ApplicationQueryBinding<Schema>>::Query>,
    {
        use WorthQueryOperationDeliveryDenial as Denial;
        if !std::ptr::eq(self.runtime.runtime(), request.application)
            || self.description.branch != request.branch
        {
            return Err(Denial::FreshRequestMismatch);
        }
        request.application
            .with_application_advancement(request.scope, |phase| {
                let observation = self.current_observation(request, &phase, policy)?;
                let inputs = <ActionDelivery<Schema, Intent, Action> as ApplicationDiscoveredBatchDelivery<
                    Schema,
                    Binding<Schema, Intent>,
                    N,
                >>::read_inputs(self.description.intent.input());
                request
                    .at(&observation)
                    .query_batch(inputs.into_iter().collect())
                    .execute_installed(policy.batch_limits, policy.query_limits, &phase)
                    .map_err(Denial::Batch)
            })
            .map_err(Denial::Execution)?
    }
    /// Reads original declared selectors at the retained settlement basis with
    /// fresh authority, without claiming currentness or advancing outputs.
    pub fn read_batch_historical<const N: usize>(&self, request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>) -> Result<DeliveredProjectionBatch<Schema, BatchRead<Schema, Intent, Action, N>>, WorthQueryOperationDeliveryDenial>
    where
    ActionDelivery<Schema, Intent, Action>: ApplicationDiscoveredBatchDelivery<Schema, Binding<Schema, Intent>, N, Root = Root>,
    <BatchRead<Schema, Intent, Action, N> as ApplicationQueryBinding<Schema>>::Input: ApplicationQueryIntent<Schema, Binding = BatchRead<Schema, Intent, Action, N>>,
    <BatchRead<Schema, Intent, Action, N> as ApplicationQueryBinding<Schema>>::ScopeBinding: ApplicationQueryScopeResolution<Schema, <BatchRead<Schema, Intent, Action, N> as ApplicationQueryBinding<Schema>>::PrincipalIdentity>,
    <<BatchRead<Schema, Intent, Action, N> as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<Schema, <BatchRead<Schema, Intent, Action, N> as ApplicationQueryBinding<Schema>>::Query>,
    {
        use WorthQueryOperationDeliveryDenial as Denial;
        if !std::ptr::eq(self.runtime.runtime(), request.application)
            || self.description.branch != request.branch
        {
            return Err(Denial::FreshRequestMismatch);
        }
        request.application
            .with_application_advancement(request.scope, |phase| {
                let inputs = <ActionDelivery<Schema, Intent, Action> as ApplicationDiscoveredBatchDelivery<
                    Schema,
                    Binding<Schema, Intent>,
                    N,
                >>::read_inputs(self.description.intent.input());
                request
                    .at(self.settlement.observation())
                    .query_batch(inputs.into_iter().collect())
                    .execute_installed(None, None, &phase)
                    .map_err(Denial::Batch)
            })
            .map_err(Denial::Execution)?
    }
}
