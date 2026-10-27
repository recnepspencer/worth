use super::WorthQueryOwnedAsyncRequestCustody;
use worth_runtime_bridge::facade::{
    BridgeAsyncCompletionRejection, BridgeAsyncRequestIdentityRejection,
    BridgeOwnedAsyncManagedDenial,
};

/// Existing diagnostic audience; cloning this denial never clones custody.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryOwnedAsyncRuntimeDenial {
    ForeignRuntime,
    SuccessorRuntime,
    ProductSelectionMismatch,
    NoCommittedSource,
    Request(BridgeAsyncRequestIdentityRejection),
    Completion(BridgeAsyncCompletionRejection),
}

#[derive(Clone, Debug)]
pub enum WorthQueryOwnedAsyncRuntimeCause {
    Admission(WorthQueryOwnedAsyncRuntimeDenial),
    Custody(BridgeOwnedAsyncManagedDenial),
}

/// A refused entry returns every actual post-admission cleanup capability.
#[derive(Debug)]
pub struct WorthQueryOwnedAsyncRuntimeStop {
    cause: WorthQueryOwnedAsyncRuntimeCause,
    custody: Option<WorthQueryOwnedAsyncRequestCustody>,
}

impl WorthQueryOwnedAsyncRuntimeStop {
    pub(super) fn after_admission(
        cause: WorthQueryOwnedAsyncRuntimeCause,
        custody: WorthQueryOwnedAsyncRequestCustody,
    ) -> Self {
        Self {
            cause,
            custody: Some(custody),
        }
    }

    pub fn cause(&self) -> &WorthQueryOwnedAsyncRuntimeCause {
        &self.cause
    }

    pub fn into_parts(
        self,
    ) -> (
        WorthQueryOwnedAsyncRuntimeCause,
        Option<WorthQueryOwnedAsyncRequestCustody>,
    ) {
        (self.cause, self.custody)
    }
}

impl From<WorthQueryOwnedAsyncRuntimeCause> for WorthQueryOwnedAsyncRuntimeStop {
    fn from(cause: WorthQueryOwnedAsyncRuntimeCause) -> Self {
        Self {
            cause,
            custody: None,
        }
    }
}
impl From<WorthQueryOwnedAsyncRuntimeDenial> for WorthQueryOwnedAsyncRuntimeCause {
    fn from(cause: WorthQueryOwnedAsyncRuntimeDenial) -> Self {
        Self::Admission(cause)
    }
}
impl From<WorthQueryOwnedAsyncRuntimeDenial> for WorthQueryOwnedAsyncRuntimeStop {
    fn from(cause: WorthQueryOwnedAsyncRuntimeDenial) -> Self {
        WorthQueryOwnedAsyncRuntimeCause::from(cause).into()
    }
}
