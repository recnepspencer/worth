//! Only the original dispatch handle survives an attempt; admission is always fresh.
use super::*;
use super::{advance::WorkflowStep, completion::WorkflowDispatchRecovery};
use worth_query_declaration::facade::application_operation::ApplicationMutationAdmissionBinding;
use worth_query_execution::facade::primary_graph::WorthQueryRecoveryHandleDenial;
use worth_query_execution::publication_boundary::workflow_advance::{
    WorthQueryGuardedWorkflowOperationCustody, WorthQueryWorkflowAdvanceAdapter,
};
/// Refuses a fresh attempt to inspect or progress the original workflow effect.
///
/// The operation retains its business evidence and any dispatch custody it still
/// owns. `NativeHandleConsumed` does not recreate the consumed handle. This
/// diagnostic grants no retry authority and does not prove settlement.
#[derive(Debug)]
pub enum WorthQueryWorkflowEffectRecoveryDenial {
    Admission(crate::application_entry::WorthQueryApplicationRecoveryRequestDenial),
    Inspection(worth_query_execution::facade::primary_graph::WorthQueryApplicationIdempotencyResolutionDenial),
    Recovery(WorthQueryRecoveryHandleDenial),
    Owner(crate::application_entry::WorthQueryWorkflowOperationOwnerPosture),
    NativeHandleConsumed,
}
impl<S, I, P, A> WorthQueryWorkflowOperation<S, I, P, A>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    I::Binding: ApplicationMutationAdmissionBinding<S>,
    <I::Binding as ApplicationMutationAdmissionBinding<S>>::Admission:
        super::super::super::mutation::authorization::WorthQueryBindingAdmission<S, I::Binding>,
    <I::Binding as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync + 'static,
    <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding: ApplicationMutationScopeResolution<
        S,
        <I::Binding as ApplicationMutationBinding<S>>::PrincipalIdentity,
    >,
{
    pub(super) fn advance_dispatch(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        retry: bool,
    ) -> Result<WorkflowStep, WorthQueryWorkflowEffectRecoveryDenial> {
        if let WorkflowDispatchRecovery::Terminal(cause) = &self.recovery {
            return Ok(WorkflowStep::RecoveryTerminal(cause.clone()));
        }
        let source = &self.source.admission;
        source
            .select(phase, request)
            .map_err(|cause| WorthQueryWorkflowEffectRecoveryDenial::Admission(cause.into()))?;
        let mut native = source
            .description
            .request_view(request, &source.installed_action);
        let prepared = source
            .authorize_recovery_view(phase, request, &mut native)
            .map_err(WorthQueryWorkflowEffectRecoveryDenial::Admission)?;
        let custody = WorthQueryWorkflowAdvanceAdapter::resolve_guarded_operation_custody(
            phase,
            source.runtime.runtime(),
            &prepared.admission,
            prepared.idempotency,
            self.required.transition_identity_bytes(),
            None,
        )
        .map_err(WorthQueryWorkflowEffectRecoveryDenial::Inspection)?;
        // A descriptive committed read cannot discharge a retained or interrupted handle.
        if matches!(self.recovery, WorkflowDispatchRecovery::Consumed) {
            return Err(WorthQueryWorkflowEffectRecoveryDenial::NativeHandleConsumed);
        }
        let (receipt, settled) = match custody {
            WorthQueryGuardedWorkflowOperationCustody::Committed(receipt)
            | WorthQueryGuardedWorkflowOperationCustody::ExternallySettled(receipt) => {
                (receipt, true)
            }
            WorthQueryGuardedWorkflowOperationCustody::DispatchPending(receipt) => (receipt, false),
            other => {
                return Err(WorthQueryWorkflowEffectRecoveryDenial::Owner(
                    crate::application_entry::workflow::operation::owner::other_custody(other),
                ))
            }
        };
        if settled {
            if !matches!(self.recovery, WorkflowDispatchRecovery::Retained(_)) {
                return Ok(WorkflowStep::Completed);
            }
            let WorkflowDispatchRecovery::Retained(handle) = &self.recovery else {
                unreachable!()
            };
            let authority=match source.runtime.runtime().admit_recovery_effect_authority(handle,&prepared.admission) {
                Ok(authority)=>authority,
                Err(cause) if cause.kind()==worth_query_execution::facade::primary_graph::WorthQueryRecoveryHandleDenialKind::AlreadyTerminal=>{
                    // Native registry has already ended this exact handle; no completion is inferred.
                    self.recovery=WorkflowDispatchRecovery::Terminal(cause.clone());
                    return Ok(WorkflowStep::RecoveryTerminal(cause));
                },
                Err(cause)=>return Err(WorthQueryWorkflowEffectRecoveryDenial::Recovery(cause)),
            };
            let WorkflowDispatchRecovery::Retained(handle) =
                std::mem::replace(&mut self.recovery, WorkflowDispatchRecovery::Consumed)
            else {
                unreachable!()
            };
            return match worth_query_execution::facade::primary_graph::dispose_recovery_handle(
                handle, &authority,
            ) {
                Ok(receipt) => {
                    self.recovery = WorkflowDispatchRecovery::Disposed(receipt);
                    Ok(WorkflowStep::Completed)
                }
                Err(cause) => {
                    let (cause, handle) = cause.into_parts();
                    self.retain_dispatch_refusal(cause, handle)
                }
            };
        }
        if matches!(
            self.recovery,
            WorkflowDispatchRecovery::Released | WorkflowDispatchRecovery::Disposed(_)
        ) {
            return Err(WorthQueryWorkflowEffectRecoveryDenial::NativeHandleConsumed);
        }
        if matches!(self.recovery, WorkflowDispatchRecovery::Unopened) {
            self.recovery = WorkflowDispatchRecovery::Retained(
                source
                    .runtime
                    .runtime()
                    .mint_recovery_handle(&receipt)
                    .map_err(WorthQueryWorkflowEffectRecoveryDenial::Recovery)?,
            );
        }
        // Initial execution retains dispatch custody; a subsequent fresh advance performs one retry.
        if !retry {
            return Ok(WorkflowStep::Progress);
        }
        let WorkflowDispatchRecovery::Retained(handle) =
            std::mem::replace(&mut self.recovery, WorkflowDispatchRecovery::Consumed)
        else {
            unreachable!("native dispatch handle retained")
        };
        match crate::application_entry::workflow::operation::recovery::retry_original_handle(
            phase,
            source.runtime.runtime(),
            &prepared.admission,
            handle,
        ) {
            Ok(result) => {
                self.recovery = WorkflowDispatchRecovery::Released;
                self.recovery_result = Some(result);
                Ok(WorkflowStep::Completed)
            }
            Err((cause, handle)) => self.retain_dispatch_refusal(cause, handle),
        }
    }
    fn retain_dispatch_refusal(
        &mut self,
        cause: WorthQueryRecoveryHandleDenial,
        handle: Option<worth_query_execution::facade::primary_graph::WorthQueryRecoveryHandle>,
    ) -> Result<WorkflowStep, WorthQueryWorkflowEffectRecoveryDenial> {
        match handle {
            Some(handle) => {
                self.recovery = WorkflowDispatchRecovery::Retained(handle);
                Err(WorthQueryWorkflowEffectRecoveryDenial::Recovery(cause))
            }
            None => {
                self.recovery = WorkflowDispatchRecovery::Terminal(cause.clone());
                Ok(WorkflowStep::RecoveryTerminal(cause))
            }
        }
    }
}
