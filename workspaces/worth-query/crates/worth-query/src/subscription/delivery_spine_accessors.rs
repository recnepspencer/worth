use crate::identity_authority::{QueryProjectionIdentity, QuerySubscriptionIdentityKind};

use super::acknowledgement::{QueryDeliveryBatchReceipt, SubscriptionAcknowledgementFrontier};
use super::attachment_request::SubscriptionConsumerAttachmentRequest;
use super::delivery::QuerySubscriptionDeliveryIntent;
use super::delivery_cause::QuerySubscriptionDeliveryCause;
use super::delivery_window::{QueryDeliveryBatch, QueryDeliveryWindow};
use super::delivery_work_packet::ActiveDeliveryWorkPacket;
use super::evidence_projection::subscription_evidence_projection;
use super::fanout::{SubscriptionFanoutPlan, SubscriptionFanoutReport};
use super::maintenance_delta::{
    QueryMaintenanceDeltaLoweringReport, QuerySubscriptionMaintenanceDelta,
};
use super::patch_group::QueryPatchGroup;
use super::performance_receipt::SubscriptionPerformanceReceipt;
use super::slice::QuerySubscriptionSliceIntent;

impl QuerySubscriptionSliceIntent {
    pub fn slice_intent_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(self.evidence_identity())
    }
}

impl QueryPatchGroup {
    pub fn patch_group_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(self.patch_group_identity())
    }
}

impl SubscriptionPerformanceReceipt {
    pub fn performance_receipt_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(self.performance_receipt_identity())
    }
}

impl QuerySubscriptionDeliveryIntent {
    pub fn delivery_intent_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.evidence_identity())
    }
}

impl QuerySubscriptionDeliveryCause {
    pub fn evidence_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(self.evidence_identity())
    }

    pub fn delivery_cause_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(self.delivery_cause_identity())
    }
}

impl SubscriptionConsumerAttachmentRequest {
    pub fn consumer_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(self.consumer_identity())
    }

    pub fn delivery_cursor_seed_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(self.delivery_cursor_seed_identity())
    }
}

impl SubscriptionFanoutPlan {
    pub fn fanout_plan_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(self.evidence_identity())
    }
}

impl SubscriptionFanoutReport {
    pub fn fanout_report_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(self.evidence_identity())
    }
}

impl SubscriptionAcknowledgementFrontier {
    pub fn frontier_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(self.evidence_identity())
    }
}

impl QueryDeliveryBatchReceipt {
    pub fn receipt_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(self.evidence_identity())
    }
}

impl QueryDeliveryWindow {
    pub fn delivery_window_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(self.evidence_identity())
    }
}

impl QueryDeliveryBatch {
    pub fn delivery_batch_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(self.evidence_identity())
    }

    pub fn delivery_window_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(self.delivery_window_identity())
    }
}

impl ActiveDeliveryWorkPacket {
    pub fn work_packet_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(self.evidence_identity())
    }
}

impl QuerySubscriptionMaintenanceDelta {
    pub fn maintenance_delta_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(self.evidence_identity())
    }

    pub fn scope_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(self.scope_identity())
    }
}

impl QueryMaintenanceDeltaLoweringReport {
    pub fn lowering_report_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(self.evidence_identity())
    }

    pub fn maintenance_delta_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(self.maintenance_delta_identity())
    }
}
