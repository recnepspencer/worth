//! Source disclosure associates the sole funded parameter capture with its source.
use super::super::application_query::CapturedApplicationQueryParameters;

pub struct OriginalSourceDisclosure(CapturedApplicationQueryParameters);

impl OriginalSourceDisclosure {
    pub(in crate::domain_computation::primary_graph) fn readmitted(
        query: worth_query_installation::facade::WorthQueryInstalledApplicationQueryIdentity,
        parameters: worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial>
    {
        CapturedApplicationQueryParameters::readmitted(query, parameters, execution).map(Self)
    }
    pub(super) fn captured(parameters: CapturedApplicationQueryParameters) -> Self {
        Self(parameters)
    }
    pub(in crate::domain_computation) fn query(
        &self,
    ) -> &worth_query_installation::facade::WorthQueryInstalledApplicationQueryIdentity {
        self.0.query()
    }
    pub(in crate::domain_computation) fn parameters(
        &self,
    ) -> &worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters{
        self.0.parameters()
    }
    pub(in crate::domain_computation) fn same_capture(&self, other: &Self) -> bool {
        self.0.same_capture(&other.0)
    }
}

mod court_doors_for_test;
