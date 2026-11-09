//! Reserve deferred fixed completion without manufacturing its postponed ingress.
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

    /// Deferred fixed completion requires the postponed ingress in public #120.
    /// Ordinary synchronous refusal has no saved-source unavailable evidence.
    pub fn output_unavailable(
        &self,
    ) -> Result<
        Option<&WorthQueryApplicationProgramOutputUnavailable>,
        WorthQueryOperationCloseDenial,
    > {
        match &self.output_completion {
            Some(TerminalOutputCompletion::Unavailable(evidence)) => Ok(Some(evidence)),
            _ => Err(deferred_ingress_unsupported()),
        }
    }

    pub(super) fn complete_unavailable_in_phase(
        &mut self,
        _phase: &WorthQueryAdvancementPhase<'_>,
        _request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<bool, WorthQueryOperationCloseDenial> {
        if matches!(
            self.output_completion,
            Some(TerminalOutputCompletion::Pending(_))
        ) {
            return Err(deferred_ingress_unsupported());
        }
        Ok(matches!(
            self.output_completion,
            Some(TerminalOutputCompletion::Unavailable(_))
        ))
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

fn deferred_ingress_unsupported() -> WorthQueryOperationCloseDenial {
    WorthQueryOperationCloseDenial::DeferredFixedCompletionUnsupported {
        issue: "https://github.com/recnepspencer/worth/issues/120",
    }
}
