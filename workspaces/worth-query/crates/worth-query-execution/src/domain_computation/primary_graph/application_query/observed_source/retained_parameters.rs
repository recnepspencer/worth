use worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters;

/// One retained clone of the admitted canonical parameter basis, shared by
/// every row and the result-set source from the same query execution.
pub(in crate::domain_computation::primary_graph::application_query) struct WorthQueryRetainedObservedParameters
{
    admitted: WorthQueryAdmittedApplicationQueryParameters,
}

impl WorthQueryRetainedObservedParameters {
    pub(in crate::domain_computation::primary_graph::application_query) fn new(
        admitted: WorthQueryAdmittedApplicationQueryParameters,
    ) -> Self {
        Self { admitted }
    }
}

impl std::ops::Deref for WorthQueryRetainedObservedParameters {
    type Target = WorthQueryAdmittedApplicationQueryParameters;

    fn deref(&self) -> &Self::Target {
        &self.admitted
    }
}
