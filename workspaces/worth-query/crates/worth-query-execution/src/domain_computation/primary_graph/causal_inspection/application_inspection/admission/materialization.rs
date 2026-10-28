//! Only fresh Inspection currentness can enter the report-only Bridge factory.
use super::super::report::{self, FundedInspectionReport};
use super::*;
use worth_query_installation::facade::ApplicationSchema;
#[cfg(test)]
mod tests;

impl<Schema: ApplicationSchema, Scope> InspectionRead<'_, '_, Schema, Scope> {
    pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) fn materialize(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<FundedInspectionReport, InspectionAdmissionDenial> {
        self.check(execution)?;
        let report = report::materialize(execution, &self.target, &self.session, self.memory)
            .map_err(InspectionAdmissionDenial::Report)?;
        require_request(execution, self.request)?;
        Ok(report)
    }
}
