use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::acknowledgement::{QueryDeliveryBatchReceipt, QueryDeliverySequence};
use super::active_budget::ActiveSubscriptionAllocationPosture;
use super::active_counters::ActiveSubscriptionCounters;
use super::active_digest::ActiveSubscriptionLaneDigest;
use super::attachment::SubscriptionConsumerAttachment;
use super::attachment_digest::SubscriptionConsumerAttachmentDigest;
use super::continuation::SubscriptionContinuationReport;
use super::delivery_budget::QueryDeliveryWindowBudget;
use super::delivery_cause::{QuerySubscriptionDeliveryCause, QuerySubscriptionDeliveryCauseKind};
use super::delivery_error::{QueryDeliveryDenialKind, QueryDeliveryError};
use super::delivery_work_packet::ActiveDeliveryWorkPacket;
#[cfg(test)]
use super::evidence_identities::{
    lifecycle_absent_performance_receipt_identity, lifecycle_absent_work_packet_identity,
};
use super::evidence_identities::{
    lifecycle_delivery_batch_identity, lifecycle_delivery_window_identity, typed_identity_drift,
};
use super::maintenance_delta::{
    QueryMaintenanceDeltaLoweringReport, QuerySubscriptionMaintenanceDelta,
};
use super::patch_group::{QueryPatchGroup, QueryPatchGroupKind};

mod delivery_batch;

pub use delivery_batch::QueryDeliveryBatch;

#[derive(Debug, Eq, PartialEq)]
pub struct QueryDeliveryWindow {
    delivery_window_identity: WorthQueryEvidenceIdentity,
    active_lane_digest: ActiveSubscriptionLaneDigest,
    attachment_digest: SubscriptionConsumerAttachmentDigest,
    next_sequence: QueryDeliverySequence,
    delivery_window_width: u64,
    patch_group_width: u64,
    maintenance_delta_width: u64,
    allocation_scope_width: u64,
    allocation_posture: ActiveSubscriptionAllocationPosture,
    counters: ActiveSubscriptionCounters,
}

impl QueryDeliveryWindow {
    pub(super) fn new(
        attachment: &SubscriptionConsumerAttachment,
        budget: QueryDeliveryWindowBudget,
    ) -> Result<Self, QueryDeliveryError> {
        let mut counters = ActiveSubscriptionCounters::default();
        if budget.exceeds_phase_three_budget() {
            counters.delivery_window_overflow_count = 1;
            return Err(QueryDeliveryError::new(
                QueryDeliveryDenialKind::DeliveryWindowBudgetExceeded,
                "delivery window exceeds its explicit Phase 3 budget",
                attachment.attachment_digest().evidence_identity().clone(),
                counters,
            ));
        }
        if budget.forbidden_allocation_posture() {
            counters.heap_allocation_denial_count = 1;
            return Err(QueryDeliveryError::new(
                QueryDeliveryDenialKind::AllocationPostureForbidden,
                "delivery windows require delivery-window allocation posture",
                attachment.attachment_digest().evidence_identity().clone(),
                counters,
            ));
        }

        let delivery_window_identity = lifecycle_delivery_window_identity(
            attachment.lane_digest().evidence_identity(),
            attachment.attachment_digest().evidence_identity(),
            attachment.next_delivery_sequence().get(),
            budget.delivery_window_width(),
            budget.patch_group_width(),
            budget.allocation_scope_width(),
            budget.allocation_posture(),
            *budget.backpressure_policy(),
        );
        counters.delivery_window_open_count = 1;
        counters.delivery_window_width = budget.delivery_window_width();
        counters.patch_group_width = budget.patch_group_width();
        if budget.allocation_posture().is_heap_debt() {
            counters.heap_allocation_debt_count = 1;
        }

        Ok(Self {
            delivery_window_identity,
            active_lane_digest: attachment.lane_digest().clone(),
            attachment_digest: attachment.attachment_digest().clone(),
            next_sequence: attachment.next_delivery_sequence(),
            delivery_window_width: budget.delivery_window_width(),
            patch_group_width: budget.patch_group_width(),
            maintenance_delta_width: budget.maintenance_delta_width(),
            allocation_scope_width: budget.allocation_scope_width(),
            allocation_posture: budget.allocation_posture(),
            counters,
        })
    }

    pub fn evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.delivery_window_identity
    }

    pub(crate) fn active_lane_digest(&self) -> &ActiveSubscriptionLaneDigest {
        &self.active_lane_digest
    }

    pub(crate) fn attachment_digest(&self) -> &SubscriptionConsumerAttachmentDigest {
        &self.attachment_digest
    }

    pub fn next_sequence(&self) -> QueryDeliverySequence {
        self.next_sequence
    }

    pub fn patch_group_width(&self) -> u64 {
        self.patch_group_width
    }

    pub fn maintenance_delta_width(&self) -> u64 {
        self.maintenance_delta_width
    }

    pub fn delivery_window_width(&self) -> u64 {
        self.delivery_window_width
    }

    pub fn allocation_scope_width(&self) -> u64 {
        self.allocation_scope_width
    }

    pub fn allocation_posture(&self) -> ActiveSubscriptionAllocationPosture {
        self.allocation_posture
    }

    pub fn counters(&self) -> &ActiveSubscriptionCounters {
        &self.counters
    }

    pub(super) fn apply_continuation(self, report: &SubscriptionContinuationReport) -> Self {
        let delivery_window_identity = WorthQueryEvidenceIdentity::compose(
            crate::evidence_identity::WorthQueryEvidenceScope::SubscriptionActivationReceipt,
        )
        .field_shape(
            crate::evidence_identity::WorthQueryEvidenceTag::new("identity_family"),
            "query_delivery_window_continuation_v1",
        )
        .field_evidence_identity(
            crate::evidence_identity::WorthQueryEvidenceTag::new("prior"),
            &self.delivery_window_identity,
        )
        .field_evidence_identity(
            crate::evidence_identity::WorthQueryEvidenceTag::new("continuation"),
            report.evidence_identity(),
        )
        .field_usize(
            crate::evidence_identity::WorthQueryEvidenceTag::new("remap_width"),
            report.remap_width() as usize,
        )
        .seal()
        .expect(
            "fixed subscription receipt: two sealed tokens, closed labels and primitive widths",
        );
        Self {
            delivery_window_identity,
            ..self
        }
    }
}

type LoweredQuerySubscriptionMaintenanceDelta = (
    QuerySubscriptionMaintenanceDelta,
    QueryMaintenanceDeltaLoweringReport,
    ActiveSubscriptionCounters,
);

pub fn lower_query_subscription_maintenance_delta(
    delta: QuerySubscriptionMaintenanceDelta,
) -> Result<LoweredQuerySubscriptionMaintenanceDelta, QueryDeliveryError> {
    let mut counters = ActiveSubscriptionCounters::default();
    counters.maintenance_delta_lowering_count = 1;
    counters.maintenance_delta_width = delta.width();
    let report = QueryMaintenanceDeltaLoweringReport::new(&delta);
    Ok((delta, report, counters))
}
