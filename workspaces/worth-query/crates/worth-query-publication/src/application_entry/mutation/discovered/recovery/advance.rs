//! Native advancement of a retained discovered-source partial.

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
    WorthQueryManagedApplicationRecoveryOutcome,
};
use worth_query_installation::facade::ApplicationSchema;

use super::super::unpublished::{DiscoveredRecoveryPhase, RecoveredCarrier};
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
        let prepared = self.authorize_discovered_recovery(application, recovery)?;
        let partial = match &recovery.phase {
            DiscoveredRecoveryPhase::Unpublished(partial) => partial,
            DiscoveredRecoveryPhase::Performed { .. } => {
                return Ok(WorthQueryDiscoveredRecoveryProgress::Performed)
            }
        };
        // A native successor may return one independently failed prior cleanup.
        // Reserve its storage before World can move; never allocate aftereffect
        // merely to keep the returned mandatory owner alive.
        recovery.prior_cleanup.try_reserve(1).map_err(|_| {
            WorthQueryApplicationRecoveryRequestDenial::Recovery(
                WorthQueryManagedApplicationRecoveryDenial::ProviderCapacity,
            )
        })?;
        let (outcome, carrier) = self
            .request
            .application
            .recover_admitted_unpublished_discovered_source(
                &worth_query_execution::publication_boundary::program_publication_access(),
                partial,
                &prepared.admission,
                prepared.idempotency,
            )
            .map_err(WorthQueryApplicationRecoveryRequestDenial::Recovery)?;
        Ok(match outcome {
            WorthQueryManagedApplicationRecoveryOutcome::NoEffect => {
                WorthQueryDiscoveredRecoveryProgress::NoEffect
            }
            WorthQueryManagedApplicationRecoveryOutcome::ProductUnpublished {
                next,
                prior_cleanup_failure,
            } => {
                recovery.phase = DiscoveredRecoveryPhase::Unpublished(next);
                if let Some(failure) = prior_cleanup_failure {
                    recovery.prior_cleanup.push(failure);
                }
                WorthQueryDiscoveredRecoveryProgress::ProductUnpublished
            }
            WorthQueryManagedApplicationRecoveryOutcome::Performed(outcome) => {
                recovery.phase = DiscoveredRecoveryPhase::Performed {
                    carrier: match carrier {
                        Some(carrier) => RecoveredCarrier::Retained(carrier),
                        None => RecoveredCarrier::Unavailable,
                    },
                    outcome,
                };
                WorthQueryDiscoveredRecoveryProgress::Performed
            }
        })
    }
}
