//! Each call enters one native phase and delegates one genuine source handoff.
use super::*;
use worth_query_declaration::facade::application_operation::ApplicationMutationAdmissionBinding;
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationIdempotencyResolution as Resolution,
    WorthQueryManagedApplicationRecoveryOutcome as Recovery,
};

type Advance<'r, S, I, P, A, M> = WorthQuerySourceOnlyAdvance<
    WorthQuerySourceOnlyOperation<'r, S, I, P, A, M>,
    WorthQueryCompletedSourceOperation<'r, S, I, P, A, M>,
    WorthQueryTerminalSourceOperation<'r, S, I, P, A, M>,
>;

impl<'r, S, I, P, A, M> WorthQuerySourceOnlyOperation<'r, S, I, P, A, M>
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
    pub fn advance(
        mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Advance<'r, S, I, P, A, M> {
        let runtime = self.admission.runtime.runtime();
        match runtime
            .with_application_advancement(request.scope, |phase| self.step(&phase, request))
        {
            Ok(Ok(Step::Progress)) => WorthQuerySourceOnlyAdvance::Progress(self),
            Ok(Ok(Step::Completed(evidence))) => {
                WorthQuerySourceOnlyAdvance::Completed(WorthQueryCompletedSourceOperation {
                    operation: self,
                    evidence,
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
                if matches!(denial, worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial::Panicked)
                    && matches!(self.state, SourceState::Initial)
                {
                    self.state = SourceState::UnresolvedAdvancement(denial);
                }
                WorthQuerySourceOnlyAdvance::Refused {
                    operation: self,
                    denial: WorthQuerySourceOnlyDenial::Execution(denial),
                }
            }
        }
    }

    fn step(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        Step<<I::Binding as ApplicationMutationBinding<S>>::Result>,
        WorthQuerySourceOnlyDenial,
    > {
        match &mut self.state {
            SourceState::Initial => {
                let outcome = self
                    .admission
                    .execute_in_advancement(phase, request)
                    .map_err(WorthQuerySourceOnlyDenial::Mutation)?;
                match outcome {
                    WorthQueryApplicationRetainedMutationOutcome::Committed {
                        receipt,
                        result,
                        retained,
                    } => Ok(Step::Completed(WorthQuerySourceCompletion::Fresh {
                        receipt,
                        result,
                        retained: Some(retained),
                    })),
                    WorthQueryApplicationRetainedMutationOutcome::Other(outcome) => {
                        self.accept_initial(outcome)
                    }
                }
            }
            SourceState::Unpublished(recovery) => {
                self.cleanup
                    .reserve_release_failure()
                    .map_err(WorthQuerySourceOnlyDenial::Cleanup)?;
                match self
                    .admission
                    .recover_in_advancement(phase, request, recovery)
                    .map_err(WorthQuerySourceOnlyDenial::Recovery)?
                {
                    Recovery::NoEffect => Ok(Step::Progress),
                    Recovery::ProductUnpublished {
                        next,
                        prior_cleanup_failure,
                    } => {
                        self.cleanup.absorb_release_failure(prior_cleanup_failure);
                        self.state = SourceState::Unpublished(next);
                        Ok(Step::Progress)
                    }
                    Recovery::Performed(performed) => {
                        self.state = SourceState::Performed(performed);
                        Ok(Step::Progress)
                    }
                }
            }
            SourceState::Performed(performed) => {
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
            SourceState::Keyed(_) | SourceState::UnresolvedAdvancement(_) => {
                let resolution = self
                    .admission
                    .resolve_keyed_in_advancement(phase, request)
                    .map_err(WorthQuerySourceOnlyDenial::Recovery)?
                    .into_resolution();
                self.accept_keyed(resolution)
            }
            SourceState::Stopped(_) | SourceState::RetainedUnavailable(_) => Ok(Step::Terminal),
        }
    }

    fn accept_initial(
        &mut self,
        outcome: NativeOutcome<S, I>,
    ) -> Result<
        Step<<I::Binding as ApplicationMutationBinding<S>>::Result>,
        WorthQuerySourceOnlyDenial,
    > {
        use WorthQueryApplicationMutationOutcome as O;
        match outcome {
            O::Committed { receipt, result } => {
                Ok(Step::Completed(WorthQuerySourceCompletion::Fresh {
                    receipt,
                    result,
                    retained: None,
                }))
            }
            O::AlreadyCommitted(receipt) if M::RETAIN_COMMIT => {
                self.state =
                    SourceState::RetainedUnavailable(WorthQuerySourceCompletion::Recovered {
                        receipt,
                        publication: None,
                        initial_posture: None,
                        initial_advancement_failure: None,
                    });
                Ok(Step::Terminal)
            }
            O::AlreadyCommitted(receipt) => {
                Ok(Step::Completed(WorthQuerySourceCompletion::Recovered {
                    receipt,
                    publication: None,
                    initial_posture: None,
                    initial_advancement_failure: None,
                }))
            }
            O::Commit(WorthQueryApplicationUncommitted::ProductUnpublished(partial)) => {
                self.state = SourceState::Unpublished(partial.into_recovery());
                Ok(Step::Progress)
            }
            O::Commit(
                posture @ (WorthQueryApplicationUncommitted::Aborted
                | WorthQueryApplicationUncommitted::Indeterminate(_)),
            ) => {
                self.state = SourceState::Keyed(posture);
                Ok(Step::Progress)
            }
            other => {
                self.state = SourceState::Stopped(other);
                Ok(Step::Terminal)
            }
        }
    }

    fn accept_keyed(
        &mut self,
        resolution: Resolution,
    ) -> Result<
        Step<<I::Binding as ApplicationMutationBinding<S>>::Result>,
        WorthQuerySourceOnlyDenial,
    > {
        match resolution {
            Resolution::AlreadyCommitted(receipt) => {
                let state = std::mem::replace(&mut self.state, SourceState::Initial);
                let (publication, initial_posture, initial_advancement_failure) = match state {
                    SourceState::Performed(performed) => (Some(performed), None, None),
                    SourceState::Keyed(posture) => (None, Some(posture), None),
                    SourceState::UnresolvedAdvancement(failure) => (None, None, Some(failure)),
                    _ => unreachable!(
                        "only native performed or unresolved custody reaches keyed resolution"
                    ),
                };
                let evidence = WorthQuerySourceCompletion::Recovered {
                    receipt,
                    publication,
                    initial_posture,
                    initial_advancement_failure,
                };
                if M::RETAIN_COMMIT {
                    self.state = SourceState::RetainedUnavailable(evidence);
                    Ok(Step::Terminal)
                } else {
                    Ok(Step::Completed(evidence))
                }
            }
            Resolution::Unseen
                if matches!(
                    self.state,
                    SourceState::Keyed(WorthQueryApplicationUncommitted::Aborted)
                ) =>
            {
                // The native Aborted outcome proves no effect; Unseen alone never does.
                self.state = SourceState::Initial;
                Ok(Step::Progress)
            }
            Resolution::Unseen => Ok(Step::Progress),
            resolution @ Resolution::IntentDrift => {
                self.keyed_terminal = Some(resolution);
                Ok(Step::Terminal)
            }
        }
    }
}

enum Step<Result> {
    Progress,
    Completed(WorthQuerySourceCompletion<Result>),
    Terminal,
}
