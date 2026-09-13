use crate::identity::data::{EntityId, RelationId};
use crate::transactions::data::{
    CreateIntent, EntityMutationIntent, EntityReference, MutationIntent, RelationMutationIntent,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum CanonicalIntentKey {
    CreateEntity {
        partition_id: crate::identity::data::PartitionId,
        kind_id: crate::identity::data::KindId,
        client_key: crate::symbols::data::ClientKey,
    },
    BulkCreateEntities {
        partition_id: crate::identity::data::PartitionId,
        kind_id: crate::identity::data::KindId,
        client_keys: Vec<crate::symbols::data::ClientKey>,
    },
    UpdateEntityFields(EntityId),
    ReplaceEntity {
        entity_id: EntityId,
        replacement_partition_id: crate::identity::data::PartitionId,
        replacement_kind_id: crate::identity::data::KindId,
        replacement_client_key: crate::symbols::data::ClientKey,
    },
    DeleteRelation(RelationId),
    CreateRelation(RelationCreateKey),
    BulkCreateRelations {
        partition_id: crate::identity::data::PartitionId,
        kind_id: crate::identity::data::KindId,
        client_keys: Vec<crate::symbols::data::ClientKey>,
        endpoints: Vec<(EntityReference, EntityReference)>,
    },
    UpdateRelationEndpoints(RelationId),
    DeleteEntity(EntityId),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct RelationCreateKey {
    pub(crate) partition_id: crate::identity::data::PartitionId,
    pub(crate) kind_id: crate::identity::data::KindId,
    pub(crate) source: EntityReference,
    pub(crate) target: EntityReference,
    pub(crate) client_key: crate::symbols::data::ClientKey,
}

pub(crate) fn canonical_intent_key(intent: &MutationIntent) -> CanonicalIntentKey {
    match intent {
        MutationIntent::Create(CreateIntent::Entity(spec)) => CanonicalIntentKey::CreateEntity {
            partition_id: spec.partition_id,
            kind_id: spec.kind_id,
            client_key: spec.client_key.clone(),
        },
        MutationIntent::Create(CreateIntent::EntityAspects(spec)) => {
            CanonicalIntentKey::CreateEntity {
                partition_id: spec.partition_id,
                kind_id: spec.kind_id,
                client_key: spec.client_key.clone(),
            }
        }
        MutationIntent::Create(CreateIntent::BulkEntities(spec)) => {
            CanonicalIntentKey::BulkCreateEntities {
                partition_id: spec.partition_id,
                kind_id: spec.kind_id,
                client_keys: spec.client_keys.clone(),
            }
        }
        MutationIntent::Entity(EntityMutationIntent::UpdateFields(spec)) => {
            CanonicalIntentKey::UpdateEntityFields(spec.entity_id)
        }
        MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(spec)) => {
            CanonicalIntentKey::UpdateEntityFields(spec.entity_id)
        }
        MutationIntent::Entity(EntityMutationIntent::Replace(spec)) => {
            CanonicalIntentKey::ReplaceEntity {
                entity_id: spec.entity_id,
                replacement_partition_id: spec.replacement.partition_id,
                replacement_kind_id: spec.replacement.kind_id,
                replacement_client_key: spec.replacement.client_key.clone(),
            }
        }
        MutationIntent::Entity(EntityMutationIntent::Delete(spec)) => {
            CanonicalIntentKey::DeleteEntity(spec.entity_id)
        }
        MutationIntent::Create(CreateIntent::Relation(spec)) => {
            CanonicalIntentKey::CreateRelation(RelationCreateKey {
                partition_id: spec.partition_id,
                kind_id: spec.kind_id,
                source: spec.source.clone(),
                target: spec.target.clone(),
                client_key: spec.client_key.clone(),
            })
        }
        MutationIntent::Create(CreateIntent::RelationAspects(spec)) => {
            CanonicalIntentKey::CreateRelation(RelationCreateKey {
                partition_id: spec.partition_id,
                kind_id: spec.kind_id,
                source: spec.source.clone(),
                target: spec.target.clone(),
                client_key: spec.client_key.clone(),
            })
        }
        MutationIntent::Create(CreateIntent::BulkRelations(spec)) => {
            CanonicalIntentKey::BulkCreateRelations {
                partition_id: spec.partition_id,
                kind_id: spec.kind_id,
                client_keys: spec.client_keys.clone(),
                endpoints: spec.endpoints.clone(),
            }
        }
        MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(spec)) => {
            CanonicalIntentKey::UpdateRelationEndpoints(spec.relation_id)
        }
        MutationIntent::Relation(RelationMutationIntent::ApplyAspectPatch(spec)) => {
            CanonicalIntentKey::UpdateRelationEndpoints(spec.relation_id)
        }
        MutationIntent::Relation(RelationMutationIntent::Delete(spec)) => {
            CanonicalIntentKey::DeleteRelation(spec.relation_id)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::canonical_intent_key;
    use crate::identity::data::{EntityId, KindId, PartitionId};
    use crate::symbols::data::ClientKey;
    use crate::transactions::data::AspectFieldPatch;
    use crate::transactions::data::{
        BulkRelationCreateIntent, CreateIntent, EntityReference, MutationIntent,
    };

    #[test]
    fn bulk_relation_canonical_keys_preserve_client_key_identity() {
        let source = EntityReference::Existing(EntityId::new(PartitionId::main(), 1, 1));
        let target = EntityReference::Existing(EntityId::new(PartitionId::main(), 2, 1));

        let alpha = MutationIntent::Create(CreateIntent::BulkRelations(BulkRelationCreateIntent {
            partition_id: PartitionId::main(),
            kind_id: KindId(2),
            client_keys: vec![ClientKey::raw("edge-alpha")],
            endpoints: vec![(source.clone(), target.clone())],
            field_patches: vec![AspectFieldPatch::default()],
        }));
        let beta = MutationIntent::Create(CreateIntent::BulkRelations(BulkRelationCreateIntent {
            partition_id: PartitionId::main(),
            kind_id: KindId(2),
            client_keys: vec![ClientKey::raw("edge-beta")],
            endpoints: vec![(source, target)],
            field_patches: vec![AspectFieldPatch::default()],
        }));

        assert_ne!(canonical_intent_key(&alpha), canonical_intent_key(&beta));
    }
}
