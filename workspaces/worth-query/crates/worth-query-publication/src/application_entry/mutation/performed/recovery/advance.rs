//! Native advancement of a retained required-source partial.

use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIntent, ApplicationMutationScopeResolution,
};
use worth_query_declaration::facade::application_program::{
    ApplicationOutputGraphShape, ApplicationProgramDefinition, ApplicationProgramOutputsShape,
    ApplicationRequiredOutputRoot,
};
use worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime;
use worth_query_execution::facade::primary_graph::WorthQueryApplicationRequiredOutputConnection;
use worth_query_installation::facade::ApplicationSchema;

use super::super::{RootConnection, WorthQueryUnpublishedRequiredApplicationMutation};
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
    /// Advances only the original required source under fresh authorization.
    /// Refusal and no effect leave its exact partial unchanged. A performed
    /// result retains every read/publication/cleanup result for promotion.
    pub fn recover_unpublished_required_in_program<Program, Root>(
        mut self,
        recovery: &mut WorthQueryUnpublishedRequiredApplicationMutation<
            Schema,
            Intent,
            Program,
            Root,
        >,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
    ) -> Result<
        crate::application_entry::WorthQueryProgramSourceRecoveryProgress,
        WorthQueryApplicationRecoveryRequestDenial,
    >
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
        let prepared =
            self.authorize_required_source::<Program, Root>(application, &recovery.source)?;
        recovery
            .phase
            .advance(
                self.request.application,
                &prepared.admission,
                prepared.idempotency,
                &mut recovery.prior_cleanup,
            )
            .map_err(WorthQueryApplicationRecoveryRequestDenial::Recovery)
    }
}
