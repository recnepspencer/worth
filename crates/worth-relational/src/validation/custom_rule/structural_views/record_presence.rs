//! Negative structural facts require the selected head metadata to be available.
use super::*;

pub(crate) enum StructuralPresenceDenial {
    Read(StructuralReadError),
    Request(crate::validation::data::StructuralWalkDenialKind),
    Native(crate::execution::RelationalExecutionDenialCause),
}

impl StructuralRelationView<'_> {
    fn available_state(
        &self,
        partition: crate::identity::data::PartitionId,
    ) -> Result<InvariantStateView<'_>, StructuralPresenceDenial> {
        self.state_view
            .state()
            .demand_head_partition(&self.execution, partition)
            .map_err(|cause| StructuralPresenceDenial::Native(cause.into()))?;
        // Preparation may have cached a nonresident miss. Observe the same
        // immutable candidate after its metadata owner admits residency.
        Ok(InvariantStateView::new(
            self.state_view.state(),
            self.state_view.version_id(),
        ))
    }

    pub(crate) fn live_entity_kind(
        &self,
        entity: EntityId,
    ) -> Result<Option<KindId>, StructuralPresenceDenial> {
        self.read_budget()
            .checkpoint(1)
            .map_err(StructuralPresenceDenial::Request)?;
        match self
            .available_state(entity.partition_id)?
            .entity_metadata(entity)
        {
            Some(metadata) if self.access.reads_entity(metadata.kind_id) => {
                Ok(Some(metadata.kind_id))
            }
            Some(_) => Err(StructuralPresenceDenial::Read(
                StructuralReadError::OutsideDeclaredAccess,
            )),
            None => Ok(None),
        }
    }

    pub(crate) fn live_relation_record(
        &self,
        id: RelationId,
    ) -> Result<Option<StructuralRelationRecord>, StructuralPresenceDenial> {
        self.read_budget()
            .checkpoint(1)
            .map_err(StructuralPresenceDenial::Request)?;
        match self.available_state(id.partition_id)?.relation_metadata(id) {
            Some(edge) if self.access.reads_relation(edge.kind_id) => {
                Ok(Some(StructuralRelationRecord {
                    relation_id: id,
                    kind_id: edge.kind_id,
                    source: edge.source,
                    target: edge.target,
                }))
            }
            Some(_) => Err(StructuralPresenceDenial::Read(
                StructuralReadError::OutsideDeclaredAccess,
            )),
            None => Ok(None),
        }
    }

    pub(crate) fn live_relations_for_entity(
        &self,
        entity: EntityId,
        outgoing: bool,
    ) -> Result<Vec<StructuralRelationRecord>, StructuralPresenceDenial> {
        let state = self.available_state(entity.partition_id)?;
        let count = state.relation_candidate_count(entity, outgoing);
        let budget = self.read_budget();
        budget
            .checkpoint((count as u64).saturating_mul(3).saturating_add(1))
            .map_err(StructuralPresenceDenial::Request)?;
        // Raw candidate deduplication, the retained ID list and result records
        // are all covered before any of those collections are materialized.
        budget
            .claim_retained((count as u64).saturating_mul(512))
            .map_err(StructuralPresenceDenial::Request)?;
        let ids = state.relation_candidates(entity, outgoing);
        let mut edges = Vec::new();
        for &id in ids.iter() {
            let state = self.available_state(id.partition_id)?;
            let Some(edge) = state.relation_metadata(id) else {
                continue;
            };
            if self.access.reads_relation(edge.kind_id)
                && if outgoing {
                    edge.source == entity
                } else {
                    edge.target == entity
                }
            {
                edges.push(StructuralRelationRecord {
                    relation_id: id,
                    kind_id: edge.kind_id,
                    source: edge.source,
                    target: edge.target,
                });
            }
        }
        Ok(edges)
    }
}
