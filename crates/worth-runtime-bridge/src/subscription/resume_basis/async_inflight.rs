use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::input::envelope::TruthBranchIdentity;
use crate::source::AdmittedBridgeAsyncRequestIdentity;
use crate::subscription::{
    BridgeSubscriptionCounters, BridgeSubscriptionRetainedInflightAsyncResumeBasisIdentity,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeRetainedInflightAsyncResumeBasis {
    retained_inflight_async_resume_basis_identity:
        BridgeSubscriptionRetainedInflightAsyncResumeBasisIdentity,
    request_identity: Arc<str>,
    truth_view_basis_digest: Arc<str>,
    truth_branch_identity: Option<TruthBranchIdentity>,
    request_generation: Option<u64>,
    attempt: u64,
    retention_complete: bool,
    counters: BridgeSubscriptionCounters,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl BridgeRetainedInflightAsyncResumeBasis {
    pub(crate) fn capture(
        request_identity: &AdmittedBridgeAsyncRequestIdentity,
        retention_complete: bool,
    ) -> Self {
        let request_generation = Some(request_identity.request_handle().generation().get());
        Self::capture_with_generation(request_identity, request_generation, retention_complete)
    }

    #[cfg(test)]
    pub(crate) fn capture_without_generation_for_test(
        request_identity: &AdmittedBridgeAsyncRequestIdentity,
        retention_complete: bool,
    ) -> Self {
        Self::capture_with_generation(request_identity, None, retention_complete)
    }

    pub(crate) fn capture_with_generation(
        request_identity: &AdmittedBridgeAsyncRequestIdentity,
        request_generation: Option<u64>,
        retention_complete: bool,
    ) -> Self {
        let truth_view_basis = request_identity.basis_binding().truth_view_basis();
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-retained-inflight-async-resume-basis|request={}|truth-view={}|branch={}|generation={}|attempt={}|retention-complete={retention_complete}",
            request_identity.request_identity().as_str(),
            truth_view_basis.digest(),
            truth_view_basis
                .truth_branch_identity()
                .map(TruthBranchIdentity::canonical_label)
                .unwrap_or_else(|| "-".to_owned()),
            request_generation
                .map(|generation| generation.to_string())
                .unwrap_or_else(|| "-".to_owned()),
            request_identity.attempt().get(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            retained_inflight_async_resume_basis_identity:
                BridgeSubscriptionRetainedInflightAsyncResumeBasisIdentity::admit_bridge_owned(
                    format!("bridge-retained-inflight-async-resume-basis-id:sha256:{digest:x}"),
                ),
            request_identity: Arc::from(request_identity.request_identity().as_str().to_owned()),
            truth_view_basis_digest: Arc::from(truth_view_basis.digest().to_owned()),
            truth_branch_identity: truth_view_basis.truth_branch_identity().cloned(),
            request_generation,
            attempt: request_identity.attempt().get(),
            retention_complete,
            counters: BridgeSubscriptionCounters::from_resume_inflight_async_basis(),
            canonical_basis,
            digest: Arc::from(format!(
                "bridge-retained-inflight-async-resume-basis:sha256:{digest:x}"
            )),
        }
    }

    pub fn truth_branch_identity(&self) -> Option<&TruthBranchIdentity> {
        self.truth_branch_identity.as_ref()
    }

    pub fn request_generation(&self) -> Option<u64> {
        self.request_generation
    }

    pub fn retention_complete(&self) -> bool {
        self.retention_complete
    }

    pub fn counters(&self) -> &BridgeSubscriptionCounters {
        &self.counters
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}
