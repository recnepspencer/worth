//! A successor round's own-output view (#119 §3.5): the reopened derivation
//! reads every record any round of its chain touched as it was immediately
//! before the chain's head appended, so it re-derives from that image under
//! the new inputs, which is the image Native retracts the chain to before it
//! applies the round as written. Native certified that no later live member
//! changed those records (`chain_overlay`); field reads and key resolution of
//! those records are served here, and any other read of a key the chain
//! wrote stops before append (`PriorEffectRead`).
//!
//! Relations are served too (#277): an adjacency read is the current list
//! minus the chain's own links, plus the links it removed, as they were at
//! the head pre-pin (`relations`). A record the chain created is then
//! unreachable through the chain's own links. A link the chain did not make
//! can still reach one; that view is incoherent, because such a read is
//! another member's effect on the chain's records, not C. `mark_incoherent`
//! records the first record found absent at the head, and
//! `mark_incoherent_relation` an anchor whose derived traversal crosses a
//! covered relation; both are shared by every clone, and a successor round
//! whose prepare then fails untyped stops typed (`PriorEffectRead`) on it.
mod relation_facts;
mod relations;
pub use relations::{merge_records, OverlayRelations};
use worth_relational::facade::{
    identity::{EntityId, KindId},
    runtime::{
        RelationalAdjacencyDirection, RelationalAppliedCandidateReadIdentity, RelationalPinnedRead,
    },
};

#[derive(Clone)]
pub struct OwnOutputOverlay {
    head: RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    exclusive: Vec<EntityId>,
    relations: OverlayRelations,
    /// The first record an incoherent read named: a covered record absent at
    /// the head, or the anchor of a derived traversal over a covered relation.
    incoherent: std::sync::Arc<std::sync::OnceLock<EntityId>>,
}

impl OwnOutputOverlay {
    pub fn from_chain(
        chain: &worth_relational::facade::mvcc::RelationalCoupledChainOverlay,
    ) -> Self {
        Self {
            head: chain.head_pre_pin().clone(),
            exclusive: chain.exclusive_entities().to_vec(),
            relations: OverlayRelations::from_chain(chain.relations()),
            incoherent: std::sync::Arc::default(),
        }
    }

    /// The chain head's pre-pin: the image before the chain's first round.
    pub fn head(&self) -> &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity> {
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

    /// A derived traversal (Native `derived_source`, not overlay-aware)
    /// probed `anchor`'s adjacency over a covered relation: its answer is the
    /// chain's effect. Unconditional, unlike `mark_incoherent`: the anchor is
    /// usually a pre-existing record the chain never wrote.
    pub fn mark_incoherent_relation(&self, anchor: EntityId) {
        let _ = self.incoherent.set(anchor);
    }

    /// A derived traversal probed `probes` (anchor, kind, outgoing) through
    /// Native `derived_source`, which is not overlay-aware: if one crosses a
    /// list a covered relation lies on, its answer holds the chain's effect,
    /// so the view is marked incoherent at that anchor. Returns whether it was.
    pub fn crossed_by_derived_traversal(
        &self,
        probes: impl IntoIterator<Item = (EntityId, KindId, bool)>,
    ) -> bool {
        let crossed = probes.into_iter().find(|&(anchor, kind, outgoing)| {
            self.touches_adjacency(
                anchor,
                kind,
                if outgoing {
                    RelationalAdjacencyDirection::Outgoing
                } else {
                    RelationalAdjacencyDirection::Incoming
                },
            )
        });
        if let Some((anchor, _, _)) = crossed {
            self.mark_incoherent_relation(anchor);
        }
        crossed.is_some()
    }

    /// An indexed selection as the view serves it: records the chain wrote
    /// exclusively match as they did at the head pre-pin, every other record
    /// as it does now (`current ∖ covered ∪ head ∩ covered`), sorted and
    /// unique. One owner for projection selections and unique-write
    /// dependencies (#277 r4 A).
    pub fn serve_candidates(&self, current: Vec<EntityId>, head: Vec<EntityId>) -> Vec<EntityId> {
        let mut candidates = current
            .into_iter()
            .filter(|entity| !self.covers(*entity))
            .chain(head.into_iter().filter(|entity| self.covers(*entity)))
            .collect::<Vec<_>>();
        candidates.sort_unstable();
        candidates.dedup();
        candidates
    }

    /// The chain's covered relations and the adjacency lists they touch.
    pub fn relations(&self) -> &OverlayRelations {
        &self.relations
    }

    /// Whether `anchor`'s `kind` list in `direction` holds a covered relation
    /// on either side, so that serving it needs the head pre-pin's list too.
    pub fn touches_adjacency(
        &self,
        anchor: EntityId,
        kind: KindId,
        direction: RelationalAdjacencyDirection,
    ) -> bool {
        self.relations.touches(anchor, kind, direction)
    }

    /// The first record a served read found absent at the head, if any.
    pub fn incoherent(&self) -> Option<EntityId> {
        self.incoherent.get().copied()
    }

    /// Whether `entity` is a record the chain created: covered, and not
    /// live at the head pre-pin. One charged point read, only when covered.
    pub fn created_by_chain(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity: EntityId,
    ) -> Result<bool, worth_relational::facade::transactions::RelationalExecutionDenialCause> {
        Ok(self.covers(entity) && !self.live_at_head(execution, entity)?)
    }

    /// Whether `entity` is live at the head pre-pin. One charged point read.
    pub fn live_at_head(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity: EntityId,
    ) -> Result<bool, worth_relational::facade::transactions::RelationalExecutionDenialCause> {
        live_in(&self.head.project_candidate(), execution, entity)
    }
}

/// Whether `entity` is live in `view`, in one charged point read. Every
/// guard of the own-output view means this by "live".
pub fn live_in(
    view: &worth_relational::facade::runtime::VisibilityProjectionView,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    entity: EntityId,
) -> Result<bool, worth_relational::facade::transactions::RelationalExecutionDenialCause> {
    view.entity_record_with_projection_scope(
        execution,
        entity,
        worth_relational::facade::runtime::ProjectionAspectScope::empty(),
        |record| {
            (record.lifecycle() == worth_relational::facade::storage::RecordLifecycleState::Live)
                .then_some(())
        },
    )
    .map(|present| present.is_some())
}
