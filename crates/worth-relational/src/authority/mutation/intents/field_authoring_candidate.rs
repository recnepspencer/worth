use std::collections::BTreeMap;

use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, AspectShape, ContractValidationInput,
    PortableAspectContractBasis, PortableAspectFieldSet, PortableAspectPatchOperation,
    PortablePatchReadmissionPurpose, PortableRecordAspectPatch, StructAspectValue,
};

use crate::schema::data::LoweredAspectContractPlan;
use crate::transactions::data::{
    AspectFieldPatch, AspectFieldTargetRejectionReason, RecordAspectPatchDenial,
};

#[derive(Clone, Copy)]
pub(crate) enum FieldAuthoringDomain {
    Entity,
    Relation,
}

pub(crate) fn lower(
    fields: &AspectFieldPatch,
    purpose: PortablePatchReadmissionPurpose,
    plan: Option<&LoweredAspectContractPlan>,
    kind_id: crate::identity::data::KindId,
    domain: FieldAuthoringDomain,
) -> Result<PortableRecordAspectPatch, RecordAspectPatchDenial> {
    if fields.is_empty() {
        return match purpose {
            PortablePatchReadmissionPurpose::RecordCreation => {
                Ok(PortableRecordAspectPatch::new([]))
            }
            PortablePatchReadmissionPurpose::RecordMutation
            | PortablePatchReadmissionPurpose::RecordDeletion => {
                Err(RecordAspectPatchDenial::EmptyFieldAuthoringPatch)
            }
        };
    }
    let plan = plan.ok_or(RecordAspectPatchDenial::MissingAspectPlan { kind_id })?;
    let mut whole_sets = BTreeMap::new();
    let mut struct_sets = BTreeMap::<AspectKey, StructFields>::new();
    for (target, value) in fields.iter() {
        let field = single_field(target)?;
        let binding = plan
            .executable_bindings
            .iter()
            .find(|binding| binding.aspect_key() == target.aspect().aspect_key())
            .ok_or_else(|| denied(target, AspectFieldTargetRejectionReason::UndeclaredAspect))?;
        let scalar = match domain {
            FieldAuthoringDomain::Entity => binding.targets_entity_scalar_field(field),
            FieldAuthoringDomain::Relation => binding.targets_relation_scalar_field(field),
        };
        let structure = match domain {
            FieldAuthoringDomain::Entity => binding.targets_entity_struct_field(field),
            FieldAuthoringDomain::Relation => binding.targets_relation_struct_field(field),
        };
        match (binding.aspect_shape(), scalar, structure) {
            (AspectShape::Scalar(_), true, _) => {
                whole_sets.insert(
                    binding.aspect_key().clone(),
                    (
                        PortableAspectContractBasis::from_contract(&binding.contract),
                        ContractValidationInput::Scalar(value.clone()),
                    ),
                );
            }
            (AspectShape::Struct(_), _, true) => {
                let entry = struct_sets
                    .entry(binding.aspect_key().clone())
                    .or_insert_with(|| StructFields {
                        basis: PortableAspectContractBasis::from_contract(&binding.contract),
                        fields: BTreeMap::new(),
                    });
                entry.fields.insert(field.clone(), value.clone());
            }
            _ => {
                return Err(denied(
                    target,
                    AspectFieldTargetRejectionReason::FieldPathNotAdmittedByAspectBinding,
                ));
            }
        }
    }
    let mut operations = whole_sets
        .into_values()
        .map(|(basis, value)| PortableAspectPatchOperation::SetWhole { basis, value })
        .collect::<Vec<_>>();
    for (aspect_key, fields) in struct_sets {
        operations.push(struct_operation(aspect_key, fields, purpose)?);
    }
    Ok(PortableRecordAspectPatch::new(operations))
}

struct StructFields {
    basis: PortableAspectContractBasis,
    fields: BTreeMap<worth_foundational::facade::FieldKey, worth_foundational::facade::AspectValue>,
}

fn struct_operation(
    aspect_key: AspectKey,
    fields: StructFields,
    purpose: PortablePatchReadmissionPurpose,
) -> Result<PortableAspectPatchOperation, RecordAspectPatchDenial> {
    if purpose == PortablePatchReadmissionPurpose::RecordCreation {
        let value = StructAspectValue::new(fields.fields)
            .map_err(|_| RecordAspectPatchDenial::StructValueConstructionDenied { aspect_key })?;
        return Ok(PortableAspectPatchOperation::SetWhole {
            basis: fields.basis,
            value: ContractValidationInput::Struct(value),
        });
    }
    let selected_fields = fields.fields.keys().cloned().collect();
    let field_sets = fields
        .fields
        .into_iter()
        .map(|(field, value)| PortableAspectFieldSet::new(field, value))
        .collect();
    Ok(PortableAspectPatchOperation::PatchFields {
        basis: fields.basis,
        selected_fields,
        field_sets,
        field_clears: Vec::new(),
    })
}

fn single_field(
    target: &AspectFieldLocator,
) -> Result<&worth_foundational::facade::FieldKey, RecordAspectPatchDenial> {
    match target.field_path().fields() {
        [field] => Ok(field),
        _ => Err(denied(
            target,
            AspectFieldTargetRejectionReason::NestedFieldPath,
        )),
    }
}

fn denied(
    target: &AspectFieldLocator,
    reason: AspectFieldTargetRejectionReason,
) -> RecordAspectPatchDenial {
    RecordAspectPatchDenial::FieldAuthoringDenied {
        target: target.clone(),
        reason,
    }
}

/// Lower only the submitted fields, using kind metadata from the actual applied
/// candidate. This does not read or reconstruct the entity's aspect state.
pub(crate) fn lower_entity(
    intent: &crate::transactions::data::UpdateEntityFieldsIntent,
    state: &impl crate::storage::overlay::CurrentPartitionAccess,
    plans: &crate::schema::data::AspectContractPlanCatalog,
) -> Result<PortableRecordAspectPatch, crate::transactions::data::CommitConflict> {
    use crate::transactions::data::{
        CommitConflict, ConflictClass, EntityFieldUpdateMissingState, RecordAspectPatchTarget,
    };
    let entity = intent.entity_id;
    super::super::stale_targets::ensure_entity_target_is_current(state, entity)?;
    let missing = |missing| {
        CommitConflict::new(ConflictClass::EntityFieldUpdateStateInconsistency {
            entity_id: entity,
            missing,
        })
    };
    let partition = state
        .get_current_partition(entity.partition_id)
        .ok_or_else(|| missing(EntityFieldUpdateMissingState::Partition))?;
    let slot = partition
        .entity_arena()
        .get_slot(entity.slot_index())
        .ok_or_else(|| missing(EntityFieldUpdateMissingState::Slot))?;
    let kind = slot
        .kind_id()
        .ok_or_else(|| missing(EntityFieldUpdateMissingState::KindId))?;
    lower(
        &intent.fields,
        PortablePatchReadmissionPurpose::RecordMutation,
        plans.entity_plans.get(&kind),
        kind,
        FieldAuthoringDomain::Entity,
    )
    .map_err(|denial| {
        super::record_aspect_patch::conflict(
            RecordAspectPatchTarget::Entity {
                entity_id: entity,
                kind_id: kind,
            },
            denial,
        )
    })
}
