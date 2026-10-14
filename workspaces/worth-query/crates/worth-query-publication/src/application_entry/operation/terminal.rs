//! Terminal native custody has no execution or delivery entrance.
use super::fixed::FixedState;
use super::*;

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
    operation: WorthQueryFixedOperation<'runtime, Schema, Intent, Program, Root, Action>,
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
        operation: WorthQueryFixedOperation<'runtime, Schema, Intent, Program, Root, Action>,
    ) -> Self {
        Self { operation }
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

    /// A native unresolved obligation is retained, never declared discharged.
    pub fn close(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<WorthQueryClosedOperation, (Self, WorthQueryOperationCloseDenial)> {
        if !std::ptr::eq(self.operation.runtime.runtime(), request.application)
            || self.operation.description.branch != request.branch
        {
            return Err((self, WorthQueryOperationCloseDenial::FreshRequestMismatch));
        }
        let no_effect = matches!(
            &self.operation.state,
            FixedState::Initial
                | FixedState::NotPerformed(
                    WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift
                        | WorthQueryApplicationMutationOutcome::DomainDenied(_)
                        | WorthQueryApplicationMutationOutcome::Cancelled
                        | WorthQueryApplicationMutationOutcome::DeadlineExceeded
                )
        );
        let result = self
            .operation
            .runtime
            .runtime()
            .with_application_advancement(request.scope, |_phase| {
                if no_effect {
                    Ok(WorthQueryClosedOperation::without_native_obligations())
                } else {
                    Err(WorthQueryOperationCloseDenial::NativeObligationRetained)
                }
            });
        match result {
            Ok(Ok(closed)) => Ok(closed),
            Ok(Err(denial)) => Err((self, denial)),
            Err(denial) => Err((self, WorthQueryOperationCloseDenial::Execution(denial))),
        }
    }
}
