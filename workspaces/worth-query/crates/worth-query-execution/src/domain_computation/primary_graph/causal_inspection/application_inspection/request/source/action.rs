//! Source and query inspection enter the same current READ/session/terminal owner.
use super::super::super::{
    admission::{self, InspectionSubject},
    source_original::SourceOriginal,
};
use super::*;
use crate::domain_computation::{
    authorization::application_disclosure::WorthQueryPendingApplicationQueryGovernance,
    primary_graph::application_query::FreshQueryPermissionStop,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryBinding;
use worth_relational::facade::identity::EntityId;
impl<Schema: ApplicationSchema, Disclosure: ApplicationSourceDisclosureBinding<Schema>>
    WorthQuerySourceInspectionRequest<'_, '_, '_, '_, Schema, Disclosure>
{
    pub(super) fn execute_with(
        self,
        pending: impl FnOnce(
            &ExecutionRequest<'_, '_>,
            &WorthQueryInstalledApplicationQueryBinding<Schema, Disclosure::Disclosure>,
            EntityId,
            EntityId,
            &mut Option<ExecutionMemoryReservation>,
        ) -> Result<
            Option<WorthQueryPendingApplicationQueryGovernance>,
            InspectionCause,
        >,
    ) -> Result<WorthQueryPerformedSourceInspection, WorthQueryApplicationInspectionDenial> {
        let mut memory = None;
        let outcome = funding::open(
            self.runtime,
            self.request,
            self.allocation_policy,
            |execution| {
                let installed = binding::installed::<Schema, Disclosure::Disclosure>(
                    self.runtime,
                    execution,
                    &mut memory,
                )
                .map_err(InspectionCause::Binding)?;
                let original = SourceOriginal::from_receipt::<Schema, Disclosure>(
                    self.runtime,
                    &installed,
                    self.source,
                )
                .map_err(InspectionCause::Admission)?;
                let selected = self
                    .runtime
                    .on_branch(original.publication.product_branch())
                    .select_with_request(execution)
                    .map_err(InspectionCause::Selection)?;
                let storage = &mut memory;
                let (report, terminal) = self
                    .runtime
                    .with_fresh_inspection_scope_identities(
                        execution,
                        &installed,
                        None,
                        self.external,
                        original.disclosure.scope(),
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
                            let mut read = admission::prepare(
                                self.runtime,
                                execution,
                                &installed,
                                &selected,
                                principal,
                                scope,
                                self.request,
                                InspectionSubject::Source(&original),
                                pending,
                                None,
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
                        FreshQueryPermissionStop::Principal(cause) => {
                            InspectionCause::Principal(cause)
                        }
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
                    .publish_source(execution, original, report, &mut memory)
                    .map_err(InspectionCause::Admission)
            },
        );
        match outcome {
            Ok(prepared) => Ok(prepared.fund(memory)),
            Err(cause) => Err(WorthQueryApplicationInspectionDenial::new(cause, memory)),
        }
    }
}
