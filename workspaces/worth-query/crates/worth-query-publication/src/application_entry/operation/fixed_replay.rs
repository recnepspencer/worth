//! Registered replay delegates receipt-bound native demand recovery without a source effect.
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
    Intent::Binding: worth_query_declaration::facade::application_operation::ApplicationMutationAdmissionBinding<Schema>,
    <Intent::Binding as worth_query_declaration::facade::application_operation::ApplicationMutationAdmissionBinding<Schema>>::Admission: crate::application_entry::mutation::authorization::WorthQueryBindingAdmission<Schema, Intent::Binding>,
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
    pub(super) fn advance_replay_in_phase(
        mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> FixedAdvance<Schema, Intent, Program, Root, Action> {
        use worth_query_execution::facade::primary_graph::{
            WorthQueryApplicationIdempotencyResolution as Resolution,
            WorthQueryManagedApplicationRecoveryDenial as RecoveryDenial,
        };
        use crate::application_entry::request::program_outputs::WorthQueryAdmittedRequiredOutputRecoveryDenial as Recovery;
        let original = match &self.state {
            FixedState::ReplayRetained(receipt) => worth_query_execution::facade::primary_graph::WorthQueryOutputSourceReplay::Retained(receipt),
            FixedState::ReplayRecorded(receipt) => worth_query_execution::facade::primary_graph::WorthQueryOutputSourceReplay::Recorded(receipt),
            _ => unreachable!("dispatch owns original replay evidence"),
        };
        let admission = (|| {
            let mut native = self.description.request_view(request, &self.installed_action);
            let prepared = native.authorize_fixed_source::<Program, Root>(phase, &self.runtime, None)?;
            let read = native.read_recovery_idempotency(phase, &prepared)?;
            let receipt = match read.into_resolution() {
                Resolution::RecoveredCommitted(receipt) => receipt,
                Resolution::ReleasedOriginal(original) => return Err(WorthQueryApplicationRecoveryRequestDenial::ReleasedOriginal(original)),
                Resolution::ProductUnpublished(_) | Resolution::ProductPreparationPending(_)
                | Resolution::IntentDrift | Resolution::Unseen => return Err(WorthQueryApplicationRecoveryRequestDenial::Recovery(RecoveryDenial::BindingMismatch)),
            };
            if !original.matches_recorded(&receipt) {
                return Err(WorthQueryApplicationRecoveryRequestDenial::Recovery(
                    RecoveryDenial::BindingMismatch,
                ));
            }
            Ok((prepared, original))
        })();
        let (prepared, receipt) = match admission {
            Ok(admitted) => admitted,
            Err(cause) => return Advance::Refused {
                operation: self,
                denial: Denial::Recovery(cause),
            },
        };
        let demand = match <Binding<Schema, Intent> as WorthQueryApplicationRequiredOutputSource<Schema, Connection<Schema, Root>>>::demand_from_source(self.description.intent.input()) {
            Ok(demand) => demand,
            Err(cause) => return Advance::Refused {
                operation: self,
                denial: Denial::Mutation(WorthQueryPerformedMutationExecutionDenial::Connection(cause)),
            },
        };
        match request.recover_admitted_required_outputs_in_advancement::<Program, Root, _, _, _>(
            phase, &self.runtime, receipt, demand,
            WorthQueryOutputDemandControls::host_policy(), &prepared.admission,
        ) {
            Ok(outputs) => {
                self.state = FixedState::Outputs(outputs);
                Advance::Progress { operation: self, progress: Progress::RegisteredOutputsRecovered }
            }
            Err(Recovery::Authorization(cause)) => Advance::Refused {
                operation: self,
                denial: Denial::Recovery(WorthQueryApplicationRecoveryRequestDenial::Recovery(
                    RecoveryDenial::Authorization(cause),
                )),
            },
            Err(Recovery::Outputs(cause)) => {
                if cause.recovery_posture() == WorthQueryRequiredOutputRecoveryPosture::Terminal {
                    self.state = FixedState::NotPerformed(match receipt {
                        worth_query_execution::facade::primary_graph::WorthQueryOutputSourceReplay::Retained(receipt) => WorthQueryApplicationMutationOutcome::AlreadyCommitted(receipt.clone()),
                        worth_query_execution::facade::primary_graph::WorthQueryOutputSourceReplay::Recorded(receipt) => WorthQueryApplicationMutationOutcome::RecoveredCommitted(receipt.clone()),
                    });
                    Advance::Terminal {
                        operation: WorthQueryTerminalFixedOperation::new(self),
                        denial: Denial::Outputs(cause),
                    }
                } else {
                    Advance::Refused { operation: self, denial: Denial::Outputs(cause) }
                }
            }
        }
    }
}
