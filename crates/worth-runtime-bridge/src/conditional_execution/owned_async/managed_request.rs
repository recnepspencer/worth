use std::sync::Arc;
use worth_signal::facade::{AdmittedResourceRequest, ResourceManagedQueueBinding};

use super::super::{signal_port::BridgeConditionalSignalPort, BridgeOwnedAsyncRequestAdmission};

#[derive(Debug)]
pub enum BridgeOwnedAsyncManagedDenial {
    ForeignOwner,
    QueueUnavailable,
    Request(crate::facade::BridgeAsyncRequestIdentityRejection),
    Signal(worth_signal::facade::branch::SignalOwnedAsyncQueueDenial),
    Service(worth_signal::facade::branch::SignalConditionalServiceExecutionDenial),
}

/// Owns the native cleanup capability even when descriptive Bridge admission refuses.
/// Dropping this value does not cancel, drain or retire its native request.
pub struct BridgeOwnedAsyncManagedRequestAdmission {
    pub(super) authority: Arc<()>,
    pub(super) port: BridgeConditionalSignalPort,
    pub(super) source: worth_signal::facade::branch::SignalOwnedAsyncSourceBinding,
    pub(super) admitted: AdmittedResourceRequest,
    pub(super) queue: Option<ResourceManagedQueueBinding>,
    pub(super) request: Option<BridgeOwnedAsyncRequestAdmission>,
    pub(super) refusal: Option<BridgeOwnedAsyncManagedDenial>,
}

impl BridgeOwnedAsyncManagedRequestAdmission {
    pub fn request(&self) -> Option<&BridgeOwnedAsyncRequestAdmission> {
        self.request.as_ref()
    }

    /// Admission refused after creating native custody. Ordinary active requests
    /// also retain queue/provider obligations regardless of this admission flag.
    pub fn cleanup_required(&self) -> bool {
        self.refusal.is_some()
    }

    pub fn refusal(&self) -> Option<&BridgeOwnedAsyncManagedDenial> {
        self.refusal.as_ref()
    }

    pub fn request_handle(&self) -> worth_signal::facade::ResourceRequestHandle {
        self.admitted.handle()
    }

    pub(super) fn require_owner(
        &self,
        authority: &Arc<()>,
    ) -> Result<(), BridgeOwnedAsyncManagedDenial> {
        if Arc::ptr_eq(&self.authority, authority) {
            Ok(())
        } else {
            Err(BridgeOwnedAsyncManagedDenial::ForeignOwner)
        }
    }
}
