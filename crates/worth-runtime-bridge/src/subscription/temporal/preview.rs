use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::temporal::AdmittedBridgeTemporalBasis;

use super::family::{BridgeTemporalSubscriptionFamily, BridgeTemporalSubscriptionFamilyKind};
use crate::subscription::{
    AdmittedBridgeSubscription, BridgeSubscriptionActivationReady, BridgeSubscriptionCounters,
    BridgeSubscriptionFamilyRegistryIdentity, BridgeSubscriptionPreviewBasisBinding,
    BridgeSubscriptionPreviewTemporalActivationReadyIdentity,
    BridgeSubscriptionPreviewTemporalAdmissionIdentity,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgePreviewTemporalSubscriptionAdmissionRejectionKind {
    TemporalFamilyDoesNotSupportBasisKind,
    SubscriptionBranchIdentityMismatch,
    SubscriptionSnapshotIdentityMismatch,
    PreviewBranchIdentityMismatch,
    PreviewSnapshotIdentityMismatch,
}

impl BridgePreviewTemporalSubscriptionAdmissionRejectionKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TemporalFamilyDoesNotSupportBasisKind => {
                "temporal_family_does_not_support_basis_kind"
            }
            Self::SubscriptionBranchIdentityMismatch => "subscription_branch_identity_mismatch",
            Self::SubscriptionSnapshotIdentityMismatch => "subscription_snapshot_identity_mismatch",
            Self::PreviewBranchIdentityMismatch => "preview_branch_identity_mismatch",
            Self::PreviewSnapshotIdentityMismatch => "preview_snapshot_identity_mismatch",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgePreviewTemporalSubscriptionAdmissionRejection {
    rejection_kind: BridgePreviewTemporalSubscriptionAdmissionRejectionKind,
    family_kind: BridgeTemporalSubscriptionFamilyKind,
    counters: BridgeSubscriptionCounters,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl BridgePreviewTemporalSubscriptionAdmissionRejection {
    fn new(
        rejection_kind: BridgePreviewTemporalSubscriptionAdmissionRejectionKind,
        family_kind: BridgeTemporalSubscriptionFamilyKind,
        admitted: &AdmittedBridgeSubscription,
        preview_basis: &BridgeSubscriptionPreviewBasisBinding,
        temporal_basis: &AdmittedBridgeTemporalBasis,
    ) -> Self {
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-preview-temporal-subscription-admission-rejection|admitted={}|preview-basis={}|family={}|subscription-basis={}|temporal-basis={}|rejection-kind={}",
            admitted.admitted_subscription_identity().as_str(),
            preview_basis.preview_basis_identity().as_str(),
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
                "bridge-preview-temporal-subscription-admission-rejection:sha256:{digest:x}"
            )),
        }
    }

    pub fn rejection_kind(&self) -> BridgePreviewTemporalSubscriptionAdmissionRejectionKind {
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
pub struct AdmittedPreviewTemporalBridgeSubscription {
    preview_temporal_admission_identity: BridgeSubscriptionPreviewTemporalAdmissionIdentity,
    admitted: AdmittedBridgeSubscription,
    preview_basis: BridgeSubscriptionPreviewBasisBinding,
    temporal_basis: AdmittedBridgeTemporalBasis,
    family: BridgeTemporalSubscriptionFamily,
    counters: BridgeSubscriptionCounters,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl AdmittedPreviewTemporalBridgeSubscription {
    pub(crate) fn admit(
        admitted: &AdmittedBridgeSubscription,
        preview_basis: &BridgeSubscriptionPreviewBasisBinding,
        temporal_basis: AdmittedBridgeTemporalBasis,
        family_kind: BridgeTemporalSubscriptionFamilyKind,
    ) -> Result<Self, BridgePreviewTemporalSubscriptionAdmissionRejection> {
        let family = BridgeTemporalSubscriptionFamily::for_kind(family_kind);
        if !family.supports_basis_kind(temporal_basis.kind()) {
            return Err(BridgePreviewTemporalSubscriptionAdmissionRejection::new(
                BridgePreviewTemporalSubscriptionAdmissionRejectionKind::TemporalFamilyDoesNotSupportBasisKind,
                family_kind,
                admitted,
                preview_basis,
                &temporal_basis,
            ));
        }

        let subscription_basis = admitted.basis_binding();
        let temporal_truth_basis = temporal_basis.truth_basis().basis();
        if subscription_basis.snapshot_identity() != temporal_truth_basis.snapshot_identity() {
            return Err(BridgePreviewTemporalSubscriptionAdmissionRejection::new(
                BridgePreviewTemporalSubscriptionAdmissionRejectionKind::SubscriptionSnapshotIdentityMismatch,
                family_kind,
                admitted,
                preview_basis,
                &temporal_basis,
            ));
        }
        if let Some(branch_identity) = subscription_basis.branch_identity()
            && branch_identity != temporal_truth_basis.branch_identity()
        {
            return Err(BridgePreviewTemporalSubscriptionAdmissionRejection::new(
                    BridgePreviewTemporalSubscriptionAdmissionRejectionKind::SubscriptionBranchIdentityMismatch,
                    family_kind,
                    admitted,
                    preview_basis,
                    &temporal_basis,
                ));
        }
        if preview_basis.truth_snapshot_identity() != temporal_truth_basis.snapshot_identity() {
            return Err(BridgePreviewTemporalSubscriptionAdmissionRejection::new(
                BridgePreviewTemporalSubscriptionAdmissionRejectionKind::PreviewSnapshotIdentityMismatch,
                family_kind,
                admitted,
                preview_basis,
                &temporal_basis,
            ));
        }
        if preview_basis.truth_branch_identity() != temporal_truth_basis.branch_identity() {
            return Err(BridgePreviewTemporalSubscriptionAdmissionRejection::new(
                BridgePreviewTemporalSubscriptionAdmissionRejectionKind::PreviewBranchIdentityMismatch,
                family_kind,
                admitted,
                preview_basis,
                &temporal_basis,
            ));
        }

        let canonical_basis = Arc::<str>::from(format!(
            "bridge-preview-temporal-subscription-admission|admitted={}|preview-basis={}|family={}|subscription-basis={}|temporal-basis={}",
            admitted.admitted_subscription_identity().as_str(),
            preview_basis.preview_basis_identity().as_str(),
            family.kind().as_str(),
            subscription_basis.basis_identity().as_str(),
            temporal_basis.identity().as_str(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Ok(Self {
            preview_temporal_admission_identity:
                BridgeSubscriptionPreviewTemporalAdmissionIdentity::admit_bridge_owned(format!(
                    "bridge-preview-temporal-subscription-admission-id:sha256:{digest:x}"
                )),
            admitted: admitted.clone(),
            preview_basis: preview_basis.clone(),
            temporal_basis,
            family,
            counters: BridgeSubscriptionCounters::from_temporal_subscription_admission(),
            canonical_basis,
            digest: Arc::from(format!(
                "bridge-preview-temporal-subscription-admission:sha256:{digest:x}"
            )),
        })
    }

    pub fn preview_temporal_admission_identity(
        &self,
    ) -> &BridgeSubscriptionPreviewTemporalAdmissionIdentity {
        &self.preview_temporal_admission_identity
    }

    pub fn admitted(&self) -> &AdmittedBridgeSubscription {
        &self.admitted
    }

    pub fn preview_basis(&self) -> &BridgeSubscriptionPreviewBasisBinding {
        &self.preview_basis
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgePreviewTemporalSubscriptionActivationReady {
    preview_temporal_activation_ready_identity:
        BridgeSubscriptionPreviewTemporalActivationReadyIdentity,
    ordinary_activation_ready: BridgeSubscriptionActivationReady,
    preview_temporal_admission: AdmittedPreviewTemporalBridgeSubscription,
    counters: BridgeSubscriptionCounters,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl BridgePreviewTemporalSubscriptionActivationReady {
    pub(crate) fn prepare(
        registry_identity: &BridgeSubscriptionFamilyRegistryIdentity,
        preview_temporal_admission: &AdmittedPreviewTemporalBridgeSubscription,
    ) -> Self {
        let ordinary_activation_ready = BridgeSubscriptionActivationReady::prepare(
            registry_identity,
            preview_temporal_admission.admitted(),
        );
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-preview-temporal-subscription-activation-ready|ordinary={}|preview-temporal-admission={}|preview-basis={}|temporal-basis={}|family={}",
            ordinary_activation_ready.digest(),
            preview_temporal_admission
                .preview_temporal_admission_identity()
                .as_str(),
            preview_temporal_admission.preview_basis().preview_basis_identity().as_str(),
            preview_temporal_admission.temporal_basis().identity().as_str(),
            preview_temporal_admission.family().kind().as_str(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            preview_temporal_activation_ready_identity:
                BridgeSubscriptionPreviewTemporalActivationReadyIdentity::admit_bridge_owned(
                    format!(
                    "bridge-preview-temporal-subscription-activation-ready-id:sha256:{digest:x}"
                ),
                ),
            ordinary_activation_ready,
            preview_temporal_admission: preview_temporal_admission.clone(),
            counters: BridgeSubscriptionCounters::from_temporal_activation_ready(),
            canonical_basis,
            digest: Arc::from(format!(
                "bridge-preview-temporal-subscription-activation-ready:sha256:{digest:x}"
            )),
        }
    }

    pub fn preview_temporal_activation_ready_identity(
        &self,
    ) -> &BridgeSubscriptionPreviewTemporalActivationReadyIdentity {
        &self.preview_temporal_activation_ready_identity
    }

    pub fn ordinary_activation_ready(&self) -> &BridgeSubscriptionActivationReady {
        &self.ordinary_activation_ready
    }

    pub fn preview_temporal_admission(&self) -> &AdmittedPreviewTemporalBridgeSubscription {
        &self.preview_temporal_admission
    }

    pub fn counters(&self) -> &BridgeSubscriptionCounters {
        &self.counters
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}
