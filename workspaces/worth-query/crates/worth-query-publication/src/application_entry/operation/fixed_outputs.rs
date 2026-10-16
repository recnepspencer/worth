//! Native fixed output start and traversal to authentic settlement.
use super::fixed::FixedState;
use super::*;
use WorthQueryOperationAdvance as Advance;
use WorthQueryOperationAdvanceDenial as Denial;
use WorthQueryOperationProgress as Progress;

impl<Schema, Intent, Program, Root, Action>
    WorthQueryFixedOperation<Schema, Intent, Program, Root, Action>
where
    Schema: ApplicationSchema + 'static,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema> + ApplicationRequiredOutputRoot,
    Connection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Binding<Schema, Intent>:
        WorthQueryApplicationRequiredOutputSource<Schema, Connection<Schema, Root>>,
    Action: ApplicationAction<Schema, Binding<Schema, Intent>>,
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
    pub(super) fn advance_preparation_in_phase(
        mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> FixedAdvance<Schema, Intent, Program, Root, Action> {
        let FixedState::Preparation(preparation) = self.state else {
            unreachable!("output dispatch owns its preparation")
        };
        let progress = {
            match preparation.start_required_outputs_in_advancement(
                phase,
                &self.runtime,
                request,
                WorthQueryOutputDemandControls::host_policy(),
            ) {
                Ok(outputs) => {
                    self.state = FixedState::Outputs(outputs);
                    Progress::OutputsStarted
                }
                Err((preparation, denial)) => {
                    let terminal = denial.recovery_posture()
                        == WorthQueryRequiredOutputRecoveryPosture::Terminal;
                    self.state = FixedState::Preparation(preparation);
                    return if terminal {
                        Advance::Terminal {
                            operation: WorthQueryTerminalFixedOperation::new(self),
                            denial: Denial::Outputs(denial),
                        }
                    } else {
                        Advance::Refused {
                            operation: self,
                            denial: Denial::Outputs(denial),
                        }
                    };
                }
            }
        };
        Advance::Progress {
            operation: self,
            progress,
        }
    }

    pub(super) fn advance_outputs_in_phase(
        mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> FixedAdvance<Schema, Intent, Program, Root, Action> {
        let FixedState::Outputs(mut outputs) = self.state else {
            unreachable!("output dispatch owns native output custody")
        };
        let progress = {
            match outputs.advance_in_advancement(phase, &self.runtime, request) {
                Ok(WorthQueryApplicationProgramOutputProgress::Pending) => {
                    self.state = FixedState::Outputs(outputs);
                    Progress::OutputsPending
                }
                Ok(WorthQueryApplicationProgramOutputProgress::Settled(settlement)) => {
                    return Advance::Settled(WorthQuerySettledFixedOperation {
                        delivery: FixedDeliveryCore {
                            installed_action: self.installed_action,
                            description: self.description,
                            settlement,
                            _action: self.action,
                            runtime: self.runtime,
                        },
                        initial_cause: self.initial_cause,
                        recovered: self.recovered,
                        result: self.result,
                        cleanup: self.cleanup,
                    });
                }
                Err(mut denial) => {
                    if let Some(required) = denial.take_managed_cleanup() {
                        self.cleanup.absorb_producer(&self.runtime, required);
                    }
                    let terminal = denial.recovery_posture()
                        == WorthQueryRequiredOutputRecoveryPosture::Terminal;
                    self.state = FixedState::Outputs(outputs);
                    return if terminal {
                        Advance::Terminal {
                            operation: WorthQueryTerminalFixedOperation::new(self).with_output_completion(
                                <Root::Dependents as super::super::mutation::program_output_continuation::ProgramOutputContinuationFactory<Schema, Program, Demand<Schema, Root>>>::unavailable_completion::<Root>(),
                            ),
                            denial: Denial::Outputs(denial),
                        }
                    } else {
                        Advance::Refused {
                            operation: self,
                            denial: Denial::Outputs(denial),
                        }
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
