//! Source and query inspection enter the same current READ/session/terminal owner.
use super::super::super::{
    admission::{self, InspectionDiagnosticPreparation, InspectionSubject},
    source_original::SourceOriginal,
};
use super::*;
use crate::domain_computation::{
    authorization::application_disclosure::WorthQueryPendingApplicationQueryGovernance,
    primary_graph::application_query::FreshQueryPermissionStop,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryBinding;
use worth_relational::facade::identity::EntityId;
enum SourceMaterialization {
    Reference(super::super::super::report::FundedInspectionReport),
    Rich(super::super::super::report::rich::FundedRichInspectionReport),
}
enum PreparedSourceOutcome {
    Reference(super::super::super::result::PreparedSourceInspection),
    Rich(super::super::super::result::PreparedMaterializedSourceInspection),
}
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
        let (prepared, memory) = self.execute_inner(None, |e, q, p, s, m| {
            pending(e, q, p, s, m).map(|metadata| (metadata, None))
        })?;
        match prepared {
            PreparedSourceOutcome::Reference(prepared) => Ok(prepared.fund(memory)),
            _ => Err(WorthQueryApplicationInspectionDenial::new(
                InspectionCause::Admission(
                    admission::InspectionAdmissionDenial::ForeignIssuedAccess,
                ),
                memory,
            )),
        }
    }
    pub(super) fn execute_rich_with(
        self,
        detail: super::super::super::rich::WorthQueryApplicationMaterializationRequest,
        pending: impl FnOnce(
            &ExecutionRequest<'_, '_>,
            &WorthQueryInstalledApplicationQueryBinding<Schema, Disclosure::Disclosure>,
            EntityId,
            EntityId,
            &mut Option<ExecutionMemoryReservation>,
        ) -> Result<
            (
                Option<WorthQueryPendingApplicationQueryGovernance>,
                WorthQueryPendingApplicationQueryGovernance,
            ),
            InspectionCause,
        >,
    ) -> Result<
        super::super::super::result::WorthQueryMaterializedPerformedSourceInspection,
        WorthQueryApplicationInspectionDenial,
    > {
        let (prepared, memory) = self.execute_inner(Some(detail), |e, q, p, s, m| {
            pending(e, q, p, s, m).map(|(metadata, diagnostic)| (metadata, Some(diagnostic)))
        })?;
        match prepared {
            PreparedSourceOutcome::Rich(prepared) => Ok(prepared.fund(memory)),
            _ => Err(WorthQueryApplicationInspectionDenial::new(
                InspectionCause::Admission(
                    admission::InspectionAdmissionDenial::ForeignIssuedAccess,
                ),
                memory,
            )),
        }
    }
    fn execute_inner(
        self,
        detail: Option<super::super::super::rich::WorthQueryApplicationMaterializationRequest>,
        pending: impl FnOnce(
            &ExecutionRequest<'_, '_>,
            &WorthQueryInstalledApplicationQueryBinding<Schema, Disclosure::Disclosure>,
            EntityId,
            EntityId,
            &mut Option<ExecutionMemoryReservation>,
        ) -> Result<
            (
                Option<WorthQueryPendingApplicationQueryGovernance>,
                Option<WorthQueryPendingApplicationQueryGovernance>,
            ),
            InspectionCause,
        >,
    ) -> Result<
        (PreparedSourceOutcome, Option<ExecutionMemoryReservation>),
        WorthQueryApplicationInspectionDenial,
    > {
        let mut pending = Some(pending);
        self.execute_inner_core(detail, &mut |e, q, p, s, m| {
            pending
                .take()
                .expect("one private source policy preparation")(e, q, p, s, m)
        })
    }
    fn execute_inner_core(
        self,
        detail: Option<super::super::super::rich::WorthQueryApplicationMaterializationRequest>,
        pending: &mut dyn FnMut(
            &ExecutionRequest<'_, '_>,
            &WorthQueryInstalledApplicationQueryBinding<Schema, Disclosure::Disclosure>,
            EntityId,
            EntityId,
            &mut Option<ExecutionMemoryReservation>,
        ) -> Result<
            (
                Option<WorthQueryPendingApplicationQueryGovernance>,
                Option<WorthQueryPendingApplicationQueryGovernance>,
            ),
            InspectionCause,
        >,
    ) -> Result<
        (PreparedSourceOutcome, Option<ExecutionMemoryReservation>),
        WorthQueryApplicationInspectionDenial,
    > {
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
                            let (metadata, diagnostic) = pending(
                                execution,
                                &installed,
                                principal.principal().principal_entity_id(),
                                scope.scope().entity_id(),
                                storage,
                            )
                            .map_err(FreshQueryPermissionStop::Inspect)?;
                            let rich = match (detail, diagnostic) {
                                (Some(request), Some(pending)) => {
                                    Some(InspectionDiagnosticPreparation::Rich { request, pending })
                                }
                                (None, None) => None,
                                _ => return Err(FreshQueryPermissionStop::Inspect(
                                    InspectionCause::Admission(
                                        admission::InspectionAdmissionDenial::ForeignIssuedAccess,
                                    ),
                                )),
                            };
                            let mut read = admission::prepare(
                                self.runtime,
                                execution,
                                &installed,
                                &selected,
                                principal,
                                scope,
                                self.request,
                                InspectionSubject::Source(&original),
                                metadata,
                                rich,
                                storage,
                            )
                            .map_err(|cause| {
                                FreshQueryPermissionStop::Inspect(InspectionCause::Admission(cause))
                            })?;
                            let report = if detail.is_some() {
                                read.materialize_source_rich(execution, installed.query())
                                    .map(SourceMaterialization::Rich)
                            } else {
                                read.materialize(execution)
                                    .map(SourceMaterialization::Reference)
                            }
                            .map_err(|cause| {
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
                match report {
                    SourceMaterialization::Reference(report) => completed
                        .publish_source(execution, original, report, &mut memory)
                        .map(PreparedSourceOutcome::Reference),
                    SourceMaterialization::Rich(report) => completed
                        .publish_source_rich(execution, original, report, &mut memory)
                        .map(PreparedSourceOutcome::Rich),
                }
                .map_err(InspectionCause::Admission)
            },
        );
        match outcome {
            Ok(prepared) => Ok((prepared, memory)),
            Err(cause) => Err(WorthQueryApplicationInspectionDenial::new(cause, memory)),
        }
    }
}
