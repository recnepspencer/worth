//! Reserve deferred fixed completion without manufacturing its postponed ingress.
use super::*;
use crate::application_entry::mutation::program_output_continuation::ProgramOutputUnavailableCompletion;

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
        self.output_completion = completion;
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
        Err(deferred_ingress_unsupported())
    }

    pub(super) fn complete_unavailable_in_phase(
        &mut self,
        _phase: &WorthQueryAdvancementPhase<'_>,
        _request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<bool, WorthQueryOperationCloseDenial> {
        // A reserved completion stays pending until its ingress exists. No
        // native unavailable completion is produced here.
        if self.output_completion.is_some() {
            return Err(deferred_ingress_unsupported());
        }
        Ok(false)
    }
}

fn deferred_ingress_unsupported() -> WorthQueryOperationCloseDenial {
    WorthQueryOperationCloseDenial::DeferredFixedCompletionUnsupported {
        issue: "https://github.com/recnepspencer/worth/issues/120",
    }
}
