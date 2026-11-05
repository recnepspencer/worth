//! Moved whole from `domain_computation::primary_graph::vocabulary::application_query::funded_parameters`: the items below and every impl they own.
use std::sync::Arc;
use worth_execution::ExecutionMemoryReservation;
use worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters;
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryIdentity;

pub(in crate::domain_computation) struct CapturedApplicationQueryParameters {
    query: WorthQueryInstalledApplicationQueryIdentity,
    parameters: Arc<WorthQueryAdmittedApplicationQueryParameters>,
    // Last to drop: parameters cannot outlive their original reservation.
    _funding: Option<ExecutionMemoryReservation>,
}

impl CapturedApplicationQueryParameters {
    pub(in crate::domain_computation) fn captured(
        query: WorthQueryInstalledApplicationQueryIdentity,
        parameters: WorthQueryAdmittedApplicationQueryParameters,
        funding: Option<ExecutionMemoryReservation>,
    ) -> Self {
        Self {
            query,
            parameters: Arc::new(parameters),
            _funding: funding,
        }
    }
    pub(in crate::domain_computation) fn query(
        &self,
    ) -> &WorthQueryInstalledApplicationQueryIdentity {
        &self.query
    }
    pub(in crate::domain_computation) fn parameters(
        &self,
    ) -> &WorthQueryAdmittedApplicationQueryParameters {
        &self.parameters
    }
    pub(in crate::domain_computation) fn same_capture(&self, other: &Self) -> bool {
        self.query == other.query && Arc::ptr_eq(&self.parameters, &other.parameters)
    }
}
