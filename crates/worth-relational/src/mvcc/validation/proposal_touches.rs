//! Semantic mutation touches projected from one owner-validated plan.

use std::collections::BTreeSet;

use worth_foundational::facade::{
    AspectFieldLocator, CanonicalFieldPath, PortableAspectPatchOperation,
};

use super::ValidatedRelationalProposal;
use crate::identity::data::{EntityId, KindId, RelationId};
use crate::storage::overlay::PartitionAccess;
use crate::transactions::data::{
    planned_aspect_field_locator, AspectFieldPatch, CreateIntent, EntityMutationIntent,
    MutationIntent, RelationMutationIntent,
};

/// One semantic locus touched by an invariant-validated Relational mutation.
///
/// The two `Unrepresentable` variants preserve owner truth for mutation forms
/// that cannot honestly be admitted as an exact field or graph-edge touch.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ValidatedMutationTouch {
    CreateEntity {
        kind: KindId,
    },
    DeleteEntity {
        kind: KindId,
    },
    WriteEntityField {
        kind: KindId,
        locator: AspectFieldLocator,
    },
    LinkRelation {
        kind: KindId,
    },
    UnlinkRelation {
        kind: KindId,
    },
    UnrepresentableEntityMutation {
        kind: KindId,
    },
    UnrepresentableRelationMutation {
        kind: KindId,
    },
}

