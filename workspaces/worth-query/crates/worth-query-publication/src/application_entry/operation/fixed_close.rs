//! Live fixed cleanup release and unexecuted Initial close use the native collection.
use super::*;
impl<Schema, Intent, Program, Root, Action>
    WorthQueryFixedOperation<Schema, Intent, Program, Root, Action>
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
    /// Abandons only an unexecuted description; pending native work stays owned.
    pub fn close(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<WorthQueryClosedOperation, (Self, WorthQueryOperationCloseDenial)> {
        if !std::ptr::eq(self.runtime.runtime(), request.application)
            || self.description.branch != request.branch
        {
            return Err((self, WorthQueryOperationCloseDenial::FreshRequestMismatch));
        }
        if !matches!(&self.state, super::fixed::FixedState::Initial) {
            return Err((
                self,
                WorthQueryOperationCloseDenial::NativeObligationRetained,
            ));
        }
        WorthQueryTerminalFixedOperation::new(self)
            .close(request)
            .map_err(|(terminal, denial)| (terminal.operation, denial))
    }
    /// Releases only the absorbed cleanup collection, without source/output progress.
    pub fn release_cleanup(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<WorthQueryOperationCleanupReceipt, WorthQueryOperationCloseDenial> {
        let runtime = self.runtime.runtime();
        if !std::ptr::eq(runtime, request.application) || self.description.branch != request.branch
        {
            return Err(WorthQueryOperationCloseDenial::FreshRequestMismatch);
        }
        request
            .application
            .with_application_advancement(request.scope, |phase| {
                self.cleanup
                    .close_in_advancement(&phase, request.application)
            })
            .map_err(WorthQueryOperationCloseDenial::Execution)?
    }
}
