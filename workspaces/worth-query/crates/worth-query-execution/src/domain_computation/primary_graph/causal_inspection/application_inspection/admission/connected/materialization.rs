//! Both diagnostic loans precede original performed-input and output discovery.
use super::*;
use crate::domain_computation::primary_graph::{
    admitted_capture::{checkpoint, run_retained_stage, RetainedStorageDenial},
    WorthQueryApplicationCommitReceipt,
};
use worth_execution::MapKernelFailure;
use worth_query_installation::facade::{ApplicationSchema, WorthQueryInstalledApplicationQuery};
pub(in super::super::super) enum ConnectedMaterialization {
    Reference(super::super::super::report::FundedConnectedReport),
    Rich(super::super::super::report::rich::FundedRichInspectionReport),
}
impl<Schema: ApplicationSchema, Scope> InspectionRead<'_, '_, Schema, Scope> {
    pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) fn materialize_connected<
        Q,
        P,
        R,
        S,
        SQ,
        SP,
        SR,
        SS,
    >(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        query: &WorthQueryInstalledApplicationQuery<Schema, Q, P, R, S>,
        source_query: &WorthQueryInstalledApplicationQuery<Schema, SQ, SP, SR, SS>,
        source: &WorthQueryApplicationCommitReceipt,
        purpose: super::super::super::connection::SourceInspectionPurpose,
    ) -> Result<ConnectedMaterialization, InspectionAdmissionDenial> {
        drop(self.check(execution)?);
        let report = self.materialize(execution)?;
        let source_read = self
            .source
            .as_ref()
            .ok_or(InspectionAdmissionDenial::ForeignIssuedAccess)?;
        let diagnostic = self
            .diagnostic
            .as_ref()
            .ok_or(InspectionAdmissionDenial::ForeignIssuedAccess)?;
        let (query_admission, source_admission) =
            run_retained_stage(execution, self.memory, |context, _| {
                let query_admission = diagnostic.admit_diagnostic_comparison(
                    context,
                    query,
                    &self.session,
                    &self.target,
                    &self.target.parameters(),
                    self.principal,
                    self.scope.entity_id(),
                )?;
                let source_admission = source_read.policy.diagnostic.admit_diagnostic_comparison(
                    context,
                    source_query,
                    &self.session,
                    &source_read.policy.target,
                    &source_read.policy.target.parameters(),
                    self.principal,
                    source_read.scope.entity_id(),
                )?;
                Ok((query_admission, source_admission))
            })
            .map_err(InspectionAdmissionDenial::DiagnosticAdmission)?;
        let explanation = super::super::super::connection::explain(
            self.runtime,
            execution,
            &self.target,
            source,
            &query_admission,
            Some(&source_admission),
            self.memory,
            purpose,
        )
        .map_err(InspectionAdmissionDenial::Connection)?;
        let body = if self.target.rich_request().is_some() {
            let history = if self
                .target
                .rich_request()
                .is_some_and(|request| request.requests_conditional_history())
            {
                run_retained_stage(
                    execution,
                    self.memory,
                    |context, _| -> Result<_, MapKernelFailure<RetainedStorageDenial>> {
                        checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
                        Ok(super::super::super::history::HistoryDetail::capture(
                            self.target.original().expect("connected query target"),
                            &query_admission,
                            &source_admission,
                        ))
                    },
                )
                .map_err(InspectionAdmissionDenial::Storage)?
            } else {
                super::super::super::history::HistoryDetail::NotRequested
            };
            let mut body = super::super::rich::capture_original_query(
                execution,
                self.memory,
                &self.target,
                &self.session,
                query_admission,
            )?;
            body.artifact.history = history;
            if body.artifact.request().requests_source_effects() {
                let source = source_read
                    .policy
                    .target
                    .source()
                    .ok_or(InspectionAdmissionDenial::ForeignIssuedAccess)?;
                body.artifact.source = super::super::super::rich::source::capture(
                    execution,
                    self.memory,
                    source,
                    source_read.scope.entity_kind(),
                    source_admission,
                    &self.runtime.primary_provider.graph.layout,
                    self.runtime.installed_schema().native_contracts(),
                )?;
            }
            if matches!(
                body.artifact.original,
                super::super::super::rich::OriginalQueryDetail::Provided(_)
            ) || matches!(
                body.artifact.source,
                super::super::super::rich::source::SourceDetail::Provided(_)
            ) || matches!(
                body.artifact.history,
                super::super::super::history::HistoryDetail::Provided(_)
            ) || body.artifact.request().requests_producer_evidence()
            {
                self.runtime
                    .primary_provider
                    .inspection_materialization_prepared();
            }
            Some(body)
        } else {
            None
        };
        drop(self.check(execution)?);
        let connected = super::super::super::report::materialize_connected(
            execution,
            report,
            explanation,
            &self.session,
            self.memory,
        )
        .map_err(InspectionAdmissionDenial::Report)?;
        drop(self.check(execution)?);
        match body {
            None => Ok(ConnectedMaterialization::Reference(connected)),
            Some(body) => {
                let report = super::super::super::report::rich::materialize_connected_rich(
                    execution,
                    connected,
                    body,
                    self.memory,
                )
                .map_err(InspectionAdmissionDenial::Report)?;
                drop(self.check(execution)?);
                Ok(ConnectedMaterialization::Rich(report))
            }
        }
    }
}
