//! Native cleanup has closed; original complete Fixed delivery meaning remains owned.
use super::*;
mod batch;
mod query;
mod unprojected;
/// Actual original Fixed settlement after all native cleanup closed.
/// Fresh declared delivery can still refuse without losing this readonly stage.
/// Drop it and returned observations promptly before whole-runtime close/OS join.
#[must_use]
pub struct WorthQueryClosedFixedDelivery<Schema, Intent, Program, Root, Action>
where
    Schema: ApplicationSchema,
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
    pub(super) closed: WorthQueryClosedOperation,
    // Keep runtime last in the entire owning destruction order.
    pub(super) delivery: FixedDeliveryCore<Schema, Intent, Program, Root, Action>,
}
impl<Schema, Intent, Program, Root, Action>
    WorthQueryClosedFixedDelivery<Schema, Intent, Program, Root, Action>
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
    pub fn branch(&self) -> worth_query_execution::facade::product::WorthQueryProductBranch {
        self.delivery.description.branch
    }
    pub fn settlement(
        &self,
    ) -> &WorthQueryApplicationProgramOutputSettlement<RootQuery<Schema, Root>> {
        &self.delivery.settlement
    }
    pub fn close_evidence(&self) -> &WorthQueryClosedOperation {
        &self.closed
    }
}
