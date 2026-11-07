use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::identity_authority::{QueryProjectionIdentity, QuerySubscriptionIdentityKind};

use super::super::active_digest::ActiveSubscriptionLaneDigest;
use super::super::delivery_density::ActiveDeliveryDensityPosture;
use super::super::evidence_projection::subscription_evidence_projection;
use super::super::future_selection::QuerySubscriptionFutureSelection;
use super::super::performance_receipt::SubscriptionPerformanceReceipt;
use super::class::SubscriptionContinuationClass;
use super::evidence::SubscriptionContinuationEvidence;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubscriptionContinuationReport {
    active_lane_digest: ActiveSubscriptionLaneDigest,
    continuation_class: SubscriptionContinuationClass,
    continuation_identity: WorthQueryEvidenceIdentity,
    future_selection: QuerySubscriptionFutureSelection,
    checkpoint_identity: WorthQueryEvidenceIdentity,
    remap_width: u64,
    performance_receipt: SubscriptionPerformanceReceipt,
    report_identity: WorthQueryEvidenceIdentity,
}

impl SubscriptionContinuationReport {
    pub(super) fn new(evidence: &SubscriptionContinuationEvidence) -> Self {
        let performance_receipt = SubscriptionPerformanceReceipt::new(
            evidence.remap_width(),
            evidence.remap_width(),
            ActiveDeliveryDensityPosture::SparseDelta,
            super::super::active_budget::ActiveSubscriptionAllocationPosture::PatchScratch,
            evidence.evidence_identity(),
        );
        let report_identity = WorthQueryEvidenceIdentity::compose(
            crate::evidence_identity::WorthQueryEvidenceScope::SubscriptionActivationReceipt,
        )
        .field_shape(
            crate::evidence_identity::WorthQueryEvidenceTag::new("identity_family"),
            "subscription_continuation_report_v1",
        )
        .field_evidence_identity(
            crate::evidence_identity::WorthQueryEvidenceTag::new("lane"),
            evidence.active_lane_digest().evidence_identity(),
        )
        .field_evidence_identity(
            crate::evidence_identity::WorthQueryEvidenceTag::new("continuation"),
            evidence.evidence_identity(),
        )
        .field_evidence_identity(
            crate::evidence_identity::WorthQueryEvidenceTag::new("performance"),
            performance_receipt.performance_receipt_identity(),
        )
        .seal().expect("bounded scalar receipt grammar: literal and enum labels, primitive values and sealed identity tokens");
        Self {
            active_lane_digest: evidence.active_lane_digest().clone(),
            continuation_class: evidence.continuation_class(),
            continuation_identity: evidence.evidence_identity().clone(),
            future_selection: evidence.future_selection().clone(),
            checkpoint_identity: evidence.checkpoint_identity().clone(),
            remap_width: evidence.remap_width(),
            performance_receipt,
            report_identity,
        }
    }

    pub(crate) fn active_lane_digest(&self) -> &ActiveSubscriptionLaneDigest {
        &self.active_lane_digest
    }

    pub fn continuation_class(&self) -> SubscriptionContinuationClass {
        self.continuation_class
    }

    pub fn continuation_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.continuation_identity)
    }

    pub fn future_selection(&self) -> &QuerySubscriptionFutureSelection {
        &self.future_selection
    }

    pub fn checkpoint_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.checkpoint_identity)
    }

    pub fn checkpoint_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.checkpoint_identity
    }

    pub fn remap_width(&self) -> u64 {
        self.remap_width
    }

    pub fn performance_receipt(&self) -> &SubscriptionPerformanceReceipt {
        &self.performance_receipt
    }

    pub fn report_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.report_identity)
    }

    pub fn evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.report_identity
    }
}
