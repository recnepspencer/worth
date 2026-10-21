//! No-projection delivery validates native settlement without query authority.
use super::*;
impl<Schema, Intent, Program, Root, Action> FixedDeliveryCore<Schema, Intent, Program, Root, Action>
where
    Schema: ApplicationSchema + 'static,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema> + ApplicationRequiredOutputRoot,
    Connection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Binding<Schema, Intent>:
        WorthQueryApplicationRequiredOutputSource<Schema, Connection<Schema, Root>>,
    Action: ApplicationAction<Schema, Binding<Schema, Intent>>,
    Delivery<Schema, Intent, Action>: ApplicationFixedActionDelivery<
        Schema,
        Binding<Schema, Intent>,
        Root = Root,
        Projection = worth_query_declaration::facade::application_program::ApplicationNoProjection,
    >,
{
    pub fn deliver_current(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<WorthQueryApplicationReadObservation, WorthQueryOperationDeliveryDenial> {
        if !self.matches(request) {
            return Err(WorthQueryOperationDeliveryDenial::FreshRequestMismatch);
        }
        request
            .application
            .with_application_advancement(request.scope, |phase| {
                self.current_observation(request, &phase)
            })
            .map_err(WorthQueryOperationDeliveryDenial::Execution)?
    }
    pub fn deliver_current_at(
        &self,
        retained: &WorthQueryApplicationRetainedRequest<'_, '_, '_, Schema>,
    ) -> Result<(), WorthQueryOperationDeliveryDenial> {
        let request = retained.base_request();
        if !self.matches(&request) {
            return Err(WorthQueryOperationDeliveryDenial::FreshRequestMismatch);
        }
        request
            .application
            .with_application_advancement(request.scope, |phase| {
                self.require_current_at(retained, &phase)
            })
            .map_err(WorthQueryOperationDeliveryDenial::Execution)?
    }
}
