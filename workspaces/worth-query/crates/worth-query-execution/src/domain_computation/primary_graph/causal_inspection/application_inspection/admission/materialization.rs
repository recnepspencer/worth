//! Only fresh Inspection currentness can enter the report-only Bridge factory.
use super::super::{
    identity::InspectionOriginalReference,
    report::{self, FundedComparisonReport, FundedInspectionReport},
};
use super::*;
use worth_query_installation::facade::{ApplicationSchema, WorthQueryInstalledApplicationQuery};
#[cfg(test)]
mod tests;

impl<Schema: ApplicationSchema, Scope> InspectionRead<'_, '_, Schema, Scope> {
    pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) fn materialize(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<FundedInspectionReport, InspectionAdmissionDenial> {
        drop(self.check(execution)?);
        let report = report::materialize(
            execution,
            &self.target,
            &self.session,
            if self.target.source().is_some() {
                InspectionOriginalReference::Source
            } else {
                InspectionOriginalReference::After
            },
            self.memory,
        )
        .map_err(InspectionAdmissionDenial::Report)?;
        require_request(execution, self.request)?;
        Ok(report)
    }
    pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) fn materialize_comparison<
        Query,
        Parameters,
        QueryResult,
        QueryScope,
    >(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        query: &WorthQueryInstalledApplicationQuery<
            Schema,
            Query,
            Parameters,
            QueryResult,
            QueryScope,
        >,
        source: Option<
            &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
        >,
    ) -> Result<FundedComparisonReport, InspectionAdmissionDenial> {
        let (reason, connection) = self.compare(execution, query, source)?;
        drop(self.check(execution)?);
        let before = report::materialize(
            execution,
            &self.target,
            &self.session,
            InspectionOriginalReference::Before,
            self.memory,
        )
        .map_err(InspectionAdmissionDenial::Report)?;
        drop(self.check(execution)?);
        let after = report::materialize(
            execution,
            &self.target,
            &self.session,
            InspectionOriginalReference::After,
            self.memory,
        )
        .map_err(InspectionAdmissionDenial::Report)?;
        drop(self.check(execution)?);
        let report = report::materialize_comparison(
            execution,
            before,
            after,
            reason,
            connection,
            self.memory,
        )
        .map_err(InspectionAdmissionDenial::Report)?;
        require_request(execution, self.request)?;
        Ok(report)
    }
}
