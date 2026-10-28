//! Opaque actual cleanup evidence survives a later original-request refusal.
/// Actual resource cleanup evidence, retained even if the request later stops.
pub struct WorthQueryClosedResource {
    #[expect(
        dead_code,
        reason = "closed receipt retains actual native cleanup evidence without disclosing it"
    )]
    pub(super) native: Option<worth_runtime_bridge::facade::BridgeOwnedAsyncReleasedResource>,
    #[expect(
        dead_code,
        reason = "clock-only progress stays owned by the opaque cleanup receipt"
    )]
    pub(super) clock_only: Option<worth_signal::facade::TemporalClockAdvanceSummary>,
    #[expect(
        dead_code,
        reason = "completed action evidence stays owned by the opaque cleanup receipt"
    )]
    pub(super) action:
        crate::domain_computation::provider_session::WorthQueryResourceReadCompletion,
    pub(super) detail: Option<super::WorthQueryResourceCleanupDetail>,
}

impl WorthQueryClosedResource {
    /// Present only when the original Close also admitted CleanupDetail.
    /// Plain Close and recovery handoff retain opaque cleanup evidence.
    pub fn detail(&self) -> Option<&super::WorthQueryResourceCleanupDetail> {
        self.detail.as_ref()
    }
}

impl std::fmt::Debug for WorthQueryClosedResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryClosedResource")
            .finish_non_exhaustive()
    }
}
