//! Discovered-output sources committed under the program their branch selected.

use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIntent, ApplicationMutationScopeResolution,
};
use worth_query_declaration::facade::application_program::{
    ApplicationDiscoveredOutputRoot, ApplicationOutputGraphShape, ApplicationProgramDefinition,
    ApplicationProgramOutputsShape,
};
use worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime;
use worth_query_execution::facade::primary_graph::WorthQueryApplicationDiscoveredOutputConnection;
use worth_query_installation::facade::ApplicationSchema;

use super::{discovered_outcome, RootConnection, WorthQueryApplicationDiscoveredMutationOutcome};
use crate::application_entry::mutation::{
    WorthQueryApplicationMutationRequestWithIdempotency, WorthQueryMutationSourcePrepared,
    WorthQueryPerformedMutationExecutionDenial,
};
use crate::application_entry::WorthQueryApplicationRequestMutationDenial;

impl<'application, 'principal, 'scope, 'key, Schema, Intent>
    WorthQueryApplicationMutationRequestWithIdempotency<
        'application,
        'principal,
        'scope,
        'key,
        Schema,
        Intent,
        WorthQueryMutationSourcePrepared,
    >
where
    Schema: ApplicationSchema + 'static,
    Intent: ApplicationMutationIntent<Schema> + Clone + Send + Sync,
    <Intent::Binding as ApplicationMutationBinding<Schema>>::Input: Clone + Send + Sync,
    <Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<
            Schema,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
        >,
{
    /// Discovered-output counterpart to
    /// [`Self::execute_performed_in_selected_program`], with the same
    /// selection, declaration and not-active-on-occurrence law.
    pub fn execute_performed_discovered_in_selected_program<Program, Root>(
        self,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
    ) -> Result<
        WorthQueryApplicationDiscoveredMutationOutcome<'application, Schema, Intent, Program, Root>,
        WorthQueryPerformedMutationExecutionDenial,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
        Program::Outputs: ApplicationProgramOutputsShape<Schema>,
        Root: ApplicationOutputGraphShape<Schema> + ApplicationDiscoveredOutputRoot,
        RootConnection<Schema, Root>:
            WorthQueryApplicationDiscoveredOutputConnection<Schema, Source = Intent::Binding>,
    {
        crate::application_entry::mutation::performed_source::require_program_output_root::<
            Schema,
            Program,
            Root,
        >(self.request.application, application)?;
        let selected = self
            .request
            .application
            .on_branch(self.request.branch)
            .select()
            .map_err(|selection| {
                WorthQueryPerformedMutationExecutionDenial::Mutation(
                    WorthQueryApplicationRequestMutationDenial::ProductSelection(selection),
                )
            })?;
        let owner = application.selected_program_owner(&selected).map_err(|denial| {
            WorthQueryPerformedMutationExecutionDenial::Mutation(
                crate::application_entry::mutation::selected_program::map_selected_program_owner_denial(denial),
            )
        })?;
        let discovery =
            RootConnection::<Schema, Root>::discovery_from_source(self.request.intent.input())
                .map_err(WorthQueryPerformedMutationExecutionDenial::Connection)?;
        let retained_discovery = discovery.clone();
        let source =
            crate::application_entry::mutation::performed_source::PerformedSourceCommit::default();
        let outcome = self
            .execute_with_preparation_and_commit(
                move |request| {
                    crate::application_entry::mutation::authorization::prepare_selected(
                        request, &selected,
                    )
                },
                |_, program, idempotency| {
                    source.record(
                        application
                            .compare_and_commit_selected_discovered_output_source::<Root, Intent::Binding>(
                                &worth_query_execution::publication_boundary::program_publication_access(),
                                &owner,
                                program,
                                idempotency,
                                retained_discovery,
                            ),
                    )
                },
            )
            .map_err(WorthQueryPerformedMutationExecutionDenial::Mutation)?;
        Ok(discovered_outcome(application, discovery, outcome, source))
    }
}
