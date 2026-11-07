use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

use super::active_budget::ActiveSubscriptionAllocationPosture;
use super::delivery_density::ActiveDeliveryDensityPosture;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubscriptionPerformanceReceipt {
    consumed_width: u64,
    budgeted_width: u64,
    remaining_width: u64,
    density_posture: ActiveDeliveryDensityPosture,
    allocation_posture: ActiveSubscriptionAllocationPosture,
    performance_receipt_identity: WorthQueryEvidenceIdentity,
}

impl SubscriptionPerformanceReceipt {
    pub(super) fn new(
        consumed_width: u64,
        budgeted_width: u64,
        density_posture: ActiveDeliveryDensityPosture,
        allocation_posture: ActiveSubscriptionAllocationPosture,
        source_identity: &WorthQueryEvidenceIdentity,
    ) -> Self {
        let remaining_width = budgeted_width.saturating_sub(consumed_width);
        let performance_receipt_identity = WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::SubscriptionActivationReceipt,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "subscription_performance_receipt_v1",
        )
        .field_evidence_identity(WorthQueryEvidenceTag::new("source"), source_identity)
        .field_usize(
            WorthQueryEvidenceTag::new("consumed_width"),
            consumed_width as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("budgeted_width"),
            budgeted_width as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("remaining_width"),
            remaining_width as usize,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("density"),
            density_posture.as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("allocation"),
            allocation_posture.as_str(),
        )
        .seal().expect("fixed digest grammar: literal tags, primitive values, closed enum labels, or sealed digest tokens");
        Self {
            consumed_width,
            budgeted_width,
            remaining_width,
            density_posture,
            allocation_posture,
            performance_receipt_identity,
        }
    }

    pub fn consumed_width(&self) -> u64 {
        self.consumed_width
    }

    pub fn budgeted_width(&self) -> u64 {
        self.budgeted_width
    }

    pub fn remaining_width(&self) -> u64 {
        self.remaining_width
    }

    pub fn density_posture(&self) -> ActiveDeliveryDensityPosture {
        self.density_posture
    }

    pub fn allocation_posture(&self) -> ActiveSubscriptionAllocationPosture {
        self.allocation_posture
    }

    pub fn performance_receipt_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.performance_receipt_identity
    }
}
