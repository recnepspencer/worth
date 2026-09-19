use worth_foundational::facade::AuthoritativeRecordAspectState;

use crate::identity::data::{EntityId, KindId, RelationId};
use crate::validation::engine::state_view::InvariantStateView;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructuralRelationRecord {
    pub relation_id: RelationId,
    pub kind_id: KindId,
    pub source: EntityId,
    pub target: EntityId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuralReadError {
    WorkBudgetExceeded,
    OutsideDeclaredAccess,
    RecordUnavailable,
}

#[derive(Clone)]
pub struct StructuralAspectStateView<'runtime> {
    state_view: InvariantStateView<'runtime>,
    work: super::CustomInvariantWorkMeter,
    access: std::sync::Arc<crate::validation::data::CustomInvariantAccessContract>,
}

impl<'runtime> StructuralAspectStateView<'runtime> {
    pub(crate) fn new(
        state_view: InvariantStateView<'runtime>,
        work: super::CustomInvariantWorkMeter,
        access: std::sync::Arc<crate::validation::data::CustomInvariantAccessContract>,
    ) -> Self {
        Self {
            state_view,
            work,
            access,
        }
    }

    pub fn entity_aspect_state(
        &self,
        entity_id: EntityId,
    ) -> Result<&'runtime AuthoritativeRecordAspectState, StructuralReadError> {
        if !self.work.try_charge(1) {
            return Err(StructuralReadError::WorkBudgetExceeded);
        }
        let metadata = self
            .state_view
            .entity_metadata(entity_id)
            .ok_or(StructuralReadError::RecordUnavailable)?;
        if !self.access.reads_entity(metadata.kind_id) {
            return Err(StructuralReadError::OutsideDeclaredAccess);
        }
        self.state_view
            .entity_aspect_state(entity_id)
            .ok_or(StructuralReadError::RecordUnavailable)
    }

    pub fn relation_aspect_state(
        &self,
        relation_id: RelationId,
    ) -> Result<&'runtime AuthoritativeRecordAspectState, StructuralReadError> {
        if !self.work.try_charge(1) {
            return Err(StructuralReadError::WorkBudgetExceeded);
        }
        let metadata = self
            .state_view
            .relation_metadata(relation_id)
            .ok_or(StructuralReadError::RecordUnavailable)?;
        if !self.access.reads_relation(metadata.kind_id) {
            return Err(StructuralReadError::OutsideDeclaredAccess);
        }
        self.state_view
            .relation_aspect_state(relation_id)
            .ok_or(StructuralReadError::RecordUnavailable)
    }
}

#[derive(Clone)]
pub struct StructuralRelationView<'runtime> {
    state_view: InvariantStateView<'runtime>,
    work: super::CustomInvariantWorkMeter,
    access: std::sync::Arc<crate::validation::data::CustomInvariantAccessContract>,
}

impl<'runtime> StructuralRelationView<'runtime> {
    pub(crate) fn new(
        state_view: InvariantStateView<'runtime>,
        work: super::CustomInvariantWorkMeter,
        access: std::sync::Arc<crate::validation::data::CustomInvariantAccessContract>,
    ) -> Self {
        Self {
            state_view,
            work,
            access,
        }
    }

    pub fn entity_kind(&self, entity_id: EntityId) -> Result<KindId, StructuralReadError> {
        if !self.work.try_charge(1) {
            return Err(StructuralReadError::WorkBudgetExceeded);
        }
        let metadata = self
            .state_view
            .entity_metadata(entity_id)
            .ok_or(StructuralReadError::RecordUnavailable)?;
        if !self.access.reads_entity(metadata.kind_id) {
            return Err(StructuralReadError::OutsideDeclaredAccess);
        }
        Ok(metadata.kind_id)
    }

