//! Per-refresh kind routing, derived once from both authoritative views.
use super::{changes::ChangedRecords, record_metadata, work::MaintenanceWork};
use crate::identity::data::{EntityId, KindId, RelationId};
use crate::indexes::data::DerivedIndexMaintenanceDenialKind as Denial;
use crate::runtime::VisibilityProjectionView;
use record_metadata::RelationMetadata;
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
    pub(super) old: Option<RelationMetadata>,
    pub(super) new: Option<RelationMetadata>,
}

pub(super) struct ChangeRouting {
    entities: BTreeMap<KindId, Vec<EntityChange>>,
    relations: BTreeMap<KindId, Vec<RelationChange>>,
}

impl ChangeRouting {
    pub(super) fn classify(
        changes: &ChangedRecords,
        before: Option<&VisibilityProjectionView<'_>>,
        after: &VisibilityProjectionView<'_>,
        work: &mut MaintenanceWork,
    ) -> Result<Self, Denial> {
        let mut routed = Self {
            entities: BTreeMap::new(),
            relations: BTreeMap::new(),
        };
        for id in &changes.entities {
            let old = record_metadata::entity_kind(before, *id, work)?;
            let new = record_metadata::entity_kind(Some(after), *id, work)?;
            let change = EntityChange { id: *id, old, new };
            for kind in old.into_iter().chain(new.filter(|kind| Some(*kind) != old)) {
                insert(&mut routed.entities, kind, change, work)?;
            }
        }
        for id in &changes.relations {
            let old = record_metadata::relation(before, *id, work)?;
            let new = record_metadata::relation(Some(after), *id, work)?;
            let change = RelationChange { id: *id, old, new };
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
