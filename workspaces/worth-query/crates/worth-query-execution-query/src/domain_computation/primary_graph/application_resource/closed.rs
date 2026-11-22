//! Opaque actual cleanup evidence survives a later original-request refusal.

/// Cleanup details exposed only through the installed CleanupDetail purpose.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryResourceCleanupDetail {
    pub provider_closed: bool,
    pub queue_drained: bool,
    pub native_released: bool,
}

/// Actual resource cleanup evidence, retained even if the request later stops.
pub struct WorthQueryClosedResource {
    #[expect(
        dead_code,
        reason = "closed receipt retains actual native cleanup evidence without disclosing it"
    )]
    native: Option<worth_runtime_bridge::facade::BridgeOwnedAsyncReleasedResource>,
    #[expect(
        dead_code,
        reason = "clock-only progress stays owned by the opaque cleanup receipt"
    )]
    clock_only: Option<worth_signal::facade::TemporalClockAdvanceSummary>,
    #[expect(
        dead_code,
        reason = "completed action evidence stays owned by the opaque cleanup receipt"
    )]
    action: crate::domain_computation::provider_session::WorthQueryResourceReadCompletion,
    detail: Option<WorthQueryResourceCleanupDetail>,
}

impl WorthQueryClosedResource {
    pub(crate) fn from_completed_action(
        native: Option<worth_runtime_bridge::facade::BridgeOwnedAsyncReleasedResource>,
        clock_only: Option<worth_signal::facade::TemporalClockAdvanceSummary>,
        action: crate::domain_computation::provider_session::WorthQueryResourceReadCompletion,
    ) -> Self {
        Self {
            native,
            clock_only,
            action,
            detail: None,
        }
    }

    pub(crate) fn with_cleanup_detail(
        mut self,
        detail: Option<WorthQueryResourceCleanupDetail>,
    ) -> Self {
        self.detail = detail;
        self
    }

    /// Present only when the original Close also admitted CleanupDetail.
    /// Plain Close and recovery handoff retain opaque cleanup evidence.
    pub fn detail(&self) -> Option<&WorthQueryResourceCleanupDetail> {
        self.detail.as_ref()
    }
}

/// Crate-boundary internal port for `WorthQueryClosedResource`.
/// Never exported on `worth-query-execution::facade` or Host.
pub trait ClosedResourceInternalPort: Sized {
    fn from_completed_action(
        native: Option<worth_runtime_bridge::facade::BridgeOwnedAsyncReleasedResource>,
        clock_only: Option<worth_signal::facade::TemporalClockAdvanceSummary>,
        action: crate::domain_computation::provider_session::WorthQueryResourceReadCompletion,
    ) -> Self;
    fn with_cleanup_detail(self, detail: Option<WorthQueryResourceCleanupDetail>) -> Self;
}

impl ClosedResourceInternalPort for WorthQueryClosedResource {
    fn from_completed_action(
        native: Option<worth_runtime_bridge::facade::BridgeOwnedAsyncReleasedResource>,
        clock_only: Option<worth_signal::facade::TemporalClockAdvanceSummary>,
        action: crate::domain_computation::provider_session::WorthQueryResourceReadCompletion,
    ) -> Self {
        Self::from_completed_action(native, clock_only, action)
    }

    fn with_cleanup_detail(self, detail: Option<WorthQueryResourceCleanupDetail>) -> Self {
        Self::with_cleanup_detail(self, detail)
    }
}

impl std::fmt::Debug for WorthQueryClosedResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryClosedResource")
            .finish_non_exhaustive()
    }
}
