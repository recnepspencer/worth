//! Completed and terminal source custody have close, never another execution entrance.
use super::*;

#[must_use = "source completion does not discharge cleanup"]
/// Completed source evidence with retained native cleanup custody; close may refuse and return this owner.
pub struct WorthQueryCompletedSourceOperation<S, I, P, A, M>
where
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
{
    pub(in crate::application_entry::operation) evidence:
        WorthQuerySourceCompletion<<I::Binding as ApplicationMutationBinding<S>>::Result>,
    // Evidence carriers release before the operation's authentic runtime keepalive.
    pub(in crate::application_entry) operation: WorthQuerySourceOnlyOperation<S, I, P, A, M>,
}
#[must_use = "retain terminal evidence and close its remaining cleanup"]
/// Native terminal custody cannot be advanced.
/// ```compile_fail,E0599
/// use worth_query_publication::facade::application_entry::*;
/// use worth_query_declaration::facade::{application_schema::ApplicationSchema, application_operation::ApplicationMutationIntent, application_program::ApplicationProgramDefinition};
/// fn cannot_advance<S: ApplicationSchema + 'static, I: ApplicationMutationIntent<S>, P: ApplicationProgramDefinition<S>, A, M>(terminal: WorthQueryTerminalSourceOperation<S, I, P, A, M>, request: &WorthQueryApplicationRequest<'_, '_, '_, S>) {
///     terminal.advance(request);
/// }
/// ```
/// Terminal observation remains a lawful counterpart.
/// ```
/// use worth_query_publication::facade::application_entry::*;
/// use worth_query_declaration::facade::{application_schema::ApplicationSchema, application_operation::ApplicationMutationIntent, application_program::ApplicationProgramDefinition};
/// fn observe<S: ApplicationSchema + 'static, I: ApplicationMutationIntent<S>, P: ApplicationProgramDefinition<S>, A, M>(terminal: WorthQueryTerminalSourceOperation<S, I, P, A, M>) {
///     let _ = terminal.native_outcome();
/// }
/// ```
pub struct WorthQueryTerminalSourceOperation<S, I, P, A, M>
where
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
{
    pub(in crate::application_entry) operation: WorthQuerySourceOnlyOperation<S, I, P, A, M>,
}
impl<S, I, P, A, M> WorthQueryCompletedSourceOperation<S, I, P, A, M>
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
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryClosedOperation, (Self, WorthQueryOperationCloseDenial)> {
        self.close_with_evidence(request).map(|(closed, _)| closed)
    }

    /// Releases native cleanup before transferring the original completion evidence.
    /// Refusal returns this owner with its receipt and performed-change witness intact.
    /// The borrowed evidence accessor cannot transfer a performed witness.
    /// ```compile_fail,E0596
    /// use worth_query_publication::facade::application_entry::*;
    /// use worth_query_declaration::facade::{application_schema::ApplicationSchema,application_operation::ApplicationMutationIntent,application_program::ApplicationProgramDefinition};
    /// fn borrowed<S:ApplicationSchema+'static,I:ApplicationMutationIntent<S>,P:ApplicationProgramDefinition<S>,A,M>(done:WorthQueryCompletedSourceOperation<S,I,P,A,M>) {
    ///     if let WorthQuerySourceCompletion::Fresh { receipt, .. }=done.evidence() { let _=receipt.take_performed_relational_product_change(); }
    /// }
    /// ```
    /// Lawful close supplies the actual owned counterpart.
    /// ```
    /// use worth_query_publication::facade::application_entry::*;
    /// use worth_query_declaration::facade::{application_schema::ApplicationSchema,application_operation::ApplicationMutationIntent,application_program::ApplicationProgramDefinition};
    /// fn transfer<S:ApplicationSchema+'static,I:ApplicationMutationIntent<S>,P:ApplicationProgramDefinition<S>,A,M>(done:WorthQueryCompletedSourceOperation<S,I,P,A,M>,request:&WorthQueryApplicationRequest<'_,'_,'_,S>) {
    ///     if let Ok((_closed,WorthQuerySourceCompletion::Fresh { mut receipt, .. }))=done.close_with_evidence(request) { let _=receipt.take_performed_relational_product_change(); }
    /// }
    /// ```
    pub fn close_with_evidence(
        mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        (
            WorthQueryClosedOperation,
            WorthQuerySourceCompletion<<I::Binding as ApplicationMutationBinding<S>>::Result>,
        ),
        (Self, WorthQueryOperationCloseDenial),
    > {
        match self.operation.close_cleanup(request) {
            Ok(closed) => Ok((closed, self.evidence)),
            Err(denial) => Err((self, denial)),
        }
    }
}
impl<S, I, P, A, M> WorthQueryTerminalSourceOperation<S, I, P, A, M>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
{
    /// Exact managed-run cleanup custody retained independently of business evidence.
    pub fn managed_cleanup_failure(
        &self,
    ) -> Option<&worth_query_execution::facade::primary_graph::WorthQueryMutationCleanupFailure>
    {
        self.operation.cleanup.managed_failure()
    }
    /// Cleanup discharge retains this terminal and does not settle its business outcome.
    pub fn release_cleanup(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryOperationCleanupReceipt, WorthQueryOperationCloseDenial> {
        self.operation.release_cleanup(request)
    }
    pub fn native_outcome(&self) -> Option<&NativeOutcome<S, I>> {
        match &self.operation.state {
            SourceState::Stopped(outcome) => Some(outcome),
            _ => None,
        }
    }
    /// Exact native preparation refusal retained without a manufactured commit outcome.
    pub fn native_admission_denial(&self) -> Option<&WorthQueryApplicationRequestMutationDenial> {
        match &self.operation.state {
            SourceState::AdmissionStopped(denial) => Some(denial),
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
    pub fn close(
        mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryClosedOperation, (Self, WorthQueryOperationCloseDenial)> {
        if !native_posture::can_close(&self.operation.state) {
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
impl<S, I, P, A, M> WorthQuerySourceOnlyOperation<S, I, P, A, M>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
{
    /// Closes unexecuted or proven retryable no-effect custody; native recovery obligations remain owned.
    pub fn close(
        mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryClosedOperation, (Self, WorthQueryOperationCloseDenial)> {
        if !matches!(
            self.state,
            SourceState::Initial | SourceState::RetryableNoEffect(_)
        ) || !native_posture::can_close(&self.state)
        {
            return Err((
                self,
                WorthQueryOperationCloseDenial::NativeObligationRetained,
            ));
        }
        match self.close_cleanup(request) {
            Ok(closed) => Ok(closed),
            Err(denial) => Err((self, denial)),
        }
    }

    fn close_cleanup(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryClosedOperation, WorthQueryOperationCloseDenial> {
        self.release_cleanup(request)
            .map(WorthQueryClosedOperation::from_cleanup)
    }

    /// Discharges this cleanup collection only; original business evidence stays owned.
    pub fn release_cleanup(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryOperationCleanupReceipt, WorthQueryOperationCloseDenial> {
        let runtime = self.admission.runtime.runtime();
        if !std::ptr::eq(runtime, request.application)
            || self.admission.description.branch != request.branch
        {
            return Err(WorthQueryOperationCloseDenial::FreshRequestMismatch);
        }
        runtime
            .with_application_advancement(request.scope, |phase| {
                self.cleanup.close_in_advancement(&phase, runtime)
            })
            .map_err(WorthQueryOperationCloseDenial::Execution)?
    }
}
