use std::sync::Arc;

use sha2::{Digest, Sha256};

use super::BridgeAsyncCompletionCounters;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeAsyncCompletionSupersessionRejectionKind {
    DeniedCompletionClassMismatch,
    MissingCurrentSubscriptionInstance,
    UnexpectedCurrentSubscriptionInstance,
    MissingDisplacingRequestIdentity,
    DisplacingRequestIdentityMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeAsyncCompletionSupersessionRejection {
    kind: BridgeAsyncCompletionSupersessionRejectionKind,
    detail: Arc<str>,
    counters: BridgeAsyncCompletionCounters,
    digest: Arc<str>,
}

impl BridgeAsyncCompletionSupersessionRejection {
    pub(crate) fn new(
        kind: BridgeAsyncCompletionSupersessionRejectionKind,
        detail: impl Into<Arc<str>>,
    ) -> Self {
        let detail = detail.into();
        let canonical_basis = format!(
            "bridge-async-completion-supersession-rejection|kind={kind:?}|detail={}",
            detail.as_ref()
        );
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            kind,
            detail,
            counters: BridgeAsyncCompletionCounters::rejected(),
            digest: Arc::from(format!(
                "bridge-async-completion-supersession-rejection:sha256:{digest:x}"
            )),
        }
    }

    pub fn kind(&self) -> BridgeAsyncCompletionSupersessionRejectionKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        self.detail.as_ref()
    }

    pub fn counters(&self) -> &BridgeAsyncCompletionCounters {
        &self.counters
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}
