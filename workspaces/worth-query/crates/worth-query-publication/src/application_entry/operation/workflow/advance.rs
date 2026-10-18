//! One admitted phase delegates one existing source or outbox advancement.
use super::super::source_only::SourceStep;
use super::*;
use worth_query_declaration::facade::application_operation::ApplicationMutationAdmissionBinding;
pub(super) enum WorkflowStep {
    Progress,
    Completed,
    Terminal,
    Spent(WorthQuerySourceOnlyDenial),
    RecoveryTerminal(worth_query_execution::facade::primary_graph::WorthQueryRecoveryHandleDenial),
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
    pub fn advance(
        mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> WorthQueryWorkflowOperationAdvance<
        Self,
        WorthQueryCompletedWorkflowOperation<S, I, P, A>,
        WorthQueryTerminalWorkflowOperation<S, I, P, A>,
    > {
        let keepalive = self.source.admission.runtime.clone();
        let outcome = keepalive
            .runtime()
            .with_application_advancement(request.scope, |phase| {
                self.advance_in_advancement(&phase, request)
            });
        match outcome {
            Ok(Ok(WorkflowStep::Progress)) => WorthQueryWorkflowOperationAdvance::Progress(self),
            Ok(Ok(WorkflowStep::Completed)) => WorthQueryWorkflowOperationAdvance::Completed(
                WorthQueryCompletedWorkflowOperation {
                    source: WorthQueryCompletedSourceOperation {
                        operation: self.source,
                    },
                    required: self.required,
                    recovery: self.recovery,
                    recovery_result: self.recovery_result,
                },
            ),
            Ok(Ok(WorkflowStep::Terminal)) => {
                WorthQueryWorkflowOperationAdvance::Terminal(WorthQueryTerminalWorkflowOperation {
                    state: super::completion::TerminalWorkflowState::Native {
                        source: WorthQueryTerminalSourceOperation {
                            operation: self.source,
                        },
                    },
                })
            }
            Ok(Ok(WorkflowStep::Spent(denial))) => {
                WorthQueryWorkflowOperationAdvance::Terminal(WorthQueryTerminalWorkflowOperation {
                    state: super::completion::TerminalWorkflowState::Spent {
                        operation: self,
                        denial,
                    },
                })
            }
            Ok(Ok(WorkflowStep::RecoveryTerminal(denial))) => {
                WorthQueryWorkflowOperationAdvance::Terminal(WorthQueryTerminalWorkflowOperation {
                    state: super::completion::TerminalWorkflowState::Recovery {
                        operation: self,
                        denial,
                    },
                })
            }
            Ok(Err(denial)) => WorthQueryWorkflowOperationAdvance::Refused {
                operation: self,
                denial,
            },
            Err(cause) => {
                self.source.record_unresolved_advancement(cause);
                WorthQueryWorkflowOperationAdvance::Refused {
                    operation: self,
                    denial: WorthQueryWorkflowOperationDenial::Source(
                        WorthQuerySourceOnlyDenial::Execution(cause),
                    ),
                }
            }
        }
    }
    fn advance_in_advancement(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorkflowStep, WorthQueryWorkflowOperationDenial> {
        if self.source_completed {
            return self
                .advance_dispatch(phase, request, true)
                .map_err(WorthQueryWorkflowOperationDenial::Recovery);
        }
        match self.source.step(phase,request,None) {
            Ok(SourceStep::Progress)=>Ok(WorkflowStep::Progress),
            Ok(SourceStep::Terminal)=>Ok(WorkflowStep::Terminal),
            Ok(SourceStep::Completed)=>{
                self.source_completed=true;
                self.advance_dispatch(phase,request,false).map_err(WorthQueryWorkflowOperationDenial::Recovery)
            },
            Err(denial) if matches!(&denial,WorthQuerySourceOnlyDenial::Mutation(crate::application_entry::WorthQueryApplicationRequestMutationDenial::WorkflowAuthoritySpent|crate::application_entry::WorthQueryApplicationRequestMutationDenial::WorkflowTransitionCurrentness(_)))=>Ok(WorkflowStep::Spent(denial)),
            Err(cause)=>Err(WorthQueryWorkflowOperationDenial::Source(cause)),
        }
    }
}
impl<S, I, P, A> WorthQueryWorkflowOperation<S, I, P, A>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
{
    pub fn required(&self) -> &RequiredWorkflowOperation {
        &self.required
    }
    pub fn close(
        mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryClosedOperation, (Self, WorthQueryOperationCloseDenial)> {
        self.close_in_place(request)
            .map_err(|denial| (self, denial))
    }

    pub(super) fn close_in_place(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryClosedOperation, WorthQueryOperationCloseDenial> {
        // An inbound-waiting owner cannot close away its original evidence before
        // a fresh advance observes native settlement. Other close postures stay native.
        if matches!(
            self.recovery,
            super::completion::WorkflowDispatchRecovery::Retained(_)
                | super::completion::WorkflowDispatchRecovery::Consumed
        ) || (self.source_completed
            && matches!(
                self.recovery,
                super::completion::WorkflowDispatchRecovery::Unopened
            )
            && match self.source.completion() {
                WorthQuerySourceCompletion::Fresh { receipt, .. }
                | WorthQuerySourceCompletion::Recovered { receipt, .. } => {
                    super::recovery::inbound_without_recovery(receipt)
                }
            })
        {
            return Err(WorthQueryOperationCloseDenial::NativeObligationRetained);
        }
        if self.source_completed {
            self.source.close_cleanup(request)
        } else {
            self.source.close_in_place(request)
        }
    }
}

impl<
        S: ApplicationSchema + 'static,
        I: ApplicationMutationIntent<S>,
        P: ApplicationProgramDefinition<S>,
        A,
    > WorthQueryWorkflowOperation<S, I, P, A>
{
    pub fn managed_cleanup_failure(
        &self,
    ) -> Option<&worth_query_execution::facade::primary_graph::WorthQueryMutationCleanupFailure>
    {
        self.source.managed_cleanup_failure()
    }
    pub fn release_cleanup(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryOperationCleanupReceipt, WorthQueryOperationCloseDenial> {
        self.source.release_cleanup(request)
    }
}
