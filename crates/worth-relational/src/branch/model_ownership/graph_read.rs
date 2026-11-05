//! Paid exact-generation relation facts at the original immutable basis.
use super::*;
use crate::{identity::data::RelationId, validation::data::StructuralRelationRecord};

impl BasisRead<'_, '_, '_, '_, '_, '_> {
    pub(in crate::branch::model_ownership) fn relation(
        &mut self,
        id: RelationId,
    ) -> Result<Option<StructuralRelationRecord>, RelationalModelOwnershipDenial> {
        self.charge(1)?;
        self.demand(id.partition_id)?;
        let live = self
            .root
            .get_current_partition(id.partition_id)
            .is_some_and(|region| region.relation(id).is_some_and(|slot| slot.is_live()));
        Ok(live
            .then(|| self.state.relation_metadata(id))
            .flatten()
            .map(|edge| StructuralRelationRecord {
                relation_id: id,
                kind_id: edge.kind_id,
                source: edge.source,
                target: edge.target,
            }))
    }

    pub(in crate::branch::model_ownership) fn relations(
        &mut self,
        entity: EntityId,
        outgoing: bool,
    ) -> Result<Vec<StructuralRelationRecord>, RelationalModelOwnershipDenial> {
        self.demand(entity.partition_id)?;
        let count = self.state.relation_candidate_count(entity, outgoing);
        self.retain_bytes(count.saturating_mul(512) as u64)?;
        self.charge(count.saturating_mul(3).saturating_add(1))?;
        let ids = self.state.relation_candidates(entity, outgoing);
        let mut edges = Vec::new();
        for &id in ids.iter() {
            if let Some(edge) = self.relation(id)? {
                let native_custody = self
                    .root
                    .schema_authority()
                    .registry()
                    .current_publication_storage
                    .as_ref()
                    .is_some_and(|storage| storage.metadata_kinds().contains(&edge.kind_id));
                if !native_custody
                    && if outgoing {
                        edge.source == entity
                    } else {
                        edge.target == entity
                    }
                {
                    edges.push(edge);
                }
            }
        }
        Ok(edges)
    }
}
