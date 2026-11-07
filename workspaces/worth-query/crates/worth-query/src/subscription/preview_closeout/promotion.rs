use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::identity_authority::{QueryProjectionIdentity, QuerySubscriptionIdentityKind};

use super::super::active_counters::ActiveSubscriptionCounters;
use super::super::active_digest::ActiveSubscriptionLaneDigest;
use super::super::active_handle::ActiveSubscriptionLaneHandle;
use super::super::attachment_digest::SubscriptionConsumerAttachmentDigest;
use super::super::delivery_density::ActiveDeliveryDensityPosture;
use super::super::evidence_identities::{
    preview_promotion_authority_identity, preview_promotion_handoff_identity,
    preview_promotion_rebinding_identity,
};
use super::super::evidence_projection::subscription_evidence_projection;
use super::super::future_selection::QuerySubscriptionFutureSelection;
use super::super::performance_receipt::SubscriptionPerformanceReceipt;
use super::super::preview_isolation::{
    PreviewSubscriptionIsolationArtifact, PreviewSubscriptionLifecycleState,
};
use super::super::preview_isolation_error::{
    PreviewSubscriptionIsolationDenialKind, PreviewSubscriptionIsolationError,
};
use super::super::preview_residue::PreviewSubscriptionResidueReport;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreviewSubscriptionPromotionHandoff {
    preview_lane_digest: ActiveSubscriptionLaneDigest,
    authoritative_active_lane_digest: ActiveSubscriptionLaneDigest,
    attachment_digest: SubscriptionConsumerAttachmentDigest,
    future_selection: QuerySubscriptionFutureSelection,
    preview_basis_binding_identity: WorthQueryEvidenceIdentity,
    authoritative_basis_binding_identity: WorthQueryEvidenceIdentity,
    preview_checkpoint_identity: WorthQueryEvidenceIdentity,
    authoritative_checkpoint_identity: WorthQueryEvidenceIdentity,
    preview_epoch_identity: WorthQueryEvidenceIdentity,
    residue_report_identity: WorthQueryEvidenceIdentity,
    authority_identity: WorthQueryEvidenceIdentity,
    rebinding_identity: WorthQueryEvidenceIdentity,
    performance_receipt: SubscriptionPerformanceReceipt,
    counters: ActiveSubscriptionCounters,
    handoff_identity: WorthQueryEvidenceIdentity,
}

impl PreviewSubscriptionPromotionHandoff {
    pub(super) fn new(
        isolation: PreviewSubscriptionIsolationArtifact,
        residue_report: &PreviewSubscriptionResidueReport,
        authoritative_lane: &ActiveSubscriptionLaneHandle,
        authority_label: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        let authority_identity = preview_promotion_authority_identity(&authority_label.into())?;
        let mut counters = ActiveSubscriptionCounters::default();
        counters.preview_promotion_handoff_count = 1;
        let performance_receipt = SubscriptionPerformanceReceipt::new(
            1,
            1,
            ActiveDeliveryDensityPosture::SparseDelta,
            super::super::active_budget::ActiveSubscriptionAllocationPosture::LifecycleArena,
            isolation.attachment_digest().evidence_identity(),
        )?;
        let rebinding_identity = preview_promotion_rebinding_identity(
            isolation.basis_binding_identity(),
            authoritative_lane.basis_binding_identity(),
            isolation.checkpoint_identity(),
            authoritative_lane.checkpoint_identity(),
        )?;
        counters.subscription_performance_receipt_count = 1;
        counters.subscription_budget_consumption_width = performance_receipt.consumed_width();
        counters.subscription_budget_remaining_width = performance_receipt.remaining_width();
        let handoff_identity = preview_promotion_handoff_identity(
            isolation.active_lane_digest().evidence_identity(),
            authoritative_lane.lane_digest().evidence_identity(),
            isolation.attachment_digest().evidence_identity(),
            isolation.future_selection().projection_identity(),
            isolation.basis_binding_identity(),
            isolation.checkpoint_identity(),
            authoritative_lane.checkpoint_identity(),
            isolation.preview_epoch_identity(),
            isolation.isolation_identity(),
            residue_report.report_identity(),
            &authority_identity,
            &rebinding_identity,
            performance_receipt.performance_receipt_identity(),
            PreviewSubscriptionLifecycleState::PreviewPromoted.as_str(),
            &counters.evidence_identity(),
        )?;
        Ok(Self {
            preview_lane_digest: isolation.active_lane_digest().clone(),
            authoritative_active_lane_digest: authoritative_lane.lane_digest().clone(),
            attachment_digest: isolation.attachment_digest().clone(),
            future_selection: isolation.future_selection().clone(),
            preview_basis_binding_identity: isolation.basis_binding_identity().clone(),
            authoritative_basis_binding_identity: authoritative_lane
                .basis_binding_identity()
                .clone(),
            preview_checkpoint_identity: isolation.checkpoint_identity().clone(),
            authoritative_checkpoint_identity: authoritative_lane.checkpoint_identity().clone(),
            preview_epoch_identity: isolation.preview_epoch_identity().clone(),
            residue_report_identity: residue_report.report_identity().clone(),
            authority_identity,
            rebinding_identity,
            performance_receipt,
            counters,
            handoff_identity,
        })
    }

