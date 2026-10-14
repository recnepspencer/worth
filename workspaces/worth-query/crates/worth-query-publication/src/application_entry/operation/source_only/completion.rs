//! Completed and terminal source custody have close, never another execution entrance.
use super::*;

#[must_use = "source completion does not discharge cleanup"]
pub struct WorthQueryCompletedSourceOperation<'r, S, I, P, A, M>
where
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
{
    pub(super) operation: WorthQuerySourceOnlyOperation<'r, S, I, P, A, M>,
    pub(super) evidence:
        WorthQuerySourceCompletion<<I::Binding as ApplicationMutationBinding<S>>::Result>,
}
#[must_use = "retain terminal evidence and close its remaining cleanup"]
/// Native terminal custody cannot be advanced.
/// ```compile_fail,E0599
/// use worth_query_publication::facade::application_entry::*;
/// use worth_query_declaration::facade::{application_schema::ApplicationSchema, application_operation::ApplicationMutationIntent, application_program::ApplicationProgramDefinition};
/// fn cannot_advance<S: ApplicationSchema + 'static, I: ApplicationMutationIntent<S>, P: ApplicationProgramDefinition<S>, A, M>(terminal: WorthQueryTerminalSourceOperation<'_, S, I, P, A, M>, request: &WorthQueryApplicationRequest<'_, '_, '_, S>) {
///     terminal.advance(request);
/// }
/// ```
/// Terminal observation remains a lawful counterpart.
/// ```
/// use worth_query_publication::facade::application_entry::*;
/// use worth_query_declaration::facade::{application_schema::ApplicationSchema, application_operation::ApplicationMutationIntent, application_program::ApplicationProgramDefinition};
/// fn observe<S: ApplicationSchema + 'static, I: ApplicationMutationIntent<S>, P: ApplicationProgramDefinition<S>, A, M>(terminal: WorthQueryTerminalSourceOperation<'_, S, I, P, A, M>) {
///     let _ = terminal.native_outcome();
/// }
/// ```
pub struct WorthQueryTerminalSourceOperation<'r, S, I, P, A, M>
where
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
{
    pub(super) operation: WorthQuerySourceOnlyOperation<'r, S, I, P, A, M>,
}
impl<'r, S, I, P, A, M> WorthQueryCompletedSourceOperation<'r, S, I, P, A, M>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
{
    pub fn evidence(
        &self,
    ) -> &WorthQuerySourceCompletion<<I::Binding as ApplicationMutationBinding<S>>::Result> {
        &self.evidence
    }
    pub fn close(
        mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryClosedOperation, (Self, WorthQueryOperationCloseDenial)> {
        match self.operation.close_cleanup(request) {
            Ok(closed) => Ok(closed),
            Err(denial) => Err((self, denial)),
        }
    }
}
impl<'r, S, I, P, A, M> WorthQueryTerminalSourceOperation<'r, S, I, P, A, M>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
{
    pub fn native_outcome(&self) -> Option<&NativeOutcome<S, I>> {
        match &self.operation.state {
            SourceState::Stopped(outcome) => Some(outcome),
            _ => None,
        }
    }
    pub fn native_keyed_resolution(
        &self,
    ) -> Option<
        &worth_query_execution::facade::primary_graph::WorthQueryApplicationIdempotencyResolution,
    > {
        self.operation.keyed_terminal.as_ref()
    }
    pub fn native_advancement_failure(
        &self,
    ) -> Option<&worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial>
    {
        match &self.operation.state {
            SourceState::UnresolvedAdvancement(failure) => Some(failure),
            _ => None,
        }
    }
    /// Retained delivery was unavailable; this is receipt evidence, never a World observation.
    pub fn unavailable_retained_delivery(
        &self,
    ) -> Option<&WorthQuerySourceCompletion<<I::Binding as ApplicationMutationBinding<S>>::Result>>
    {
        match &self.operation.state {
            SourceState::RetainedUnavailable(evidence) => Some(evidence),
            _ => None,
        }
    }
    pub fn close(
        mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryClosedOperation, (Self, WorthQueryOperationCloseDenial)> {
        if !matches!(
            self.operation.state,
            SourceState::RetainedUnavailable(_)
                | SourceState::Stopped(
                    WorthQueryApplicationMutationOutcome::DomainDenied(_)
                        | WorthQueryApplicationMutationOutcome::Cancelled
                        | WorthQueryApplicationMutationOutcome::DeadlineExceeded
                        | WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift
                        | WorthQueryApplicationMutationOutcome::PreviouslyCommitted(_)
                        | WorthQueryApplicationMutationOutcome::Commit(
                            WorthQueryApplicationUncommitted::Denied(_)
                                | WorthQueryApplicationUncommitted::Cancelled
                                | WorthQueryApplicationUncommitted::TimedOut
                                | WorthQueryApplicationUncommitted::Stale(_)
                        )
                )
        ) {
            return Err((
                self,
                WorthQueryOperationCloseDenial::NativeObligationRetained,
            ));
        }
        match self.operation.close_cleanup(request) {
            Ok(closed) => Ok(closed),
            Err(denial) => Err((self, denial)),
        }
    }
}
impl<'r, S, I, P, A, M> WorthQuerySourceOnlyOperation<'r, S, I, P, A, M>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
{
    fn close_cleanup(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryClosedOperation, WorthQueryOperationCloseDenial> {
        let runtime = self.admission.runtime.runtime();
        if !std::ptr::eq(runtime, request.application)
            || self.admission.description.branch != request.branch
        {
            return Err(WorthQueryOperationCloseDenial::FreshRequestMismatch);
        }
        runtime
            .with_application_advancement(request.scope, |phase| {
                self.cleanup
                    .close_in_advancement(&phase, runtime)
                    .map(|receipt| {
                        WorthQueryClosedOperation::from_cleanup(receipt.retired_component_count())
                    })
            })
            .map_err(WorthQueryOperationCloseDenial::Execution)?
    }
}
