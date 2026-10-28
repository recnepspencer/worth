//! Fresh guards end before the actual Product and consuming basis terminal.
use super::*;
use crate::domain_computation::{
    authorization::application_disclosure::WorthQueryPendingApplicationQueryGovernance,
    primary_graph::{
        application_query::FreshQueryPermissionStop, WorthQueryApplicationBasisSelectionIdentity,
    },
};
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryBinding;
use worth_relational::facade::identity::EntityId;

impl<'o, Schema: ApplicationSchema, Binding: ApplicationQueryBinding<Schema>>
    WorthQueryApplicationInspectionRequest<'_, '_, '_, 'o, Schema, Binding>
{
    pub(super) fn execute_with(
        self,
        pending: impl FnOnce(
            &ExecutionRequest<'_, '_>,
            &WorthQueryInstalledApplicationQueryBinding<Schema, Binding>,
            EntityId,
            EntityId,
            &mut Option<ExecutionMemoryReservation>,
        ) -> Result<
            Option<WorthQueryPendingApplicationQueryGovernance>,
            InspectionCause,
        >,
    ) -> Result<WorthQueryObservedApplicationInspection<'o>, WorthQueryApplicationInspectionDenial>
    {
        // This sole owner outlives callback temporaries, every inner error, and
        // the advancement handoff. Success and denial move exactly this ticket.
        let mut memory = None;
        let result = self.open(|execution| {
            let installed =
                binding::installed::<Schema, Binding>(self.runtime, execution, &mut memory)
                    .map_err(InspectionCause::Binding)?;
            super::super::admission::validate_original_affinity(
                self.runtime,
                &installed,
                self.original,
            )
            .map_err(InspectionCause::Admission)?;
            let branch = match &self.original.source.selection {
                WorthQueryApplicationBasisSelectionIdentity::Product(original) => {
                    original.product_branch()
                }
                WorthQueryApplicationBasisSelectionIdentity::Relational => {
                    return Err(InspectionCause::Unavailable)
                }
            };
            let selected = self
                .runtime
                .on_branch(branch)
                .select_with_request(execution)
                .map_err(InspectionCause::Selection)?;
            let storage = &mut memory;
            let (report, terminal) = self
                .runtime
                .with_fresh_retained_query_identities(
                    execution,
                    &installed,
                    None,
                    self.external,
                    &self.original.source,
                    &selected,
                    self.request,
                    |_, principal, scope| {
                        let pending = pending(
                            execution,
                            &installed,
                            principal.principal().principal_entity_id(),
                            scope.scope().entity_id(),
                            storage,
                        )
                        .map_err(FreshQueryPermissionStop::Inspect)?;
                        let mut read = super::super::admission::prepare(
                            self.runtime,
                            execution,
                            &installed,
                            &selected,
                            principal,
                            scope,
                            self.request,
                            self.original,
                            pending,
                            storage,
                        )
                        .map_err(|cause| {
                            FreshQueryPermissionStop::Inspect(InspectionCause::Admission(cause))
                        })?;
                        let report = read.materialize(execution).map_err(|cause| {
                            FreshQueryPermissionStop::Inspect(InspectionCause::Admission(cause))
                        })?;
                        let terminal = read.terminal(execution).map_err(|cause| {
                            FreshQueryPermissionStop::Inspect(InspectionCause::Admission(cause))
                        })?;
                        Ok((report, terminal))
                    },
                )
                .map_err(|cause| match cause {
                    FreshQueryPermissionStop::Principal(cause) => InspectionCause::Principal(cause),
                    FreshQueryPermissionStop::Scope(cause) => InspectionCause::Scope(cause),
                    FreshQueryPermissionStop::Query(cause) => InspectionCause::Query(cause),
                    FreshQueryPermissionStop::Inspect(cause) => cause,
                })?;
            let (_, product, basis) = selected.into_parts();
            drop(product);
            drop(installed);
            let completed = terminal
                .complete(execution, basis.release())
                .map_err(InspectionCause::Admission)?;
            completed
                .publish(execution, self.original, report, &mut memory)
                .map_err(InspectionCause::Admission)
        });
        Self::finish(result, memory)
    }
}
