//! Freeze correctable legacy input before its handler reads it.
use super::super::WorthQueryApplicationMutationRequestWithIdempotency;
use crate::application_entry::WorthQueryApplicationRequestMutationDenial as Denial;
use worth_query_declaration::facade::{
    application_operation::{
        ApplicationEncodedInput, ApplicationMutationBinding, ApplicationMutationIntent,
    },
    application_schema::ApplicationSchema,
};
use worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase;
use worth_query_installation::facade::InstalledCorrectionAuthority;

impl<Schema, Intent, SourcePreparation>
    WorthQueryApplicationMutationRequestWithIdempotency<
        '_,
        '_,
        '_,
        '_,
        Schema,
        Intent,
        SourcePreparation,
    >
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    <Intent::Binding as ApplicationMutationBinding<Schema>>::Input: Clone + Send + Sync,
{
    pub(super) fn capture_original_input(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<
        Option<
            ApplicationEncodedInput<
                <Intent::Binding as ApplicationMutationBinding<Schema>>::InputBinding,
            >,
        >,
        Denial,
    > {
        if self.operation_input.is_some() || self.workflow_input.is_some() {
            return Ok(None);
        }
        let binding = self
            .request
            .application
            .installed_schema()
            .installed_mutation_binding::<Intent::Binding>()
            .map_err(Denial::BindingInstallation)?;
        let retains_original =
            binding
                .operation()
                .contracts()
                .aftermath()
                .is_some_and(|contract| {
                    contract.authority() != InstalledCorrectionAuthority::NotCorrectable
                });
        if !retains_original {
            return Ok(None);
        }
        self.request.application.capture_operation_input_in_advancement::<<Intent::Binding as ApplicationMutationBinding<Schema>>::InputBinding>(
            phase,
            &worth_query_execution::publication_boundary::program_publication_access(),
            self.request.scope,
            self.request.intent.input(),
        ).map(Some).map_err(Denial::InputCapture)
    }
}
