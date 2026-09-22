use super::{
    BulkRelationCreateIntent, CreateIntent, DeleteEntityIntent, EntityMutationIntent,
    ExistingRecordTarget, MutationIntent, RelationIdentity, RelationMutationIntent,
    ReplaceEntityIntent, RollbackEffect,
};
use crate::identity::data::PartitionId;
use crate::validation::data::{InvariantGroup, InvariantGroupSet, InvariantPlanContract};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitTopology {
    FlatEntityBatch,
    GraphMutation,
    BranchMerge,
}

impl CommitTopology {
    pub const fn mask(self) -> u32 {
        match self {
            Self::FlatEntityBatch => 1 << 0,
            Self::GraphMutation => 1 << 1,
            Self::BranchMerge => 1 << 2,
        }
    }
}

impl MutationIntent {
    pub(crate) fn seed_touched_partitions(
        &self,
        touched: &mut std::collections::BTreeSet<PartitionId>,
    ) {
        match self {
            Self::Create(CreateIntent::Entity(spec)) => {
                touched.insert(spec.partition_id);
            }
            Self::Create(CreateIntent::EntityAspects(spec)) => {
                touched.insert(spec.partition_id);
            }
            Self::Create(CreateIntent::BulkEntities(spec)) => {
                touched.insert(spec.partition_id);
            }
            Self::Entity(EntityMutationIntent::UpdateFields(spec)) => {
                touched.insert(spec.entity_id.partition_id);
            }
            Self::Entity(EntityMutationIntent::ApplyAspectPatch(spec)) => {
                touched.insert(spec.entity_id.partition_id);
            }
            Self::Entity(EntityMutationIntent::Replace(spec)) => {
                touched.insert(spec.entity_id.partition_id);
                touched.insert(spec.replacement.partition_id);
            }
            Self::Entity(EntityMutationIntent::Delete(spec)) => {
                touched.insert(spec.entity_id.partition_id);
            }
            Self::Entity(EntityMutationIntent::Revalidate(spec)) => {
                touched.insert(spec.entity_id.partition_id);
            }
            Self::Create(CreateIntent::Relation(spec)) => {
                touched.insert(spec.partition_id);
                touched.insert(spec.source.partition_id());
                touched.insert(spec.target.partition_id());
            }
            Self::Create(CreateIntent::RelationAspects(spec)) => {
                touched.insert(spec.partition_id);
                touched.insert(spec.source.partition_id());
                touched.insert(spec.target.partition_id());
            }
            Self::Create(CreateIntent::BulkRelations(spec)) => {
                touched.insert(spec.partition_id);
                for (source, target) in &spec.endpoints {
                    touched.insert(source.partition_id());
                    touched.insert(target.partition_id());
                }
            }
            Self::Relation(RelationMutationIntent::UpdateEndpoints(spec)) => {
                touched.insert(spec.relation_id.partition_id);
                touched.insert(spec.source.partition_id());
                touched.insert(spec.target.partition_id());
            }
            Self::Relation(RelationMutationIntent::ApplyAspectPatch(spec)) => {
                touched.insert(spec.relation_id.partition_id);
            }
            Self::Relation(RelationMutationIntent::Delete(spec)) => {
                touched.insert(spec.relation_id.partition_id);
            }
            Self::Materialization(intent) => {
                touched.insert(intent.partition_id());
            }
        }
    }

    pub(crate) fn bulk_entity_reservation(&self) -> Option<(PartitionId, usize)> {
        match self {
            Self::Create(CreateIntent::BulkEntities(spec)) => {
                Some((spec.partition_id, spec.field_patches.len()))
            }
            _ => None,
        }
    }

    pub(crate) fn bulk_relation_reservation(&self) -> Option<(PartitionId, usize)> {
        match self {
            Self::Create(CreateIntent::BulkRelations(spec)) => {
                Some((spec.partition_id, spec.endpoints.len()))
            }
            _ => None,
        }
    }

    /// What discarding this intent undoes.
    ///
    /// Not every intent leaves something to undo: a revalidation demand never
    /// changed the record, so discarding it restores nothing and discards no
    /// creation. Reporting it as a restoration would inflate the rollback
    /// summary with work that never happened, so it yields no effect at all.
    pub(crate) fn rollback_effect(&self) -> Option<RollbackEffect> {
        match self {
            Self::Entity(EntityMutationIntent::Revalidate(_)) => None,
            Self::Create(CreateIntent::Entity(_))
            | Self::Create(CreateIntent::EntityAspects(_))
            | Self::Create(CreateIntent::BulkEntities(_)) => {
                Some(RollbackEffect::DiscardedEntityCreation)
            }
            Self::Entity(EntityMutationIntent::UpdateFields(super::UpdateEntityFieldsIntent {
                entity_id,
                ..
            }))
            | Self::Entity(EntityMutationIntent::ApplyAspectPatch(
                super::ApplyEntityAspectPatchIntent { entity_id, .. },
            ))
            | Self::Entity(EntityMutationIntent::Replace(ReplaceEntityIntent {
                entity_id, ..
            }))
            | Self::Entity(EntityMutationIntent::Delete(DeleteEntityIntent { entity_id })) => {
                Some(RollbackEffect::RestoredEntity(*entity_id))
            }
            Self::Create(CreateIntent::Relation(_))
            | Self::Create(CreateIntent::RelationAspects(_))
            | Self::Create(CreateIntent::BulkRelations(_)) => {
                Some(RollbackEffect::DiscardedRelationCreation)
            }
            Self::Relation(RelationMutationIntent::UpdateEndpoints(spec)) => {
                Some(RollbackEffect::RestoredRelation(spec.relation_id))
            }
            Self::Relation(RelationMutationIntent::ApplyAspectPatch(spec)) => {
                Some(RollbackEffect::RestoredRelation(spec.relation_id))
            }
            Self::Relation(RelationMutationIntent::Delete(spec)) => {
                Some(RollbackEffect::RestoredRelation(spec.relation_id))
            }
            Self::Materialization(intent) => Some(match intent.record() {
                super::RecordRef::Entity(entity_id) => RollbackEffect::RestoredEntity(entity_id),
                super::RecordRef::Relation(relation_id) => {
                    RollbackEffect::RestoredRelation(relation_id)
                }
            }),
        }
    }

