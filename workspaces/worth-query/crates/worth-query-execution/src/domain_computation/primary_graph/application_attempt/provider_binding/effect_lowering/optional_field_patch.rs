use std::collections::BTreeMap;

use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, FieldKey, PortableAspectContractBasis, PortableAspectFieldSet,
    PortableAspectPatchOperation, PortableRecordAspectPatch,
};
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::transactions::{
    ApplyEntityAspectPatchIntent, EntityMutationIntent, MutationIntent,
};

use super::{effect_step, mutation, ObservedFactIndex, WorthQueryLoweredProviderEffect};
use crate::domain_computation::primary_graph::application_attempt::effect_program::WorthQueryApplicationOptionalFieldWrite;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAttemptDenial;
use crate::domain_computation::WorthQueryProvisionalEffectAction;

struct AspectFieldPatch {
    basis: PortableAspectContractBasis,
    selected_fields: Vec<FieldKey>,
    field_sets: Vec<PortableAspectFieldSet>,
    field_clears: Vec<FieldKey>,
}

pub(super) fn lower(
    facts: &ObservedFactIndex<'_>,
    entity_id: EntityId,
    fields: BTreeMap<AspectFieldLocator, WorthQueryApplicationOptionalFieldWrite>,
) -> Result<WorthQueryLoweredProviderEffect, WorthQueryApplicationAttemptDenial> {
    let steps = fields
        .iter()
        .map(|(locator, write)| {
            let identity = facts.field_identity(entity_id, locator)?.into();
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
    let aspect_patch = portable_patch(fields);
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
    fields: BTreeMap<AspectFieldLocator, WorthQueryApplicationOptionalFieldWrite>,
) -> PortableRecordAspectPatch {
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
                field_sets: Vec::new(),
                field_clears: Vec::new(),
            });
        patch.selected_fields.push(field.clone());
        match write.value {
            Some(value) => patch
                .field_sets
                .push(PortableAspectFieldSet::new(field, value)),
            None => patch.field_clears.push(field),
        }
    }
    PortableRecordAspectPatch::new(aspects.into_values().map(|patch| {
        PortableAspectPatchOperation::PatchFields {
            basis: patch.basis,
            selected_fields: patch.selected_fields,
            field_sets: patch.field_sets,
            field_clears: patch.field_clears,
        }
    }))
}
