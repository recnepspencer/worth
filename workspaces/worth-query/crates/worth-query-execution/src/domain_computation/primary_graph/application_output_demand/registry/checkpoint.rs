use crate::domain_computation::execution_runtime::product_world::WorthQueryPerformedRelationalProductChange;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryOutputDemandDenial,
};

use super::WorthQueryCompletedOutputDemand;

pub(in crate::domain_computation::primary_graph) enum WorthQueryPendingOutputDelivery {
    Change(WorthQueryPerformedRelationalProductChange),
    NoChange,
}

pub(in crate::domain_computation::primary_graph) enum WorthQueryOutputCheckpoint {
    Published {
        receipt: WorthQueryApplicationCommitReceipt,
        delivery: WorthQueryPendingOutputDelivery,
    },
    Delivered {
        receipt: WorthQueryApplicationCommitReceipt,
        delivery: Option<worth_runtime_bridge::facade::BridgeCorrespondenceDeliveryReceipt>,
    },
    Ready(WorthQueryCompletedOutputDemand),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryOutputClaimIdentity(
    pub(super) u64,
);

impl WorthQueryOutputCheckpoint {
    pub(in crate::domain_computation::primary_graph) fn receipt(
        &self,
    ) -> &WorthQueryApplicationCommitReceipt {
        match self {
            Self::Published { receipt, .. } | Self::Delivered { receipt, .. } => receipt,
            Self::Ready(completion) => &completion.receipt,
        }
    }
}

pub(super) enum WorthQueryOutputAdvancement {
    Idle,
    Claimed(WorthQueryOutputClaimIdentity),
    Stopped {
        denial: WorthQueryOutputDemandDenial,
        interrupted_claim: Option<WorthQueryOutputClaimIdentity>,
    },
}

pub(super) struct WorthQueryOutputProgress {
    pub(super) receipt: WorthQueryApplicationCommitReceipt,
    pub(super) checkpoint: Option<WorthQueryOutputCheckpoint>,
    pub(super) advancement: WorthQueryOutputAdvancement,
    pub(super) next_claim: u64,
}

impl WorthQueryOutputProgress {
    pub(super) fn new(checkpoint: WorthQueryOutputCheckpoint) -> Self {
        let _ = checkpoint
            .receipt()
            .committed_product_publication()
            .take_output_demand_observation();
        Self {
            receipt: checkpoint.receipt().clone(),
            checkpoint: Some(checkpoint),
            advancement: WorthQueryOutputAdvancement::Idle,
            next_claim: 0,
        }
    }

    pub(super) fn stop(&mut self, denial: WorthQueryOutputDemandDenial) {
        if !matches!(
            self.advancement,
            WorthQueryOutputAdvancement::Stopped { .. }
        ) {
            let interrupted_claim = match self.advancement {
                WorthQueryOutputAdvancement::Claimed(claim) => Some(claim),
                _ => None,
            };
            self.advancement = WorthQueryOutputAdvancement::Stopped {
                denial,
                interrupted_claim,
            };
        }
    }
}
