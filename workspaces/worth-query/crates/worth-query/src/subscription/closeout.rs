use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::identity_authority::{QueryProjectionIdentity, QuerySubscriptionIdentityKind};

use super::active_budget::ActiveSubscriptionAllocationPosture;
use super::active_counters::ActiveSubscriptionCounters;
use super::active_digest::ActiveSubscriptionLaneDigest;
use super::attachment::SubscriptionConsumerAttachment;
use super::attachment_digest::SubscriptionConsumerAttachmentDigest;
use super::delivery_density::ActiveDeliveryDensityPosture;
use super::evidence_identities::lifecycle_closeout_identity;
use super::evidence_projection::subscription_evidence_projection;
use super::future_selection::QuerySubscriptionFutureSelection;
use super::performance_receipt::SubscriptionPerformanceReceipt;
use super::preview_isolation::{
    PreviewSubscriptionDiscardCloseout, PreviewSubscriptionPromotionHandoff,
};
use super::support::QuerySubscriptionSupportProfile;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SubscriptionLifecycleCloseoutKind {
    ConsumerDetached,
    ConsumerTerminated,
    PreviewDiscarded,
    PreviewPromoted,
}

impl SubscriptionLifecycleCloseoutKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ConsumerDetached => "consumer_detached",
            Self::ConsumerTerminated => "consumer_terminated",
            Self::PreviewDiscarded => "preview_discarded",
            Self::PreviewPromoted => "preview_promoted",
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum SubscriptionLifecycleCloseRequest {
    DetachConsumer(SubscriptionConsumerAttachment),
    TerminateConsumer(SubscriptionConsumerAttachment),
    PreviewDiscard(PreviewSubscriptionDiscardCloseout),
    PreviewPromotion(PreviewSubscriptionPromotionHandoff),
}

impl SubscriptionLifecycleCloseRequest {
    pub(super) fn lane_digest(&self) -> &ActiveSubscriptionLaneDigest {
        match self {
            Self::DetachConsumer(attachment) | Self::TerminateConsumer(attachment) => {
                attachment.lane_digest()
            }
            Self::PreviewDiscard(closeout) => closeout.active_lane_digest(),
            Self::PreviewPromotion(handoff) => handoff.preview_lane_digest(),
        }
    }

    pub(super) fn attachment_digest(&self) -> &SubscriptionConsumerAttachmentDigest {
        match self {
            Self::DetachConsumer(attachment) | Self::TerminateConsumer(attachment) => {
                attachment.attachment_digest()
            }
            Self::PreviewDiscard(closeout) => closeout.attachment_digest(),
            Self::PreviewPromotion(handoff) => handoff.attachment_digest(),
        }
    }

    pub(super) fn closeout_kind(&self) -> SubscriptionLifecycleCloseoutKind {
        match self {
            Self::DetachConsumer(_) => SubscriptionLifecycleCloseoutKind::ConsumerDetached,
            Self::TerminateConsumer(_) => SubscriptionLifecycleCloseoutKind::ConsumerTerminated,
            Self::PreviewDiscard(_) => SubscriptionLifecycleCloseoutKind::PreviewDiscarded,
            Self::PreviewPromotion(_) => SubscriptionLifecycleCloseoutKind::PreviewPromoted,
        }
    }

    pub(super) fn source_identity(&self) -> WorthQueryEvidenceIdentity {
        match self {
            Self::DetachConsumer(attachment) | Self::TerminateConsumer(attachment) => {
                attachment.attachment_digest().evidence_identity().clone()
            }
            Self::PreviewDiscard(closeout) => closeout.closeout_identity().clone(),
            Self::PreviewPromotion(handoff) => handoff.handoff_identity().clone(),
        }
    }

    pub(super) fn future_selection(&self) -> &QuerySubscriptionFutureSelection {
        match self {
            Self::DetachConsumer(attachment) | Self::TerminateConsumer(attachment) => {
                attachment.future_selection()
            }
            Self::PreviewDiscard(closeout) => closeout.future_selection(),
            Self::PreviewPromotion(handoff) => handoff.future_selection(),
        }
    }

    pub(super) fn basis_binding_identity(&self) -> &WorthQueryEvidenceIdentity {
        match self {
            Self::DetachConsumer(attachment) | Self::TerminateConsumer(attachment) => {
                attachment.basis_binding_identity()
            }
            Self::PreviewDiscard(closeout) => closeout.basis_binding_identity(),
            Self::PreviewPromotion(handoff) => handoff.preview_basis_binding_identity(),
        }
    }

