//! Preserve genuine negative completion independently of later cleanup retry.
use super::*;
use crate::application_entry::mutation::program_output_continuation::ProgramOutputUnavailableCompletion;

pub(super) enum TerminalOutputCompletion<Schema, Program, Root>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema> + ApplicationRequiredOutputRoot,
    Connection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
{
    Pending(ProgramOutputUnavailableCompletion<Schema, Program, Root>),
    Unavailable(WorthQueryApplicationProgramOutputUnavailable),
}

impl<Schema, Intent, Program, Root, Action>
    WorthQueryTerminalFixedOperation<Schema, Intent, Program, Root, Action>
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
{
    pub(in crate::application_entry::operation) fn with_output_completion(
        mut self,
        completion: Option<ProgramOutputUnavailableCompletion<Schema, Program, Root>>,
    ) -> Self {
        self.output_completion = completion.map(TerminalOutputCompletion::Pending);
        self
    }

    /// Native unavailable evidence survives a refused cleanup or close attempt.
    pub fn output_unavailable(&self) -> Option<&WorthQueryApplicationProgramOutputUnavailable> {
        match &self.output_completion {
            Some(TerminalOutputCompletion::Unavailable(evidence)) => Some(evidence),
            _ => None,
        }
    }

    pub(super) fn complete_unavailable_in_phase(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<bool, WorthQueryOperationCloseDenial> {
        if let Some(TerminalOutputCompletion::Pending(complete)) = &self.output_completion {
            let FixedState::Outputs(outputs) = &mut self.operation.state else {
                return Err(WorthQueryOperationCloseDenial::NativeObligationRetained);
            };
            let evidence = complete(outputs, phase, &self.operation.runtime, request)
                .map_err(WorthQueryOperationCloseDenial::Outputs)?;
            // The native transition is not replayed if subsequent cleanup refuses.
            self.output_completion = Some(TerminalOutputCompletion::Unavailable(evidence));
        }
        Ok(self.output_unavailable().is_some())
    }

    pub(super) fn take_output_unavailable(
        &mut self,
    ) -> Option<WorthQueryApplicationProgramOutputUnavailable> {
        match self.output_completion.take() {
            Some(TerminalOutputCompletion::Unavailable(evidence)) => Some(evidence),
            _ => None,
        }
    }
}
