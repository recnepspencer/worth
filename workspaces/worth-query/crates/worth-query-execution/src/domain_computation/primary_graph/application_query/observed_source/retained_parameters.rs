use worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters;
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryBindingObservation;

/// One retained clone of the admitted canonical parameter basis, shared by
/// every row and the result-set source from the same query execution.
/// The original compiled binding interprets those parameters; it does not
/// provide permission to read or inspect them.
pub(in crate::domain_computation::primary_graph::application_query) struct WorthQueryRetainedObservedParameters
{
    admitted: WorthQueryAdmittedApplicationQueryParameters,
    binding: Option<WorthQueryInstalledApplicationQueryBindingObservation>,
}

impl WorthQueryRetainedObservedParameters {
    pub(in crate::domain_computation::primary_graph::application_query) fn new(
        admitted: WorthQueryAdmittedApplicationQueryParameters,
        binding: Option<WorthQueryInstalledApplicationQueryBindingObservation>,
    ) -> Self {
        Self { admitted, binding }
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn binding_observation(
        &self,
    ) -> Option<&WorthQueryInstalledApplicationQueryBindingObservation> {
        self.binding.as_ref()
    }
}

impl std::ops::Deref for WorthQueryRetainedObservedParameters {
    type Target = WorthQueryAdmittedApplicationQueryParameters;

    fn deref(&self) -> &Self::Target {
        &self.admitted
    }
}
