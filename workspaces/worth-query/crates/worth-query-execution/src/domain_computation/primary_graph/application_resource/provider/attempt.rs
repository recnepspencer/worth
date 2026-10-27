//! Issued correlation retains no Native progression or Product snapshot authority.
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, WorkCeilingDenial};
use worth_runtime_bridge::facade::BridgeOwnedAsyncManagedRequestAdmission;

pub(in crate::domain_computation::primary_graph::application_resource) struct AttemptAuthority {
    pub(super) handle: worth_signal::facade::ResourceRequestHandle,
    pub(super) attempt: worth_signal::facade::ResourceAttemptId,
    correlation: Arc<str>,
    // Last to drop with every provider/writer clone of the issued correlation.
    _funding: Option<ExecutionMemoryReservation>,
}

/// Opaque correlation issued only from the real admitted Native occurrence.
/// Its text is a transport selector; it grants no resource or completion authority.
#[derive(Clone)]
pub struct WorthQueryResourceProviderAttempt {
    pub(super) authority: Arc<AttemptAuthority>,
}

impl WorthQueryResourceProviderAttempt {
    pub(in crate::domain_computation::primary_graph::application_resource) fn matches_native(
        &self,
        native: &BridgeOwnedAsyncManagedRequestAdmission,
    ) -> bool {
        native.request().is_some_and(|request| {
            self.authority.handle == request.request().request_handle()
                && self.authority.attempt == request.request().attempt()
        })
    }
    pub fn correlation(&self) -> &str {
        &self.authority.correlation
    }

    pub(in crate::domain_computation::primary_graph::application_resource) fn issue(
        execution: &ExecutionRequest<'_, '_>,
        native: &BridgeOwnedAsyncManagedRequestAdmission,
    ) -> Result<Self, AttemptIssuanceDenial> {
        let request = native
            .request()
            .ok_or(AttemptIssuanceDenial::NativeWitnessUnavailable)?;
        let identity = request.request();
        let correlation = identity.request_identity_for_reporting();
        let bytes = std::mem::size_of::<AttemptAuthority>()
            .checked_add(4 * std::mem::size_of::<usize>())
            .and_then(|bytes| bytes.checked_add(correlation.len()))
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(AttemptIssuanceDenial::AccountingOverflow)?;
        let funding = (*execution)
            .reserve_retained_memory(bytes)
            .map_err(AttemptIssuanceDenial::Request)?;
        Ok(Self {
            authority: Arc::new(AttemptAuthority {
                handle: identity.request_handle(),
                attempt: identity.attempt(),
                correlation: Arc::from(correlation),
                _funding: funding,
            }),
        })
    }
}

#[derive(Debug)]
pub(in crate::domain_computation::primary_graph::application_resource) enum AttemptIssuanceDenial {
    NativeWitnessUnavailable,
    AccountingOverflow,
    Request(WorkCeilingDenial),
}
