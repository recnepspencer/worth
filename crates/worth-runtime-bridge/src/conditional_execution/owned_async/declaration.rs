use std::sync::Arc;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeOwnedAsyncRequestResponseDeclaration {
    pub(super) identity: Arc<str>,
    pub(super) payload_contract: u64,
    pub(super) max_payload_bytes: u64,
    pub(super) retry_max_attempts: u32,
    pub(super) retry_delay_ticks: u64,
    pub(super) timeout_ticks: u64,
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
        }
    }
}
