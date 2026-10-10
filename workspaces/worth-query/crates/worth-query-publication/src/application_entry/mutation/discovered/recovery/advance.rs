//! Native advancement of a retained discovered-source partial.

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

use super::super::{
    RootConnection, WorthQueryDiscoveredRecoveryProgress,
    WorthQueryUnpublishedDiscoveredApplicationMutation,
};
use crate::application_entry::mutation::{
    WorthQueryApplicationMutationRequestWithIdempotency, WorthQueryApplicationRecoveryRequestDenial,
};

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
    /// Advances only the original discovered source under fresh authorization.
    /// Refusal and no effect leave its exact partial unchanged. A performed
    /// result retains every read/publication/cleanup result for promotion.
    pub fn recover_unpublished_discovered_in_program<Program, Root>(
        mut self,
        recovery: &mut WorthQueryUnpublishedDiscoveredApplicationMutation<
            Schema,
            Intent,
            Program,
            Root,
        >,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
    ) -> Result<WorthQueryDiscoveredRecoveryProgress, WorthQueryApplicationRecoveryRequestDenial>
    where
        Program: ApplicationProgramDefinition<Schema>,
        Program::Outputs: ApplicationProgramOutputsShape<Schema>,
        Root: ApplicationOutputGraphShape<Schema> + ApplicationDiscoveredOutputRoot,
        RootConnection<Schema, Root>:
            WorthQueryApplicationDiscoveredOutputConnection<Schema, Source = Intent::Binding>,
    {
        let runtime = self.request.application;
        let scope = self.request.scope.clone();
        runtime
            .with_application_advancement(&scope, |phase| {
                let prepared = self.authorize_discovered_source::<Program, Root>(
                    &phase,
                    application,
                    &recovery.source,
                )?;
                recovery
                    .phase
                    .advance(
                        &phase,
                        self.request.application,
                        &prepared.admission,
                        prepared.idempotency,
                        &mut recovery.prior_cleanup,
                    )
                    .map_err(WorthQueryApplicationRecoveryRequestDenial::Recovery)
            })
            .map_err(WorthQueryApplicationRecoveryRequestDenial::advancement)?
    }
}
