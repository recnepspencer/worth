use super::source_identity::WorthQueryObservedSourceMeaning;
use super::WorthQueryObservedSourceFootprint;
use std::sync::Arc;
use worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters;
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryBindingObservation;

/// One retained clone of the admitted canonical parameter basis, shared by
/// every row and the result-set source from the same query execution.
/// The original compiled binding interprets those parameters; it does not
/// provide permission to read or inspect them.
/// The exact row meanings share their original immutable footprints; the roster
/// owns its original request's fixed backing and retains no business rows.
pub(in crate::domain_computation::primary_graph::application_query) struct WorthQueryRetainedObservedParameters
{
    admitted: WorthQueryAdmittedApplicationQueryParameters,
    binding: Option<WorthQueryInstalledApplicationQueryBindingObservation>,
    rows: worth_execution::ExecutionArray<Arc<WorthQueryObservedSourceMeaning>>,
}

impl WorthQueryRetainedObservedParameters {
    pub(in crate::domain_computation::primary_graph::application_query) fn new(
        admitted: WorthQueryAdmittedApplicationQueryParameters,
        binding: Option<WorthQueryInstalledApplicationQueryBindingObservation>,
        rows: worth_execution::ExecutionArray<Arc<WorthQueryObservedSourceMeaning>>,
    ) -> Self {
        Self {
            admitted,
            binding,
            rows,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn row_meanings(
        &self,
    ) -> &[Arc<WorthQueryObservedSourceMeaning>] {
        self.rows.elements()
    }

    pub(super) fn row_footprints(
        &self,
    ) -> impl ExactSizeIterator<Item = &WorthQueryObservedSourceFootprint> {
        self.rows.iter().map(|meaning| meaning.footprint())
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn binding_observation(
        &self,
    ) -> Option<&WorthQueryInstalledApplicationQueryBindingObservation> {
        self.binding.as_ref()
    }
}

#[cfg(test)]
mod tests;

impl std::ops::Deref for WorthQueryRetainedObservedParameters {
    type Target = WorthQueryAdmittedApplicationQueryParameters;

    fn deref(&self) -> &Self::Target {
        &self.admitted
    }
}
