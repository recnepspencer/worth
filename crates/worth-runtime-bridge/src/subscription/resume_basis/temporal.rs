use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::input::envelope::TruthBranchIdentity;
use crate::subscription::{
    BridgeSubscriptionCounters, BridgeSubscriptionRetainedTemporalResumeBasisIdentity,
    RetainedHistoricalPreviousValueEvidence,
};
use crate::temporal::AdmittedBridgeTemporalBasis;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeRetainedTemporalWakePosture {
    Pending,
    Ready,
}

impl BridgeRetainedTemporalWakePosture {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Ready => "ready",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeRetainedTemporalResumeBasis {
    retained_temporal_resume_basis_identity: BridgeSubscriptionRetainedTemporalResumeBasisIdentity,
    temporal_basis_identity: Arc<str>,
    truth_branch_identity: TruthBranchIdentity,
    wake_posture: BridgeRetainedTemporalWakePosture,
    previous_value_evidence_digest: Option<Arc<str>>,
    retention_complete: bool,
    counters: BridgeSubscriptionCounters,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl BridgeRetainedTemporalResumeBasis {
    pub(crate) fn capture(
        admitted_temporal_basis: &AdmittedBridgeTemporalBasis,
        wake_posture: BridgeRetainedTemporalWakePosture,
        previous_value_evidence: Option<&RetainedHistoricalPreviousValueEvidence>,
        retention_complete: bool,
    ) -> Self {
        let previous_value_evidence_digest =
            previous_value_evidence.map(|evidence| Arc::from(evidence.digest().to_owned()));
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-retained-temporal-resume-basis|temporal-basis={}|branch={}|wake-posture={}|previous-value={}|retention-complete={retention_complete}",
            admitted_temporal_basis.identity().as_str(),
            admitted_temporal_basis.truth_basis().basis().branch_identity().canonical_label(),
            wake_posture.as_str(),
            previous_value_evidence_digest.as_deref().unwrap_or("-"),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            retained_temporal_resume_basis_identity:
                BridgeSubscriptionRetainedTemporalResumeBasisIdentity::admit_bridge_owned(format!(
                    "bridge-retained-temporal-resume-basis-id:sha256:{digest:x}"
                )),
            temporal_basis_identity: Arc::from(
                admitted_temporal_basis.identity().as_str().to_owned(),
            ),
            truth_branch_identity: admitted_temporal_basis
                .truth_basis()
                .basis()
                .branch_identity()
                .clone(),
            wake_posture,
            previous_value_evidence_digest,
            retention_complete,
            counters: BridgeSubscriptionCounters::from_resume_temporal_basis(),
            canonical_basis,
            digest: Arc::from(format!(
                "bridge-retained-temporal-resume-basis:sha256:{digest:x}"
            )),
        }
    }

    pub fn truth_branch_identity(&self) -> &TruthBranchIdentity {
        &self.truth_branch_identity
    }

    pub fn wake_posture(&self) -> BridgeRetainedTemporalWakePosture {
        self.wake_posture
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
