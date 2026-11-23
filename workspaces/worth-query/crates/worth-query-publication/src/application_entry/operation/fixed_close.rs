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
        if self.readmitted_recovery.is_some()
            || !matches!(&*self.state, super::fixed::FixedState::Initial)
        {
            return Err((
                self,
                WorthQueryOperationCloseDenial::NativeObligationRetained,
            ));
        }
        WorthQueryTerminalFixedOperation::new(self)
            .close(request)
            .map_err(|(terminal, denial)| (terminal.operation, denial))
    }
    pub fn handoff_for_reopen(
        mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<
        (
            WorthQueryOriginalOperation,
            WorthQueryOperationCleanupReceipt,
        ),
        (Self, WorthQueryOperationCloseDenial),
    > {
        let original = match self.description.original_lookup() {
            Ok(original) => original,
            Err(cause) => return Err((self, WorthQueryOperationCloseDenial::Original(cause))),
        };
        if let Err(cause) = original.admit(request) {
            return Err((self, WorthQueryOperationCloseDenial::Original(cause)));
        }
        if self.readmitted_recovery.is_some()
            || !matches!(
                &*self.state,
                super::fixed::FixedState::Initial
                    | super::fixed::FixedState::Outputs(_)
                    | super::fixed::FixedState::ReplayRetained(_)
                    | super::fixed::FixedState::ReplayRecorded(_)
            )
        {
            return Err((
                self,
                WorthQueryOperationCloseDenial::NativeObligationRetained,
            ));
        }
        match self.release_cleanup(request) {
            Ok(cleanup) => Ok((original, cleanup)),
            Err(cause) => Err((self, cause)),
        }
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
