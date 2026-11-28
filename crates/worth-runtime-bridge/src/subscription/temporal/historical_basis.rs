use std::sync::Arc;

use sha2::{Digest, Sha256};
use worth_signal::facade::TemporalPreviousValueReference;

use crate::input::envelope::TruthBranchIdentity;
use crate::snapshot::TruthSnapshotIdentity;
use crate::subscription::{
    BridgeSubscriptionCounters, BridgeSubscriptionHistoricalPreviousValueEvidenceIdentity,
    BridgeSubscriptionHistoricalTruthBasisIdentity,
};
use crate::temporal::{AdmittedBridgeTemporalTruthViewBasis, BridgeTemporalBasisKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeHistoricalTruthBasisAdmissionRejectionKind {
    TemporalTruthBasisNotHistorical,
}

impl BridgeHistoricalTruthBasisAdmissionRejectionKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TemporalTruthBasisNotHistorical => "temporal_truth_basis_not_historical",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeHistoricalTruthBasisAdmissionRejection {
    rejection_kind: BridgeHistoricalTruthBasisAdmissionRejectionKind,
    counters: BridgeSubscriptionCounters,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl BridgeHistoricalTruthBasisAdmissionRejection {
    fn new(
        rejection_kind: BridgeHistoricalTruthBasisAdmissionRejectionKind,
        truth_basis: &AdmittedBridgeTemporalTruthViewBasis,
    ) -> Self {
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-historical-truth-basis-admission-rejection|rejection-kind={}|truth-kind={}|truth-branch={}|truth-snapshot={}|truth-locator={}",
            rejection_kind.as_str(),
            truth_basis.basis().kind().canonical_label(),
            truth_basis.basis().branch_identity().canonical_label(),
            truth_basis.basis().snapshot_identity().canonical_label(),
            truth_basis.basis().native_truth_locator(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            rejection_kind,
            counters: BridgeSubscriptionCounters::from_historical_temporal_replay_rejection(),
            canonical_basis,
            digest: Arc::from(format!(
                "bridge-historical-truth-basis-admission-rejection:sha256:{digest:x}"
            )),
        }
    }

    pub fn rejection_kind(&self) -> BridgeHistoricalTruthBasisAdmissionRejectionKind {
        self.rejection_kind
    }

    pub fn counters(&self) -> &BridgeSubscriptionCounters {
        &self.counters
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedBridgeHistoricalTruthViewBasis {
    historical_truth_basis_identity: BridgeSubscriptionHistoricalTruthBasisIdentity,
    truth_basis: AdmittedBridgeTemporalTruthViewBasis,
    counters: BridgeSubscriptionCounters,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl AdmittedBridgeHistoricalTruthViewBasis {
    pub(crate) fn admit(
        truth_basis: &AdmittedBridgeTemporalTruthViewBasis,
    ) -> Result<Self, BridgeHistoricalTruthBasisAdmissionRejection> {
        let basis = truth_basis.basis();
        if !matches!(
            basis.kind(),
            BridgeTemporalBasisKind::Historical | BridgeTemporalBasisKind::CdcCursor
        ) {
            return Err(BridgeHistoricalTruthBasisAdmissionRejection::new(
                BridgeHistoricalTruthBasisAdmissionRejectionKind::TemporalTruthBasisNotHistorical,
                truth_basis,
            ));
        }
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-historical-truth-view-basis|temporal-truth-digest={:?}",
            truth_basis.canonical_digest().value().bytes(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Ok(Self {
            historical_truth_basis_identity:
                BridgeSubscriptionHistoricalTruthBasisIdentity::admit_bridge_owned(format!(
                    "bridge-historical-truth-view-basis-id:sha256:{digest:x}"
                )),
            truth_basis: truth_basis.clone(),
            counters: BridgeSubscriptionCounters::from_historical_truth_basis_admission(),
            canonical_basis,
            digest: Arc::from(format!(
                "bridge-historical-truth-view-basis:sha256:{digest:x}"
            )),
        })
    }

    pub fn historical_truth_basis_identity(
        &self,
    ) -> &BridgeSubscriptionHistoricalTruthBasisIdentity {
        &self.historical_truth_basis_identity
    }

    pub fn truth_basis(&self) -> &AdmittedBridgeTemporalTruthViewBasis {
        &self.truth_basis
    }

    pub fn counters(&self) -> &BridgeSubscriptionCounters {
        &self.counters
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetainedHistoricalPreviousValueEvidence {
    previous_value_evidence_identity: BridgeSubscriptionHistoricalPreviousValueEvidenceIdentity,
    truth_branch_identity: TruthBranchIdentity,
    truth_snapshot_identity: TruthSnapshotIdentity,
    references: Vec<TemporalPreviousValueReference>,
    counters: BridgeSubscriptionCounters,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl RetainedHistoricalPreviousValueEvidence {
    pub(crate) fn retain(
        truth_branch_identity: TruthBranchIdentity,
        truth_snapshot_identity: TruthSnapshotIdentity,
        references: Vec<TemporalPreviousValueReference>,
    ) -> Self {
        let evidence_rows = references
            .iter()
            .map(|reference| {
                format!(
                    "{}:{}:{}:{}:{}:{}:{}",
                    reference.revision().get(),
                    reference.branch_id().0,
                    reference.access_wake_id().get(),
                    reference.node(),
                    reference.captured_at_tick().get(),
                    serde_json::to_string(&reference.aspect_version())
                        .expect("aspect version should serialize"),
                    reference
                        .output_identity()
                        .map(|it| it.as_str())
                        .unwrap_or("-"),
                )
            })
            .collect::<Vec<_>>()
            .join("|");
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-historical-previous-value-evidence|branch={}|snapshot={}|count={}|rows={evidence_rows}",
            truth_branch_identity.canonical_label(),
            truth_snapshot_identity.canonical_label(),
            references.len(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            previous_value_evidence_identity:
                BridgeSubscriptionHistoricalPreviousValueEvidenceIdentity::admit_bridge_owned(
                    format!("bridge-historical-previous-value-evidence-id:sha256:{digest:x}"),
                ),
            truth_branch_identity,
            truth_snapshot_identity,
            references,
            counters: BridgeSubscriptionCounters::from_historical_previous_value_evidence(),
            canonical_basis,
            digest: Arc::from(format!(
                "bridge-historical-previous-value-evidence:sha256:{digest:x}"
            )),
        }
    }

    pub fn previous_value_evidence_identity(
        &self,
    ) -> &BridgeSubscriptionHistoricalPreviousValueEvidenceIdentity {
        &self.previous_value_evidence_identity
    }

    pub fn truth_snapshot_identity(&self) -> &TruthSnapshotIdentity {
        &self.truth_snapshot_identity
    }

    pub fn truth_branch_identity(&self) -> &TruthBranchIdentity {
        &self.truth_branch_identity
    }

    pub fn references(&self) -> &[TemporalPreviousValueReference] {
        &self.references
    }

    pub fn is_empty(&self) -> bool {
        self.references.is_empty()
    }

    pub fn counters(&self) -> &BridgeSubscriptionCounters {
        &self.counters
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}
