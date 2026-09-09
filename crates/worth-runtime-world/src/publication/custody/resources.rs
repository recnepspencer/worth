use std::sync::Arc;

use crate::history::{
    CompositeRuntimeWorldCommit, ProductHeadHistoryProtectionObligation,
    ReservedCompositeCommitCapacity,
};
use crate::identity::CompositeCommitIdentity;
use crate::lifecycle::owner::{
    ReservedPublicationAttemptCapacity, RuntimeWorldOperationReservation,
};
use crate::retention::{
    PublicationRetentionObligation, ReservedComponentPinPairCapacity,
    RetainedPartialRetentionObligation, RetentionObligationDenial,
};

/// Mutable reservations have exactly one holder in the owner record. A short
/// resource lease may take them out while calling an owner, then restores them.
#[derive(Debug)]
pub(crate) struct ActiveAttemptResources {
    pub(super) conditional_definition: Option<Arc<super::ConditionalDefinitionAttemptCustody>>,
    pub(crate) product_comparison_costs:
        Option<crate::publication::CompositePublicationCostCounters>,
    pub(super) commit_identity: CompositeCommitIdentity,
    pub(super) commit: Option<Arc<CompositeRuntimeWorldCommit>>,
    pub(super) history_custody: ActiveHistoryCustody,
    pub(super) pins: ActivePinCustody,
    pub(super) history_pins: Option<crate::retention::HistoryRetentionObligation>,
    pub(super) pin_denial: Option<RetentionObligationDenial>,
    pub(super) product_head: Option<crate::branch::ProductBranchHeadProtection>,
    pub(super) delivery: Option<crate::history::PublicationDeliveryClaim>,
    pub(super) creation: Option<super::ActiveCreationResources>,
    pub(super) operation: Option<RuntimeWorldOperationReservation>,
    pub(super) publication_capacity: Option<ReservedPublicationAttemptCapacity>,
}

#[derive(Debug)]
pub(super) enum ActiveHistoryCustody {
    Reserved(ReservedCompositeCommitCapacity),
    Installed(ProductHeadHistoryProtectionObligation),
    Released,
    TransferredToProduct,
}

#[derive(Debug)]
pub(super) enum ActivePinCustody {
    Reserved(ReservedComponentPinPairCapacity),
    Bound(PublicationRetentionObligation),
    Retained {
        _obligation: RetainedPartialRetentionObligation,
    },
    TransferredToProduct,
}

impl ActiveAttemptResources {
    pub(crate) fn retains_signal_definition(&self) -> bool {
        self.conditional_definition
            .as_ref()
            .is_some_and(|custody| custody.retains_definition())
    }

    /// Abandonment retains the exact custody already held; inspection does not
    /// acquire pins or retag their dependency class.
    pub(crate) fn retention_posture(&self) -> crate::recovery::ProductUnpublishedRetentionPosture {
        if self.product_head.is_some() || self.creation.as_ref().is_some_and(|c| c.cell.is_some()) {
            return crate::recovery::ProductUnpublishedRetentionPosture::ProductHeadPinsRetained;
        }
        match &self.pins {
            ActivePinCustody::Reserved(_) if self.pin_denial.is_some() => {
                crate::recovery::ProductUnpublishedRetentionPosture::ReacquisitionPending
            }
            ActivePinCustody::Reserved(_) => {
                crate::recovery::ProductUnpublishedRetentionPosture::BindingReserved
            }
            ActivePinCustody::Bound(_) => {
                crate::recovery::ProductUnpublishedRetentionPosture::PublicationPinsRetained
            }
            ActivePinCustody::Retained { .. } => {
                crate::recovery::ProductUnpublishedRetentionPosture::RetainedExact
            }
            ActivePinCustody::TransferredToProduct => {
                unreachable!("performed product custody is never unpublished")
            }
        }
    }

    pub(crate) fn holds_history_obligation(&self) -> bool {
        self.product_head.is_some()
            || self.creation.as_ref().is_some_and(|c| c.cell.is_some())
            || !matches!(
                self.history_custody,
                ActiveHistoryCustody::TransferredToProduct | ActiveHistoryCustody::Released
            )
    }

    pub(crate) fn successor_commit(&self) -> Option<&CompositeCommitIdentity> {
        if self.creation.as_ref().is_some_and(|c| c.cell.is_some()) {
            return self.commit.as_ref().map(|commit| commit.identity());
        }
        if let Some(head) = &self.product_head {
            return Some(head.product_head_history().commit_identity());
        }
        match &self.history_custody {
            ActiveHistoryCustody::Installed(protection) => Some(protection.commit_identity()),
            _ => None,
        }
    }

    pub(crate) fn live_obligations(&self) -> crate::recovery::ProductUnpublishedLiveObligations {
        let counts = crate::recovery::ProductUnpublishedLiveObligations::from_custody(
            2 + 2 * usize::from(self.history_pins.is_some()),
            self.holds_history_obligation(),
        );
        match self.creation.as_ref().and_then(|c| c.observation.as_ref()) {
            Some(observation) => counts.with_observation(observation),
            None => counts,
        }
    }
}