    pub(crate) fn preview_lane_digest(&self) -> &ActiveSubscriptionLaneDigest {
        &self.preview_lane_digest
    }

    pub(crate) fn authoritative_active_lane_digest(&self) -> &ActiveSubscriptionLaneDigest {
        &self.authoritative_active_lane_digest
    }

    pub(crate) fn attachment_digest(&self) -> &SubscriptionConsumerAttachmentDigest {
        &self.attachment_digest
    }

    pub fn future_selection(&self) -> &QuerySubscriptionFutureSelection {
        &self.future_selection
    }

    pub fn preview_basis_binding_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.preview_basis_binding_identity)
    }

    pub fn preview_basis_binding_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.preview_basis_binding_identity
    }

    pub fn authoritative_basis_binding_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.authoritative_basis_binding_identity)
    }

    pub fn authoritative_basis_binding_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.authoritative_basis_binding_identity
    }

    pub fn preview_checkpoint_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.preview_checkpoint_identity)
    }

    pub fn preview_checkpoint_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.preview_checkpoint_identity
    }

    pub fn authoritative_checkpoint_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.authoritative_checkpoint_identity)
    }

    pub fn authoritative_checkpoint_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.authoritative_checkpoint_identity
    }

    pub fn preview_epoch_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.preview_epoch_identity)
    }

    pub fn preview_epoch_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.preview_epoch_identity
    }

    pub fn residue_report_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.residue_report_identity)
    }

    pub fn residue_report_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.residue_report_identity
    }

    pub fn authority_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.authority_identity)
    }

    pub fn authority_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.authority_identity
    }

    pub fn rebinding_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.rebinding_identity)
    }

    pub fn rebinding_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.rebinding_identity
    }

    pub fn counters(&self) -> &ActiveSubscriptionCounters {
        &self.counters
    }

    pub fn performance_receipt(&self) -> &SubscriptionPerformanceReceipt {
        &self.performance_receipt
    }

    pub fn handoff_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.handoff_identity)
    }

    pub fn handoff_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.handoff_identity
    }
}

pub fn promote_preview_subscription(
    isolation: PreviewSubscriptionIsolationArtifact,
    residue_report: &PreviewSubscriptionResidueReport,
    authoritative_lane: &ActiveSubscriptionLaneHandle,
    authority_label: impl Into<String>,
) -> Result<PreviewSubscriptionPromotionHandoff, PreviewSubscriptionIsolationError> {
    if isolation.lifecycle_state() != PreviewSubscriptionLifecycleState::PreviewActive {
        let mut counters = ActiveSubscriptionCounters::default();
        counters.preview_promotion_handoff_count = 1;
        return Err(PreviewSubscriptionIsolationError::new(
            PreviewSubscriptionIsolationDenialKind::PreviewLifecycleStateMismatch,
            "preview promotion requires an active preview isolation artifact",
            isolation.isolation_identity().clone(),
            counters,
        ));
    }

    Ok(PreviewSubscriptionPromotionHandoff::new(
        isolation,
        residue_report,
        authoritative_lane,
        authority_label,
    ))
}
