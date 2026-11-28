use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::subscription::{
    BridgeSubscriptionCounters, BridgeSubscriptionFamilyRegistryIdentity,
    BridgeSubscriptionHistoricalTemporalReadinessIdentity,
    BridgeSubscriptionHistoricalTemporalReplayBasisIdentity,
    BridgeSubscriptionHistoricalTemporalReplayRequestIdentity,
};
use crate::temporal::AdmittedBridgeTemporalBasis;

use super::admission::AdmittedTemporalBridgeSubscription;
use super::family::BridgeTemporalSubscriptionFamilyKind;
use super::historical_basis::{
    AdmittedBridgeHistoricalTruthViewBasis, RetainedHistoricalPreviousValueEvidence,
};
use super::historical_rejection::{
    BridgeHistoricalTemporalReplayRejection, BridgeHistoricalTemporalReplayRejectionKind,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedHistoricalTemporalReplayBasis {
    historical_temporal_replay_basis_identity:
        BridgeSubscriptionHistoricalTemporalReplayBasisIdentity,
    temporal_admission: AdmittedTemporalBridgeSubscription,
    historical_truth_basis: AdmittedBridgeHistoricalTruthViewBasis,
    retained_previous_values: RetainedHistoricalPreviousValueEvidence,
    counters: BridgeSubscriptionCounters,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl AdmittedHistoricalTemporalReplayBasis {
    pub(crate) fn admit(
        temporal_admission: &AdmittedTemporalBridgeSubscription,
        historical_truth_basis: &AdmittedBridgeHistoricalTruthViewBasis,
        retained_previous_values: RetainedHistoricalPreviousValueEvidence,
    ) -> Result<Self, BridgeHistoricalTemporalReplayRejection> {
        if temporal_admission.family().kind()
            != BridgeTemporalSubscriptionFamilyKind::HistoricalReplay
        {
            return Err(BridgeHistoricalTemporalReplayRejection::new(
                BridgeHistoricalTemporalReplayRejectionKind::TemporalAdmissionFamilyNotHistoricalReplay,
                temporal_admission.family().kind(),
                temporal_admission.temporal_admission_identity(),
                None,
                temporal_admission.temporal_basis().identity().as_str(),
            ));
        }

        let truth_basis = temporal_admission.temporal_basis().truth_basis().basis();
        if truth_basis.kind() != historical_truth_basis.truth_basis().basis().kind() {
            return Err(BridgeHistoricalTemporalReplayRejection::new(
                BridgeHistoricalTemporalReplayRejectionKind::TemporalTruthBasisNotHistorical,
                temporal_admission.family().kind(),
                temporal_admission.temporal_admission_identity(),
                None,
                &truth_basis.native_truth_locator(),
            ));
        }
        let historical_truth = historical_truth_basis.truth_basis().basis();
        if truth_basis.snapshot_identity() != historical_truth.snapshot_identity() {
            return Err(BridgeHistoricalTemporalReplayRejection::new(
                BridgeHistoricalTemporalReplayRejectionKind::HistoricalTruthSnapshotIdentityMismatch,
                temporal_admission.family().kind(),
                temporal_admission.temporal_admission_identity(),
                None,
                &historical_truth.snapshot_identity().canonical_label(),
            ));
        }
        if truth_basis.branch_identity() != historical_truth.branch_identity() {
            return Err(BridgeHistoricalTemporalReplayRejection::new(
                BridgeHistoricalTemporalReplayRejectionKind::HistoricalTruthBranchIdentityMismatch,
                temporal_admission.family().kind(),
                temporal_admission.temporal_admission_identity(),
                None,
                &historical_truth.branch_identity().canonical_label(),
            ));
        }
        if temporal_admission.temporal_basis().truth_basis() != historical_truth_basis.truth_basis()
        {
            return Err(BridgeHistoricalTemporalReplayRejection::new(
                BridgeHistoricalTemporalReplayRejectionKind::TemporalBasisIdentityMismatch,
                temporal_admission.family().kind(),
                temporal_admission.temporal_admission_identity(),
                None,
                historical_truth_basis
                    .historical_truth_basis_identity()
                    .as_str(),
            ));
        }
        if truth_basis.snapshot_identity() != retained_previous_values.truth_snapshot_identity() {
            return Err(BridgeHistoricalTemporalReplayRejection::new(
                BridgeHistoricalTemporalReplayRejectionKind::PreviousValueEvidenceSnapshotMismatch,
                temporal_admission.family().kind(),
                temporal_admission.temporal_admission_identity(),
                None,
                &retained_previous_values
                    .truth_snapshot_identity()
                    .canonical_label(),
            ));
        }
        if retained_previous_values.is_empty() {
            return Err(BridgeHistoricalTemporalReplayRejection::new(
                BridgeHistoricalTemporalReplayRejectionKind::MissingPreviousValueEvidence,
                temporal_admission.family().kind(),
                temporal_admission.temporal_admission_identity(),
                None,
                retained_previous_values.digest(),
            ));
        }
        if retained_previous_values.truth_branch_identity() != truth_basis.branch_identity() {
            return Err(BridgeHistoricalTemporalReplayRejection::new(
                BridgeHistoricalTemporalReplayRejectionKind::PreviousValueEvidenceBranchMismatch,
                temporal_admission.family().kind(),
                temporal_admission.temporal_admission_identity(),
                None,
                &truth_basis.branch_identity().canonical_label(),
            ));
        }

        let canonical_basis = Arc::<str>::from(format!(
            "bridge-historical-temporal-replay-basis|temporal-admission={}|temporal-basis={}|historical-truth={}|previous-values={}",
            temporal_admission.temporal_admission_identity().as_str(),
            temporal_admission.temporal_basis().identity().as_str(),
            historical_truth_basis.historical_truth_basis_identity().as_str(),
            retained_previous_values.previous_value_evidence_identity().as_str(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Ok(Self {
            historical_temporal_replay_basis_identity:
                BridgeSubscriptionHistoricalTemporalReplayBasisIdentity::admit_bridge_owned(
                    format!("bridge-historical-temporal-replay-basis-id:sha256:{digest:x}"),
                ),
            temporal_admission: temporal_admission.clone(),
            historical_truth_basis: historical_truth_basis.clone(),
            retained_previous_values,
            counters: BridgeSubscriptionCounters::from_historical_temporal_replay_basis_admission(),
            canonical_basis,
            digest: Arc::from(format!(
                "bridge-historical-temporal-replay-basis:sha256:{digest:x}"
            )),
        })
    }

    pub fn historical_temporal_replay_basis_identity(
        &self,
    ) -> &BridgeSubscriptionHistoricalTemporalReplayBasisIdentity {
        &self.historical_temporal_replay_basis_identity
    }

    pub fn temporal_admission(&self) -> &AdmittedTemporalBridgeSubscription {
        &self.temporal_admission
    }

    pub fn historical_truth_basis(&self) -> &AdmittedBridgeHistoricalTruthViewBasis {
        &self.historical_truth_basis
    }

    pub fn retained_previous_values(&self) -> &RetainedHistoricalPreviousValueEvidence {
        &self.retained_previous_values
    }

    pub fn counters(&self) -> &BridgeSubscriptionCounters {
        &self.counters
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeHistoricalTemporalSubscriptionReplayRequest {
    historical_temporal_replay_request_identity:
        BridgeSubscriptionHistoricalTemporalReplayRequestIdentity,
    replay_basis: AdmittedHistoricalTemporalReplayBasis,
    counters: BridgeSubscriptionCounters,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl BridgeHistoricalTemporalSubscriptionReplayRequest {
    pub(crate) fn prepare(replay_basis: &AdmittedHistoricalTemporalReplayBasis) -> Self {
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-historical-temporal-subscription-replay-request|replay-basis={}",
            replay_basis
                .historical_temporal_replay_basis_identity()
                .as_str(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            historical_temporal_replay_request_identity:
                BridgeSubscriptionHistoricalTemporalReplayRequestIdentity::admit_bridge_owned(
                    format!(
                    "bridge-historical-temporal-subscription-replay-request-id:sha256:{digest:x}"
                ),
                ),
            replay_basis: replay_basis.clone(),
            counters: BridgeSubscriptionCounters::from_historical_temporal_replay_basis_admission(),
            canonical_basis,
            digest: Arc::from(format!(
                "bridge-historical-temporal-subscription-replay-request:sha256:{digest:x}"
            )),
        }
    }

    pub fn historical_temporal_replay_request_identity(
        &self,
    ) -> &BridgeSubscriptionHistoricalTemporalReplayRequestIdentity {
        &self.historical_temporal_replay_request_identity
    }

    pub fn replay_basis(&self) -> &AdmittedHistoricalTemporalReplayBasis {
        &self.replay_basis
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeHistoricalTemporalReadiness {
    historical_temporal_readiness_identity: BridgeSubscriptionHistoricalTemporalReadinessIdentity,
    replay_request: BridgeHistoricalTemporalSubscriptionReplayRequest,
    counters: BridgeSubscriptionCounters,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl BridgeHistoricalTemporalReadiness {
    pub(crate) fn prepare(
        registry_identity: &BridgeSubscriptionFamilyRegistryIdentity,
        replay_request: &BridgeHistoricalTemporalSubscriptionReplayRequest,
    ) -> Self {
        let replay_basis = replay_request.replay_basis();
        let temporal_basis: &AdmittedBridgeTemporalBasis =
            replay_basis.temporal_admission().temporal_basis();
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-historical-temporal-readiness|registry={}|replay-request={}|temporal-admission={}|historical-truth={}|temporal-basis={}|previous-values={}",
            registry_identity.as_str(),
            replay_request.historical_temporal_replay_request_identity().as_str(),
            replay_basis.temporal_admission().temporal_admission_identity().as_str(),
            replay_basis.historical_truth_basis().historical_truth_basis_identity().as_str(),
            temporal_basis.identity().as_str(),
            replay_basis
                .retained_previous_values()
                .previous_value_evidence_identity()
                .as_str(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            historical_temporal_readiness_identity:
                BridgeSubscriptionHistoricalTemporalReadinessIdentity::admit_bridge_owned(format!(
                    "bridge-historical-temporal-readiness-id:sha256:{digest:x}"
                )),
            replay_request: replay_request.clone(),
            counters: BridgeSubscriptionCounters::from_historical_temporal_readiness(),
            canonical_basis,
            digest: Arc::from(format!(
                "bridge-historical-temporal-readiness:sha256:{digest:x}"
            )),
        }
    }

    pub fn historical_temporal_readiness_identity(
        &self,
    ) -> &BridgeSubscriptionHistoricalTemporalReadinessIdentity {
        &self.historical_temporal_readiness_identity
    }

    pub fn replay_request(&self) -> &BridgeHistoricalTemporalSubscriptionReplayRequest {
        &self.replay_request
    }

    pub fn counters(&self) -> &BridgeSubscriptionCounters {
        &self.counters
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}
