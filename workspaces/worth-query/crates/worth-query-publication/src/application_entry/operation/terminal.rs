//! Terminal native custody has no execution or delivery entrance.
use super::fixed::FixedState;
mod unavailable;
use super::*;
mod closed;
pub use closed::WorthQueryClosedFixedTerminal;
use unavailable::TerminalOutputCompletion;
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationCommitOutcome, WorthQueryManagedApplicationRecoveryPerformed,
};

/// Terminal custody preserves native observations but cannot advance.
/// ```compile_fail,E0599
/// use worth_query_publication::facade::application_entry::WorthQueryTerminalFixedOperation;
/// use worth_query_declaration::facade::{application_operation::*, application_program::*, application_schema::ApplicationSchema};
/// use worth_query_execution::facade::{application_installation::WorthQueryProgramApplicationRuntime, primary_graph::{WorthQueryApplicationRequiredOutputConnection, WorthQueryApplicationRequiredOutputSource}};
/// fn cannot_advance<S, I, P, R, A>(terminal: WorthQueryTerminalFixedOperation<S, I, P, R, A>, request: &worth_query_publication::facade::application_entry::WorthQueryApplicationRequest<'_, '_, '_, S>)
/// where
///     S: ApplicationSchema + 'static,
///     I: ApplicationMutationIntent<S>,
///     P: ApplicationProgramDefinition<S>,
///     R: ApplicationOutputGraphShape<S> + ApplicationRequiredOutputRoot,
///     <R::RootConnection as ApplicationConnectionShape<S>>::Binding: WorthQueryApplicationRequiredOutputConnection<S>,
///     I::Binding: WorthQueryApplicationRequiredOutputSource<S, <R::RootConnection as ApplicationConnectionShape<S>>::Binding>,
///     A: ApplicationAction<S, I::Binding>,
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
/// fn observe_terminal<S, I, P, R, A>(terminal: WorthQueryTerminalFixedOperation<S, I, P, R, A>, request: &worth_query_publication::facade::application_entry::WorthQueryApplicationRequest<'_, '_, '_, S>)
/// where
///     S: ApplicationSchema + 'static,
///     I: ApplicationMutationIntent<S>,
///     P: ApplicationProgramDefinition<S>,
///     R: ApplicationOutputGraphShape<S> + ApplicationRequiredOutputRoot,
///     <R::RootConnection as ApplicationConnectionShape<S>>::Binding: WorthQueryApplicationRequiredOutputConnection<S>,
///     I::Binding: WorthQueryApplicationRequiredOutputSource<S, <R::RootConnection as ApplicationConnectionShape<S>>::Binding>,
///     A: ApplicationAction<S, I::Binding>,
///     A::Delivery: ApplicationFixedActionDelivery<S, I::Binding, Root = R>,
/// {
///     let _ = terminal.native_outcome(); // A valid terminal observation remains available.
///     let _ = request;
/// }
/// ```
pub struct WorthQueryTerminalFixedOperation<Schema, Intent, Program, Root, Action>
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
    output_completion: Option<TerminalOutputCompletion<Schema, Program, Root>>,
    pub(super) advance_denial: Option<WorthQueryOperationAdvanceDenial>,
    pub(super) operation: WorthQueryFixedOperation<Schema, Intent, Program, Root, Action>,
}
impl<Schema, Intent, Program, Root, Action>
    WorthQueryTerminalFixedOperation<Schema, Intent, Program, Root, Action>
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
    pub(super) fn new(
        mut operation: WorthQueryFixedOperation<Schema, Intent, Program, Root, Action>,
    ) -> Self {
        // Before promotion, the partial is the sole owner of its prior cleanup.
        if let FixedState::Unpublished(partial) = &mut operation.state {
            operation
                .cleanup
                .retain_original_failures(partial.take_prior_cleanup());
        }
        Self {
            operation,
            output_completion: None,
            advance_denial: None,
        }
    }

    /// Original output-producer evidence is not replaced by its cleanup receipt.
    pub fn producer_native_outcome(&self) -> Option<&WorthQueryApplicationCommitOutcome> {
        self.operation.cleanup.producer_native_outcome()
    }

    /// The original advancement refusal, when transferred into this owner by
    /// an enclosing history edit. Retaining it does not authorize a retry.
    pub fn advance_denial(&self) -> Option<&WorthQueryOperationAdvanceDenial> {
        self.advance_denial.as_ref()
    }
    /// Exact managed-run cleanup custody retained independently of business evidence.
    pub fn managed_cleanup_failures(
        &self,
    ) -> impl ExactSizeIterator<
        Item = &worth_query_execution::facade::primary_graph::WorthQueryMutationCleanupFailure,
    > {
        self.operation.cleanup.managed_failures()
    }
    /// Interrupt this receiver only after actual native cleanup has released the prefix.
    pub fn cancel_after_managed_releases_for_test(
        &mut self,
        count: std::num::NonZeroUsize,
        cancellation:worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource,
    ) -> Result<
        super::cleanup::WorthQueryCleanupReleaseInterruptionForTest,
        super::cleanup::WorthQueryCleanupReleaseRegistrationDenial,
    > {
        self.operation
            .cleanup
            .cancel_after_managed_releases(count, cancellation)
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
        request
            .application
            .with_application_advancement(request.scope, |phase| {
                self.operation
                    .cleanup
                    .close_in_advancement(&phase, request.application)
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
    /// Eligible fixed-leaf domain refusal completes as unavailable through its original
    /// native owner before cleanup. All other unresolved output custody still refuses.
    pub fn close(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<WorthQueryClosedOperation, (Self, WorthQueryOperationCloseDenial)> {
        self.close_with_evidence(request)
            .map(WorthQueryClosedFixedTerminal::into_close_evidence)
    }

    /// Complete the original native close before moving its genuine diagnostic
    /// evidence into a readonly result. Refusal retains this actual owner.
    pub fn close_with_evidence(
        mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<WorthQueryClosedFixedTerminal<Schema, Intent>, (Self, WorthQueryOperationCloseDenial)>
    {
        match self.close_native(request) {
            Ok(closed) => Ok(WorthQueryClosedFixedTerminal::from_terminal(self, closed)),
            Err(denial) => Err((self, denial)),
        }
    }

    fn close_native(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<WorthQueryClosedOperation, WorthQueryOperationCloseDenial> {
        if !std::ptr::eq(self.operation.runtime.runtime(), request.application)
            || self.operation.description.branch != request.branch
        {
            return Err(WorthQueryOperationCloseDenial::FreshRequestMismatch);
        }
        let closable = match &self.operation.state {
            FixedState::Initial => true,
            FixedState::NotPerformed(outcome) => super::close::terminal_outcome_can_close(outcome),
            _ => false,
        };
        let result = request
            .application
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
                let unavailable = self.complete_unavailable_in_phase(&phase, request)?;
                if !closable && !terminal_carrier && !unavailable {
                    return Err(WorthQueryOperationCloseDenial::NativeObligationRetained);
                }
                if let Some(performed) = self.operation.recovered.as_mut() {
                    self.operation.cleanup.absorb_recovered(performed)?;
                }
                let receipt = self
                    .operation
                    .cleanup
                    .close_in_advancement(&phase, request.application)?;
                Ok(WorthQueryClosedOperation::from_cleanup(receipt))
            });
        match result {
            Ok(Ok(closed)) => Ok(closed.with_output_unavailable(self.take_output_unavailable())),
            Ok(Err(denial)) => Err(denial),
            Err(denial) => Err(WorthQueryOperationCloseDenial::Execution(denial)),
        }
    }
}
