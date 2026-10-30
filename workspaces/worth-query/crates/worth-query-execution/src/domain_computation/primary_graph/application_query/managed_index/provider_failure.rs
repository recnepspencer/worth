//! Concrete pure refusals keep their type and original-request wrapper custody.
use super::{WorthQueryManagedIndexControl, WorthQueryManagedIndexStop};
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

#[derive(Debug)]
struct PaidProviderDenial<D> {
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
}

impl WorthQueryManagedIndexProviderFailure {
    pub(super) fn retain<D>(
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
        Ok(Self { cause })
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
