use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::temporal::AdmittedBridgeTemporalBasis;

use super::family::{BridgeTemporalSubscriptionFamily, BridgeTemporalSubscriptionFamilyKind};
use crate::subscription::{
    AdmittedBridgeSubscription, BridgeSubscriptionCounters,
    BridgeSubscriptionTemporalAdmissionIdentity,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeTemporalSubscriptionAdmissionRejectionKind {
    TemporalFamilyDoesNotSupportBasisKind,
    BranchIdentityMismatch,
    SnapshotIdentityMismatch,
}

impl BridgeTemporalSubscriptionAdmissionRejectionKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TemporalFamilyDoesNotSupportBasisKind => {
                "temporal_family_does_not_support_basis_kind"
            }
            Self::BranchIdentityMismatch => "branch_identity_mismatch",
            Self::SnapshotIdentityMismatch => "snapshot_identity_mismatch",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeTemporalSubscriptionAdmissionRejection {
    rejection_kind: BridgeTemporalSubscriptionAdmissionRejectionKind,
    family_kind: BridgeTemporalSubscriptionFamilyKind,
    counters: BridgeSubscriptionCounters,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl BridgeTemporalSubscriptionAdmissionRejection {
    fn new(
        rejection_kind: BridgeTemporalSubscriptionAdmissionRejectionKind,
        family_kind: BridgeTemporalSubscriptionFamilyKind,
        admitted: &AdmittedBridgeSubscription,
        temporal_basis: &AdmittedBridgeTemporalBasis,
    ) -> Self {
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-temporal-subscription-admission-rejection|admitted={}|family={}|subscription-basis={}|temporal-basis={}|rejection-kind={}",
            admitted.admitted_subscription_identity().as_str(),
            family_kind.as_str(),
            admitted.basis_binding().basis_identity().as_str(),
            temporal_basis.identity().as_str(),
            rejection_kind.as_str(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            rejection_kind,
            family_kind,
            counters: BridgeSubscriptionCounters::from_temporal_subscription_rejection(),
            canonical_basis,
            digest: Arc::from(format!(
                "bridge-temporal-subscription-admission-rejection:sha256:{digest:x}"
            )),
        }
    }

    pub fn rejection_kind(&self) -> BridgeTemporalSubscriptionAdmissionRejectionKind {
        self.rejection_kind
    }

    pub fn family_kind(&self) -> BridgeTemporalSubscriptionFamilyKind {
        self.family_kind
    }

    pub fn counters(&self) -> &BridgeSubscriptionCounters {
        &self.counters
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedTemporalBridgeSubscription {
    temporal_admission_identity: BridgeSubscriptionTemporalAdmissionIdentity,
    admitted: AdmittedBridgeSubscription,
    temporal_basis: AdmittedBridgeTemporalBasis,
    family: BridgeTemporalSubscriptionFamily,
    counters: BridgeSubscriptionCounters,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl AdmittedTemporalBridgeSubscription {
    pub(crate) fn admit(
        admitted: &AdmittedBridgeSubscription,
        temporal_basis: AdmittedBridgeTemporalBasis,
        family_kind: BridgeTemporalSubscriptionFamilyKind,
    ) -> Result<Self, BridgeTemporalSubscriptionAdmissionRejection> {
        let family = BridgeTemporalSubscriptionFamily::for_kind(family_kind);
        if !family.supports_basis_kind(temporal_basis.kind()) {
            return Err(BridgeTemporalSubscriptionAdmissionRejection::new(
                BridgeTemporalSubscriptionAdmissionRejectionKind::TemporalFamilyDoesNotSupportBasisKind,
                family_kind,
                admitted,
                &temporal_basis,
            ));
        }

        let subscription_basis = admitted.basis_binding();
        let temporal_truth_basis = temporal_basis.truth_basis().basis();
        if subscription_basis.snapshot_identity() != temporal_truth_basis.snapshot_identity() {
            return Err(BridgeTemporalSubscriptionAdmissionRejection::new(
                BridgeTemporalSubscriptionAdmissionRejectionKind::SnapshotIdentityMismatch,
                family_kind,
                admitted,
                &temporal_basis,
            ));
        }
        if let Some(branch_identity) = subscription_basis.branch_identity()
            && branch_identity != temporal_truth_basis.branch_identity()
        {
            return Err(BridgeTemporalSubscriptionAdmissionRejection::new(
                BridgeTemporalSubscriptionAdmissionRejectionKind::BranchIdentityMismatch,
                family_kind,
                admitted,
                &temporal_basis,
            ));
        }

        let canonical_basis = Arc::<str>::from(format!(
            "bridge-temporal-subscription-admission|admitted={}|family={}|subscription-basis={}|temporal-basis={}",
            admitted.admitted_subscription_identity().as_str(),
            family.kind().as_str(),
            subscription_basis.basis_identity().as_str(),
            temporal_basis.identity().as_str(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Ok(Self {
            temporal_admission_identity:
                BridgeSubscriptionTemporalAdmissionIdentity::admit_bridge_owned(format!(
                    "bridge-temporal-subscription-admission-id:sha256:{digest:x}"
                )),
            admitted: admitted.clone(),
            temporal_basis,
            family,
            counters: BridgeSubscriptionCounters::from_temporal_subscription_admission(),
            canonical_basis,
            digest: Arc::from(format!(
                "bridge-temporal-subscription-admission:sha256:{digest:x}"
            )),
        })
    }

    pub fn temporal_admission_identity(&self) -> &BridgeSubscriptionTemporalAdmissionIdentity {
        &self.temporal_admission_identity
    }

    pub fn admitted(&self) -> &AdmittedBridgeSubscription {
        &self.admitted
    }

    pub fn temporal_basis(&self) -> &AdmittedBridgeTemporalBasis {
        &self.temporal_basis
    }

    pub fn family(&self) -> BridgeTemporalSubscriptionFamily {
        self.family
    }

    pub fn counters(&self) -> &BridgeSubscriptionCounters {
        &self.counters
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}
