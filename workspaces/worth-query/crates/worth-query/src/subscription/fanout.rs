use crate::evidence_identity::WorthQueryEvidenceIdentity;

use super::active_digest::ActiveSubscriptionLaneDigest;
use super::evidence_identities::{
    subscription_fanout_plan_identity, subscription_fanout_report_identity,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubscriptionFanoutPlan {
    lane_digest: ActiveSubscriptionLaneDigest,
    affected_consumer_attachment_width: u64,
    fanout_plan_identity: WorthQueryEvidenceIdentity,
}

impl SubscriptionFanoutPlan {
    pub(super) fn new(
        lane_digest: ActiveSubscriptionLaneDigest,
        affected_consumer_attachment_width: u64,
    ) -> Self {
        let fanout_plan_identity = subscription_fanout_plan_identity(
            lane_digest.evidence_identity(),
            affected_consumer_attachment_width,
        );
        Self {
            lane_digest,
            affected_consumer_attachment_width,
            fanout_plan_identity,
        }
    }

    pub(crate) fn lane_digest(&self) -> &ActiveSubscriptionLaneDigest {
        &self.lane_digest
    }

    pub fn affected_consumer_attachment_width(&self) -> u64 {
        self.affected_consumer_attachment_width
    }

    pub fn evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.fanout_plan_identity
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubscriptionFanoutReport {
    plan: SubscriptionFanoutPlan,
    shared_lane_count: u64,
    fanout_width: u64,
    report_identity: WorthQueryEvidenceIdentity,
}

impl SubscriptionFanoutReport {
    pub(super) fn new(plan: SubscriptionFanoutPlan, shared_lane_count: u64) -> Self {
        let fanout_width = plan.affected_consumer_attachment_width();
        let report_identity = subscription_fanout_report_identity(
            plan.evidence_identity(),
            shared_lane_count,
            fanout_width,
        );
        Self {
            plan,
            shared_lane_count,
            fanout_width,
            report_identity,
        }
    }

    pub fn plan(&self) -> &SubscriptionFanoutPlan {
        &self.plan
    }

    pub fn shared_lane_count(&self) -> u64 {
        self.shared_lane_count
    }

    pub fn fanout_width(&self) -> u64 {
        self.fanout_width
    }

    pub fn evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.report_identity
    }
}
