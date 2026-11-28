//! Which persistent names are live entity members once this commit lands.
//!
//! Group finalization runs after the commit's entity and relation lineage is
//! final, so it reads the commit's own canonical events: a birth or a kept
//! name binds a name to the record its target lineage was created for, and a
//! lineage that a commit event leaves (a source that is not also a target)
//! ends the name it carried. Every other name keeps its basis holder.

use std::collections::{BTreeMap, BTreeSet};

use contracts::identity::data::primitives::LineageId;
use contracts::identity::data::record_ids::EntityId;
use contracts::lineage::data::artifacts::finalization_artifact::PreparedLineageFinalization;
use contracts::lineage::data::events::{LineageEventKind, LineageSubject};
use contracts::lineage::data::naming::{PersistentNameHolder, PersistentNameIdentity};
use contracts::lineage::data::postures::LineageBasisView;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_lineage::internal_port::runtime::state::subsystems::lineage::naming_basis_binding::BranchNamingBasisGuard;

/// The commit's effect on name liveness, read against its basis.
pub(super) struct CommitNameOutcome<'basis, 'guard> {
    basis: &'basis BranchNamingBasisGuard<'guard>,
    /// Names this commit bound: to an entity, or to a relation (`None`).
    bound: BTreeMap<PersistentNameIdentity, Option<EntityId>>,
    ended: BTreeSet<LineageId>,
    bound_entities: BTreeMap<EntityId, PersistentNameIdentity>,
}

impl<'basis, 'guard> CommitNameOutcome<'basis, 'guard> {
    /// Complexity: O(E log E) in the commit's lineage events and new nodes.
    pub(super) fn of(
        prepared: &PreparedLineageFinalization,
        basis: &'basis BranchNamingBasisGuard<'guard>,
    ) -> Self {
        let nodes: BTreeMap<LineageId, EntityId> = prepared
            .new_nodes()
            .iter()
            .map(|node| (node.lineage_id(), node.entity_id()))
            .collect();
        let mut bound = BTreeMap::new();
        let mut ended = BTreeSet::new();
        for event in prepared.artifact().event_batch().events() {
            let name = event
                .name_birth()
                .map(|birth| birth.identity())
                .or(event.kept_name());
            if let (Some(name), Some(target)) = (name, event.targets().first()) {
                match (event.subject(), nodes.get(target)) {
                    (LineageSubject::Entity, Some(entity)) => {
                        bound.insert(name, Some(*entity));
                    }
                    (LineageSubject::Relation, _) => {
                        bound.insert(name, None);
                    }
                    (LineageSubject::Entity, None) => {}
                }
            }
            if event.kind() != LineageEventKind::Create {
                let leaving = event
                    .sources()
                    .iter()
                    .filter(|source| !event.targets().contains(source));
                ended.extend(leaving.copied());
            }
        }
        let bound_entities = bound
            .iter()
            .filter_map(|(&name, entity)| entity.map(|entity| (entity, name)))
            .collect();
        Self {
            basis,
            bound_entities,
            bound,
            ended,
        }
    }

    /// The entity `name` designates once the commit lands, or `None` when it
    /// is unknown, retired, or names a relation.
    ///
    /// Complexity: O(log N) basis probes.
    pub(super) fn member_after(&self, name: &PersistentNameIdentity) -> Option<EntityId> {
        if let Some(bound) = self.bound.get(name) {
            return *bound;
        }
        let holder = self.basis.live_holder(name)?;
        let PersistentNameHolder::Entity(entity) = &holder else {
            return None;
        };
        let lineage = self.basis.live_lineage(&holder)?;
        (!self.ended.contains(&lineage)).then_some(*entity)
    }

    pub(super) fn name_before(&self, entity: EntityId) -> Option<PersistentNameIdentity> {
        let holder = PersistentNameHolder::Entity(entity);
        let name = self.basis.holder_name(&holder)?;
        (self.basis.live_holder(&name) == Some(holder)).then_some(name)
    }

    pub(super) fn name_after(&self, entity: EntityId) -> Option<PersistentNameIdentity> {
        self.bound_entities.get(&entity).copied().or_else(|| {
            let name = self.name_before(entity)?;
            (self.member_after(&name) == Some(entity)).then_some(name)
        })
    }

    /// Names this commit binds, with the entity each now designates (`None`
    /// for a relation).
    ///
    /// Complexity: O(1) to start; O(bound names) to walk.
    pub(super) fn bound_names(
        &self,
    ) -> impl Iterator<Item = (&PersistentNameIdentity, &Option<EntityId>)> {
        self.bound.iter()
    }

    /// Basis names this commit ends without rebinding them.
    ///
    /// Complexity: O(ended lineages * log N).
    pub(super) fn retired_names(&self) -> Vec<PersistentNameIdentity> {
        self.ended
            .iter()
            .filter_map(|lineage| self.basis.holder_and_name_for_lineage(*lineage))
            .map(|(_, name)| name)
            .filter(|name| !self.bound.contains_key(name))
            .filter(|name| self.basis.live_holder(name).is_some())
            .collect()
    }
}
