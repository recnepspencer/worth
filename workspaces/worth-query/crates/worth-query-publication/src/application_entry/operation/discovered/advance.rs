//! One ordinary advance opens one native phase and delegates one owning handoff.
use super::*;
use WorthQueryOperationAdvance as Advance;
use WorthQueryOperationAdvanceDenial as Denial;
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
    pub fn advance(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> DiscoveredAdvance<S, I, P, R, A> {
        if !std::ptr::eq(self.runtime.runtime(), request.application)
            || self.description.branch != request.branch
        {
            return Advance::Refused {
                operation: self,
                denial: Denial::FreshRequestMismatch,
            };
        }
        let runtime = request.application;
        let mut retained = Some(self);
        match runtime.with_application_advancement(request.scope, |phase| {
            retained
                .take()
                .expect("one opened phase owns its operation")
                .advance_in_phase(&phase, request)
        }) {
            Ok(outcome) => outcome,
            Err(cause) => Advance::Refused {
                operation: retained.expect("refused opening retains custody"),
                denial: Denial::Execution(cause),
            },
        }
    }
    fn advance_in_phase(
        self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> DiscoveredAdvance<S, I, P, R, A> {
        if !matches!(
            &self.state,
            State::Initial | State::NotPerformed(_) | State::Blocked(_)
        ) {
            let resolution = (|| {
                let selected = request
                    .application
                    .on_branch(request.branch)
                    .select(phase)
                    .map_err(Denial::Observation)?;
                let owner = self
                    .runtime
                    .selected_program_owner(&selected)
                    .map_err(Denial::Program)?;
                owner
                    .resolve_declared_action::<I::Binding>(&self.installed_action)
                    .map_err(Denial::Action)?;
                Ok::<_, Denial>(())
            })();
            if let Err(denial) = resolution {
                return Advance::Refused {
                    operation: self,
                    denial,
                };
            }
        }
        match &self.state {
            State::Initial => self.advance_source(phase, request),
            State::Replay(_) => self.advance_replay(phase, request),
            State::Unpublished(_) => self.advance_partial(phase, request),
            State::Retention(_) => self.advance_retention(phase, request),
            State::Performed(_) => self.advance_start(phase, request),
            State::Outputs(_) => self.advance_outputs(phase, request),
            State::NotPerformed(_) | State::Blocked(_) => self.terminal(Denial::NativeTerminal),
        }
    }
}
