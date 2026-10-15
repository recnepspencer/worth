//! Terminal native custody has no execution or delivery entrance.
use super::fixed::FixedState;
use super::*;
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationUncommitted, WorthQueryManagedApplicationRecoveryPerformed,
};

/// Terminal custody preserves native observations but cannot advance.
/// ```compile_fail,E0599
/// use worth_query_publication::facade::application_entry::WorthQueryTerminalFixedOperation;
/// use worth_query_declaration::facade::{application_operation::*, application_program::*, application_schema::ApplicationSchema};
/// use worth_query_execution::facade::{application_installation::WorthQueryProgramApplicationRuntime, primary_graph::{WorthQueryApplicationRequiredOutputConnection, WorthQueryApplicationRequiredOutputSource}};
/// fn cannot_advance<'r, S, I, P, R, A>(terminal: WorthQueryTerminalFixedOperation<'r, S, I, P, R, A>, request: &worth_query_publication::facade::application_entry::WorthQueryApplicationRequest<'_, '_, '_, S>)
/// where
///     S: ApplicationSchema + 'static,
///     I: ApplicationMutationIntent<S>,
///     P: ApplicationProgramDefinition<S>,
///     R: ApplicationOutputGraphShape<S> + ApplicationRequiredOutputRoot,
///     <R::RootConnection as ApplicationConnectionShape<S>>::Binding: WorthQueryApplicationRequiredOutputConnection<S>,
///     I::Binding: WorthQueryApplicationRequiredOutputSource<S, <R::RootConnection as ApplicationConnectionShape<S>>::Binding>,
///     A: ApplicationFixedAction<S, I::Binding>,
///     A::Delivery: ApplicationFixedActionDelivery<S, I::Binding, Root = R>,
/// {
///     let _ = terminal.native_outcome(); // A valid terminal observation remains available.
///     let _ = terminal.advance(request);
/// }
/// ```
/// The same correctly bounded terminal owner has a valid observation entrance.
/// ```
/// use worth_query_publication::facade::application_entry::WorthQueryTerminalFixedOperation;
/// use worth_query_declaration::facade::{application_operation::*, application_program::*, application_schema::ApplicationSchema};
/// use worth_query_execution::facade::{application_installation::WorthQueryProgramApplicationRuntime, primary_graph::{WorthQueryApplicationRequiredOutputConnection, WorthQueryApplicationRequiredOutputSource}};
/// fn observe_terminal<'r, S, I, P, R, A>(terminal: WorthQueryTerminalFixedOperation<'r, S, I, P, R, A>, request: &worth_query_publication::facade::application_entry::WorthQueryApplicationRequest<'_, '_, '_, S>)
/// where
///     S: ApplicationSchema + 'static,
///     I: ApplicationMutationIntent<S>,
///     P: ApplicationProgramDefinition<S>,
///     R: ApplicationOutputGraphShape<S> + ApplicationRequiredOutputRoot,
///     <R::RootConnection as ApplicationConnectionShape<S>>::Binding: WorthQueryApplicationRequiredOutputConnection<S>,
///     I::Binding: WorthQueryApplicationRequiredOutputSource<S, <R::RootConnection as ApplicationConnectionShape<S>>::Binding>,
///     A: ApplicationFixedAction<S, I::Binding>,
///     A::Delivery: ApplicationFixedActionDelivery<S, I::Binding, Root = R>,
/// {
///     let _ = terminal.native_outcome(); // A valid terminal observation remains available.
///     let _ = request;
/// }
/// ```
pub struct WorthQueryTerminalFixedOperation<'runtime, Schema, Intent, Program, Root, Action>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema> + ApplicationRequiredOutputRoot,
    Connection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Binding<Schema, Intent>:
        WorthQueryApplicationRequiredOutputSource<Schema, Connection<Schema, Root>>,
    Action: ApplicationFixedAction<Schema, Binding<Schema, Intent>>,
    Delivery<Schema, Intent, Action>:
        ApplicationFixedActionDelivery<Schema, Binding<Schema, Intent>, Root = Root>,
{
    pub(super) operation: WorthQueryFixedOperation<'runtime, Schema, Intent, Program, Root, Action>,
}
impl<'runtime, Schema, Intent, Program, Root, Action>
    WorthQueryTerminalFixedOperation<'runtime, Schema, Intent, Program, Root, Action>
