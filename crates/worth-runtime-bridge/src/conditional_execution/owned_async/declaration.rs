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

/// Declaration meaning only; it recreates no installed Signal source or port.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BridgeOwnedAsyncRetention {
    Mandatory,
    ProcessLocal,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeOwnedAsyncDeclarationDescription {
    pub identity: String,
    pub payload_contract: u64,
    pub max_payload_bytes: u64,
    pub retry_max_attempts: u32,
    pub retry_delay_ticks: u64,
    pub timeout_ticks: u64,
    pub managed_queue_capacity: Option<std::num::NonZeroU64>,
    pub retention: BridgeOwnedAsyncRetention,
}
impl BridgeOwnedAsyncRequestResponseDeclaration {
    /// Reconstruct declaration meaning. Installation still issues fresh Native owners.
    pub fn from_description(description: BridgeOwnedAsyncDeclarationDescription) -> Self {
        Self {
            identity: Arc::from(description.identity),
            payload_contract: description.payload_contract,
            max_payload_bytes: description.max_payload_bytes,
            retry_max_attempts: description.retry_max_attempts,
            retry_delay_ticks: description.retry_delay_ticks,
            timeout_ticks: description.timeout_ticks,
            managed_queue_capacity: description.managed_queue_capacity,
            process_local_retention: description.retention
                == BridgeOwnedAsyncRetention::ProcessLocal,
        }
    }
    pub fn description(&self) -> BridgeOwnedAsyncDeclarationDescription {
        BridgeOwnedAsyncDeclarationDescription {
            identity: self.identity.to_string(),
            payload_contract: self.payload_contract,
            max_payload_bytes: self.max_payload_bytes,
            retry_max_attempts: self.retry_max_attempts,
            retry_delay_ticks: self.retry_delay_ticks,
            timeout_ticks: self.timeout_ticks,
            managed_queue_capacity: self.managed_queue_capacity,
            retention: if self.process_local_retention {
                BridgeOwnedAsyncRetention::ProcessLocal
            } else {
                BridgeOwnedAsyncRetention::Mandatory
            },
        }
    }
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
#[cfg(test)]
#[path = "declaration_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
