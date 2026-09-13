use std::sync::Arc;

use sha2::{Digest, Sha256};

use super::BridgeAsyncCompletionCounters;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeAsyncCompletionRejectionKind {
    EnvelopeHandleMismatch,
    EnvelopeAttemptMismatch,
    PayloadContractDigestMismatch,
    FamilyKindMismatch,
    SignalRuntimeThreadAffinityViolation,
    SignalCompletionAdmissionUnavailable,
    ForeignOwnerObservationAuthority,
    SupersessionMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeAsyncCompletionRejection {
    kind: BridgeAsyncCompletionRejectionKind,
    detail: Arc<str>,
    counters: BridgeAsyncCompletionCounters,
    digest: Arc<str>,
}

impl BridgeAsyncCompletionRejection {
    pub(crate) fn new(
        kind: BridgeAsyncCompletionRejectionKind,
        detail: impl Into<Arc<str>>,
    ) -> Self {
        let detail = detail.into();
        let counters = match kind {
            BridgeAsyncCompletionRejectionKind::EnvelopeHandleMismatch
            | BridgeAsyncCompletionRejectionKind::EnvelopeAttemptMismatch
            | BridgeAsyncCompletionRejectionKind::PayloadContractDigestMismatch => {
                BridgeAsyncCompletionCounters::invalid_envelope()
            }
            _ => BridgeAsyncCompletionCounters::rejected(),
        };
        let canonical_basis = format!(
            "bridge-async-completion-rejection|kind={kind:?}|detail={}",
            detail.as_ref()
        );
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            kind,
            detail,
            counters,
            digest: Arc::from(format!(
                "bridge-async-completion-rejection:sha256:{digest:x}"
            )),
        }
    }

    pub fn kind(&self) -> BridgeAsyncCompletionRejectionKind {
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