where
    Schema: ApplicationSchema + 'static,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema> + ApplicationRequiredOutputRoot,
    Connection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Binding<Schema, Intent>:
        WorthQueryApplicationRequiredOutputSource<Schema, Connection<Schema, Root>>,
    Action: ApplicationFixedAction<Schema, Binding<Schema, Intent>>,
    Delivery<Schema, Intent, Action>:
        ApplicationFixedActionDelivery<Schema, Binding<Schema, Intent>, Root = Root>,
{
    pub(super) fn new(
        mut operation: WorthQueryFixedOperation<'runtime, Schema, Intent, Program, Root, Action>,
    ) -> Self {
        // Before promotion, the partial is the sole owner of its prior cleanup.
        if let FixedState::Unpublished(partial) = &mut operation.state {
            operation
                .cleanup
                .retain_original_failures(partial.take_prior_cleanup());
        }
        Self { operation }
    }

    /// Original output-producer evidence is not replaced by its cleanup receipt.
    pub fn producer_native_outcome(&self) -> Option<&WorthQueryApplicationUncommitted> {
        self.operation.cleanup.producer_native_outcome()
    }
    /// Exact managed-run cleanup custody retained independently of business evidence.
    pub fn managed_cleanup_failure(
        &self,
    ) -> Option<&worth_query_execution::facade::primary_graph::WorthQueryMutationCleanupFailure>
    {
        self.operation.cleanup.managed_failure()
    }
    /// Discharge of this collection retains terminal business evidence and required-output obligations.
    pub fn release_cleanup(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<WorthQueryOperationCleanupReceipt, WorthQueryOperationCloseDenial> {
        let runtime = self.operation.runtime.runtime();
        if !std::ptr::eq(runtime, request.application)
            || self.operation.description.branch != request.branch
        {
            return Err(WorthQueryOperationCloseDenial::FreshRequestMismatch);
        }
        runtime
            .with_application_advancement(request.scope, |phase| {
                self.operation.cleanup.close_in_advancement(&phase, runtime)
            })
            .map_err(WorthQueryOperationCloseDenial::Execution)?
    }
    /// Exact native source outcome, including every retained unresolved posture.
    pub fn native_outcome(
        &self,
    ) -> Option<
        &WorthQueryApplicationMutationOutcome<
            <Binding<Schema, Intent> as ApplicationMutationBinding<Schema>>::Denial,
            <Binding<Schema, Intent> as ApplicationMutationBinding<Schema>>::Result,
        >,
    > {
        match &self.operation.state {
            FixedState::NotPerformed(outcome) => Some(outcome),
            FixedState::Blocked(blocked) => Some(blocked.outcome()),
            _ => None,
        }
    }

    /// Native recovered publication evidence, including terminal carrier refusal.
    /// This observation does not certify required-output settlement.
    pub fn recovered_progress(&self) -> Option<&WorthQueryManagedApplicationRecoveryPerformed> {
        match &self.operation.state {
            FixedState::Unpublished(partial) => partial.performed(),
            _ => self.operation.recovered.as_ref(),
        }
    }
    /// A native unresolved obligation is retained, never declared discharged.
    pub fn close(
        mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<WorthQueryClosedOperation, (Self, WorthQueryOperationCloseDenial)> {
        if !std::ptr::eq(self.operation.runtime.runtime(), request.application)
            || self.operation.description.branch != request.branch
        {
            return Err((self, WorthQueryOperationCloseDenial::FreshRequestMismatch));
        }
        let closable = match &self.operation.state {
            FixedState::Initial => true,
            FixedState::NotPerformed(outcome) => super::close::terminal_outcome_can_close(outcome),
            _ => false,
        };
        let result = self
            .operation
            .runtime
            .runtime()
            .with_application_advancement(request.scope, |phase| {
                let terminal_carrier = match &mut self.operation.state {
                    FixedState::Unpublished(partial)
                        if partial.recovery_posture()
                            == WorthQueryRequiredOutputRecoveryPosture::Terminal =>
                    {
                        let Some(performed) = partial.performed_mut() else {
                            return Err(WorthQueryOperationCloseDenial::NativeObligationRetained);
                        };
                        self.operation.cleanup.absorb_recovered(performed)?;
                        true
                    }
                    _ => false,
                };
                if !closable && !terminal_carrier {
                    return Err(WorthQueryOperationCloseDenial::NativeObligationRetained);
                }
                let receipt = self
                    .operation
                    .cleanup
                    .close_in_advancement(&phase, self.operation.runtime.runtime())?;
                Ok(WorthQueryClosedOperation::from_cleanup(receipt))
            });
        match result {
            Ok(Ok(closed)) => Ok(closed),
            Ok(Err(denial)) => Err((self, denial)),
            Err(denial) => Err((self, WorthQueryOperationCloseDenial::Execution(denial))),
        }
    }
}
