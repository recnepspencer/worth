use worth_relational::facade::runtime::StructuralReadError;

use super::{
    map_structural_error, WorthQueryApplicationInvariantEntity,
    WorthQueryApplicationInvariantFieldBinding, WorthQueryApplicationInvariantReadView,
    WorthQueryInvariantAccessDenial,
};
use crate::domain_computation::primary_graph::application_invariant::WorthQueryApplicationInvariantEntityBinding;

impl<Schema> WorthQueryApplicationInvariantReadView<'_, Schema> {
    /// Returns directly mutated visible entities of the bound kind, including
    /// old and new relation endpoints but excluding traversal-only neighbors.
    pub fn touched_entities_of<Entity>(
        &self,
        entity: &WorthQueryApplicationInvariantEntityBinding<Schema, Entity>,
    ) -> Result<
        Vec<WorthQueryApplicationInvariantEntity<Schema, Entity>>,
        WorthQueryInvariantAccessDenial,
    > {
        self.check_binding(&entity.binding_identity)?;
        self.relations
            .require_entity_kind(entity.entity_kind)
            .map_err(|error| map_structural_error(error, "touched entity kind"))?;
        let mut entities = Vec::new();
        for entity_id in self.touched_entities.iter().copied() {
            let kind = match self.relations.readable_entity_kind(entity_id) {
                Ok(kind) => kind,
                Err(StructuralReadError::RecordUnavailable) => continue,
                Err(error) => return Err(map_structural_error(error, "touched entity")),
            };
            self.reads.endpoint();
            if kind == Some(entity.entity_kind) {
                self.admission.entity(entity_id)?;
                entities.push(self.entity(entity_id, entity.entity_kind));
            }
        }
        Ok(entities)
    }

    /// Returns directly mutated visible entities carrying the bound field.
    /// Relation before- and after-image endpoints are included; adjacency-only
    /// traversal neighbors are excluded.
    pub fn touched_entities<Entity, Value>(
        &self,
        field: &WorthQueryApplicationInvariantFieldBinding<Schema, Entity, Value>,
    ) -> Result<
        Vec<WorthQueryApplicationInvariantEntity<Schema, Entity>>,
        WorthQueryInvariantAccessDenial,
    > {
        self.check_binding(&field.binding_identity)?;
        self.relations
            .require_entity_kind(field.entity_kind)
            .map_err(|error| map_structural_error(error, "touched entity kind"))?;
        let mut entities = Vec::new();
        for entity_id in self.touched_entities.iter().copied() {
            let kind = match self.relations.readable_entity_kind(entity_id) {
                Ok(kind) => kind,
                Err(StructuralReadError::RecordUnavailable) => continue,
                Err(error) => return Err(map_structural_error(error, "touched entity")),
            };
            self.reads.endpoint();
            if kind == Some(field.entity_kind) {
                self.admission.entity(entity_id)?;
                entities.push(self.entity(entity_id, field.entity_kind));
            }
        }
        Ok(entities)
    }
}
