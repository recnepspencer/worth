//! One complete Native population and endpoint index, without a second schema engine.
use super::*;
use worth_relational::facade::runtime::VisibilityProjectionView;

pub(super) struct WorkflowPopulation {
    entities: BTreeMap<KindId, Vec<EntityId>>,
    outgoing: BTreeMap<(KindId, EntityId), Vec<EntityId>>,
    incoming: BTreeMap<(KindId, EntityId), Vec<EntityId>>,
}
impl WorkflowPopulation {
    pub(super) fn read(
        view: &VisibilityProjectionView,
        execution: &ExecutionRequest<'_, '_>,
        layout: &WorthQueryWorkflowLayout,
    ) -> Result<Self, Denial> {
        let (entities, relations) = layout.home_readmission_kinds();
        let mut result = Self {
            entities: BTreeMap::new(),
            outgoing: BTreeMap::new(),
            incoming: BTreeMap::new(),
        };
        for kind in entities {
            let rows = view
                .bounded_entities_of_kind(execution, kind, usize::MAX)
                .map_err(
                    crate::domain_computation::primary_graph::bootstrap::request_read_denial::read,
                )
                .map_err(Denial::Source)?
                .map_err(Denial::EntityPopulation)?;
            let mut ids = rows
                .into_records()
                .into_iter()
                .map(|record| record.entity_id)
                .collect::<Vec<_>>();
            ids.sort_unstable();
            result.entities.insert(kind, ids);
        }
        for kind in relations {
            let rows = view
                .bounded_relations_of_kind(execution, kind, usize::MAX)
                .map_err(
                    crate::domain_computation::primary_graph::bootstrap::request_read_denial::read,
                )
                .map_err(Denial::Source)?
                .map_err(Denial::RelationPopulation)?;
            for relation in rows.into_records() {
                result
                    .outgoing
                    .entry((kind, relation.source))
                    .or_default()
                    .push(relation.target);
                result
                    .incoming
                    .entry((kind, relation.target))
                    .or_default()
                    .push(relation.source);
            }
        }
        Ok(result)
    }
    pub(super) fn is_empty(&self) -> bool {
        self.entities.values().all(Vec::is_empty) && self.outgoing.is_empty()
    }
    pub(super) fn entities(&self, kind: KindId) -> &[EntityId] {
        self.entities
            .get(&kind)
            .expect("exhaustive workflow kind inventory")
    }
    pub(super) fn contains(&self, kind: KindId, entity: EntityId) -> bool {
        self.entities(kind).binary_search(&entity).is_ok()
    }
    pub(super) fn outgoing(&self, kind: KindId, source: EntityId) -> &[EntityId] {
        self.outgoing
            .get(&(kind, source))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
    pub(super) fn incoming(&self, kind: KindId, target: EntityId) -> &[EntityId] {
        self.incoming
            .get(&(kind, target))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
    pub(super) fn one(&self, kind: KindId, source: EntityId) -> Result<Option<EntityId>, Denial> {
        match self.outgoing(kind, source) {
            [] => Ok(None),
            [one] => Ok(Some(*one)),
            _ => Err(Denial::CorruptRecord {
                entity: source,
                kind,
            }),
        }
    }
    pub(super) fn parent(&self, kind: KindId, target: EntityId) -> Result<EntityId, Denial> {
        match self.incoming(kind, target) {
            [one] => Ok(*one),
            _ => Err(Denial::CorruptRecord {
                entity: target,
                kind,
            }),
        }
    }
}
