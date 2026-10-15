//! Declared homogeneous batches reuse native complete-only execution.
use super::*;
use worth_query_declaration::facade::application_program::ApplicationFixedBatchDelivery;
type BatchRead<S, I, A, const N: usize> =
    <Delivery<S, I, A> as ApplicationFixedBatchDelivery<S, Binding<S, I>, N>>::Read;
impl<Schema, Intent, Program, Root, Action>
    WorthQuerySettledFixedOperation<Schema, Intent, Program, Root, Action>
where
    Schema: ApplicationSchema + 'static,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema> + ApplicationRequiredOutputRoot,
    Connection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Binding<Schema, Intent>:
        WorthQueryApplicationRequiredOutputSource<Schema, Connection<Schema, Root>>,
    Action: ApplicationAction<Schema, Binding<Schema, Intent>>,
    Delivery<Schema, Intent, Action>:
        ApplicationFixedActionDelivery<Schema, Binding<Schema, Intent>, Root = Root>,
{
    /// Reads the installed ordered batch at one checked current observation.
    /// A refusal publishes no partial collection and retains this owner.
    pub fn read_batch_current<const N: usize>(&self, request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>) -> Result<DeliveredProjectionBatch<Schema, BatchRead<Schema, Intent, Action, N>>, WorthQueryOperationDeliveryDenial>
    where
    Delivery<Schema, Intent, Action>: ApplicationFixedBatchDelivery<Schema, Binding<Schema, Intent>, N, Root = Root>,
    <BatchRead<Schema, Intent, Action, N> as ApplicationQueryBinding<Schema>>::Input: ApplicationQueryIntent<Schema, Binding = BatchRead<Schema, Intent, Action, N>>,
    <BatchRead<Schema, Intent, Action, N> as ApplicationQueryBinding<Schema>>::ScopeBinding: ApplicationQueryScopeResolution<Schema, <BatchRead<Schema, Intent, Action, N> as ApplicationQueryBinding<Schema>>::PrincipalIdentity>,
    <<BatchRead<Schema, Intent, Action, N> as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<Schema, <BatchRead<Schema, Intent, Action, N> as ApplicationQueryBinding<Schema>>::Query>,
    {
        self.read_batch_current_with_policy::<N>(
            request,
            WorthQueryOperationDeliveryPolicy::currentness(
                std::num::NonZeroUsize::new(
                    self.runtime
                        .runtime()
                        .output_demand_resource_profile()
                        .limits()
                        .source_currentness_work(),
                )
                .expect("installed finite positive currentness policy"),
            ),
        )
    }
    /// Narrows native batch ceilings for a fresh delivery attempt.
    pub fn read_batch_current_with_policy<const N: usize>(&self, request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>, policy: WorthQueryOperationDeliveryPolicy) -> Result<DeliveredProjectionBatch<Schema, BatchRead<Schema, Intent, Action, N>>, WorthQueryOperationDeliveryDenial>
    where
    Delivery<Schema, Intent, Action>: ApplicationFixedBatchDelivery<Schema, Binding<Schema, Intent>, N, Root = Root>,
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
        request
            .application
            .with_application_advancement(request.scope, |phase| {
                let observation = self.current_observation(request, &phase, policy)?;
                let inputs = <Delivery<Schema, Intent, Action> as ApplicationFixedBatchDelivery<
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
    Delivery<Schema, Intent, Action>: ApplicationFixedBatchDelivery<Schema, Binding<Schema, Intent>, N, Root = Root>,
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
        request
            .application
            .with_application_advancement(request.scope, |phase| {
                let inputs = <Delivery<Schema, Intent, Action> as ApplicationFixedBatchDelivery<
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
