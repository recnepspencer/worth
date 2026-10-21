//! Declared scalar reads at the original or freshly validated native basis.
use super::*;
impl<Schema, Intent, Program, Root, Action> FixedDeliveryCore<Schema, Intent, Program, Root, Action>
where
    Schema: ApplicationSchema + 'static,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema> + ApplicationRequiredOutputRoot,
    Connection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Binding<Schema, Intent>: WorthQueryApplicationRequiredOutputSource<Schema, Connection<Schema, Root>>,
    Action: ApplicationAction<Schema, Binding<Schema, Intent>>,
    Delivery<Schema, Intent, Action>: ApplicationFixedQueryDelivery<Schema, Binding<Schema, Intent>, Root = Root>,
    <Read<Schema, Intent, Action> as ApplicationQueryBinding<Schema>>::Input: ApplicationQueryIntent<Schema, Binding = Read<Schema, Intent, Action>>,
    <Read<Schema, Intent, Action> as ApplicationQueryBinding<Schema>>::ScopeBinding: ApplicationQueryScopeResolution<Schema, <Read<Schema, Intent, Action> as ApplicationQueryBinding<Schema>>::PrincipalIdentity>,
    <<Read<Schema, Intent, Action> as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<Schema, <Read<Schema, Intent, Action> as ApplicationQueryBinding<Schema>>::Query> + Clone,
{
    pub fn read_current(&self, request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>) -> Result<DeliveredProjection<Schema, Read<Schema, Intent, Action>>, WorthQueryOperationDeliveryDenial> {
        self.read_current_with_policy(request, WorthQueryOperationDeliveryPolicy::default())
    }

    pub fn read_current_with_policy(&self, request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>, policy: WorthQueryOperationDeliveryPolicy) -> Result<DeliveredProjection<Schema, Read<Schema, Intent, Action>>, WorthQueryOperationDeliveryDenial> {
        use WorthQueryOperationDeliveryDenial as Denial;
        if !std::ptr::eq(self.runtime.runtime(), request.application) || self.description.branch != request.branch { return Err(Denial::FreshRequestMismatch); }
        request.application.with_application_advancement(request.scope, |phase| {
            let observation = self.current_observation(request, &phase)?;
            let retained = request.at(&observation);
            let query = retained.query(Delivery::<Schema, Intent, Action>::read_input(self.description.intent.input()));
            let query = match policy.query_limits { Some(results) => query.limit_results(results), None => query };
            query.execute_in_advancement(&phase).map_err(Denial::Query)
        }).map_err(Denial::Execution)?
    }
    pub fn read_historical(&self, request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>) -> Result<DeliveredProjection<Schema, Read<Schema, Intent, Action>>, WorthQueryOperationDeliveryDenial> {
        use WorthQueryOperationDeliveryDenial as Denial;
        if !std::ptr::eq(self.runtime.runtime(), request.application) || self.description.branch != request.branch { return Err(Denial::FreshRequestMismatch); }
        request.application.with_application_advancement(request.scope, |phase| {
            request.at(self.settlement.observation()).query(Delivery::<Schema, Intent, Action>::read_input(self.description.intent.input())).execute_in_advancement(&phase).map_err(Denial::Query)
        }).map_err(Denial::Execution)?
    }

}
