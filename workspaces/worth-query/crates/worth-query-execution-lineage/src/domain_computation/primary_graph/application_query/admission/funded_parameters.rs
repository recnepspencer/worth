//! Admitted immutable query parameter custody.
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, WorkCeilingDenial};
use worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters;
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryIdentity;
use crate::domain_computation::primary_graph::{
    application_contribution::execution_denial::publication_execution_kind,
    WorthQueryApplicationAttemptDenial as Denial, WorthQueryApplicationAttemptDenialKind as Kind,
};

pub fn overflow() -> Denial {
    request_stopped(WorkCeilingDenial::Admission(
        worth_execution::LeaseDenial::ChargedBytesOverflow,
    ))
}

pub fn request_stopped(cause: WorkCeilingDenial) -> Denial {
    Denial::source_disclosure_execution(publication_execution_kind(cause))
}

pub struct CapturedApplicationQueryParameters {
    query: WorthQueryInstalledApplicationQueryIdentity,
    parameters: Arc<WorthQueryAdmittedApplicationQueryParameters>,
    // Last to drop: parameters cannot outlive their original reservation.
    _funding: Option<ExecutionMemoryReservation>,
}

impl CapturedApplicationQueryParameters {
    pub fn readmitted(
        query: WorthQueryInstalledApplicationQueryIdentity,
        parameters: WorthQueryAdmittedApplicationQueryParameters,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Denial> {
        let bytes = parameters
            .retained_owned_capacity_bytes()
            .and_then(|bytes| {
                bytes.checked_add(std::mem::size_of::<Self>() + 2 * std::mem::size_of::<usize>())
            })
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(overflow)?;
        let funding = execution
            .reserve_retained_memory(bytes)
            .map_err(request_stopped)?;
        Ok(Self::captured(query, parameters, funding))
    }
    pub fn captured(
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
    pub fn query(
        &self,
    ) -> &WorthQueryInstalledApplicationQueryIdentity {
        &self.query
    }
    pub fn parameters(
        &self,
    ) -> &WorthQueryAdmittedApplicationQueryParameters {
        &self.parameters
    }
    pub fn same_capture(&self, other: &Self) -> bool {
        self.query == other.query && Arc::ptr_eq(&self.parameters, &other.parameters)
    }
}
