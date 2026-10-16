//! Fresh borrowed admission recovers, promotes or retries the exact retained native source.
use super::*;
use WorthQueryOperationAdvance as Advance;
use WorthQueryOperationAdvanceDenial as Denial;
use WorthQueryOperationProgress as Progress;
impl<S, I, P, R, A> WorthQueryDiscoveredOperation<S, I, P, R, A>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationDiscoveredOutputRoot,
    Connection<S, R>: WorthQueryApplicationDiscoveredOutputConnection<S, Source = I::Binding>,
    A: ApplicationAction<S, I::Binding>,
    ActionDelivery<S, I, A>: ApplicationDiscoveredActionDelivery<S, I::Binding, Root = R>,
    I::Binding: ApplicationMutationAdmissionBinding<S>,
    Admission<S, I>: WorthQueryBindingAdmission<S, I::Binding>,
    MutationInput<S, I>: Clone + Send + Sync,
    MutationScope<S, I>: ApplicationMutationScopeResolution<S, MutationPrincipal<S, I>>,
    RootDemand<S, R>: Clone,
    DiscoveryValue<S, R>: WorthQueryApplicationProjection<S, DiscoveryQuery<S, R>> + Clone,
    DiscoveryScope<S, R>: ApplicationQueryScopeResolution<S, DiscoveryPrincipal<S, R>>,
    RootInput<S, R>: ApplicationQueryIntent<S, Binding = RootSource<S, R>>,
    RootScope<S, R>: ApplicationQueryScopeResolution<S, RootPrincipal<S, R>>,
    RootValue<S, R>: WorthQueryApplicationProjection<S, RootQuery<S, R>> + Clone,
    R::Dependents: ProgramOutputContinuationFactory<S, P, RootDemand<S, R>>,
{
    pub(super) fn advance_partial(
        mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> DiscoveredAdvance<S, I, P, R, A> {
        let State::Unpublished(partial) = self.state else {
            unreachable!("dispatch owns an unpublished source")
        };
        if partial.recovery_posture() == WorthQueryRequiredOutputRecoveryPosture::Terminal {
            self.state = State::Unpublished(partial);
            return self.terminal(Denial::RecoveryCarrierUnavailable);
        }
        let mut borrowed = partial.into_same_binding_intent::<&I>();
        if borrowed.performed().is_none() {
            let progress = self
                .description
                .request_view(request, &self.installed_action)
                .recover_unpublished_discovered_in_program_in_advancement::<P, R>(
                    phase,
                    &mut borrowed,
                    &self.runtime,
                );
            self.state = State::Unpublished(borrowed.into_same_binding_intent::<I>());
            return match progress {
                Ok(progress) => Advance::Progress {
                    operation: self,
                    progress: Progress::Recovery(progress),
                },
                Err(denial) => Advance::Refused {
                    operation: self,
                    denial: Denial::Recovery(denial),
                },
            };
        }
        let promoted = self
            .description
            .request_view(request, &self.installed_action)
            .promote_recovered_discovered_outputs_in_advancement::<P, R>(
                phase,
                borrowed,
                &self.runtime,
                WorthQueryOutputDemandControls::host_policy(),
            );
        match promoted {
            Ok(promoted) => {
                let (outputs, cause, performed, prior_cleanup) = promoted.into_parts();
                self.state = State::Outputs(outputs);
                self.initial_cause = Some(cause);
                self.recovered = Some(performed);
                self.prior_cleanup = prior_cleanup;
                Advance::Progress {
                    operation: self,
                    progress: Progress::SourcePromoted,
                }
            }
            Err((denial, partial)) => {
                self.state = State::Unpublished(partial.into_same_binding_intent::<I>());
                if matches!(&self.state, State::Unpublished(partial) if partial.recovery_posture() == WorthQueryRequiredOutputRecoveryPosture::Terminal)
                {
                    self.terminal(Denial::Recovery(denial))
                } else {
                    Advance::Refused {
                        operation: self,
                        denial: Denial::Recovery(denial),
                    }
                }
            }
        }
    }
    pub(super) fn advance_retention(
        mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> DiscoveredAdvance<S, I, P, R, A> {
        let State::Retention(failure) = self.state else {
            unreachable!("dispatch owns the unregistered original source")
        };
        let terminal =
            failure.recovery_posture() == WorthQueryRequiredOutputRecoveryPosture::Terminal;
        self.state = State::Retention(failure);
        if terminal {
            return self.terminal(Denial::RecoveryCarrierUnavailable);
        }
        let State::Retention(failure) = self.state else {
            unreachable!("the same original failure remains owned")
        };
        let retry = self
            .description
            .request_view(request, &self.installed_action)
            .retry_discovered_output_retention_in_advancement::<P, R>(
                phase,
                failure.into_same_binding_intent::<&I>(),
                &self.runtime,
            );
        match retry {
            Ok(performed) => {
                self.state = State::Performed(performed.into_same_binding_intent::<I>());
                Advance::Progress {
                    operation: self,
                    progress: Progress::SourcePerformed,
                }
            }
            Err((denial, failure)) => {
                let terminal =
                    failure.recovery_posture() == WorthQueryRequiredOutputRecoveryPosture::Terminal;
                self.state = State::Retention(failure.into_same_binding_intent::<I>());
                if terminal {
                    self.terminal(Denial::Recovery(denial))
                } else {
                    Advance::Refused {
                        operation: self,
                        denial: Denial::Recovery(denial),
                    }
                }
            }
        }
    }
}