    pub(super) fn checkpoint_identity(&self) -> &WorthQueryEvidenceIdentity {
        match self {
            Self::DetachConsumer(attachment) | Self::TerminateConsumer(attachment) => {
                attachment.checkpoint_identity()
            }
            Self::PreviewDiscard(closeout) => closeout.checkpoint_identity(),
            Self::PreviewPromotion(handoff) => handoff.preview_checkpoint_identity(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SubscriptionLifecycleCloseDenialKind {
    LaneHandleMismatch,
    AttachmentLaneMismatch,
    AttachmentNotActive,
}

impl SubscriptionLifecycleCloseDenialKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LaneHandleMismatch => "lane_handle_mismatch",
            Self::AttachmentLaneMismatch => "attachment_lane_mismatch",
            Self::AttachmentNotActive => "attachment_not_active",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubscriptionLifecycleCloseError {
    denial_kind: SubscriptionLifecycleCloseDenialKind,
    message: String,
    pub(in crate::subscription) source_identity: WorthQueryEvidenceIdentity,
    counters: ActiveSubscriptionCounters,
}

impl SubscriptionLifecycleCloseError {
    pub(super) fn new(
        denial_kind: SubscriptionLifecycleCloseDenialKind,
        message: impl Into<String>,
        source_identity: WorthQueryEvidenceIdentity,
        counters: ActiveSubscriptionCounters,
    ) -> Self {
        Self {
            denial_kind,
            message: message.into(),
            source_identity,
            counters,
        }
    }

    pub fn denial_kind(&self) -> &SubscriptionLifecycleCloseDenialKind {
        &self.denial_kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn counters(&self) -> &ActiveSubscriptionCounters {
        &self.counters
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubscriptionLifecycleCloseout {
    active_lane_digest: ActiveSubscriptionLaneDigest,
    attachment_digest: SubscriptionConsumerAttachmentDigest,
    future_selection: QuerySubscriptionFutureSelection,
    basis_binding_identity: WorthQueryEvidenceIdentity,
    checkpoint_identity: WorthQueryEvidenceIdentity,
    closeout_kind: SubscriptionLifecycleCloseoutKind,
    lane_terminal: bool,
    source_identity: WorthQueryEvidenceIdentity,
    support_profile: QuerySubscriptionSupportProfile,
    performance_receipt: SubscriptionPerformanceReceipt,
    counters: ActiveSubscriptionCounters,
    closeout_identity: WorthQueryEvidenceIdentity,
}

impl SubscriptionLifecycleCloseout {
    pub(super) fn new(
        request: SubscriptionLifecycleCloseRequest,
        lane_terminal: bool,
    ) -> Result<Self, IdentityDerivationDenial> {
        let closeout_kind = request.closeout_kind();
        let mut counters = ActiveSubscriptionCounters::default();
        counters.consumer_attachment_close_count = 1;
        counters.subscription_lifecycle_closeout_count = 1;
        if lane_terminal {
            counters.active_lane_close_count = 1;
        }
        let source_identity = request.source_identity();
        let support_profile =
            QuerySubscriptionSupportProfile::active_runtime_admitted(&source_identity)?;
        let active_lane_digest = request.lane_digest().clone();
        let attachment_digest = request.attachment_digest().clone();
        let future_selection = request.future_selection().clone();
        let basis_binding_identity = request.basis_binding_identity().clone();
        let checkpoint_identity = request.checkpoint_identity().clone();
        let consumed_width = if lane_terminal { 2 } else { 1 };
        let performance_receipt = SubscriptionPerformanceReceipt::new(
            consumed_width,
            2,
            ActiveDeliveryDensityPosture::SparseDelta,
            ActiveSubscriptionAllocationPosture::LifecycleArena,
            &source_identity,
        )?;
        counters.subscription_performance_receipt_count = 1;
        counters.subscription_budget_consumption_width = performance_receipt.consumed_width();
        counters.subscription_budget_remaining_width = performance_receipt.remaining_width();
        let closeout_identity = lifecycle_closeout_identity(
            active_lane_digest.evidence_identity(),
            attachment_digest.evidence_identity(),
            future_selection.projection_identity(),
            &basis_binding_identity,
            &checkpoint_identity,
            closeout_kind.as_str(),
            lane_terminal,
            support_profile.profile_identity(),
            performance_receipt.performance_receipt_identity(),
            &counters.evidence_identity(),
            &source_identity,
        )?;
        Ok(Self {
            active_lane_digest,
            attachment_digest,
            future_selection,
            basis_binding_identity,
            checkpoint_identity,
            closeout_kind,
            lane_terminal,
            source_identity,
            support_profile,
            performance_receipt,
            counters,
            closeout_identity,
        })
    }

    pub(crate) fn active_lane_digest(&self) -> &ActiveSubscriptionLaneDigest {
        &self.active_lane_digest
    }

    pub(crate) fn attachment_digest(&self) -> &SubscriptionConsumerAttachmentDigest {
        &self.attachment_digest
    }

    pub fn future_selection(&self) -> &QuerySubscriptionFutureSelection {
        &self.future_selection
    }

    pub fn basis_binding_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.basis_binding_identity)
    }

    pub fn basis_binding_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.basis_binding_identity
    }

    pub fn checkpoint_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.checkpoint_identity)
    }

    pub fn checkpoint_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.checkpoint_identity
    }

    pub fn closeout_kind(&self) -> &SubscriptionLifecycleCloseoutKind {
        &self.closeout_kind
    }

    pub fn lane_terminal(&self) -> bool {
        self.lane_terminal
    }

    pub fn source_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.source_identity
    }

    pub fn source_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.source_identity)
    }

    pub fn support_profile(&self) -> &QuerySubscriptionSupportProfile {
        &self.support_profile
    }

    pub fn performance_receipt(&self) -> &SubscriptionPerformanceReceipt {
        &self.performance_receipt
    }

    pub fn counters(&self) -> &ActiveSubscriptionCounters {
        &self.counters
    }

    pub fn closeout_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.closeout_identity)
    }

    pub fn evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.closeout_identity
    }
}