impl ValidatedMutationTouch {
    pub const fn kind(&self) -> KindId {
        match self {
            Self::CreateEntity { kind }
            | Self::DeleteEntity { kind }
            | Self::WriteEntityField { kind, .. }
            | Self::LinkRelation { kind }
            | Self::UnlinkRelation { kind }
            | Self::UnrepresentableEntityMutation { kind }
            | Self::UnrepresentableRelationMutation { kind } => *kind,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ValidatedMutationTouchProjectionWork {
    validated_intents_examined: usize,
    mutation_targets_materialized: usize,
    owner_state_lookups: usize,
}

impl ValidatedMutationTouchProjectionWork {
    pub const fn validated_intents_examined(self) -> usize {
        self.validated_intents_examined
    }

    pub const fn mutation_targets_materialized(self) -> usize {
        self.mutation_targets_materialized
    }

    pub const fn owner_state_lookups(self) -> usize {
        self.owner_state_lookups
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedMutationTouches {
    touches: Vec<ValidatedMutationTouch>,
    work: ValidatedMutationTouchProjectionWork,
}

impl ValidatedMutationTouches {
    pub fn touches(&self) -> &[ValidatedMutationTouch] {
        &self.touches
    }

    pub const fn work(&self) -> ValidatedMutationTouchProjectionWork {
        self.work
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidatedMutationTouchProjectionError {
    MissingEntityKind(EntityId),
    MissingRelationKind(RelationId),
}

impl ValidatedRelationalProposal {
    /// Projects the exact semantic touch set from this owner-validated plan.
    pub fn mutation_touches(
        &self,
    ) -> Result<ValidatedMutationTouches, ValidatedMutationTouchProjectionError> {
        let mut touches = BTreeSet::new();
        let mut work = ValidatedMutationTouchProjectionWork::default();
        for intent in &self.prepared.merged_plan.merged_intents {
            work.validated_intents_examined += 1;
            project_intent(
                intent,
                &self.prepared.working_state,
                &mut touches,
                &mut work,
            )?;
        }
        work.mutation_targets_materialized = touches.len();
        Ok(ValidatedMutationTouches {
            touches: touches.into_iter().collect(),
            work,
        })
    }
}

fn project_intent(
    intent: &MutationIntent,
    state: &crate::storage::overlay::WorkingState,
    touches: &mut BTreeSet<ValidatedMutationTouch>,
    work: &mut ValidatedMutationTouchProjectionWork,
) -> Result<(), ValidatedMutationTouchProjectionError> {
    match intent {
        MutationIntent::Create(intent) => {
            project_create(intent, touches);
            Ok(())
        }
        MutationIntent::Entity(intent) => project_entity(intent, state, touches, work),
        MutationIntent::Relation(intent) => project_relation(intent, state, touches, work),
        MutationIntent::Materialization(intent) => {
            project_materialization(intent, state, touches, work)
        }
    }
}

fn project_materialization(
    intent: &crate::transactions::data::MaterializationMutationIntent,
    state: &crate::storage::overlay::WorkingState,
    touches: &mut BTreeSet<ValidatedMutationTouch>,
    work: &mut ValidatedMutationTouchProjectionWork,
) -> Result<(), ValidatedMutationTouchProjectionError> {
    use crate::transactions::data::MaterializationMutationIntent as Intent;
    match intent {
        Intent::SuspendEntity(spec) => {
            let kind = entity_kind(state, spec.entity_id, work)?;
            touches.insert(ValidatedMutationTouch::UnrepresentableEntityMutation { kind });
        }
        Intent::SuspendRelation(spec) => {
            let kind = relation_kind(state, spec.relation_id, work)?;
            touches.insert(ValidatedMutationTouch::UnlinkRelation { kind });
        }
        Intent::RematerializeEntity(spec) => {
            touches.insert(ValidatedMutationTouch::UnrepresentableEntityMutation {
                kind: spec.kind_id,
            });
            project_fields(spec.kind_id, &spec.fields, touches);
        }
        Intent::RematerializeRelation(spec) => {
            touches.insert(ValidatedMutationTouch::LinkRelation { kind: spec.kind_id });
            project_relation_fields(spec.kind_id, &spec.fields, touches);
        }
    }
    Ok(())
}

fn project_create(intent: &CreateIntent, touches: &mut BTreeSet<ValidatedMutationTouch>) {
    match intent {
        CreateIntent::Entity(spec) => {
            touches.insert(ValidatedMutationTouch::CreateEntity { kind: spec.kind_id });
            project_fields(spec.kind_id, &spec.fields, touches);
        }
        CreateIntent::EntityAspects(spec) => {
            touches.insert(ValidatedMutationTouch::CreateEntity { kind: spec.kind_id });
            project_entity_aspect_patch(spec.kind_id, &spec.aspect_patch, touches);
        }
        CreateIntent::BulkEntities(spec) => {
            touches.insert(ValidatedMutationTouch::CreateEntity { kind: spec.kind_id });
            for fields in &spec.field_patches {
                project_fields(spec.kind_id, fields, touches);
            }
        }
        CreateIntent::Relation(spec) => {
            touches.insert(ValidatedMutationTouch::LinkRelation { kind: spec.kind_id });
            project_relation_fields(spec.kind_id, &spec.fields, touches);
        }
        CreateIntent::RelationAspects(spec) => {
            touches.insert(ValidatedMutationTouch::LinkRelation { kind: spec.kind_id });
            if !spec.aspect_patch.operations().is_empty() {
                touches.insert(ValidatedMutationTouch::UnrepresentableRelationMutation {
                    kind: spec.kind_id,
                });
            }
        }
        CreateIntent::BulkRelations(spec) => {
            touches.insert(ValidatedMutationTouch::LinkRelation { kind: spec.kind_id });
            if spec
                .field_patches
                .iter()
                .any(|fields| fields.locators().next().is_some())
            {
                touches.insert(ValidatedMutationTouch::UnrepresentableRelationMutation {
                    kind: spec.kind_id,
                });
            }
        }
    }
}

fn project_entity(
    intent: &EntityMutationIntent,
    state: &crate::storage::overlay::WorkingState,
    touches: &mut BTreeSet<ValidatedMutationTouch>,
    work: &mut ValidatedMutationTouchProjectionWork,
) -> Result<(), ValidatedMutationTouchProjectionError> {
    let entity = match intent {
        EntityMutationIntent::UpdateFields(spec) => spec.entity_id,
        EntityMutationIntent::ApplyAspectPatch(spec) => spec.entity_id,
        EntityMutationIntent::Replace(spec) => spec.entity_id,
        EntityMutationIntent::Delete(spec) => spec.entity_id,
    };
    let kind = entity_kind(state, entity, work)?;
    match intent {
        EntityMutationIntent::UpdateFields(spec) => project_fields(kind, &spec.fields, touches),
        EntityMutationIntent::ApplyAspectPatch(spec) => {
            project_entity_aspect_patch(kind, &spec.aspect_patch, touches)
        }
        EntityMutationIntent::Replace(_) => {
            touches.insert(ValidatedMutationTouch::UnrepresentableEntityMutation { kind });
        }
        EntityMutationIntent::Delete(_) => {
            touches.insert(ValidatedMutationTouch::DeleteEntity { kind });
        }
    }
    Ok(())
}

fn project_relation(
    intent: &RelationMutationIntent,
    state: &crate::storage::overlay::WorkingState,
    touches: &mut BTreeSet<ValidatedMutationTouch>,
    work: &mut ValidatedMutationTouchProjectionWork,
) -> Result<(), ValidatedMutationTouchProjectionError> {
    match intent {
        RelationMutationIntent::UpdateEndpoints(spec) => {
            touches.insert(ValidatedMutationTouch::UnlinkRelation { kind: spec.kind_id });
            touches.insert(ValidatedMutationTouch::LinkRelation { kind: spec.kind_id });
        }
        RelationMutationIntent::ApplyAspectPatch(spec) => {
            let kind = relation_kind(state, spec.relation_id, work)?;
            touches.insert(ValidatedMutationTouch::UnrepresentableRelationMutation { kind });
        }
        RelationMutationIntent::Delete(spec) => {
            let kind = relation_kind(state, spec.relation_id, work)?;
            touches.insert(ValidatedMutationTouch::UnlinkRelation { kind });
        }
    }
    Ok(())
}

fn project_fields(
    kind: KindId,
    fields: &AspectFieldPatch,
    touches: &mut BTreeSet<ValidatedMutationTouch>,
) {
    touches.extend(
        fields
            .locators()
            .cloned()
            .map(|locator| ValidatedMutationTouch::WriteEntityField { kind, locator }),
    );
}

fn project_entity_aspect_patch(
    kind: KindId,
    patch: &worth_foundational::facade::PortableRecordAspectPatch,
    touches: &mut BTreeSet<ValidatedMutationTouch>,
) {
    for operation in patch.operations() {
        match operation {
            PortableAspectPatchOperation::PatchFields {
                basis,
                selected_fields,
                ..
            } => touches.extend(selected_fields.iter().cloned().map(|field| {
                ValidatedMutationTouch::WriteEntityField {
                    kind,
                    locator: planned_aspect_field_locator(
                        basis.key().clone(),
                        CanonicalFieldPath::single(field),
                    ),
                }
            })),
            PortableAspectPatchOperation::SetWhole { .. }
            | PortableAspectPatchOperation::ClearWhole { .. } => {
                touches.insert(ValidatedMutationTouch::UnrepresentableEntityMutation { kind });
            }
        }
    }
}

fn project_relation_fields(
    kind: KindId,
    fields: &AspectFieldPatch,
    touches: &mut BTreeSet<ValidatedMutationTouch>,
) {
    if fields.locators().next().is_some() {
        touches.insert(ValidatedMutationTouch::UnrepresentableRelationMutation { kind });
    }
}

fn entity_kind(
    state: &crate::storage::overlay::WorkingState,
    entity: EntityId,
    work: &mut ValidatedMutationTouchProjectionWork,
) -> Result<KindId, ValidatedMutationTouchProjectionError> {
    work.owner_state_lookups += 1;
    state
        .get_partition(entity.partition_id)
        .and_then(|partition| partition.entity_arena.get(&entity))
        .and_then(|record| record.kind_id())
        .ok_or(ValidatedMutationTouchProjectionError::MissingEntityKind(
            entity,
        ))
}

fn relation_kind(
    state: &crate::storage::overlay::WorkingState,
    relation: RelationId,
    work: &mut ValidatedMutationTouchProjectionWork,
) -> Result<KindId, ValidatedMutationTouchProjectionError> {
    work.owner_state_lookups += 1;
    state
        .get_partition(relation.partition_id)
        .and_then(|partition| partition.relation_arena.get(&relation))
        .and_then(|record| record.kind_id())
        .ok_or(ValidatedMutationTouchProjectionError::MissingRelationKind(
            relation,
        ))
}

#[cfg(test)]
#[path = "proposal_touches/tests.rs"]
mod tests;
