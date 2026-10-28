//! Closing discharges native cleanup before transferring original command evidence.
use super::super::operation::native_publication::native_terminal_can_close;
use super::*;

/// A completed native command still owns independent cleanup obligations.
#[must_use = "close the completed command before transferring its evidence"]
pub struct WorthQueryCompletedCapabilityCommand<
    'r,
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    D,
> {
    pub(super) command: WorthQueryCapabilityCommand<'r, S, P, D>,
    pub(super) evidence: WorthQueryCapabilityCommandCompletion,
}
/// Terminal command custody grants no execution or ordinary retry.
#[must_use = "retain terminal native evidence until lawful close"]
pub struct WorthQueryTerminalCapabilityCommand<
    'r,
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    D,
> {
    pub(super) command: WorthQueryCapabilityCommand<'r, S, P, D>,
}
impl<'r, S: ApplicationSchema + 'static, P: ApplicationProgramDefinition<S>, D>
    WorthQueryCapabilityCommand<'r, S, P, D>
{
    pub(super) fn matches(&self, request: &WorthQueryApplicationRequest<'_, '_, '_, S>) -> bool {
        std::ptr::eq(self.runtime.runtime(), request.application) && self.branch == request.branch
    }
    /// Releases only this collection, without repeating effects or resolving business custody.
    pub fn release_cleanup(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryOperationCleanupReceipt, WorthQueryCapabilityCommandDenial<D>> {
        if !self.matches(request) {
            return Err(WorthQueryCapabilityCommandDenial::FreshRequestMismatch);
        }
        let keepalive = self.runtime.clone();
        keepalive
            .runtime()
            .with_application_advancement(request.scope, |phase| {
                self.backend
                    .validate(&phase, request)
                    .map_err(WorthQueryCapabilityCommandDenial::Native)?;
                self.cleanup
                    .close_in_advancement(&phase, keepalive.runtime())
                    .map_err(WorthQueryCapabilityCommandDenial::Cleanup)
            })
            .map_err(WorthQueryCapabilityCommandDenial::Execution)?
    }
    /// Initial or proven no-effect owners may close; ambiguous and performed custody cannot.
    pub fn close(
        mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryClosedOperation, (Self, WorthQueryCapabilityCommandDenial<D>)> {
        if !matches!(
            self.state,
            NativePublicationState::Initial | NativePublicationState::RetryableNoEffect(_)
        ) {
            return Err((
                self,
                WorthQueryCapabilityCommandDenial::Cleanup(
                    WorthQueryOperationCloseDenial::NativeObligationRetained,
                ),
            ));
        }
        match self.release_cleanup(request) {
            Ok(receipt) => Ok(WorthQueryClosedOperation::from_cleanup(receipt)),
            Err(cause) => Err((self, cause)),
        }
    }
    /// Borrowed native failure; no failure observation authorizes execution.
    pub fn managed_cleanup_failure(
        &self,
    ) -> Option<&worth_query_execution::facade::primary_graph::WorthQueryMutationCleanupFailure>
    {
        self.cleanup.managed_failure()
    }
}
impl<'r, S: ApplicationSchema + 'static, P: ApplicationProgramDefinition<S>, D>
    WorthQueryCompletedCapabilityCommand<'r, S, P, D>
{
    pub fn evidence(&self) -> &WorthQueryCapabilityCommandCompletion {
        &self.evidence
    }
    pub fn release_cleanup(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryOperationCleanupReceipt, WorthQueryCapabilityCommandDenial<D>> {
        self.command.release_cleanup(request)
    }
    pub fn close(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryClosedOperation, (Self, WorthQueryCapabilityCommandDenial<D>)> {
        self.close_with_evidence(request).map(|(closed, _)| closed)
    }
    pub fn close_with_evidence(
        mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        (
            WorthQueryClosedOperation,
            WorthQueryCapabilityCommandCompletion,
        ),
        (Self, WorthQueryCapabilityCommandDenial<D>),
    > {
        match self.command.release_cleanup(request) {
            Ok(receipt) => Ok((
                WorthQueryClosedOperation::from_cleanup(receipt),
                self.evidence,
            )),
            Err(cause) => Err((self, cause)),
        }
    }
}
impl<'r, S: ApplicationSchema + 'static, P: ApplicationProgramDefinition<S>, D>
    WorthQueryTerminalCapabilityCommand<'r, S, P, D>
{
    pub fn native_outcome(&self) -> Option<&WorthQueryApplicationUncommitted> {
        match &self.command.state {
            NativePublicationState::Stopped(outcome) => Some(outcome),
            _ => None,
        }
    }
    pub fn idempotency_resolution(&self) -> Option<&WorthQueryApplicationIdempotencyResolution> {
        self.command.keyed_terminal.as_ref()
    }
    pub fn release_cleanup(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryOperationCleanupReceipt, WorthQueryCapabilityCommandDenial<D>> {
        self.command.release_cleanup(request)
    }
    pub fn close(
        mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryClosedOperation, (Self, WorthQueryCapabilityCommandDenial<D>)> {
        // Key drift is not proof that original ambiguous/performed custody disappeared.
        let can_close = matches!(&self.command.state,
            NativePublicationState::Stopped(posture) if native_terminal_can_close(posture));
        if !can_close {
            return Err((
                self,
                WorthQueryCapabilityCommandDenial::Cleanup(
                    WorthQueryOperationCloseDenial::NativeObligationRetained,
                ),
            ));
        }
        match self.command.release_cleanup(request) {
            Ok(receipt) => Ok(WorthQueryClosedOperation::from_cleanup(receipt)),
            Err(cause) => Err((self, cause)),
        }
    }
}
