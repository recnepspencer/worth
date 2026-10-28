//! Source disclosure associates the sole funded parameter capture with its source.
use super::super::application_query::CapturedApplicationQueryParameters;

pub(in crate::domain_computation) struct OriginalSourceDisclosure(
    CapturedApplicationQueryParameters,
);

impl OriginalSourceDisclosure {
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
