//! Fresh admission of the original required-source request, without a handler.

use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIntent, ApplicationMutationScopeResolution,
};
use worth_query_declaration::facade::application_program::{
    ApplicationOutputGraphShape, ApplicationProgramDefinition, ApplicationProgramOutputsShape,
    ApplicationRequiredOutputRoot,
};
use worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime;
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationRequiredOutputConnection, WorthQueryManagedApplicationRecoveryDenial,
};
use worth_query_installation::facade::ApplicationSchema;

use super::super::RootConnection;
use crate::application_entry::mutation::authorization::PreparedMutation;
use crate::application_entry::mutation::{
    WorthQueryApplicationMutationRequestWithIdempotency, WorthQueryApplicationRecoveryRequestDenial,
};
use crate::application_entry::WorthQueryApplicationRequestMutationDenial;

impl<'application, 'principal, 'scope, 'key, Schema, Intent, SourcePreparation>
    WorthQueryApplicationMutationRequestWithIdempotency<
        'application,
        'principal,
        'scope,
        'key,
        Schema,
        Intent,
        SourcePreparation,
    >
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    <Intent::Binding as ApplicationMutationBinding<Schema>>::Input: Clone + Send + Sync,
    <Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<
            Schema,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
        >,
{
    pub(in crate::application_entry::mutation::performed) fn authorize_required_source<
        Program,
        Root,
    >(
        &mut self,
        phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<
            '_,
        >,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        source: &worth_query_execution::facade::application_installation::WorthQueryUnpublishedProgramOutputSource,
    ) -> Result<PreparedMutation<Schema, Intent::Binding>, WorthQueryApplicationRecoveryRequestDenial>
    where
        Program: ApplicationProgramDefinition<Schema>,
        Program::Outputs: ApplicationProgramOutputsShape<Schema>,
        Root: ApplicationOutputGraphShape<Schema> + ApplicationRequiredOutputRoot,
        RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
        Intent::Binding:
            worth_query_execution::facade::primary_graph::WorthQueryApplicationRequiredOutputSource<
                Schema,
                RootConnection<Schema, Root>,
            >,
    {
        self.request
            .application
            .validate_application_advancement(phase)
            .map_err(|cause| {
                WorthQueryApplicationRecoveryRequestDenial::advancement(cause.into())
            })?;

        if <Intent::Binding as ApplicationMutationBinding<Schema>>::WORKFLOW_CONTROL
            || <Intent::Binding as ApplicationMutationBinding<Schema>>::REQUIRES_WORKFLOW_AUTHORITY
            || self.workflow_transition_identity.is_some()
            || self.workflow_input.is_some()
            || self.workflow_authority.is_some()
        {
            return Err(WorthQueryApplicationRecoveryRequestDenial::WorkflowUnsupported);
        }
        self.check_recovery_liveness()?;
        if !std::ptr::eq(application.runtime(), self.request.application) {
            return Err(
                WorthQueryApplicationRequestMutationDenial::ApplicationProgramMismatch.into(),
            );
        }
        let selected = self
            .request
            .application
            .on_branch(self.request.branch)
            .select()
            .map_err(WorthQueryApplicationRequestMutationDenial::ProductSelection)?;
        let owner = application.selected_program_owner(&selected).map_err(
            crate::application_entry::mutation::selected_program::map_selected_program_owner_denial,
        )?;
        let staged = self.stage()?;
        let identities = self.identities()?;
        // Only bind the original source proof. The retained native attempt owns
        // its original facts; pending_source must not author another candidate.
        let prepared = crate::application_entry::mutation::authorization::prepare_selected(
            phase,
            self,
            &identities,
            staged,
            &selected,
        )?;
        application
            .validate_unpublished_required_source::<Root, Intent::Binding>(
                &worth_query_execution::publication_boundary::program_publication_access(),
                &owner,
                source,
                prepared.idempotency,
                self.request.branch,
            )
            .map_err(|denial| {
                WorthQueryApplicationRecoveryRequestDenial::Recovery(
                    WorthQueryManagedApplicationRecoveryDenial::Demand(denial),
                )
            })?;
        self.check_recovery_liveness()?;
        Ok(prepared)
    }
}
