//! Exclusive delivery of canonical publication facts. Dropping an unconsumed
//! claim permits another delivery; consuming it permanently closes that lane.

use std::sync::atomic::Ordering;
use std::sync::Arc;

use crate::history::ExplicitCommitHistoryProtectionObligation;

use super::CanonicalPublicationEnvelope;

pub(super) const AVAILABLE: u8 = 0;
const CLAIMED: u8 = 1;
const CONSUMED: u8 = 2;

#[derive(Debug)]
#[must_use = "a publication delivery claim is linear"]
pub(crate) struct PublicationDeliveryClaim {
    envelope: Arc<CanonicalPublicationEnvelope>,
    _history: ExplicitCommitHistoryProtectionObligation,
    successor_observation: Option<crate::branch::ProductBranchObservation>,
    consumed: bool,
}

impl CanonicalPublicationEnvelope {
    /// The normal publisher reserves delivery before the CAS. Recovery uses
    /// the same exclusive claim only after committed facts are visible.
    pub(crate) fn claim_delivery(
        self: &Arc<Self>,
        history: ExplicitCommitHistoryProtectionObligation,
    ) -> Option<PublicationDeliveryClaim> {
        self.try_claim_delivery(history).ok()
    }
    pub(crate) fn try_claim_delivery(
        self: &Arc<Self>,
        history: ExplicitCommitHistoryProtectionObligation,
    ) -> Result<PublicationDeliveryClaim, crate::recovery::PerformedPublicationRecoveryDenial> {
        use crate::recovery::PerformedPublicationRecoveryDenial as Denial;
        if history.commit_identity() != self.commit_identity() {
            return Err(Denial::ProtectionMismatch);
        }
        self.delivery
            .compare_exchange(AVAILABLE, CLAIMED, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|state| {
                if state == CONSUMED {
                    Denial::Consumed
                } else {
                    Denial::Claimed
                }
            })?;
        Ok(PublicationDeliveryClaim {
            successor_observation: self.take_successor_observation(),
            envelope: Arc::clone(self),
            _history: history,
            consumed: false,
        })
    }
}

impl PublicationDeliveryClaim {
    pub(crate) fn envelope(&self) -> &CanonicalPublicationEnvelope {
        &self.envelope
    }

    /// The future product handoff must consume this claim, not an inspection
    /// image. There is no transition back from consumed to available.
    pub(crate) fn consume(mut self) -> Self {
        assert!(self.envelope.facts().is_some());
        self.envelope.delivery.store(CONSUMED, Ordering::Release);
        self.consumed = true;
        self
    }

    pub(crate) fn take_successor_observation(
        &mut self,
    ) -> Option<crate::branch::ProductBranchObservation> {
        assert!(
            self.consumed,
            "only consumed delivery exposes product custody"
        );
        self.successor_observation.take()
    }
}

impl Drop for PublicationDeliveryClaim {
    fn drop(&mut self) {
        if !self.consumed {
            if let Some(observation) = self.successor_observation.take() {
                self.envelope.restore_successor_observation(observation);
            }
            self.envelope.delivery.store(AVAILABLE, Ordering::Release);
        }
    }
}
