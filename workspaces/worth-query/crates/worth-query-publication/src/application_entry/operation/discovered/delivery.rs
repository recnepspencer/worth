//! Original declared scalar selector runs at the guarded observation, or at authentic retained history.
use super::*;
use worth_query_declaration::facade::application_program::ApplicationDiscoveredQueryDelivery;
impl<Schema, Intent, Program, Root, Action> WorthQuerySettledDiscoveredOperation<Schema, Intent, Program, Root, Action>
where
    Schema: ApplicationSchema + 'static,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema> + ApplicationDiscoveredOutputRoot,
    Connection<Schema, Root>: WorthQueryApplicationDiscoveredOutputConnection<Schema, Source=Intent::Binding>,
    Action: ApplicationAction<Schema, Binding<Schema, Intent>>,
    ActionDelivery<Schema, Intent, Action>: ApplicationDiscoveredQueryDelivery<Schema, Binding<Schema, Intent>, Root = Root>,
    <ProjectedRead<Schema, Intent, Action> as ApplicationQueryBinding<Schema>>::Input: ApplicationQueryIntent<Schema, Binding = ProjectedRead<Schema, Intent, Action>>,
    <ProjectedRead<Schema, Intent, Action> as ApplicationQueryBinding<Schema>>::ScopeBinding: ApplicationQueryScopeResolution<Schema, <ProjectedRead<Schema, Intent, Action> as ApplicationQueryBinding<Schema>>::PrincipalIdentity>,
    <<ProjectedRead<Schema, Intent, Action> as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<Schema, <ProjectedRead<Schema, Intent, Action> as ApplicationQueryBinding<Schema>>::Query> + Clone,
{
    pub fn read_current(&self, request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>) -> Result<DeliveredProjection<Schema, ProjectedRead<Schema, Intent, Action>>, WorthQueryOperationDeliveryDenial> {
        self.read_current_with_policy(request, WorthQueryOperationDeliveryPolicy::default())
    }

    /// Attempts delivery under fresh authority and narrower finite native ceilings.
    /// Refusal leaves this settled owner and its cleanup custody unchanged.
    pub fn read_current_with_policy(&self, request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>, policy: WorthQueryOperationDeliveryPolicy) -> Result<DeliveredProjection<Schema, ProjectedRead<Schema, Intent, Action>>, WorthQueryOperationDeliveryDenial> {
        use WorthQueryOperationDeliveryDenial as Denial;
        if !std::ptr::eq(self.runtime.runtime(), request.application) || self.description.branch != request.branch { return Err(Denial::FreshRequestMismatch); }
        request.application.with_application_advancement(request.scope, |phase| {
            let observation = self.current_observation(request, &phase)?;
            let retained = request.at(&observation);
            let query = retained.query(ActionDelivery::<Schema, Intent, Action>::read_input(self.description.intent.input()));
            let query = match policy.query_limits { Some(results) => query.limit_results(results), None => query };
            query.execute_in_advancement(&phase).map_err(Denial::Query)
        }).map_err(Denial::Execution)?
    }
    /// Reads the original declared projection at its retained settlement basis with fresh authority.
    /// This does not advance outputs or claim that their consumed facts are current.
    pub fn read_historical(&self, request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>) -> Result<DeliveredProjection<Schema, ProjectedRead<Schema, Intent, Action>>, WorthQueryOperationDeliveryDenial> {
        use WorthQueryOperationDeliveryDenial as Denial;
        if !std::ptr::eq(self.runtime.runtime(), request.application) || self.description.branch != request.branch { return Err(Denial::FreshRequestMismatch); }
        request.application.with_application_advancement(request.scope, |phase| {
            request.at(self.settlement.observation()).query(ActionDelivery::<Schema, Intent, Action>::read_input(self.description.intent.input())).execute_in_advancement(&phase).map_err(Denial::Query)
        }).map_err(Denial::Execution)?
    }

}
