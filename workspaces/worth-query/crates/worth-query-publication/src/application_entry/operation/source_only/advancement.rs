//! Each call enters one native phase and delegates one genuine source handoff.
use super::*;
use worth_query_declaration::facade::application_operation::ApplicationMutationAdmissionBinding;
use worth_query_execution::facade::primary_graph::WorthQueryApplicationIdempotencyResolution as Resolution;

pub(super) type Advance<S, I, P, A, M> = WorthQuerySourceOnlyAdvance<
    WorthQuerySourceOnlyOperation<S, I, P, A, M>,
    WorthQueryCompletedSourceOperation<S, I, P, A, M>,
    WorthQueryTerminalSourceOperation<S, I, P, A, M>,
>;

impl<S, I, P, A, M> WorthQuerySourceOnlyOperation<S, I, P, A, M>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    M: WorthQuerySourceDeliveryMode,
    I::Binding: ApplicationMutationAdmissionBinding<S>,
    <I::Binding as ApplicationMutationAdmissionBinding<S>>::Admission:
        super::super::super::mutation::authorization::WorthQueryBindingAdmission<S, I::Binding>,
    <I::Binding as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync,
    <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding: ApplicationMutationScopeResolution<
        S,
        <I::Binding as ApplicationMutationBinding<S>>::PrincipalIdentity,
    >,
{
    pub(super) fn advance_with_history(
        mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        maximum_history_work: Option<std::num::NonZeroUsize>,
    ) -> Advance<S, I, P, A, M> {
        let phase_keepalive = self.admission.runtime.clone();
        let runtime = phase_keepalive.runtime();
        match runtime.with_application_advancement(request.scope, |phase| {
            self.step(&phase, request, maximum_history_work)
        }) {
            Ok(Ok(Step::Progress)) => WorthQuerySourceOnlyAdvance::Progress(self),
            Ok(Ok(Step::Completed)) => {
                WorthQuerySourceOnlyAdvance::Completed(WorthQueryCompletedSourceOperation {
                    operation: self,
                })
            }
            Ok(Ok(Step::Terminal)) => {
                WorthQuerySourceOnlyAdvance::Terminal(WorthQueryTerminalSourceOperation {
                    operation: self,
                })
            }
            Ok(Err(denial)) => WorthQuerySourceOnlyAdvance::Refused {
                operation: self,
                denial,
            },
            Err(denial) => {
                self.record_unresolved_advancement(denial);
                WorthQuerySourceOnlyAdvance::Refused {
                    operation: self,
                    denial: WorthQuerySourceOnlyDenial::Execution(denial),
                }
            }
        }
    }

    pub(in crate::application_entry::operation) fn step(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        maximum_history_work: Option<std::num::NonZeroUsize>,
    ) -> Result<Step, WorthQuerySourceOnlyDenial> {
        match &mut self.state {
            SourceState::<S, I>::Initial | SourceState::<S, I>::RetryableNoEffect(_) => {
                let outcome = match self.admission.execute_in_advancement(phase, request) {
                    Ok(outcome) => outcome,
                    Err(denial) if native_posture::pinned_source_moved(&denial) => {
                        self.state = SourceState::<S, I>::AdmissionStopped(denial);
                        return Ok(Step::Terminal);
                    }
                    Err(denial) => return Err(WorthQuerySourceOnlyDenial::Mutation(denial)),
                };
                match outcome {
                    WorthQueryApplicationRetainedMutationOutcome::Committed {
                        receipt,
                        result,
                        retained,
                    } => Ok(self.complete(WorthQuerySourceCompletion::Fresh {
                        receipt,
                        result,
                        retained: Some(retained),
                    })),
                    WorthQueryApplicationRetainedMutationOutcome::Other(outcome) => {
                        self.accept_initial(outcome)
                    }
                }
            }
            SourceState::<S, I>::Unpublished(recovery) => {
                self.cleanup
                    .reserve_release_failure()
                    .map_err(WorthQuerySourceOnlyDenial::Cleanup)?;
                let outcome = self
                    .admission
                    .recover_in_advancement(phase, request, recovery)
                    .map_err(WorthQuerySourceOnlyDenial::Recovery)?;
                self.state.accept_recovery(outcome, &mut self.cleanup);
                Ok(Step::Progress)
            }

            SourceState::<S, I>::Performed(performed) => {
                self.cleanup
                    .absorb_recovered(performed)
                    .map_err(WorthQuerySourceOnlyDenial::Cleanup)?;
                let resolution = self
                    .admission
                    .resolve_keyed_in_advancement(phase, request)
                    .map_err(WorthQuerySourceOnlyDenial::Recovery)?
                    .into_resolution();
                self.accept_keyed(resolution)
            }
            SourceState::<S, I>::Keyed(_) | SourceState::<S, I>::UnresolvedAdvancement(_) => {
                let resolution = self
                    .admission
                    .resolve_keyed_in_advancement(phase, request)
                    .map_err(WorthQuerySourceOnlyDenial::Recovery)?
                    .into_resolution();
                self.accept_keyed(resolution)
            }
            SourceState::<S, I>::RetainedPending(_) => {
                match self
                    .admission
                    .resolve_keyed_in_advancement(phase, request)
                    .map_err(WorthQuerySourceOnlyDenial::Recovery)?
                    .into_resolution()
                {
                    Resolution::AlreadyCommitted(_) => {}
                    Resolution::Unseen => return Ok(Step::Progress),
                    resolution @ Resolution::IntentDrift => {
                        self.keyed_terminal = Some(resolution);
                        return Ok(Step::Terminal);
                    }
                }
                self.deliver_retained(
                    phase,
                    request,
                    maximum_history_work
                        .expect("retained mode requires a per-attempt history bound"),
                )
                .map(|()| Step::Completed)
            }
            SourceState::<S, I>::Stopped(_) | SourceState::<S, I>::AdmissionStopped(_) => {
                Ok(Step::Terminal)
            }
        }
    }

    fn accept_initial(
        &mut self,
        outcome: NativeOutcome<S, I>,
    ) -> Result<Step, WorthQuerySourceOnlyDenial> {
        use WorthQueryApplicationMutationOutcome as O;
        let outcome = self
            .cleanup
            .absorb_native_outcome(&self.admission.runtime, outcome);
        match outcome {
            O::Committed { receipt, result } => {
                Ok(self.complete(WorthQuerySourceCompletion::Fresh {
                    receipt,
                    result,
                    retained: None,
                }))
            }
            O::AlreadyCommitted(receipt) if M::RETAIN_COMMIT => {
                self.state =
                    SourceState::<S, I>::RetainedPending(WorthQuerySourceCompletion::Recovered {
                        retained: None,
                        receipt,
                        publication: None,
                        initial_posture: None,
                        initial_advancement_failure: None,
                    });
                Ok(Step::Progress)
            }
            O::AlreadyCommitted(receipt) => {
                Ok(self.complete(WorthQuerySourceCompletion::Recovered {
                    retained: None,
                    receipt,
                    publication: None,
                    initial_posture: None,
                    initial_advancement_failure: None,
                }))
            }
            O::Commit(posture) => match self.state.accept_uncommitted(posture) {
                Ok(()) => Ok(Step::Progress),
                Err(posture) => self.accept_no_effect_or_terminal(O::Commit(posture)),
            },
            other => self.accept_no_effect_or_terminal(other),
        }
    }

    fn accept_no_effect_or_terminal(
        &mut self,
        outcome: NativeOutcome<S, I>,
    ) -> Result<Step, WorthQuerySourceOnlyDenial> {
        match outcome {
            other
                if self.admission.description.source.is_none()
                    && native_posture::fresh_retry_eligible::<S, I>(&other) =>
            {
                self.state = SourceState::<S, I>::RetryableNoEffect(other);
                Ok(Step::Progress)
            }
            other => {
                self.state = SourceState::<S, I>::Stopped(other);
                Ok(Step::Terminal)
            }
        }
    }

    fn accept_keyed(&mut self, resolution: Resolution) -> Result<Step, WorthQuerySourceOnlyDenial> {
        use super::super::native_publication::NativeKeyedProgress;
        match self
            .state
            .accept_keyed(resolution, &self.cleanup)
            .map_err(WorthQuerySourceOnlyDenial::Cleanup)?
        {
            NativeKeyedProgress::Completed(recovered) => {
                let evidence = WorthQuerySourceCompletion::Recovered {
                    retained: None,
                    receipt: recovered.receipt,
                    publication: recovered.publication,
                    initial_posture: recovered.initial_posture,
                    initial_advancement_failure: recovered.initial_advancement_failure,
                };
                if M::RETAIN_COMMIT {
                    self.state = SourceState::<S, I>::RetainedPending(evidence);
                    Ok(Step::Progress)
                } else {
                    Ok(self.complete(evidence))
                }
            }
            NativeKeyedProgress::Progress { proved_abort } => {
                if self.admission.workflow.is_some() && proved_abort.is_some() {
                    self.guarded_abort = proved_abort;
                }
                Ok(Step::Progress)
            }
            NativeKeyedProgress::Terminal(resolution) => {
                self.keyed_terminal = Some(resolution);
                Ok(Step::Terminal)
            }
        }
    }
}

pub(in crate::application_entry::operation) enum Step {
    Progress,
    Completed,
    Terminal,
}
