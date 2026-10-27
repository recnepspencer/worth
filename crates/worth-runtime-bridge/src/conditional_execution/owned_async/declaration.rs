use std::sync::Arc;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeOwnedAsyncRequestResponseDeclaration {
    pub(super) identity: Arc<str>,
    pub(super) payload_contract: u64,
    pub(super) max_payload_bytes: u64,
    pub(super) retry_max_attempts: u32,
    pub(super) retry_delay_ticks: u64,
    pub(super) timeout_ticks: u64,
    pub(super) managed_queue_capacity: Option<std::num::NonZeroU64>,
    pub(super) process_local_retention: bool,
}

impl BridgeOwnedAsyncRequestResponseDeclaration {
    pub fn new(
        identity: impl Into<Arc<str>>,
        payload_contract: u64,
        max_payload_bytes: u64,
        retry_max_attempts: u32,
        retry_delay_ticks: u64,
        timeout_ticks: u64,
    ) -> Self {
        Self {
            identity: identity.into(),
            payload_contract,
            max_payload_bytes,
            retry_max_attempts,
            retry_delay_ticks,
            timeout_ticks,
            managed_queue_capacity: None,
            process_local_retention: false,
        }
    }

    pub fn with_managed_queue_capacity(mut self, capacity: std::num::NonZeroU64) -> Self {
        self.managed_queue_capacity = Some(capacity);
        self
    }

    /// Opts this queued resource into exact process-local terminal release.
    /// Installation refuses this posture without a managed queue capacity.
    pub fn with_process_local_retention(mut self) -> Self {
        self.process_local_retention = true;
        self
    }
}