    pub(crate) fn existing_record_target(&self) -> Option<ExistingRecordTarget> {
        match self {
            Self::Entity(EntityMutationIntent::UpdateFields(spec)) => {
                Some(ExistingRecordTarget::Entity(spec.entity_id))
            }
            Self::Entity(EntityMutationIntent::ApplyAspectPatch(spec)) => {
                Some(ExistingRecordTarget::Entity(spec.entity_id))
            }
            Self::Entity(EntityMutationIntent::Replace(spec)) => {
                Some(ExistingRecordTarget::Entity(spec.entity_id))
            }
            Self::Entity(EntityMutationIntent::Delete(spec)) => {
                Some(ExistingRecordTarget::Entity(spec.entity_id))
            }
            Self::Entity(EntityMutationIntent::Revalidate(spec)) => {
                Some(ExistingRecordTarget::Entity(spec.entity_id))
            }
            Self::Relation(RelationMutationIntent::UpdateEndpoints(spec)) => {
                Some(ExistingRecordTarget::Relation(spec.relation_id))
            }
            Self::Relation(RelationMutationIntent::ApplyAspectPatch(spec)) => {
                Some(ExistingRecordTarget::Relation(spec.relation_id))
            }
            Self::Relation(RelationMutationIntent::Delete(spec)) => {
                Some(ExistingRecordTarget::Relation(spec.relation_id))
            }
            Self::Materialization(intent) => Some(match intent.record() {
                super::RecordRef::Entity(entity_id) => ExistingRecordTarget::Entity(entity_id),
                super::RecordRef::Relation(relation_id) => {
                    ExistingRecordTarget::Relation(relation_id)
                }
            }),
            Self::Create(_) => None,
        }
    }

    pub(crate) fn collect_relation_identities(&self, identities: &mut Vec<RelationIdentity>) {
        match self {
            Self::Create(CreateIntent::Relation(spec)) => identities.push(RelationIdentity {
                partition_id: spec.partition_id,
                kind_id: spec.kind_id,
                source: spec.source.clone(),
                target: spec.target.clone(),
            }),
            Self::Create(CreateIntent::RelationAspects(spec)) => {
                identities.push(RelationIdentity {
                    partition_id: spec.partition_id,
                    kind_id: spec.kind_id,
                    source: spec.source.clone(),
                    target: spec.target.clone(),
                })
            }
            Self::Create(CreateIntent::BulkRelations(BulkRelationCreateIntent {
                partition_id,
                kind_id,
                endpoints,
                ..
            })) => {
                for (source, target) in endpoints {
                    identities.push(RelationIdentity {
                        partition_id: *partition_id,
                        kind_id: *kind_id,
                        source: source.clone(),
                        target: target.clone(),
                    });
                }
            }
            Self::Relation(RelationMutationIntent::UpdateEndpoints(spec)) => {
                identities.push(RelationIdentity {
                    partition_id: spec.relation_id.partition_id,
                    kind_id: spec.kind_id,
                    source: spec.source.clone(),
                    target: spec.target.clone(),
                });
            }
            _ => {}
        }
    }
}

impl super::MergedCommitPlan {
    pub fn invariant_contract(&self) -> InvariantPlanContract {
        InvariantPlanContract::from_merged_plan(self)
    }

    pub fn inferred_topology(&self, merge_parent_count: usize) -> CommitTopology {
        if merge_parent_count > 0 {
            return CommitTopology::BranchMerge;
        }

        let invalidated_groups = self.invariant_contract().may_invalidate_groups();
        if invalidated_groups == InvariantGroupSet::of(InvariantGroup::StorageCoherence) {
            CommitTopology::FlatEntityBatch
        } else if invalidated_groups.contains(InvariantGroup::AdjacencyIntegrity)
            || invalidated_groups.contains(InvariantGroup::RelationIntegrity)
        {
            CommitTopology::GraphMutation
        } else {
            CommitTopology::FlatEntityBatch
        }
    }
}
