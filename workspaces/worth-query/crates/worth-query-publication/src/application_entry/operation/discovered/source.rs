//! Initial execution borrows the original description and moves same-binding native custody.
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
    pub(super) fn advance_source(
        mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> DiscoveredAdvance<S, I, P, R, A> {
        let execution =
            match phase.checked_execution_request_for(self.runtime.runtime().product_runtime()) {
                Ok(execution) => execution,
                Err(cause) => {
                    return Advance::Refused {
                        operation: self,
                        denial: Denial::Execution(cause),
                    };
                }
            };
        let outcome = self
            .description
            .request_view(request, &self.installed_action)
            .execute_performed_discovered_in_advancement::<P, R>(
                phase,
                &self.runtime,
                execution.allocation_policy(),
            )
            .map(|outcome| outcome.into_same_binding_intent::<I>());
        let mut outcome = match outcome {
            Ok(outcome) => outcome,
            Err(denial @ WorthQueryPerformedMutationExecutionDenial::Action(_)) => {
                return self.terminal(Denial::Mutation(denial));
            }
            Err(denial) => {
                return Advance::Refused {
                    operation: self,
                    denial: Denial::Mutation(denial),
                };
            }
        };
        // The native classifier owns cleanup independently of business/source classification.
        while let WorthQueryApplicationDiscoveredMutationOutcome::CleanupRequired {
            cleanup,
            outcome: inner,
        } = outcome
        {
            self.cleanup.absorb_managed(&self.runtime, cleanup);
            outcome = *inner;
        }
        let progress = match outcome {
            WorthQueryApplicationDiscoveredMutationOutcome::ProductUnpublished(partial) => {
                self.initial_cause = Some(partial.initial_cause());
                self.state = State::Unpublished(partial);
                Progress::SourceUnpublished
            }
            WorthQueryApplicationDiscoveredMutationOutcome::RequiredOutputDenied(failure) => {
                self.state = State::Retention(failure);
                Progress::SourceRetentionDenied
            }
            WorthQueryApplicationDiscoveredMutationOutcome::Performed(performed) => {
                self.state = State::Performed(performed);
                Progress::SourcePerformed
            }
            WorthQueryApplicationDiscoveredMutationOutcome::Blocked(blocked) => {
                self.state = State::Blocked(blocked);
                return self.terminal(Denial::NativeTerminal);
            }
            WorthQueryApplicationDiscoveredMutationOutcome::NotPerformed(
                WorthQueryApplicationMutationOutcome::AlreadyCommitted(receipt),
            ) => {
                self.state = State::Replay(receipt);
                Progress::SourceReplayObserved
            }
            WorthQueryApplicationDiscoveredMutationOutcome::NotPerformed(outcome) => {
                self.state =
                    State::NotPerformed(self.cleanup.absorb_native_outcome(&self.runtime, outcome));
                return self.terminal(Denial::NativeTerminal);
            }
            WorthQueryApplicationDiscoveredMutationOutcome::CleanupRequired { .. } => {
                unreachable!("cleanup has been moved before classification")
            }
        };
        Advance::Progress {
            operation: self,
            progress,
        }
    }
}
