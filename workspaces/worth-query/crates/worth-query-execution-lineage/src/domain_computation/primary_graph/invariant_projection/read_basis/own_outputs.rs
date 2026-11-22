//! A successor round's own-output view (PR C §1.5): the reopened derivation
//! reads every record its own chain wrote exclusively as it was immediately
//! before the chain's head appended, so it re-derives from that image under
//! the new inputs. Native certified the exclusivity (`chain_overlay`); field
//! reads and key resolution of those records are served here, and any other
//! read of a key the chain wrote stops before append (`PriorEffectRead`).
//!
//! The view is entity-only (#277): a relation or selection read is served
//! at the current image, so it can name a record the chain created, which
//! is absent at the head pre-pin. That view is incoherent: such a read is
//! the chain's effect, not C. `mark_incoherent` records the first record
//! found absent there, shared by every clone, and a successor round whose
//! prepare then fails stops typed (`PriorEffectRead`) on it.
use worth_relational::facade::{
    identity::EntityId,
    runtime::{RelationalAppliedCandidateReadIdentity, RelationalPinnedRead},
};

#[derive(Clone)]
pub struct OwnOutputOverlay {
    head: RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    exclusive: Vec<EntityId>,
    /// The first covered record a served read found absent at the head.
    incoherent: std::sync::Arc<std::sync::OnceLock<EntityId>>,
}

impl OwnOutputOverlay {
    pub fn from_chain(
        chain: &worth_relational::facade::mvcc::RelationalCoupledChainOverlay,
    ) -> Self {
        Self {
            head: chain.head_pre_pin().clone(),
            exclusive: chain.exclusive_entities().to_vec(),
            incoherent: std::sync::Arc::default(),
        }
    }

    /// The chain head's pre-pin: the image before the chain's first round.
    pub fn head(
        &self,
    ) -> &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity> {
        &self.head
    }

    /// The chain head's pre-pin, for a record the chain wrote exclusively.
    pub fn head_for(
        &self,
        entity: EntityId,
    ) -> Option<&RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>> {
        self.exclusive
            .binary_search(&entity)
            .is_ok()
            .then_some(&self.head)
    }

    pub fn covers(&self, entity: EntityId) -> bool {
        self.exclusive.binary_search(&entity).is_ok()
    }

    /// A read served here found covered record `entity` absent at the head
    /// pre-pin: the chain created it, so the view is incoherent.
    pub fn mark_incoherent(&self, entity: EntityId) {
        if self.covers(entity) {
            let _ = self.incoherent.set(entity);
        }
    }

    /// The first record a served read found absent at the head, if any.
    pub fn incoherent(&self) -> Option<EntityId> {
        self.incoherent.get().copied()
    }
}
