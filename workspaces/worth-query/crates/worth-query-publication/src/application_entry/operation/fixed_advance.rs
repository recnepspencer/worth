//! One fixed operation advance delegates one admitted native phase.
use super::fixed::FixedState;
use super::*;
use WorthQueryOperationAdvance as Advance;
use WorthQueryOperationAdvanceDenial as Denial;

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
    pub fn advance(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> FixedAdvance<'runtime, Schema, Intent, Program, Root, Action> {
        if !std::ptr::eq(self.runtime.runtime(), request.application)
            || self.description.branch != request.branch
        {
            return Advance::Refused {
                operation: self,
                denial: Denial::FreshRequestMismatch,
            };
        }
        let runtime = self.runtime.runtime();
        let mut retained = Some(self);
        match runtime.with_application_advancement(request.scope, |phase| {
            retained
                .take()
                .expect("one admitted advance owns its operation")
                .advance_in_phase(&phase, request)
        }) {
            Ok(outcome) => outcome,
            Err(cause) => Advance::Refused {
                operation: retained.expect("refused opening retains the operation"),
                denial: Denial::Execution(cause),
            },
        }
    }

    fn advance_in_phase(
        self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> FixedAdvance<'runtime, Schema, Intent, Program, Root, Action> {
        if !matches!(
            self.state,
            FixedState::Initial | FixedState::Blocked(_) | FixedState::NotPerformed(_)
        ) {
            let admission = (|| {
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
                    .resolve_declared_action::<Binding<Schema, Intent>>(self.installed_action)
                    .map_err(Denial::Action)?;
                Ok::<_, Denial>(())
            })();
            if let Err(denial) = admission {
                return Advance::Refused {
                    operation: self,
                    denial,
                };
            }
        }
        match &self.state {
            FixedState::Initial => self.advance_initial_in_phase(phase, request),
            FixedState::Replay(_) => self.advance_replay_in_phase(phase, request),
            FixedState::Unpublished(_) => self.advance_partial_in_phase(phase, request),
            FixedState::Retention(_) => self.advance_retention_in_phase(phase, request),
            FixedState::Preparation(_) => self.advance_preparation_in_phase(phase, request),
            FixedState::Outputs(_) => self.advance_outputs_in_phase(phase, request),
            FixedState::Blocked(_) | FixedState::NotPerformed(_) => Advance::Terminal {
                operation: WorthQueryTerminalFixedOperation::new(self),
                denial: Denial::NativeTerminal,
            },
        }
    }
}
