//! Fresh declared scalar delivery delegates the original native core.
use super::*;
impl<Schema, Intent, Program, Root, Action> WorthQueryClosedFixedDelivery<Schema, Intent, Program, Root, Action>
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
        self.delivery.read_current(request)
    }

    /// Attempts delivery under fresh authority and narrower finite native ceilings.
    /// Refusal retains this already-closed readonly owner.
    pub fn read_current_with_policy(&self, request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>, policy: WorthQueryOperationDeliveryPolicy) -> Result<DeliveredProjection<Schema, Read<Schema, Intent, Action>>, WorthQueryOperationDeliveryDenial> {
        self.delivery.read_current_with_policy(request, policy)
    }


}
