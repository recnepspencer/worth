use serde::{Deserialize, Serialize};

use crate::identity::data::{KindId, PartitionId, RelationId};
use crate::symbols::data::ClientKey;
use worth_foundational::facade::PortableRecordAspectPatch;

use super::super::AspectFieldPatch;

use super::super::{EntityReference, EntitySpec, MaterializationMutationIntent, RelationSpec};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BulkEntityCreateIntent {
    pub partition_id: PartitionId,
    pub kind_id: KindId,
    pub client_keys: Vec<ClientKey>,
    pub field_patches: Vec<AspectFieldPatch>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityAspectCreateIntent {
    pub partition_id: PartitionId,
    pub kind_id: KindId,
    pub client_key: ClientKey,
    pub aspect_patch: PortableRecordAspectPatch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApplyEntityAspectPatchIntent {
    pub entity_id: crate::identity::data::EntityId,
    pub aspect_patch: PortableRecordAspectPatch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateEntityFieldsIntent {
    pub entity_id: crate::identity::data::EntityId,
    pub fields: AspectFieldPatch,
}

/// A demand that one unchanged record be judged again under the rules this
/// candidate proposes.
///
/// Rules run only over what a candidate touches, and a candidate that changes
/// the meaning rules are read under changes nothing about the records that
/// meaning now governs. Without a way to say so, such a candidate would publish
/// with the new meaning and the existing records never judged by it. This
/// intent says exactly that and nothing more: the record is not rewritten, its
/// stored state and version are left alone, and it carries no change and no
/// event downstream.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevalidateEntityIntent {
    pub entity_id: crate::identity::data::EntityId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplaceEntityIntent {
    pub entity_id: crate::identity::data::EntityId,
    pub replacement: EntitySpec,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeleteEntityIntent {
    pub entity_id: crate::identity::data::EntityId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BulkRelationCreateIntent {
    pub partition_id: PartitionId,
    pub kind_id: KindId,
    pub client_keys: Vec<ClientKey>,
    pub endpoints: Vec<(EntityReference, EntityReference)>,
    pub field_patches: Vec<AspectFieldPatch>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationAspectCreateIntent {
    pub partition_id: PartitionId,
    pub kind_id: KindId,
    pub client_key: ClientKey,
    pub source: EntityReference,
    pub target: EntityReference,
    pub aspect_patch: PortableRecordAspectPatch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApplyRelationAspectPatchIntent {
    pub relation_id: RelationId,
    pub aspect_patch: PortableRecordAspectPatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeleteRelationIntent {
    pub relation_id: RelationId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateRelationEndpointsIntent {
    pub relation_id: RelationId,
    pub kind_id: KindId,
    pub source: EntityReference,
    pub target: EntityReference,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CreateIntent {
    Entity(EntitySpec),
    EntityAspects(EntityAspectCreateIntent),
    BulkEntities(BulkEntityCreateIntent),
    Relation(RelationSpec),
    RelationAspects(RelationAspectCreateIntent),
    BulkRelations(BulkRelationCreateIntent),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityMutationIntent {
    UpdateFields(UpdateEntityFieldsIntent),
    ApplyAspectPatch(ApplyEntityAspectPatchIntent),
    Replace(ReplaceEntityIntent),
    Delete(DeleteEntityIntent),
    Revalidate(RevalidateEntityIntent),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationMutationIntent {
    UpdateEndpoints(UpdateRelationEndpointsIntent),
    ApplyAspectPatch(ApplyRelationAspectPatchIntent),
    Delete(DeleteRelationIntent),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MutationIntent {
    Create(CreateIntent),
    Entity(EntityMutationIntent),
    Relation(RelationMutationIntent),
    Materialization(MaterializationMutationIntent),
}

impl MutationIntent {
    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        match self {
            Self::Create(intent) => intent.owned_allocation_capacity_bytes(),
            Self::Entity(intent) => intent.owned_allocation_capacity_bytes(),
            Self::Relation(intent) => intent.owned_allocation_capacity_bytes(),
            Self::Materialization(intent) => intent.owned_allocation_capacity_bytes(),
        }
    }
}

impl MaterializationMutationIntent {
    fn owned_allocation_capacity_bytes(&self) -> u64 {
        match self {
            Self::SuspendEntity(_) | Self::SuspendRelation(_) => 0,
            Self::RematerializeEntity(intent) => intent.fields.owned_allocation_capacity_bytes(),
            Self::RematerializeRelation(intent) => intent.fields.owned_allocation_capacity_bytes(),
        }
    }
}

impl CreateIntent {
    fn owned_allocation_capacity_bytes(&self) -> u64 {
        match self {
            Self::Entity(spec) => entity_spec_bytes(spec),
            Self::EntityAspects(intent) => intent
                .client_key
                .owned_allocation_capacity_bytes()
                .saturating_add(intent.aspect_patch.owned_allocation_capacity_bytes() as u64),
            Self::BulkEntities(intent) => {
                client_key_vector_bytes(&intent.client_keys, intent.client_keys.capacity())
                    .saturating_add(field_patch_vector_bytes(
                        &intent.field_patches,
                        intent.field_patches.capacity(),
                    ))
            }
            Self::Relation(spec) => relation_spec_bytes(spec),
            Self::RelationAspects(intent) => intent
                .client_key
                .owned_allocation_capacity_bytes()
                .saturating_add(entity_reference_bytes(&intent.source))
                .saturating_add(entity_reference_bytes(&intent.target))
                .saturating_add(intent.aspect_patch.owned_allocation_capacity_bytes() as u64),
            Self::BulkRelations(intent) => {
                client_key_vector_bytes(&intent.client_keys, intent.client_keys.capacity())
                    .saturating_add(
                        (intent.endpoints.capacity()
                            * std::mem::size_of::<(EntityReference, EntityReference)>())
                            as u64,
                    )
                    .saturating_add(
                        intent
                            .endpoints
                            .iter()
                            .map(|(source, target)| {
                                entity_reference_bytes(source)
                                    .saturating_add(entity_reference_bytes(target))
                            })
                            .sum(),
                    )
                    .saturating_add(field_patch_vector_bytes(
                        &intent.field_patches,
                        intent.field_patches.capacity(),
                    ))
            }
        }
    }
}

impl EntityMutationIntent {
    fn owned_allocation_capacity_bytes(&self) -> u64 {
        match self {
            Self::UpdateFields(intent) => intent.fields.owned_allocation_capacity_bytes(),
            Self::ApplyAspectPatch(intent) => {
                intent.aspect_patch.owned_allocation_capacity_bytes() as u64
            }
            Self::Replace(intent) => entity_spec_bytes(&intent.replacement),
            Self::Delete(_) | Self::Revalidate(_) => 0,
        }
    }
}

impl RelationMutationIntent {
    fn owned_allocation_capacity_bytes(&self) -> u64 {
        match self {
            Self::UpdateEndpoints(intent) => entity_reference_bytes(&intent.source)
                .saturating_add(entity_reference_bytes(&intent.target)),
            Self::ApplyAspectPatch(intent) => {
                intent.aspect_patch.owned_allocation_capacity_bytes() as u64
            }
            Self::Delete(_) => 0,
        }
    }
}

fn entity_spec_bytes(spec: &EntitySpec) -> u64 {
    spec.client_key
        .owned_allocation_capacity_bytes()
        .saturating_add(spec.fields.owned_allocation_capacity_bytes())
}

fn relation_spec_bytes(spec: &RelationSpec) -> u64 {
    spec.client_key
        .owned_allocation_capacity_bytes()
        .saturating_add(entity_reference_bytes(&spec.source))
        .saturating_add(entity_reference_bytes(&spec.target))
        .saturating_add(spec.fields.owned_allocation_capacity_bytes())
}

fn entity_reference_bytes(reference: &EntityReference) -> u64 {
    match reference {
        EntityReference::Existing(_) => 0,
        EntityReference::Created(created) => created.client_key.owned_allocation_capacity_bytes(),
    }
}

fn client_key_vector_bytes(values: &[ClientKey], capacity: usize) -> u64 {
    (capacity * std::mem::size_of::<ClientKey>()) as u64
        + values
            .iter()
            .map(ClientKey::owned_allocation_capacity_bytes)
            .sum::<u64>()
}

fn field_patch_vector_bytes(values: &[AspectFieldPatch], capacity: usize) -> u64 {
    (capacity * std::mem::size_of::<AspectFieldPatch>()) as u64
        + values
            .iter()
            .map(AspectFieldPatch::owned_allocation_capacity_bytes)
            .sum::<u64>()
}
