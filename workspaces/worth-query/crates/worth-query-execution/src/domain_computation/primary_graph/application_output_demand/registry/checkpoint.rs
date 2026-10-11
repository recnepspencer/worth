use crate::domain_computation::execution_runtime::product_world::WorthQueryPerformedRelationalProductChange;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryOutputDemandDenial,
};
use worth_runtime_world::facade::CompositeCommitIdentity;

use super::ReadyCompletion;

pub(in crate::domain_computation::primary_graph) enum WorthQueryPendingOutputDelivery {
    Change(WorthQueryPerformedRelationalProductChange),
    NoChange,
}

pub(in crate::domain_computation::primary_graph) enum WorthQueryOutputCheckpoint {
    Published {
        receipt: WorthQueryApplicationCommitReceipt,
        delivery: WorthQueryPendingOutputDelivery,
        ready_backing: super::PreparedReadyBacking,
    },
    Delivered {
        receipt: WorthQueryApplicationCommitReceipt,
        delivery: Option<worth_runtime_bridge::facade::BridgeCorrespondenceDeliveryReceipt>,
        ready_backing: super::PreparedReadyBacking,
    },
    Ready(ReadyCompletion),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryOutputClaimIdentity(
    pub(super) u64,
);

impl WorthQueryOutputCheckpoint {
    pub(in crate::domain_computation::primary_graph) fn receipt(
        &self,
    ) -> Option<&WorthQueryApplicationCommitReceipt> {
        match self {
            Self::Published { receipt, .. } | Self::Delivered { receipt, .. } => Some(receipt),
            Self::Ready(completion) => match &completion.authority {
                super::WorthQueryAcceptedOutputAuthority::Committed(receipt) => Some(receipt),
                super::WorthQueryAcceptedOutputAuthority::Stable(_) => None,
                super::WorthQueryAcceptedOutputAuthority::Restored(_) => None,
            },
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
    pub(super) undelivered: Option<super::publication_head::UndeliveredPublication>,
    /// Exact owner-issued commit retained while the sole full checkpoint is
    /// temporarily claimed for delivery. This fixed identity replaces a deep
    /// descriptive receipt clone after producer effects.
    pub(super) published_commit: Option<CompositeCommitIdentity>,
    pub(super) checkpoint: Option<WorthQueryOutputCheckpoint>,
    pub(super) advancement: WorthQueryOutputAdvancement,
    pub(super) next_claim: u64,
}

impl WorthQueryOutputProgress {
    #[cfg(test)]
    pub(super) fn new(checkpoint: WorthQueryOutputCheckpoint) -> Self {
        let (output, retired_observation) = Self::new_with_detached_observation(checkpoint);
        drop(retired_observation);
        output
    }

    /// Keep the original one-time observation consumption, while allowing a
    /// selected registry finish to release its final owner after unlocking.
    pub(super) fn new_with_detached_observation(
        checkpoint: WorthQueryOutputCheckpoint,
    ) -> (
        Self,
        Option<worth_runtime_world::facade::ProductBranchObservation>,
    ) {
        let published_commit = checkpoint.receipt().map(|receipt| {
            receipt
                .committed_product_publication()
                .composite_commit()
                .clone()
        });
        let retired_observation = checkpoint.receipt().and_then(|receipt| {
            receipt
                .committed_product_publication()
                .take_output_demand_observation()
        });
        (
            Self {
                published_commit,
                undelivered: None,
                checkpoint: Some(checkpoint),
                advancement: WorthQueryOutputAdvancement::Idle,
                next_claim: 0,
            },
            retired_observation,
        )
    }

    pub(super) fn restored(completion: ReadyCompletion) -> Self {
        Self {
            published_commit: None,
            undelivered: None,
            checkpoint: Some(WorthQueryOutputCheckpoint::Ready(completion)),
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
            // A stopped row cannot deliver its checkpoint; it no longer freezes the head.
            self.undelivered.take();
            self.advancement = WorthQueryOutputAdvancement::Stopped {
                denial,
                interrupted_claim,
            };
        }
    }
}

impl WorthQueryOutputProgress {
    pub(super) fn bind_publication_head(
        &mut self,
        counter: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    ) {
        if matches!(
            &self.checkpoint,
            Some(WorthQueryOutputCheckpoint::Published { .. })
        ) {
            assert!(self.undelivered.is_none());
            self.undelivered = Some(super::publication_head::UndeliveredPublication::new(
                counter,
            ));
        }
    }
    /// The row owns the hold while delivery claims its checkpoint. Only a
    /// delivered/ready finish releases it; a restored Published keeps it.
    pub(super) fn install_checkpoint(&mut self, checkpoint: WorthQueryOutputCheckpoint) {
        match &checkpoint {
            WorthQueryOutputCheckpoint::Published { .. } => {}
            WorthQueryOutputCheckpoint::Delivered { .. } | WorthQueryOutputCheckpoint::Ready(_) => {
                self.undelivered.take();
            }
        }
        self.checkpoint = Some(checkpoint);
    }
}
