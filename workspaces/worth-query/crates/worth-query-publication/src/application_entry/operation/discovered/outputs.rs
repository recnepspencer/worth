//! Start and advance only the existing native discovered traversal; retain all refused owners.
use super::*;
use WorthQueryOperationAdvance as Advance;
use WorthQueryOperationAdvanceDenial as Denial;
use WorthQueryOperationProgress as Progress;
impl<'r, S, I, P, R, A> WorthQueryDiscoveredOperation<'r, S, I, P, R, A>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationDiscoveredOutputRoot,
    Connection<S, R>: WorthQueryApplicationDiscoveredOutputConnection<S, Source = I::Binding>,
    A: ApplicationDiscoveredAction<S, I::Binding>,
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
    pub(super) fn advance_start(
        mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> DiscoveredAdvance<'r, S, I, P, R, A> {
        let admission = self
            .description
            .request_view(request, self.installed_action)
            .authorize_discovered_source::<P, R>(phase, self.runtime, None);
        let _admission = match admission {
            Ok(admission) => admission,
            Err(denial) => {
                return Advance::Refused {
                    operation: self,
                    denial: Denial::Recovery(denial),
                }
            }
        };
        let State::Performed(performed) = self.state else {
            unreachable!("dispatch owns genuine performed source")
        };
        match performed.start_required_outputs_in_advancement(
            phase,
            self.runtime,
            request,
            WorthQueryOutputDemandControls::host_policy(),
        ) {
            Ok(started) => {
                let (_receipt, result, outputs) = started.into_parts();
                self.result = Some(result);
                self.state = State::Outputs(outputs);
                Advance::Progress {
                    operation: self,
                    progress: Progress::OutputsStarted,
                }
            }
            Err(failure) => {
                let (performed, denial) = failure.into_parts();
                let terminal =
                    denial.recovery_posture() == WorthQueryRequiredOutputRecoveryPosture::Terminal;
                self.state = State::Performed(performed);
                if terminal {
                    self.terminal(Denial::Outputs(denial))
                } else {
                    Advance::Refused {
                        operation: self,
                        denial: Denial::Outputs(denial),
                    }
                }
            }
        }
    }
    pub(super) fn advance_outputs(
        mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> DiscoveredAdvance<'r, S, I, P, R, A> {
        let admission = self
            .description
            .request_view(request, self.installed_action)
            .authorize_discovered_source::<P, R>(phase, self.runtime, None);
        let _admission = match admission {
            Ok(admission) => admission,
            Err(denial) => {
                return Advance::Refused {
                    operation: self,
                    denial: Denial::Recovery(denial),
                }
            }
        };
        let State::Outputs(mut outputs) = self.state else {
            unreachable!("dispatch owns native output continuation")
        };
        match outputs.advance_in_advancement(phase, self.runtime, request) {
            Ok(WorthQueryDiscoveredProgramOutputProgress::Pending) => {
                self.state = State::Outputs(outputs);
                Advance::Progress {
                    operation: self,
                    progress: Progress::OutputsPending,
                }
            }
            Ok(WorthQueryDiscoveredProgramOutputProgress::Settled(settlement)) => {
                Advance::Settled(WorthQuerySettledDiscoveredOperation {
                    runtime: self.runtime,
                    installed_action: self.installed_action,
                    description: self.description,
                    settlement,
                    initial_cause: self.initial_cause,
                    result: self.result,
                    recovered: self.recovered,
                    prior_cleanup: self.prior_cleanup,
                    cleanup: self.cleanup,
                    action: self.action,
                })
            }
            Err(mut denial) => {
                if let Some(required) = denial.take_managed_cleanup() {
                    self.cleanup.absorb_producer(self.runtime, required);
                }
                let terminal =
                    denial.recovery_posture() == WorthQueryRequiredOutputRecoveryPosture::Terminal;
                self.state = State::Outputs(outputs);
                if terminal {
                    self.terminal(Denial::Outputs(denial))
                } else {
                    Advance::Refused {
                        operation: self,
                        denial: Denial::Outputs(denial),
                    }
                }
            }
        }
    }
}
