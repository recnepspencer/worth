use std::sync::Arc;
use worth_signal::facade::branch::{
    SignalOwnedAsyncManagedAdmission, SignalOwnedAsyncManagedReport,
};

use super::super::{signal_port::BridgeConditionalSignalPort, BridgeOwnedAsyncRequestAdmission};

#[derive(Debug, Clone)]
pub enum BridgeOwnedAsyncManagedDenial {
    ForeignOwner,
    QueueUnavailable,
    RequestUnavailable,
    ProgressDenied,
    Request(crate::facade::BridgeAsyncRequestIdentityRejection),
    Completion(crate::facade::BridgeAsyncCompletionRejection),
    Lineage(crate::facade::BridgeAsyncForwardCausalityRejection),
    Signal(worth_signal::facade::branch::SignalOwnedAsyncQueueDenial),
    Service(worth_signal::facade::branch::SignalConditionalServiceExecutionDenial),
}

/// Owns the native cleanup capability even when descriptive Bridge admission refuses.
/// Dropping this value does not cancel, drain or retire its native request.
pub struct BridgeOwnedAsyncManagedRequestAdmission {
    pub(super) authority: Arc<()>,
    pub(super) port: BridgeConditionalSignalPort,
    pub(super) native: SignalOwnedAsyncManagedAdmission,
    pub(super) business: crate::facade::BridgeAsyncRequestAdmissionRequest,
    pub(super) request: Option<BridgeOwnedAsyncRequestAdmission>,
    pub(super) refusal: Option<BridgeOwnedAsyncManagedDenial>,
}

impl BridgeOwnedAsyncManagedRequestAdmission {
    pub fn request(&self) -> Option<&BridgeOwnedAsyncRequestAdmission> {
        self.request.as_ref()
    }

    /// A retained refusal or wake cleanup still needs attention. Active requests
    /// also retain queue/provider obligations regardless of this diagnostic.
    pub fn cleanup_required(&self) -> bool {
        self.refusal.is_some()
            || self.native.refusal().is_some()
            || self.native.wake_cleanup_pending()
    }

    pub fn refusal(&self) -> Option<BridgeOwnedAsyncManagedDenial> {
        self.native
            .refusal()
            .cloned()
            .map(BridgeOwnedAsyncManagedDenial::Signal)
            .or_else(|| self.refusal.clone())
    }

    pub fn request_handle(&self) -> worth_signal::facade::ResourceRequestHandle {
        self.native.admitted_request().handle()
    }

    pub fn native_report(&self) -> &SignalOwnedAsyncManagedReport {
        self.native.report()
    }

    pub fn wake_cleanup_pending(&self) -> bool {
        self.native.wake_cleanup_pending()
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