    pub fn readable_entity_kind(
        &self,
        entity_id: EntityId,
    ) -> Result<Option<KindId>, StructuralReadError> {
        if !self.work.try_charge(1) {
            return Err(StructuralReadError::WorkBudgetExceeded);
        }
        let metadata = self
            .state_view
            .entity_metadata(entity_id)
            .ok_or(StructuralReadError::RecordUnavailable)?;
        Ok(self
            .access
            .reads_entity(metadata.kind_id)
            .then_some(metadata.kind_id))
    }

    pub fn require_entity_kind(&self, kind_id: KindId) -> Result<(), StructuralReadError> {
        if self.access.reads_entity(kind_id) {
            Ok(())
        } else {
            Err(StructuralReadError::OutsideDeclaredAccess)
        }
    }

    pub fn relation(
        &self,
        relation_id: RelationId,
    ) -> Result<StructuralRelationRecord, StructuralReadError> {
        if !self.work.try_charge(1) {
            return Err(StructuralReadError::WorkBudgetExceeded);
        }
        let metadata = self
            .state_view
            .relation_metadata(relation_id)
            .ok_or(StructuralReadError::RecordUnavailable)?;
        if !self.access.reads_relation(metadata.kind_id) {
            return Err(StructuralReadError::OutsideDeclaredAccess);
        }
        Ok(StructuralRelationRecord {
            relation_id: metadata.relation_id,
            kind_id: metadata.kind_id,
            source: metadata.source,
            target: metadata.target,
        })
    }

    pub fn require_relation_kind(&self, kind_id: KindId) -> Result<(), StructuralReadError> {
        if self.access.reads_relation(kind_id) {
            Ok(())
        } else {
            Err(StructuralReadError::OutsideDeclaredAccess)
        }
    }

    pub fn outgoing_relations_for_entity(
        &self,
        entity_id: EntityId,
    ) -> Result<Vec<RelationId>, crate::validation::data::CustomInvariantTraversalError> {
        self.adjacency(entity_id, true)
    }
    pub fn incoming_relations_for_entity(
        &self,
        entity_id: EntityId,
    ) -> Result<Vec<RelationId>, crate::validation::data::CustomInvariantTraversalError> {
        self.adjacency(entity_id, false)
    }
    fn adjacency(
        &self,
        entity_id: EntityId,
        outgoing: bool,
    ) -> Result<Vec<RelationId>, crate::validation::data::CustomInvariantTraversalError> {
        if !self.work.try_charge(1) {
            return Err(work_exhausted());
        }
        let raw_count = self
            .state_view
            .relation_candidate_count(entity_id, outgoing);
        // Charge raw adjacency collection, owner visibility filtering, and declared
        // kind filtering before materializing or reading any candidate metadata.
        if !self.work.try_charge(raw_count.saturating_mul(3)) {
            return Err(work_exhausted());
        }
        let relations = if outgoing {
            self.state_view.outgoing_relations_for_entity(entity_id)
        } else {
            self.state_view.incoming_relations_for_entity(entity_id)
        };
        Ok(relations
            .into_iter()
            .filter(|id| {
                self.state_view
                    .relation_metadata(*id)
                    .is_some_and(|metadata| self.access.reads_relation(metadata.kind_id))
            })
            .collect())
    }
    pub fn all_relations_for_entity(
        &self,
        entity_id: EntityId,
    ) -> Result<Vec<RelationId>, crate::validation::data::CustomInvariantTraversalError> {
        let outgoing = self.outgoing_relations_for_entity(entity_id)?;
        let incoming = self.incoming_relations_for_entity(entity_id)?;
        if !self
            .work
            .try_charge(outgoing.len().saturating_add(incoming.len()))
        {
            return Err(work_exhausted());
        }
        Ok(outgoing
            .into_iter()
            .chain(incoming)
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect())
    }
}

fn work_exhausted() -> crate::validation::data::CustomInvariantTraversalError {
    crate::validation::data::CustomInvariantTraversalError::new(
        "custom invariant adjacency read exceeded its work budget",
    )
}
