//! Freshly admitted native source recovery, promotion and retention.
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
    Intent::Binding: worth_query_declaration::facade::application_operation::ApplicationMutationAdmissionBinding<Schema>,
    <Intent::Binding as worth_query_declaration::facade::application_operation::ApplicationMutationAdmissionBinding<Schema>>::Admission: crate::application_entry::mutation::authorization::WorthQueryBindingAdmission<Schema, Intent::Binding>,
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
    pub(super) fn advance_partial_in_phase(
        mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> FixedAdvance<'runtime, Schema, Intent, Program, Root, Action> {
        let FixedState::Unpublished(mut partial) = self.state else {
            unreachable!("source dispatch retains partial custody")
        };
        if partial.recovery_posture() == WorthQueryRequiredOutputRecoveryPosture::Terminal {
            self.state = FixedState::Unpublished(partial);
            return Advance::Terminal {
                operation: WorthQueryTerminalFixedOperation::new(self),
                denial: Denial::RecoveryCarrierUnavailable,
            };
        }
        let progress = {
            if partial.performed().is_none() {
                let outcome = self
                    .description
                    .request_view(request, self.installed_action)
                    .recover_unpublished_required_in_advancement(phase, &mut partial, self.runtime);
                self.state = FixedState::Unpublished(partial);
                match outcome {
                    Ok(progress) => Progress::Recovery(progress),
                    Err(denial) => {
                        return Advance::Refused {
                            operation: self,
                            denial: Denial::Recovery(denial),
                        }
                    }
                }
            } else {
                match self
                    .description
                    .request_view(request, self.installed_action)
                    .promote_recovered_required_outputs_in_advancement(phase, partial, self.runtime)
                {
                    Ok(recovered) => {
                        let (preparation, _initial_cause, performed, cleanup) =
                            recovered.into_parts();
                        self.state = FixedState::Preparation(preparation);
                        self.recovered = Some(performed);
                        self.cleanup.retain_original_failures(cleanup);
                        Progress::SourcePromoted
                    }
                    Err((denial, partial)) => {
                        let terminal = partial.recovery_posture() == WorthQueryRequiredOutputRecoveryPosture::Terminal;
                        self.state = FixedState::Unpublished(partial);
                        if terminal { return Advance::Terminal { operation: WorthQueryTerminalFixedOperation::new(self), denial: Denial::Recovery(denial) }; }
                        return Advance::Refused {
                            operation: self,
                            denial: Denial::Recovery(denial),
                        };
                    }
                }
            }
        };
        Advance::Progress {
            operation: self,
            progress,
        }
    }

    pub(super) fn advance_retention_in_phase(
        mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> FixedAdvance<'runtime, Schema, Intent, Program, Root, Action> {
        let FixedState::Retention(failure) = self.state else {
            unreachable!("source dispatch retains retention custody")
        };
        let progress = {
            match self
                .description
                .request_view(request, self.installed_action)
                .retry_required_output_retention_in_advancement(phase, failure, self.runtime)
            {
                Ok(performed) => {
                    let (result, preparation) = performed.into_preparation();
                    self.result = Some(result);
                    self.state = FixedState::Preparation(preparation);
                    Progress::SourcePerformed
                }
                Err((denial, failure)) => {
                    self.state = FixedState::Retention(failure);
                    return Advance::Refused {
                        operation: self,
                        denial: Denial::Recovery(denial),
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
