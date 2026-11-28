use std::sync::Arc;

use sha2::{Digest, Sha256};

use super::BridgeSubscriptionPreviewLifecyclePromotion;
use crate::subscription::{
    BridgeAdmittedSubscriptionIdentity, BridgeSubscriptionActivationReady,
    BridgeSubscriptionCounters, BridgeSubscriptionLifecycleIdentity,
    BridgeSubscriptionPreviewAuthoritativeReadmissionIdentity,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeSubscriptionAuthoritativePreviewReadmissionClass {
    ReAdmittedAuthoritativeBoundary,
}

impl BridgeSubscriptionAuthoritativePreviewReadmissionClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReAdmittedAuthoritativeBoundary => "re_admitted_authoritative_boundary",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeSubscriptionAuthoritativePreviewReadmissionRejectionKind {
    PromotedSubscriptionMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSubscriptionAuthoritativePreviewReadmissionRejection {
    rejection_kind: BridgeSubscriptionAuthoritativePreviewReadmissionRejectionKind,
    rejection_context: Arc<str>,
    counters: BridgeSubscriptionCounters,
    digest: Arc<str>,
}

impl BridgeSubscriptionAuthoritativePreviewReadmissionRejection {
    fn promoted_subscription_mismatch(
        promotion: &BridgeSubscriptionPreviewLifecyclePromotion,
        promoted_activation_ready: &BridgeSubscriptionActivationReady,
    ) -> Self {
        let rejection_context = Arc::<str>::from(format!(
            "preview-active={}|preview-admitted={}|promoted-admitted={}",
            promotion.preview_active_subscription_identity().as_str(),
            promotion.admitted_subscription_identity().as_str(),
            promoted_activation_ready
                .admitted()
                .admitted_subscription_identity()
                .as_str(),
        ));
        let canonical_basis = format!(
            "bridge-subscription-preview-authoritative-readmission-rejection|kind=promoted_subscription_mismatch|context={rejection_context}"
        );
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            rejection_kind:
                BridgeSubscriptionAuthoritativePreviewReadmissionRejectionKind::PromotedSubscriptionMismatch,
            rejection_context,
            counters: BridgeSubscriptionCounters::from_subscription_preview_promotion_rejection(
                false,
                false,
            ),
            digest: Arc::from(format!(
                "bridge-subscription-preview-authoritative-readmission-rejection:sha256:{digest:x}"
            )),
        }
    }

    pub fn rejection_kind(&self) -> BridgeSubscriptionAuthoritativePreviewReadmissionRejectionKind {
        self.rejection_kind
    }

    pub fn rejection_context(&self) -> &str {
        self.rejection_context.as_ref()
    }

    pub fn counters(&self) -> &BridgeSubscriptionCounters {
        &self.counters
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSubscriptionAuthoritativePreviewReadmission {
    readmission_identity: BridgeSubscriptionPreviewAuthoritativeReadmissionIdentity,
    readmission_class: BridgeSubscriptionAuthoritativePreviewReadmissionClass,
    promotion_identity: Arc<str>,
    preview_active_subscription_identity: Arc<str>,
    authoritative_admitted_subscription_identity: BridgeAdmittedSubscriptionIdentity,
    authoritative_lifecycle_identity: BridgeSubscriptionLifecycleIdentity,
    counters: BridgeSubscriptionCounters,
    digest: Arc<str>,
}

impl BridgeSubscriptionAuthoritativePreviewReadmission {
    pub(crate) fn prepare(
        promotion: BridgeSubscriptionPreviewLifecyclePromotion,
        promoted_activation_ready: &BridgeSubscriptionActivationReady,
    ) -> Result<
        Self,
        (
            BridgeSubscriptionPreviewLifecyclePromotion,
            BridgeSubscriptionAuthoritativePreviewReadmissionRejection,
        ),
    > {
        if promoted_activation_ready
            .admitted()
            .admitted_subscription_identity()
            != promotion.admitted_subscription_identity()
        {
            let denial =
                BridgeSubscriptionAuthoritativePreviewReadmissionRejection::promoted_subscription_mismatch(
                    &promotion,
                    promoted_activation_ready,
                );
            return Err((promotion, denial));
        }

        let readmission_class =
            BridgeSubscriptionAuthoritativePreviewReadmissionClass::ReAdmittedAuthoritativeBoundary;
        let canonical_basis = format!(
            "bridge-subscription-preview-authoritative-readmission|class={}|promotion={}|preview-active={}|authoritative-admitted={}|authoritative-lifecycle={}",
            readmission_class.as_str(),
            promotion.promotion_identity().as_str(),
            promotion.preview_active_subscription_identity().as_str(),
            promoted_activation_ready
                .admitted()
                .admitted_subscription_identity()
                .as_str(),
            promoted_activation_ready
                .lifecycle_record()
                .lifecycle_identity()
                .as_str(),
        );
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Ok(Self {
            readmission_identity:
                BridgeSubscriptionPreviewAuthoritativeReadmissionIdentity::admit_bridge_owned(
                    format!(
                    "bridge-subscription-preview-authoritative-readmission-id:sha256:{digest:x}"
                ),
                ),
            readmission_class,
            promotion_identity: Arc::from(promotion.promotion_identity().as_str()),
            preview_active_subscription_identity: Arc::from(
                promotion.preview_active_subscription_identity().as_str(),
            ),
            authoritative_admitted_subscription_identity: promoted_activation_ready
                .admitted()
                .admitted_subscription_identity()
                .clone(),
            authoritative_lifecycle_identity: promoted_activation_ready
                .lifecycle_record()
                .lifecycle_identity()
                .clone(),
            counters:
                BridgeSubscriptionCounters::from_subscription_preview_authoritative_readmission(),
            digest: Arc::from(format!(
                "bridge-subscription-preview-authoritative-readmission:sha256:{digest:x}"
            )),
        })
    }

    pub fn readmission_identity(
        &self,
    ) -> &BridgeSubscriptionPreviewAuthoritativeReadmissionIdentity {
        &self.readmission_identity
    }

    pub fn readmission_class(&self) -> BridgeSubscriptionAuthoritativePreviewReadmissionClass {
        self.readmission_class
    }

    pub fn promotion_identity(&self) -> &str {
        self.promotion_identity.as_ref()
    }

    pub fn preview_active_subscription_identity(&self) -> &str {
        self.preview_active_subscription_identity.as_ref()
    }

    pub fn authoritative_admitted_subscription_identity(
        &self,
    ) -> &BridgeAdmittedSubscriptionIdentity {
        &self.authoritative_admitted_subscription_identity
    }

    pub fn authoritative_lifecycle_identity(&self) -> &BridgeSubscriptionLifecycleIdentity {
        &self.authoritative_lifecycle_identity
    }

    pub fn counters(&self) -> &BridgeSubscriptionCounters {
        &self.counters
    }
}
