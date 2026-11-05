//! Moved whole from `domain_computation::primary_graph::source_disclosure::original`: the items below and every impl they own.
use crate::domain_computation::vocabulary::primary_graph::funded_parameters::CapturedApplicationQueryParameters;

pub struct OriginalSourceDisclosure(CapturedApplicationQueryParameters);

impl OriginalSourceDisclosure {
    pub(in crate::domain_computation) fn captured(parameters: CapturedApplicationQueryParameters) -> Self {
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

impl OriginalSourceDisclosure {
    #[doc(hidden)]
    pub fn query_for_test(
        &self,
    ) -> &worth_query_installation::facade::WorthQueryInstalledApplicationQueryIdentity {
        Self::query(self)
    }
}
