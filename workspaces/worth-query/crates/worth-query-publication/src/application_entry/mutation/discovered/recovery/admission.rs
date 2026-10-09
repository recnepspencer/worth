//! Fresh admission of the original discovered-source request, without a handler.

use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIntent, ApplicationMutationScopeResolution,
};
use worth_query_declaration::facade::application_program::{
    ApplicationDiscoveredOutputRoot, ApplicationOutputGraphShape, ApplicationProgramDefinition,
    ApplicationProgramOutputsShape,
};
use worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime;
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationDiscoveredOutputConnection, WorthQueryManagedApplicationRecoveryDenial,
};
use worth_query_installation::facade::ApplicationSchema;

use super::super::{RootConnection, WorthQueryUnpublishedDiscoveredApplicationMutation};
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
    pub(in crate::application_entry::mutation::discovered) fn authorize_discovered_recovery<
        Program,
        Root,
    >(
        &mut self,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        recovery: &WorthQueryUnpublishedDiscoveredApplicationMutation<
            Schema,
            Intent,
            Program,
            Root,
        >,
    ) -> Result<PreparedMutation<Schema, Intent::Binding>, WorthQueryApplicationRecoveryRequestDenial>
    where
        Program: ApplicationProgramDefinition<Schema>,
        Program::Outputs: ApplicationProgramOutputsShape<Schema>,
        Root: ApplicationOutputGraphShape<Schema> + ApplicationDiscoveredOutputRoot,
        RootConnection<Schema, Root>:
            WorthQueryApplicationDiscoveredOutputConnection<Schema, Source = Intent::Binding>,
    {
        if <Intent::Binding as ApplicationMutationBinding<Schema>>::WORKFLOW_CONTROL
            || <Intent::Binding as ApplicationMutationBinding<Schema>>::REQUIRES_WORKFLOW_AUTHORITY
            || self.workflow_transition_identity.is_some()
            || self.workflow_input.is_some()
            || self.workflow_authority.is_some()
        {
            return Err(WorthQueryApplicationRecoveryRequestDenial::WorkflowUnsupported);
        }
        self.check_discovered_recovery_liveness()?;
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
            self,
            &identities,
            staged,
            &selected,
        )?;
        application
            .validate_unpublished_discovered_source::<Root, Intent::Binding>(
                &worth_query_execution::publication_boundary::program_publication_access(),
                &owner,
                &recovery.source,
                prepared.idempotency,
                self.request.branch,
            )
            .map_err(|denial| {
                WorthQueryApplicationRecoveryRequestDenial::Recovery(
                    WorthQueryManagedApplicationRecoveryDenial::Demand(denial),
                )
            })?;
        self.check_discovered_recovery_liveness()?;
        Ok(prepared)
    }

    pub(in crate::application_entry::mutation::discovered) fn check_discovered_recovery_liveness(
        &self,
    ) -> Result<(), WorthQueryApplicationRecoveryRequestDenial> {
        match self.request.scope.interruption() {
            Some(stop) => Err(WorthQueryApplicationRecoveryRequestDenial::Interrupted(
                stop,
            )),
            None => Ok(()),
        }
    }
}
