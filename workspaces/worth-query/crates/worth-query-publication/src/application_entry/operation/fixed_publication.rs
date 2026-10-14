//! Original source admission and publication through the existing native funnel.
use super::fixed::FixedState;
use super::*;
use WorthQueryOperationAdvance as Advance;
use WorthQueryOperationAdvanceDenial as Denial;
use WorthQueryOperationProgress as Progress;

impl<'runtime, Schema, Intent, Program, Root, Action>
    WorthQueryFixedOperation<'runtime, Schema, Intent, Program, Root, Action>
where
    Schema: ApplicationSchema + 'static,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema> + ApplicationRequiredOutputRoot,
    Connection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Binding<Schema, Intent>:
        WorthQueryApplicationRequiredOutputSource<Schema, Connection<Schema, Root>>,
    Action: ApplicationFixedAction<Schema, Binding<Schema, Intent>>,
    Delivery<Schema, Intent, Action>:
        ApplicationFixedActionDelivery<Schema, Binding<Schema, Intent>, Root = Root>,
    Program::Outputs: ApplicationProgramOutputsShape<Schema>,
    MutationInput<Schema, Intent>: Clone + Send + Sync,
    MutationScope<Schema, Intent>:
        ApplicationMutationScopeResolution<Schema, MutationPrincipal<Schema, Intent>>,
    Demand<Schema, Root>: Clone,
    SourceIntent<Schema, Root>: ApplicationQueryIntent<Schema, Binding = Source<Schema, Root>>,
    SourceScope<Schema, Root>:
        ApplicationQueryScopeResolution<Schema, SourcePrincipal<Schema, Root>>,
    SourceValue<Schema, Root>:
        WorthQueryApplicationProjection<Schema, RootQuery<Schema, Root>> + Clone,
    Root::Dependents:
        super::super::mutation::program_output_continuation::ProgramOutputContinuationFactory<
            Schema,
            Program,
            Demand<Schema, Root>,
        >,
{
    pub(super) fn advance_initial_in_phase(
        mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> FixedAdvance<'runtime, Schema, Intent, Program, Root, Action> {
        debug_assert!(matches!(self.state, FixedState::Initial));
        let progress = {
            let execution = match phase
                .checked_execution_request_for(self.runtime.runtime().product_runtime())
            {
                Ok(execution) => execution,
                Err(cause) => {
                    return Advance::Refused {
                        operation: self,
                        denial: Denial::Execution(cause),
                    }
                }
            };
            let mut work = worth_query_execution::facade::primary_graph::WorthQueryMutationHandlerWork::NotStarted;
            let outcome = self
                .description
                .request_view(request, self.installed_action)
                .execute_performed_with_work::<Program, Root>(
                    phase,
                    self.runtime,
                    &mut work,
                    execution.allocation_policy(),
                );
            match outcome {
                Err(denial @ WorthQueryPerformedMutationExecutionDenial::Action(_)) => {
                    return Advance::Terminal {
                        operation: WorthQueryTerminalFixedOperation::new(self),
                        denial: Denial::Mutation(denial),
                    }
                }
                Err(denial) => {
                    return Advance::Refused {
                        operation: self,
                        denial: Denial::Mutation(denial),
                    }
                }
                Ok(WorthQueryApplicationPerformedMutationOutcome::ProductUnpublished(partial)) => {
                    self.initial_cause = Some(partial.initial_cause());
                    self.state = FixedState::Unpublished(partial);
                    Progress::SourceUnpublished
                }
                Ok(WorthQueryApplicationPerformedMutationOutcome::Performed(performed)) => {
                    let (result, preparation) = performed.into_preparation();
                    self.result = Some(result);
                    self.state = FixedState::Preparation(preparation);
                    Progress::SourcePerformed
                }
                Ok(WorthQueryApplicationPerformedMutationOutcome::RequiredOutputDenied(
                    failure,
                )) => {
                    self.state = FixedState::Retention(failure);
                    Progress::SourceRetentionDenied
                }
                Ok(WorthQueryApplicationPerformedMutationOutcome::Blocked(blocked)) => {
                    self.state = FixedState::Blocked(blocked);
                    return Advance::Terminal {
                        operation: WorthQueryTerminalFixedOperation::new(self),
                        denial: Denial::NativeTerminal,
                    };
                }
                Ok(WorthQueryApplicationPerformedMutationOutcome::NotPerformed(outcome)) => {
                    self.state = FixedState::NotPerformed(outcome);
                    return Advance::Terminal {
                        operation: WorthQueryTerminalFixedOperation::new(self),
                        denial: Denial::NativeTerminal,
                    };
                }
            }
        };
        Advance::Progress {
            operation: self,
            progress,
        }
    }
}
