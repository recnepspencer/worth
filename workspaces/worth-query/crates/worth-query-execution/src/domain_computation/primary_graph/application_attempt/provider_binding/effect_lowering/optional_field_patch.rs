use crate::domain_computation::primary_graph::application_checkpoint::facts::FactSequence;
use std::collections::BTreeMap;

use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, ContractValidationInput, FieldKey, PortableAspectContractBasis,
    PortableAspectFieldSet, PortableAspectPatchOperation, PortableRecordAspectPatch,
    StructAspectValue,
};
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::transactions::{
    ApplyEntityAspectPatchIntent, EntityMutationIntent, MutationIntent,
};

use super::{
    effect_step, mutation, ObservedFactIndex, UniqueValueWrites, WorthQueryLoweredProviderEffect,
};
use crate::domain_computation::primary_graph::application_attempt::effect_program::WorthQueryApplicationOptionalFieldWrite;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAttemptDenial;
use crate::domain_computation::WorthQueryProvisionalEffectAction;

struct AspectFieldPatch {
    basis: PortableAspectContractBasis,
    selected_fields: Vec<FieldKey>,
    field_sets: BTreeMap<FieldKey, worth_foundational::facade::AspectValue>,
    field_clears: Vec<FieldKey>,
}

#[cfg(test)]
mod tests;

pub(super) fn lower(
    facts: &ObservedFactIndex<'_, impl FactSequence + ?Sized>,
    unique: &mut UniqueValueWrites<'_>,
    entity_id: EntityId,
    fields: BTreeMap<AspectFieldLocator, WorthQueryApplicationOptionalFieldWrite>,
) -> Result<WorthQueryLoweredProviderEffect, WorthQueryApplicationAttemptDenial> {
    let steps = fields
        .iter()
        .map(|(locator, write)| {
            let identity = facts.field_identity(entity_id, locator)?.into();
            if let Some(value) = &write.value {
                let kind = facts.field_kind(entity_id, locator)?;
                unique.admit(facts, kind, Some(entity_id), locator, value)?;
            }
            let action = if write.value.is_some() {
                WorthQueryProvisionalEffectAction::Replace {
                    target_identity: identity,
                }
            } else {
                WorthQueryProvisionalEffectAction::Retire {
                    target_identity: identity,
                }
            };
            effect_step(action)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let aspect_patch = portable_patch(facts, entity_id, fields)?;
    mutation(
        steps,
        MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(
            ApplyEntityAspectPatchIntent {
                entity_id,
                aspect_patch,
            },
        )),
    )
}

fn portable_patch(
    facts: &ObservedFactIndex<'_, impl FactSequence + ?Sized>,
    entity: EntityId,
    fields: BTreeMap<AspectFieldLocator, WorthQueryApplicationOptionalFieldWrite>,
) -> Result<PortableRecordAspectPatch, WorthQueryApplicationAttemptDenial> {
    let mut aspects = BTreeMap::<AspectKey, AspectFieldPatch>::new();
    for (locator, write) in fields {
        let field = locator
            .field_path()
            .fields()
            .first()
            .expect("installed application fields have one field path")
            .clone();
        let patch = aspects
            .entry(locator.aspect().aspect_key().clone())
            .or_insert_with(|| AspectFieldPatch {
                basis: write.contract.clone(),
                selected_fields: Vec::new(),
                field_sets: BTreeMap::new(),
                field_clears: Vec::new(),
            });
        patch.selected_fields.push(field.clone());
        match write.value {
            Some(value) => {
                patch.field_sets.insert(field, value);
            }
            None => patch.field_clears.push(field),
        }
    }
    let mut operations = Vec::new();
    for (aspect, patch) in aspects {
        if facts.aspect_presence(entity, &aspect)? {
            operations.push(PortableAspectPatchOperation::PatchFields {
                basis: patch.basis,
                selected_fields: patch.selected_fields,
                field_sets: patch
                    .field_sets
                    .into_iter()
                    .map(|(field, value)| PortableAspectFieldSet::new(field, value))
                    .collect(),
                field_clears: patch.field_clears,
            });
        } else if !patch.field_sets.is_empty() {
            let value = StructAspectValue::new(patch.field_sets)
                .map_err(|_| super::super::progression_denial())?;
            operations.push(PortableAspectPatchOperation::SetWhole {
                basis: patch.basis,
                value: ContractValidationInput::Struct(value),
            });
        }
    }
    Ok(PortableRecordAspectPatch::new(operations))
}
