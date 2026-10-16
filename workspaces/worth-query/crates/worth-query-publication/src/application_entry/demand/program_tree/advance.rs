//! Native original-tree progress under one freshly authenticated phase.
use super::*;
/// Why fresh advancement of the original complete output tree was refused.
#[derive(Debug)]
pub enum WorthQueryProgramOutputAdvanceDenial {
    FreshRequestMismatch,
    Execution(worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial),
    Program(worth_query_execution::facade::application_installation::WorthQuerySelectedProgramOwnerDenial),
    Outputs(WorthQueryRequiredOutputPreparationDenial),
}
/// The native next phase, retaining the original tree on progress or refusal.
#[must_use = "retain the returned original tree custody on every progression"]
pub enum WorthQueryProgramOutputOperationAdvance<S, P, R>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    Pending(WorthQueryProgramOutputOperation<S, P, R>),
    Refused {
        operation: WorthQueryProgramOutputOperation<S, P, R>,
        denial: WorthQueryProgramOutputAdvanceDenial,
    },
    Settled(WorthQuerySettledProgramOutputOperation<S, P, R>),
    Terminal(WorthQueryTerminalProgramOutputOperation<S, P, R>),
}
impl<S, P, R> WorthQueryProgramOutputOperation<S, P, R>
where
    S: ApplicationSchema + 'static,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
    Demand<S, R>: Clone,
    SourceIntent<S, R>: ApplicationQueryIntent<S, Binding = Source<S, R>>,
    SourceScope<S, R>: ApplicationQueryScopeResolution<S, SourcePrincipal<S, R>>,
    Value<S, R>: WorthQueryApplicationProjection<S, Query<S, R>> + Clone,
    R::Dependents: ProgramOutputContinuationFactory<S, P, Demand<S, R>>,
{
    pub fn advance(
        mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> WorthQueryProgramOutputOperationAdvance<S, P, R> {
        use WorthQueryProgramOutputAdvanceDenial as Denial;
        use WorthQueryProgramOutputOperationAdvance as Advance;
        if !self.core.matches(request) {
            return Advance::Refused {
                operation: self,
                denial: Denial::FreshRequestMismatch,
            };
        }
        let result = request.application.with_application_advancement(request.scope, |phase| {
            // Validate fresh selected membership without retaining another loan
            // during the native tree pass. Original source basis never changes.
            {
                let current = request.retain_read(&phase).map_err(|stop| Denial::Outputs(WorthQueryRequiredOutputPreparationDenial::ReadObservation(stop)))?;
                self.core.runtime.validate_selected_output_root::<R>(&phase, &worth_query_execution::publication_boundary::program_publication_access(), &current.retained).map_err(Denial::Program)?;
            }
            if self.core.tree.is_none() {
                let basis = self.core.basis.as_ref().expect("construction owns its issued basis");
                self.core.tree = Some(request.start_program_outputs_at_in_advancement::<P,R>(&phase, &self.core.runtime, self.core.demand.clone(), self.core.controls, basis).map_err(Denial::Outputs)?);
            }
            self.core.tree.as_mut().expect("initial admission retains its native tree")
                .advance_in_advancement(&phase, &self.core.runtime, request).map_err(Denial::Outputs)
        }).map_err(Denial::Execution).and_then(|result| result);
        match result {
            Ok(WorthQueryApplicationProgramOutputProgress::Pending) => Advance::Pending(self),
            Ok(WorthQueryApplicationProgramOutputProgress::Settled(settlement)) => {
                Advance::Settled(WorthQuerySettledProgramOutputOperation {
                    settlement,
                    core: self.core,
                })
            }
            Err(mut denial) => {
                let terminal = if let Denial::Outputs(output) = &mut denial {
                    if let Some(cleanup) = output.take_read_cleanup() {
                        debug_assert!(
                            self.core.cleanup.is_none(),
                            "a terminal native producer transfers cleanup once"
                        );
                        self.core.cleanup = Some(cleanup);
                    }
                    self.core.cleanup.is_some()
                        || output.recovery_posture()
                            == WorthQueryRequiredOutputRecoveryPosture::Terminal
                } else {
                    matches!(denial, Denial::Program(worth_query_execution::facade::application_installation::WorthQuerySelectedProgramOwnerDenial::UndeclaredOutputRoot | worth_query_execution::facade::application_installation::WorthQuerySelectedProgramOwnerDenial::InstalledOwnerUnavailable))
                };
                if terminal {
                    Advance::Terminal(WorthQueryTerminalProgramOutputOperation {
                        core: self.core,
                        denial,
                    })
                } else {
                    Advance::Refused {
                        operation: self,
                        denial,
                    }
                }
            }
        }
    }
}
