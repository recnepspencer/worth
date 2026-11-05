//! Moved whole from `domain_computation::primary_graph::vocabulary::application_query::provider_failure`: the items below and every impl they own.
use crate::domain_computation::vocabulary::primary_graph::control::WorthQueryManagedIndexControl;
use crate::domain_computation::vocabulary::primary_graph::stop::WorthQueryManagedIndexStop;
use std::sync::Arc;
use worth_execution::ExecutionMemoryReservation;
use worth_execution::ExecutionRequest;

#[derive(Debug)]
pub(in crate::domain_computation) struct PaidProviderDenial<D> {
    cause: D,
    _memory: Option<ExecutionMemoryReservation>,
}

impl<D: std::error::Error> std::fmt::Display for PaidProviderDenial<D> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.cause, formatter)
    }
}

impl<D: std::error::Error + 'static> std::error::Error for PaidProviderDenial<D> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.cause)
    }
}

/// A provider's concrete refusal retained without formatting or losing its type.
#[derive(Clone, Debug)]
pub struct WorthQueryManagedIndexProviderFailure {
    cause: Arc<dyn std::error::Error + Send + Sync>,
    retained_bytes: u64,
    equal: fn(
        &(dyn std::error::Error + Send + Sync + 'static),
        &(dyn std::error::Error + Send + Sync + 'static),
    ) -> bool,
}

pub(in crate::domain_computation) fn equal_cause<D: Eq + std::error::Error + Send + Sync + 'static>(
    left: &(dyn std::error::Error + Send + Sync + 'static),
    right: &(dyn std::error::Error + Send + Sync + 'static),
) -> bool {
    match (
        left.downcast_ref::<PaidProviderDenial<D>>(),
        right.downcast_ref::<PaidProviderDenial<D>>(),
    ) {
        (Some(left), Some(right)) => left.cause == right.cause,
        _ => false,
    }
}

impl PartialEq for WorthQueryManagedIndexProviderFailure {
    fn eq(&self, other: &Self) -> bool {
        (self.equal)(self.cause.as_ref(), other.cause.as_ref())
    }
}

impl Eq for WorthQueryManagedIndexProviderFailure {}

impl WorthQueryManagedIndexProviderFailure {
    pub(in crate::domain_computation) fn retained_payload_bytes(&self) -> u64 {
        self.retained_bytes
    }
    pub(in crate::domain_computation) fn retain<D>(
        execution: &ExecutionRequest<'_, '_>,
        control: &WorthQueryManagedIndexControl<'_>,
        cause: D,
    ) -> Result<Self, WorthQueryManagedIndexStop>
    where
        D: Copy + Eq + std::error::Error + Send + Sync + 'static,
    {
        control.poll()?;
        let bytes = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<PaidProviderDenial<D>>())
            .ok()
            .map(|(layout, _)| layout.pad_to_align().size())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(|| control.refuse(WorthQueryManagedIndexStop::CapacityOverflow))?;
        let memory = execution
            .reserve_retained_memory(bytes)
            .map_err(|cause| control.refuse(WorthQueryManagedIndexStop::Request(cause)))?;
        // Original admission precedes the only wrapper allocation.
        let cause = Arc::new(PaidProviderDenial {
            cause,
            _memory: memory,
        });
        control.poll()?;
        Ok(Self {
            cause,
            retained_bytes: bytes,
            equal: equal_cause::<D>,
        })
    }

    /// Inspect the provider's original concrete denial type.
    pub fn downcast_ref<D>(&self) -> Option<&D>
    where
        D: std::error::Error + Send + Sync + 'static,
    {
        self.cause
            .downcast_ref::<PaidProviderDenial<D>>()
            .map(|paid| &paid.cause)
    }
}

impl std::fmt::Display for WorthQueryManagedIndexProviderFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.cause, formatter)
    }
}

impl std::error::Error for WorthQueryManagedIndexProviderFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.cause.as_ref())
    }
}
