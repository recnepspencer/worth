//! Per-refresh kind routing, derived once from both authoritative views.
use super::root_view::IndexRootView;
use super::{changes::ChangedRecords, record_metadata, work::MaintenanceWork};
use crate::identity::data::{EntityId, KindId, RelationId};
use crate::indexes::data::DerivedIndexMaintenanceDenialKind as Denial;
use std::collections::BTreeMap;

#[derive(Clone, Copy)]
pub(super) struct EntityChange {
    pub(super) id: EntityId,
    pub(super) old: Option<KindId>,
    pub(super) new: Option<KindId>,
}

#[derive(Clone, Copy)]
pub(super) struct RelationChange {
    pub(super) id: RelationId,
}

pub(super) struct ChangeRouting {
    entities: BTreeMap<KindId, Vec<EntityChange>>,
    relations: BTreeMap<KindId, Vec<RelationChange>>,
}

impl ChangeRouting {
    pub(super) fn classify(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        changes: &ChangedRecords,
        before: Option<&IndexRootView>,
        after: &IndexRootView,
        work: &mut MaintenanceWork,
    ) -> Result<Self, Denial> {
        if before.is_some() {
            return Self::from_change(
                after
                    .root()
                    .commit_change()
                    .ok_or(Denial::ColdReconstructionRequired)?,
                work,
            );
        }
        let mut routed = Self {
            entities: BTreeMap::new(),
            relations: BTreeMap::new(),
        };
        for id in &changes.entities {
            let old = record_metadata::entity_kind(execution, before, *id, work)?;
            let new = record_metadata::entity_kind(execution, Some(after), *id, work)?;
            let change = EntityChange { id: *id, old, new };
            for kind in old.into_iter().chain(new.filter(|kind| Some(*kind) != old)) {
                insert(&mut routed.entities, kind, change, work)?;
            }
        }
        for id in &changes.relations {
            let old = record_metadata::relation(execution, before, *id, work)?;
            let new = record_metadata::relation(execution, Some(after), *id, work)?;
            let change = RelationChange { id: *id };
            let old_kind = old.map(|r| r.kind);
            for kind in old_kind
                .into_iter()
                .chain(new.map(|r| r.kind).filter(|kind| Some(*kind) != old_kind))
            {
                insert(&mut routed.relations, kind, change, work)?;
            }
        }
        Ok(routed)
    }

    fn from_change(
        change: &crate::storage::substrate::maintenance::CommitChange,
        work: &mut MaintenanceWork,
    ) -> Result<Self, Denial> {
        use crate::storage::data::RecordLifecycleState;
        use crate::transactions::data::RecordRef;
        let mut routed = Self {
            entities: BTreeMap::new(),
            relations: BTreeMap::new(),
        };
        for (_, partition) in change.partitions() {
            for record in partition.records() {
                work.charge(1)?;
                let mut identities = Vec::new();
                for state in record.before.iter().chain(&record.after) {
                    if !identities.contains(&state.record) {
                        work.grow_vec(&mut identities)?;
                        identities.push(state.record.clone());
                    }
                }
                for id in identities {
                    let old = record.before.as_ref().filter(|state| state.record == id);
                    let new = record.after.as_ref().filter(|state| state.record == id);
                    match id {
                        RecordRef::Entity(id) => {
                            let old = old
                                .filter(|state| state.lifecycle == RecordLifecycleState::Live)
                                .map(|state| state.kind);
                            let new = new
                                .filter(|state| state.lifecycle == RecordLifecycleState::Live)
                                .map(|state| state.kind);
                            for kind in old.into_iter().chain(new.filter(|kind| Some(*kind) != old))
                            {
                                insert(
                                    &mut routed.entities,
                                    kind,
                                    EntityChange { id, old, new },
                                    work,
                                )?;
                            }
                        }
                        RecordRef::Relation(id) => {
                            let visible = |state: &&crate::storage::substrate::maintenance::RecordChangeState| matches!(state.lifecycle, RecordLifecycleState::Live | RecordLifecycleState::RetainedDanglingForAudit);
                            let old = old.filter(visible).map(|state| state.kind);
                            let new = new.filter(visible).map(|state| state.kind);
                            for kind in old.into_iter().chain(new.filter(|kind| Some(*kind) != old))
                            {
                                insert(&mut routed.relations, kind, RelationChange { id }, work)?;
                            }
                        }
                    }
                }
            }
        }
        Ok(routed)
    }

    pub(super) fn kinds(&self) -> impl Iterator<Item = (KindId, bool)> + '_ {
        self.entities
            .keys()
            .map(|kind| (*kind, true))
            .chain(self.relations.keys().map(|kind| (*kind, false)))
    }

    pub(super) fn entities(
        &self,
        kind: KindId,
        work: &mut MaintenanceWork,
    ) -> Result<&[EntityChange], Denial> {
        work.lookup(self.entities.len(), 1)?;
        work.charge(1)?;
        Ok(self.entities.get(&kind).map_or(&[], Vec::as_slice))
    }

    pub(super) fn relations(
        &self,
        kind: KindId,
        work: &mut MaintenanceWork,
    ) -> Result<&[RelationChange], Denial> {
        work.lookup(self.relations.len(), 1)?;
        work.charge(1)?;
        Ok(self.relations.get(&kind).map_or(&[], Vec::as_slice))
    }
}

fn insert<T>(
    map: &mut BTreeMap<KindId, Vec<T>>,
    kind: KindId,
    change: T,
    work: &mut MaintenanceWork,
) -> Result<(), Denial> {
    work.charge(1)?;
    work.ordered::<KindId, Vec<T>>(map.len(), 1, 0)?;
    let records = map.entry(kind).or_default();
    work.grow_vec(records)?;
    records.push(change);
    Ok(())
}
