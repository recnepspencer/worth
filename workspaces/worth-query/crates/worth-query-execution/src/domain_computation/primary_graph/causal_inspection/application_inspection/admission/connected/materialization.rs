//! Both diagnostic loans precede original performed-input and output discovery.
use super::*;
use crate::domain_computation::primary_graph::{
    admitted_capture::run_retained_stage, WorthQueryApplicationCommitReceipt,
};
use worth_query_installation::facade::{ApplicationSchema, WorthQueryInstalledApplicationQuery};
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
    ) -> Result<super::super::super::report::FundedConnectedReport, InspectionAdmissionDenial> {
        self.check(execution)?;
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
                let source_admission = source_read.diagnostic.admit_diagnostic_comparison(
                    context,
                    source_query,
                    &self.session,
                    &source_read.target,
                    &source_read.target.parameters(),
                    self.principal,
                    self.scope.entity_id(),
                )?;
                Ok((query_admission, source_admission))
            })
            .map_err(InspectionAdmissionDenial::DiagnosticAdmission)?;
        let connection = super::super::super::connection::connect(
            self.runtime,
            execution,
            &self.target,
            source,
            &query_admission,
            Some(&source_admission),
            self.memory,
        )
        .map_err(InspectionAdmissionDenial::Connection)?;
        self.check(execution)?;
        let report = self.materialize(execution)?;
        let connected = super::super::super::report::materialize_connected(
            execution,
            report,
            connection,
            &self.session,
            self.memory,
        )
        .map_err(InspectionAdmissionDenial::Report)?;
        self.check(execution)?;
        Ok(connected)
    }
}
