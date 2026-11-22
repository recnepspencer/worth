//! A successor round's own-output view (PR C §1.5): the reopened derivation
//! reads every record its own chain wrote exclusively as it was immediately
//! before the chain's head appended, so it re-derives from that image under
//! the new inputs. Native certified the exclusivity (`chain_overlay`); field
//! reads and key resolution of those records are served here, and any other
//! read of a key the chain wrote stops before append (`PriorEffectRead`).
use worth_relational::facade::{
    identity::EntityId,
    runtime::{RelationalAppliedCandidateReadIdentity, RelationalPinnedRead},
};

#[derive(Clone)]
pub struct OwnOutputOverlay {
    head: RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    exclusive: Vec<EntityId>,
}

impl OwnOutputOverlay {
    pub(in crate::domain_computation::primary_graph) fn from_chain(
        chain: &worth_relational::facade::mvcc::RelationalCoupledChainOverlay,
    ) -> Self {
        Self {
            head: chain.head_pre_pin().clone(),
            exclusive: chain.exclusive_entities().to_vec(),
        }
    }

    /// The chain head's pre-pin: the image before the chain's first round.
    pub(in crate::domain_computation::primary_graph) fn head(
        &self,
    ) -> &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity> {
        &self.head
    }

    /// The chain head's pre-pin, for a record the chain wrote exclusively.
    pub(in crate::domain_computation::primary_graph) fn head_for(
        &self,
        entity: EntityId,
    ) -> Option<&RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>> {
        self.exclusive
            .binary_search(&entity)
            .is_ok()
            .then_some(&self.head)
    }

    pub(in crate::domain_computation::primary_graph) fn covers(&self, entity: EntityId) -> bool {
        self.exclusive.binary_search(&entity).is_ok()
    }
}
