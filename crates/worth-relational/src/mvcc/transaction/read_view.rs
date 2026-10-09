use crate::identity::data::{EntityId, RelationId};
use crate::storage::data::{EntityReadRecord, RelationReadRecord};
use crate::transactions::data::{
    CommitConflict, CreateIntent, CreatedEntityRef, CreatedRelationRef, EntityMutationIntent,
    RelationMutationIntent,
};

use super::read_projection::{
    project_entity, project_relation, RelationalTransactionRelationValue,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationalTransactionEntityRead {
    base: Option<EntityReadRecord>,
    staged: Vec<EntityMutationIntent>,
    effective: Option<EntityReadRecord>,
}

impl RelationalTransactionEntityRead {
    pub fn base(&self) -> Option<&EntityReadRecord> {
        self.base.as_ref()
    }

    pub fn staged_mutations(&self) -> &[EntityMutationIntent] {
        &self.staged
    }

    pub fn effective(&self) -> Option<&EntityReadRecord> {
        self.effective.as_ref()
    }

    pub fn is_deleted(&self) -> bool {
        self.staged
            .iter()
            .any(|intent| matches!(intent, EntityMutationIntent::Delete(_)))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationalTransactionRelationRead {
    base: Option<RelationReadRecord>,
    staged: Vec<RelationMutationIntent>,
    effective: Option<RelationalTransactionRelationValue>,
}

impl RelationalTransactionRelationRead {
    pub fn base(&self) -> Option<&RelationReadRecord> {
        self.base.as_ref()
    }

    pub fn staged_mutations(&self) -> &[RelationMutationIntent] {
        &self.staged
    }

    pub fn effective(&self) -> Option<&RelationalTransactionRelationValue> {
        self.effective.as_ref()
    }

    pub fn is_deleted(&self) -> bool {
        self.staged
            .iter()
            .any(|intent| matches!(intent, RelationMutationIntent::Delete(_)))
    }
}

impl super::BranchBoundRelationalTransaction {
    /// Read one existing entity from the immutable exact basis and include all
    /// staged mutations for that identity in authoring order.
    pub fn read_entity(
        &mut self,
        entity_id: EntityId,
    ) -> Result<RelationalTransactionEntityRead, CommitConflict> {
        self.footprint
            .admit_read(super::RelationalTransactionReadLocus::Existing(
                crate::transactions::data::RecordRef::Entity(entity_id),
            ))
            .map_err(super::RelationalTransactionStagingDenial::into_conflict)?;
        let root = self.basis.inner.root.as_ref();
        let base = root.partition_state(entity_id.partition_id).and_then(|partition| {
            crate::visibility::materialization::read_records::materialization::materialize_current_authoritative_entity_record(
                root.schema_authority().registry(),
                partition,
                entity_id.partition_id,
                entity_id.slot_index(),
            )
        }).filter(|record| {
            entity_id.generation.is_zero() || record.entity_id.generation == entity_id.generation
        });
        let staged = self
            .overlay
            .entity_mutations(entity_id)
            .cloned()
            .collect::<Vec<_>>();
        let effective = project_entity(self.schema_authority.as_ref(), base.clone(), &staged)?;
        Ok(RelationalTransactionEntityRead {
            base,
            staged,
            effective,
        })
    }

    /// Read one existing relation from the immutable exact basis and include
    /// all staged mutations for that identity in authoring order.
    pub fn read_relation(
        &mut self,
        relation_id: RelationId,
    ) -> Result<RelationalTransactionRelationRead, CommitConflict> {
        self.footprint
            .admit_read(super::RelationalTransactionReadLocus::Existing(
                crate::transactions::data::RecordRef::Relation(relation_id),
            ))
            .map_err(super::RelationalTransactionStagingDenial::into_conflict)?;
        let root = self.basis.inner.root.as_ref();
        let base = root.partition_state(relation_id.partition_id).and_then(|partition| {
            crate::visibility::materialization::read_records::materialization::materialize_current_authoritative_relation_record(
                root.schema_authority().registry(),
                partition,
                relation_id.partition_id,
                relation_id.slot_index(),
            )
        }).filter(|record| {
            relation_id.generation.is_zero()
                || record.relation_id.generation == relation_id.generation
        });
        let staged = self
            .overlay
            .relation_mutations(relation_id)
            .cloned()
            .collect::<Vec<_>>();
        let effective = project_relation(self.schema_authority.as_ref(), base.clone(), &staged)?;
        Ok(RelationalTransactionRelationRead {
            base,
            staged,
            effective,
        })
    }

    pub fn read_created_entity<'a>(
        &'a mut self,
        entity: &CreatedEntityRef,
    ) -> Result<
        Option<impl ExactSizeIterator<Item = &'a CreateIntent> + 'a>,
        super::RelationalTransactionStagingDenial,
    > {
        let entity = self.overlay.canonical_created_entity_ref(entity);
        self.footprint
            .admit_read(super::RelationalTransactionReadLocus::CreatedEntity(
                entity.clone(),
            ))?;
        Ok(self.overlay.created_entity(&entity))
    }

    pub fn read_created_relation<'a>(
        &'a mut self,
        relation: &CreatedRelationRef,
    ) -> Result<
        Option<impl ExactSizeIterator<Item = &'a CreateIntent> + 'a>,
        super::RelationalTransactionStagingDenial,
    > {
        let relation = self.overlay.canonical_created_relation_ref(relation);
        self.footprint
            .admit_read(super::RelationalTransactionReadLocus::CreatedRelation(
                relation.clone(),
            ))?;
        Ok(self.overlay.created_relation(&relation))
    }
}
