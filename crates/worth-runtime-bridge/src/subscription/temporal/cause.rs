use std::sync::Arc;

use sha2::{Digest, Sha256};
use worth_signal::facade::{ClockTick, TemporalWakeId, WakeOrdinal};

use crate::input::envelope::TruthPatchIdentity;
use crate::subscription::{
    BridgeSubscriptionCounters, BridgeSubscriptionPreviewBasisIdentity,
    BridgeSubscriptionTemporalCauseRecordIdentity,
};
use crate::temporal::BridgeTemporalBasisIdentity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeTemporalCauseClassification {
    TimeOnly,
    TruthPlusTime,
}

impl BridgeTemporalCauseClassification {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TimeOnly => "time_only",
            Self::TruthPlusTime => "truth_plus_time",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeTemporalRoutingLaneKind {
    Authoritative,
    Preview,
}

impl BridgeTemporalRoutingLaneKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Authoritative => "authoritative",
            Self::Preview => "preview",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeTemporalCauseRecord {
    cause_record_identity: BridgeSubscriptionTemporalCauseRecordIdentity,
    routing_lane_kind: BridgeTemporalRoutingLaneKind,
    subscription_identity: Arc<str>,
    activation_lane_identity: Arc<str>,
    temporal_basis_identity: BridgeTemporalBasisIdentity,
    preview_basis_identity: Option<BridgeSubscriptionPreviewBasisIdentity>,
    classification: BridgeTemporalCauseClassification,
    wake_id: TemporalWakeId,
    wake_ready_ordinal: WakeOrdinal,
    wake_tick: ClockTick,
    truth_patch_identity: Option<TruthPatchIdentity>,
    truth_patch_digest: Option<Arc<str>>,
    counters: BridgeSubscriptionCounters,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

pub(crate) struct BridgeTemporalCauseRecordInput {
    pub(crate) routing_lane_kind: BridgeTemporalRoutingLaneKind,
    pub(crate) subscription_identity: Arc<str>,
    pub(crate) activation_lane_identity: Arc<str>,
    pub(crate) temporal_basis_identity: BridgeTemporalBasisIdentity,
    pub(crate) preview_basis_identity: Option<BridgeSubscriptionPreviewBasisIdentity>,
    pub(crate) classification: BridgeTemporalCauseClassification,
    pub(crate) wake_id: TemporalWakeId,
    pub(crate) wake_ready_ordinal: WakeOrdinal,
    pub(crate) wake_tick: ClockTick,
    pub(crate) truth_patch_identity: Option<TruthPatchIdentity>,
    pub(crate) truth_patch_digest: Option<Arc<str>>,
}

impl BridgeTemporalCauseRecord {
    pub(crate) fn route(input: BridgeTemporalCauseRecordInput) -> Self {
        let truth_patch_identity = input
            .truth_patch_identity
            .as_ref()
            .map(TruthPatchIdentity::canonical_label)
            .unwrap_or_else(|| "none".to_owned());
        let truth_patch_digest = input.truth_patch_digest.as_deref().unwrap_or("none");
        let preview_basis_identity = input
            .preview_basis_identity
            .as_ref()
            .map(BridgeSubscriptionPreviewBasisIdentity::as_str)
            .unwrap_or("none");
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-temporal-cause-record|lane={}|activation={}|temporal-basis={}|preview-basis={}|classification={}|wake-id={}|wake-ready-ordinal={}|wake-tick={}|truth-patch={}|truth-patch-digest={}",
            input.routing_lane_kind.as_str(),
            input.activation_lane_identity.as_ref(),
            input.temporal_basis_identity.as_str(),
            preview_basis_identity,
            input.classification.as_str(),
            input.wake_id.get(),
            input.wake_ready_ordinal.get(),
            input.wake_tick.get(),
            truth_patch_identity,
            truth_patch_digest,
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            cause_record_identity:
                BridgeSubscriptionTemporalCauseRecordIdentity::admit_bridge_owned(format!(
                    "bridge-temporal-cause-record-id:sha256:{digest:x}"
                )),
            routing_lane_kind: input.routing_lane_kind,
            subscription_identity: input.subscription_identity,
            activation_lane_identity: input.activation_lane_identity,
            temporal_basis_identity: input.temporal_basis_identity,
            preview_basis_identity: input.preview_basis_identity,
            classification: input.classification,
            wake_id: input.wake_id,
            wake_ready_ordinal: input.wake_ready_ordinal,
            wake_tick: input.wake_tick,
            truth_patch_identity: input.truth_patch_identity,
            truth_patch_digest: input.truth_patch_digest,
            counters: match input.classification {
                BridgeTemporalCauseClassification::TimeOnly => {
                    BridgeSubscriptionCounters::from_temporal_time_only_cause()
                }
                BridgeTemporalCauseClassification::TruthPlusTime => {
                    BridgeSubscriptionCounters::from_temporal_truth_plus_time_cause()
                }
            },
            canonical_basis,
            digest: Arc::from(format!("bridge-temporal-cause-record:sha256:{digest:x}")),
        }
    }

    pub fn cause_record_identity(&self) -> &BridgeSubscriptionTemporalCauseRecordIdentity {
        &self.cause_record_identity
    }

    pub fn routing_lane_kind(&self) -> BridgeTemporalRoutingLaneKind {
        self.routing_lane_kind
    }

    pub fn temporal_basis_identity(&self) -> &BridgeTemporalBasisIdentity {
        &self.temporal_basis_identity
    }

    pub fn preview_basis_identity(&self) -> Option<&BridgeSubscriptionPreviewBasisIdentity> {
        self.preview_basis_identity.as_ref()
    }

    pub fn classification(&self) -> BridgeTemporalCauseClassification {
        self.classification
    }

    pub const fn wake_id(&self) -> TemporalWakeId {
        self.wake_id
    }

    pub const fn wake_ready_ordinal(&self) -> WakeOrdinal {
        self.wake_ready_ordinal
    }

    pub const fn wake_tick(&self) -> ClockTick {
        self.wake_tick
    }

    pub fn truth_patch_identity(&self) -> Option<&TruthPatchIdentity> {
        self.truth_patch_identity.as_ref()
    }

    pub fn counters(&self) -> &BridgeSubscriptionCounters {
        &self.counters
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }

    pub(crate) fn subscription_identity(&self) -> &str {
        self.subscription_identity.as_ref()
    }
}
