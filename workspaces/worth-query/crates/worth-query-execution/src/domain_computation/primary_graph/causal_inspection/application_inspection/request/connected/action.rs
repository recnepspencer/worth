//! Independent issued identities and policy preparation feed one common session.
pub(super) use super::super::super::admission::connected::PendingInspectionPolicies;
use super::super::super::{admission, source_original::SourceOriginal};
use super::*;
use crate::domain_computation::primary_graph::{
    application_query::FreshQueryPermissionStop, WorthQueryApplicationBasisSelectionIdentity,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryBinding;
use worth_relational::facade::identity::EntityId;
impl<
        'o,
        Schema: ApplicationSchema,
        Binding: ApplicationQueryBinding<Schema>,
        Disclosure: ApplicationSourceDisclosureBinding<Schema>,
    > WorthQueryConnectedInspectionRequest<'_, '_, '_, 'o, '_, Schema, Binding, Disclosure>
{
    pub(super) fn execute_with(
        self,
        query_pending: impl FnOnce(
            &ExecutionRequest<'_, '_>,
            &WorthQueryInstalledApplicationQueryBinding<Schema, Binding>,
            EntityId,
            EntityId,
            &mut Option<ExecutionMemoryReservation>,
        ) -> Result<PendingInspectionPolicies, InspectionCause>,
        source_pending: impl FnOnce(
            &ExecutionRequest<'_, '_>,
            &WorthQueryInstalledApplicationQueryBinding<Schema, Disclosure::Disclosure>,
            EntityId,
            EntityId,
            &mut Option<ExecutionMemoryReservation>,
        ) -> Result<PendingInspectionPolicies, InspectionCause>,
    ) -> Result<WorthQueryConnectedApplicationInspection<'o>, WorthQueryApplicationInspectionDenial>
    {
        let mut memory = None;
        let result = self.request.open(|execution| {
            let query =
                binding::installed::<Schema, Binding>(self.request.runtime, execution, &mut memory)
                    .map_err(InspectionCause::Binding)?;
            let source_binding = binding::installed::<Schema, Disclosure::Disclosure>(
                self.request.runtime,
                execution,
                &mut memory,
            )
            .map_err(InspectionCause::Binding)?;
            admission::validate_original_affinity(
                self.request.runtime,
                &query,
                self.request.original,
            )
            .map_err(InspectionCause::Admission)?;
            let WorthQueryApplicationBasisSelectionIdentity::Product(original) =
                &self.request.original.source.selection
            else {
                return Err(InspectionCause::Unavailable);
            };
            let selected = self
                .request
                .runtime
                .on_branch(original.product_branch())
                .select_with_request(execution)
                .map_err(InspectionCause::Selection)?;
            let (report, terminal) = self
                .request
                .runtime
                .with_fresh_retained_query_identities(
                    execution,
                    &query,
                    None,
                    self.request.external,
                    &self.request.original.source,
                    &selected,
                    self.request.request,
                    |_, principal, scope| {
                        let principal_id = principal.principal().principal_entity_id();
                        let scope_id = scope.scope().entity_id();
                        let policies =
                            query_pending(execution, &query, principal_id, scope_id, &mut memory)
                                .map_err(FreshQueryPermissionStop::Inspect)?;
                        let query_prepared = admission::connected::prepare_query(
                            self.request.runtime,
                            execution,
                            &query,
                            &selected,
                            self.request.request,
                            principal,
                            scope,
                            self.request.original,
                            policies,
                            &mut memory,
                        )
                        .map_err(|cause| {
                            FreshQueryPermissionStop::Inspect(InspectionCause::Admission(cause))
                        })?;
                        let source = SourceOriginal::from_receipt::<Schema, Disclosure>(
                            self.request.runtime,
                            &source_binding,
                            self.source,
                        )
                        .map_err(|cause| {
                            FreshQueryPermissionStop::Inspect(InspectionCause::Admission(cause))
                        })?;
                        self.request.runtime.with_fresh_inspection_scope_identities(
                            execution,
                            &source_binding,
                            None,
                            self.request.external,
                            source.disclosure.scope(),
                            &selected,
                            self.request.request,
                            |_, source_principal, source_scope| {
                                let inspect = || -> Result<_, InspectionCause> {
                                    let policies = source_pending(
                                        execution,
                                        &source_binding,
                                        source_principal.principal().principal_entity_id(),
                                        source_scope.scope().entity_id(),
                                        &mut memory,
                                    )?;
                                    let mut read = admission::connected::prepare::<
                                        Schema,
                                        Binding,
                                        Disclosure::Disclosure,
                                    >(
                                        self.request.runtime,
                                        execution,
                                        &source_binding,
                                        &selected,
                                        self.request.request,
                                        principal,
                                        scope,
                                        source_principal,
                                        source_scope,
                                        self.request.original,
                                        &source,
                                        query_prepared,
                                        policies,
                                        &mut memory,
                                    )
                                    .map_err(InspectionCause::Admission)?;
                                    let report = read
                                        .materialize_connected(
                                            execution,
                                            query.query(),
                                            source_binding.query(),
                                            self.source,
                                        )
                                        .map_err(InspectionCause::Admission)?;
                                    let terminal = read
                                        .terminal(execution)
                                        .map_err(InspectionCause::Admission)?;
                                    Ok((report, terminal))
                                };
                                inspect().map_err(FreshQueryPermissionStop::Inspect)
                            },
                        )
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
            drop(query);
            drop(source_binding);
            let completed = terminal
                .complete(execution, basis.release())
                .map_err(InspectionCause::Admission)?;
            completed
                .publish_connected(execution, self.request.original, report, &mut memory)
                .map_err(InspectionCause::Admission)
        });
        match result {
            Ok(prepared) => Ok(prepared.fund(memory)),
            Err(cause) => Err(WorthQueryApplicationInspectionDenial::new(cause, memory)),
        }
    }
}
