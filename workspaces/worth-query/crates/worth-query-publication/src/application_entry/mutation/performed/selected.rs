//! Performed sources committed under the program their branch selected.

use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIntent, ApplicationMutationScopeResolution,
};
use worth_query_declaration::facade::application_program::{
    ApplicationOutputGraphShape, ApplicationProgramDefinition, ApplicationProgramOutputsShape,
    ApplicationRequiredOutputRoot,
};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution,
};
use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
use worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime;
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationRequiredOutputConnection,
    WorthQueryApplicationRequiredOutputSource,
};
use worth_query_installation::facade::ApplicationSchema;

use super::{
    performed_outcome, DemandSource, RootConnection, WorthQueryApplicationPerformedMutationOutcome,
    WorthQueryPerformedMutationExecutionDenial,
};
use crate::application_entry::mutation::{
    WorthQueryApplicationMutationRequestWithIdempotency, WorthQueryMutationSourcePrepared,
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
    /// Performs this source under the program carried by the request's exact
    /// branch, so a branch that adopted a rostered successor commits its
    /// source and starts its required outputs under that successor.
    ///
    /// The selected program must declare this source and `Root`, and the
    /// host's initial output shape must declare `Root`, since the returned
    /// custody is typed by that shape. A branch whose program changes after
    /// selection is refused at commit as not active on its occurrence.
    pub fn execute_performed_in_selected_program<Program, Root>(
        self,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
    ) -> Result<
        WorthQueryApplicationPerformedMutationOutcome<'application, Schema, Intent, Program, Root>,
        WorthQueryPerformedMutationExecutionDenial,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
        Program::Outputs: ApplicationProgramOutputsShape<Schema>,
        Root: ApplicationOutputGraphShape<Schema> + ApplicationRequiredOutputRoot,
        RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
        Intent::Binding:
            WorthQueryApplicationRequiredOutputSource<Schema, RootConnection<Schema, Root>>,
        <DemandSource<Schema, Root> as ApplicationQueryBinding<Schema>>::Input:
            ApplicationQueryIntent<Schema, Binding = DemandSource<Schema, Root>>,
        <DemandSource<Schema, Root> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                Schema,
                <DemandSource<Schema, Root> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >,
        <<DemandSource<Schema, Root> as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value:
            WorthQueryApplicationProjection<
                    Schema,
                    <DemandSource<Schema, Root> as ApplicationQueryBinding<Schema>>::Query,
                > + Clone,
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
        let demand = <Intent::Binding as WorthQueryApplicationRequiredOutputSource<
            Schema,
            RootConnection<Schema, Root>,
        >>::demand_from_source(self.request.intent.input())
        .map_err(WorthQueryPerformedMutationExecutionDenial::Connection)?;
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
                            .compare_and_commit_selected_required_output_source::<Root, Intent::Binding>(
                                &worth_query_execution::publication_boundary::program_publication_access(),
                                &owner,
                                program,
                                idempotency,
                            ),
                    )
                },
            )
            .map_err(WorthQueryPerformedMutationExecutionDenial::Mutation)?;
        Ok(performed_outcome(application, demand, outcome, source))
    }
}
