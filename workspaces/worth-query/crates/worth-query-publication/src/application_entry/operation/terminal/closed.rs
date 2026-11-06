//! Original terminal data after all execution/output/read owners have dropped.
use super::*;

/// Completed native cleanup with original source diagnostics. This owns no
/// operation description, output handle, runtime continuation or effect entrance.
/// ```
/// use worth_query_publication::facade::application_entry::{WorthQueryClosedFixedTerminal, WorthQueryOperationAdvanceDenial};
/// use worth_query_declaration::facade::{application_schema::ApplicationSchema, application_operation::ApplicationMutationIntent};
/// fn reason<S:ApplicationSchema,I:ApplicationMutationIntent<S>>(closed:&WorthQueryClosedFixedTerminal<S,I>)->Option<&WorthQueryOperationAdvanceDenial>{closed.advance_denial()}
/// ```
/// ```compile_fail,E0599
/// use worth_query_publication::facade::application_entry::WorthQueryClosedFixedTerminal;
/// use worth_query_declaration::facade::{application_schema::ApplicationSchema, application_operation::ApplicationMutationIntent};
/// fn resume<S:ApplicationSchema,I:ApplicationMutationIntent<S>>(closed:WorthQueryClosedFixedTerminal<S,I>){closed.advance();}
/// ```
pub struct WorthQueryClosedFixedTerminal<S: ApplicationSchema, I: ApplicationMutationIntent<S>> {
    source_outcome: Option<
        WorthQueryApplicationMutationOutcome<
            <I::Binding as ApplicationMutationBinding<S>>::Denial,
            <I::Binding as ApplicationMutationBinding<S>>::Result,
        >,
    >,
    source_result: Option<<I::Binding as ApplicationMutationBinding<S>>::Result>,
    advance_denial: Option<WorthQueryOperationAdvanceDenial>,
    closed: WorthQueryClosedOperation,
}
impl<S: ApplicationSchema, I: ApplicationMutationIntent<S>> WorthQueryClosedFixedTerminal<S, I> {
    pub fn advance_denial(&self) -> Option<&WorthQueryOperationAdvanceDenial> {
        self.advance_denial.as_ref()
    }
    pub fn source_outcome(
        &self,
    ) -> Option<
        &WorthQueryApplicationMutationOutcome<
            <I::Binding as ApplicationMutationBinding<S>>::Denial,
            <I::Binding as ApplicationMutationBinding<S>>::Result,
        >,
    > {
        self.source_outcome.as_ref()
    }
    /// The original performed source result can exist even when its output was
    /// unavailable. Absence is not a claim that no source effect occurred.
    pub fn source_result(&self) -> Option<&<I::Binding as ApplicationMutationBinding<S>>::Result> {
        self.source_result.as_ref()
    }
    pub fn close_evidence(&self) -> &WorthQueryClosedOperation {
        &self.closed
    }
    pub fn into_close_evidence(self) -> WorthQueryClosedOperation {
        self.closed
    }

    pub(super) fn from_terminal<P, R, A>(
        mut terminal: WorthQueryTerminalFixedOperation<S, I, P, R, A>,
        closed: WorthQueryClosedOperation,
    ) -> Self
    where
        P: ApplicationProgramDefinition<S>,
        R: ApplicationOutputGraphShape<S> + ApplicationRequiredOutputRoot,
        Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
        I::Binding: WorthQueryApplicationRequiredOutputSource<S, Connection<S, R>>,
        A: ApplicationAction<S, I::Binding>,
        A::Delivery: ApplicationFixedActionDelivery<S, I::Binding, Root = R>,
    {
        let source_result = terminal.operation.result.take();
        let advance_denial = terminal.advance_denial.take();
        let source_outcome = match terminal.operation.state {
            FixedState::NotPerformed(outcome) => Some(outcome),
            _ => None,
        };
        // The remaining description, including any history restriction, and
        // Outputs.source_observation drop here before a candidate may retire.
        Self {
            source_outcome,
            source_result,
            advance_denial,
            closed,
        }
    }
}
