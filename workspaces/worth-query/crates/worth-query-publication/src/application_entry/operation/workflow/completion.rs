//! Workflow effect evidence retains cleanup independently of instance progression.
use super::*;
use crate::application_entry::WorkflowProgressOutcome;
use worth_query_execution::facade::primary_graph::WorthQueryRecoveryHandle;
pub(super) enum WorkflowDispatchRecovery {
    Unopened,
    Retained(WorthQueryRecoveryHandle),
    Released,
    Disposed(worth_query_execution::facade::primary_graph::WorthQueryRecoveryDisposalReceipt),
    Consumed,
    Terminal(worth_query_execution::facade::primary_graph::WorthQueryRecoveryHandleDenial),
}
/// Retains completed source evidence and the workflow's original requirement.
///
/// A local guarded effect may already have settled its operation node atomically.
/// Completion does not by itself advance the successor or prove workflow
/// terminality, and dispatch recovery and managed cleanup remain separate
/// obligations. `close` returns this owner intact when that custody cannot
/// lawfully be released.
#[must_use = "effect completion is not workflow terminality or native cleanup"]
pub struct WorthQueryCompletedWorkflowOperation<S, I, P, A>
where
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
{
    pub(in crate::application_entry) required: RequiredWorkflowOperation,
    pub(super) recovery: WorkflowDispatchRecovery,
    pub(in crate::application_entry) recovery_result:
        Option<worth_query_execution::facade::primary_graph::WorthQueryRecoverySafeRetryAdmission>,
    // Native recovery/result carriers drop before the source's runtime keepalive.
    pub(in crate::application_entry) source:
        WorthQueryCompletedSourceOperation<S, I, P, A, ApplicationSourceCompletion>,
}
/// Retains a terminal workflow operation's native denial and remaining custody.
///
/// Inspect source, abort or recovery evidence without treating terminality as
/// effect success. `close` uses a fresh request and preserves this owner if its
/// actual native obligations cannot yet be released.
#[must_use = "retain native terminal evidence and close its actual cleanup"]
pub struct WorthQueryTerminalWorkflowOperation<S, I, P, A>
where
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
{
    pub(super) state: TerminalWorkflowState<S, I, P, A>,
}
pub(super) enum TerminalWorkflowState<S, I, P, A>
where
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
{
    Recovery {
        denial: worth_query_execution::facade::primary_graph::WorthQueryRecoveryHandleDenial,
        operation: WorthQueryWorkflowOperation<S, I, P, A>,
    },
    Native {
        source: WorthQueryTerminalSourceOperation<S, I, P, A, ApplicationSourceCompletion>,
    },
    Spent {
        denial: WorthQuerySourceOnlyDenial,
        operation: WorthQueryWorkflowOperation<S, I, P, A>,
    },
}
/// Returns native workflow progress together with the continuing effect owner.
///
/// Keep `operation` after inspecting `progress`: accepting workflow evidence
/// does not independently release dispatch recovery or managed cleanup.
#[must_use = "native workflow progression does not discharge effect cleanup"]
pub struct WorthQueryWorkflowAcceptance<O> {
    pub progress: WorkflowProgressOutcome,
    pub operation: O,
}
impl<S, I, P, A> WorthQueryCompletedWorkflowOperation<S, I, P, A>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
{
    pub fn required(&self) -> &RequiredWorkflowOperation {
        &self.required
    }
    pub fn dispatch_disposal_receipt(
        &self,
    ) -> Option<&worth_query_execution::facade::primary_graph::WorthQueryRecoveryDisposalReceipt>
    {
        match &self.recovery {
            WorkflowDispatchRecovery::Disposed(receipt) => Some(receipt),
            _ => None,
        }
    }
    pub fn evidence(
        &self,
    ) -> &WorthQuerySourceCompletion<<I::Binding as ApplicationMutationBinding<S>>::Result> {
        self.source.evidence()
    }
    pub fn close(
        mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryClosedOperation, (Self, WorthQueryOperationCloseDenial)> {
        if !matches!(
            self.recovery,
            WorkflowDispatchRecovery::Unopened
                | WorkflowDispatchRecovery::Released
                | WorkflowDispatchRecovery::Disposed(_)
        ) {
            return Err((
                self,
                WorthQueryOperationCloseDenial::NativeObligationRetained,
            ));
        }
        self.source
            .operation
            .close_cleanup(request)
            .map_err(|denial| (self, denial))
    }
}
impl<S, I, P, A> WorthQueryTerminalWorkflowOperation<S, I, P, A>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
{
    pub fn close(
        mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryClosedOperation, (Self, WorthQueryOperationCloseDenial)> {
        let outcome = match &mut self.state {
            TerminalWorkflowState::Recovery { operation, .. }
            | TerminalWorkflowState::Spent { operation, .. } => operation.close_in_place(request),
            TerminalWorkflowState::Native { source, .. } => source.close_in_place(request),
        };
        outcome.map_err(|denial| (self, denial))
    }
    pub fn original_abort(
        &self,
    ) -> Option<&worth_query_execution::facade::primary_graph::WorthQueryApplicationUncommitted>
    {
        match &self.state {
            TerminalWorkflowState::Spent { operation, .. }
            | TerminalWorkflowState::Recovery { operation, .. } => operation.source.guarded_abort(),
            TerminalWorkflowState::Native { source, .. } => source.operation.guarded_abort(),
        }
    }
    pub fn denial(&self) -> Option<&WorthQuerySourceOnlyDenial> {
        match &self.state {
            TerminalWorkflowState::Spent { denial, .. } => Some(denial),
            _ => None,
        }
    }
    pub fn effect_evidence(
        &self,
    ) -> Option<&WorthQuerySourceCompletion<<I::Binding as ApplicationMutationBinding<S>>::Result>>
    {
        match &self.state {
            TerminalWorkflowState::Recovery { operation, .. } => operation
                .source_completed
                .then(|| operation.source.completion()),
            _ => None,
        }
    }
    pub fn recovery_denial(
        &self,
    ) -> Option<&worth_query_execution::facade::primary_graph::WorthQueryRecoveryHandleDenial> {
        match &self.state {
            TerminalWorkflowState::Recovery { denial, .. } => Some(denial),
            _ => None,
        }
    }
    pub fn native_terminal(
        &self,
    ) -> Option<&WorthQueryTerminalSourceOperation<S, I, P, A, ApplicationSourceCompletion>> {
        match &self.state {
            TerminalWorkflowState::Native { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl<S, I, P, A> WorthQueryCompletedWorkflowOperation<S, I, P, A>
where
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
{
    pub(in crate::application_entry) fn binding_view<'a>(
        &'a self,
        request: &'a WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> crate::application_entry::WorthQueryBoundWorkflowOperation<
        'a,
        'a,
        'a,
        'a,
        S,
        &'a I,
        crate::application_entry::WorthQueryMutationSourcePrepared,
    > {
        let admission = &self.source.operation.admission;
        let mut native = admission
            .description
            .request_view(request, &admission.installed_action);
        native.workflow_transition_identity = Some(*self.required.transition_identity_bytes());
        crate::application_entry::WorthQueryBoundWorkflowOperation::from_owned_view(native)
    }
}

impl<
        S: ApplicationSchema + 'static,
        I: ApplicationMutationIntent<S>,
        P: ApplicationProgramDefinition<S>,
        A,
    > WorthQueryCompletedWorkflowOperation<S, I, P, A>
{
    pub fn managed_cleanup_failure(
        &self,
    ) -> Option<&worth_query_execution::facade::primary_graph::WorthQueryMutationCleanupFailure>
    {
        self.source.operation.managed_cleanup_failure()
    }
    pub fn release_cleanup(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryOperationCleanupReceipt, WorthQueryOperationCloseDenial> {
        self.source.operation.release_cleanup(request)
    }
}
