use crate::contracts::schema::data::aspect_contracts::lowered_plan::LoweredAspectContractBinding;
use crate::contracts::storage::data::read_records::{
    EntityReadRecord, RecordLifecycleState, RelationReadRecord,
};
use crate::merge::data::VisibleMergeRecord;
use worth_foundational::facade::AspectBinding;
use worth_foundational::facade::{
    AuthoritativeRecordAspectState, ContractValidatedAspectValueView, StructAspectValue,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum MergeAspectComponent {
    AspectValue(worth_foundational::facade::AspectValue),
    StructValue(StructAspectValue),
    EntityEndpoint(crate::contracts::identity::data::record_ids::EntityId),
    Lifecycle(RecordLifecycleState),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VisibleRecordSide {
    Source,
    Target,
}

pub(crate) fn binding_component_from_visible_record(
    record: &VisibleMergeRecord,
    binding: &LoweredAspectContractBinding,
    side: VisibleRecordSide,
) -> Option<MergeAspectComponent> {
    let entity = match side {
        VisibleRecordSide::Source => record.source_entity.as_ref(),
        VisibleRecordSide::Target => record.target_entity.as_ref(),
    };
    let relation = match side {
        VisibleRecordSide::Source => record.source_relation.as_ref(),
        VisibleRecordSide::Target => record.target_relation.as_ref(),
    };

    match (
        &record.record_kind,
        &binding.target,
        binding.contract.shape(),
    ) {
        (
            crate::merge::data::VisibleMergeRecordKind::Entity,
            AspectBinding::EntityField { .. },
            worth_foundational::AspectShape::Scalar(_),
        ) => entity.and_then(|entity| entity_scalar_aspect_component(entity, binding)),
        (
            crate::merge::data::VisibleMergeRecordKind::Entity,
            AspectBinding::EntityField { .. },
            worth_foundational::AspectShape::Struct(_),
        ) => entity.and_then(|entity| entity_struct_aspect_component(entity, binding)),
        (
            crate::merge::data::VisibleMergeRecordKind::Relation,
            AspectBinding::RelationField { .. },
            worth_foundational::AspectShape::Scalar(_),
        ) => relation.and_then(|relation| relation_scalar_aspect_component(relation, binding)),
        (
            crate::merge::data::VisibleMergeRecordKind::Relation,
            AspectBinding::RelationField { .. },
            worth_foundational::AspectShape::Struct(_),
        ) => relation.and_then(|relation| relation_struct_aspect_component(relation, binding)),
        (
            crate::merge::data::VisibleMergeRecordKind::Relation,
            AspectBinding::RelationSourceEndpoint,
            _,
        ) => relation.map(|relation| MergeAspectComponent::EntityEndpoint(relation.source)),
        (
            crate::merge::data::VisibleMergeRecordKind::Relation,
            AspectBinding::RelationTargetEndpoint,
            _,
        ) => relation.map(|relation| MergeAspectComponent::EntityEndpoint(relation.target)),
        (_, AspectBinding::LifecycleTransition, _) => entity
            .map(|entity| MergeAspectComponent::Lifecycle(entity.lifecycle))
            .or_else(|| {
                relation.map(|relation| MergeAspectComponent::Lifecycle(relation.lifecycle))
            }),
        _ => None,
    }
}

fn entity_scalar_aspect_component(
    entity: &EntityReadRecord,
    binding: &LoweredAspectContractBinding,
) -> Option<MergeAspectComponent> {
    scalar_aspect_component(entity.authoritative_aspect_state.as_ref(), binding)
}

fn entity_struct_aspect_component(
    entity: &EntityReadRecord,
    binding: &LoweredAspectContractBinding,
) -> Option<MergeAspectComponent> {
    struct_aspect_component(entity.authoritative_aspect_state.as_ref(), binding)
}

fn relation_scalar_aspect_component(
    relation: &RelationReadRecord,
    binding: &LoweredAspectContractBinding,
) -> Option<MergeAspectComponent> {
    scalar_aspect_component(relation.authoritative_aspect_state.as_ref(), binding)
}

fn relation_struct_aspect_component(
    relation: &RelationReadRecord,
    binding: &LoweredAspectContractBinding,
) -> Option<MergeAspectComponent> {
    struct_aspect_component(relation.authoritative_aspect_state.as_ref(), binding)
}

fn scalar_aspect_component(
    authoritative_state: Option<&AuthoritativeRecordAspectState>,
    binding: &LoweredAspectContractBinding,
) -> Option<MergeAspectComponent> {
    let entry = authoritative_state?.get(binding.aspect_key())?;
    match entry.view() {
        ContractValidatedAspectValueView::Scalar(value) => {
            Some(MergeAspectComponent::AspectValue(value.clone()))
        }
        ContractValidatedAspectValueView::Struct(_) => None,
    }
}

fn struct_aspect_component(
    authoritative_state: Option<&AuthoritativeRecordAspectState>,
    binding: &LoweredAspectContractBinding,
) -> Option<MergeAspectComponent> {
    let entry = authoritative_state?.get(binding.aspect_key())?;
    match entry.view() {
        ContractValidatedAspectValueView::Scalar(_) => None,
        ContractValidatedAspectValueView::Struct(value) => {
            Some(MergeAspectComponent::StructValue(value.clone()))
        }
    }
}
