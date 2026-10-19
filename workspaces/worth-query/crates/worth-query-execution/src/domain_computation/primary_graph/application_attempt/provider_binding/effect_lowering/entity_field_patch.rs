//! Lower writes whose sealed observations include whole-aspect absence.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::effect_program::WorthQueryApplicationOptionalFieldWrite;
use worth_foundational::facade::{
    AbsenceLaw, AspectContract, AspectKey, AspectMask, AspectShape, AuthoritativeRecordAspectPatch,
    CanonicalFieldPath, ContractValidationInput, FieldKey, MutationMask,
    PortableAspectContractBasis, PortableAspectFieldSet, PortableAspectPatchOperation,
    PortableRecordAspectPatch, StructAspectValue,
};
use worth_relational::facade::transactions::ApplyEntityAspectPatchIntent;

pub(super) fn ordinary(
    facts: &ObservedFactIndex<'_, impl FactSequence + ?Sized>,
    unique: &mut UniqueValueWrites<'_>,
    contracts: &AspectContracts<'_>,
    entity: &str,
    id: EntityId,
    fields: BTreeMap<AspectFieldLocator, AspectValue>,
) -> Result<WorthQueryLoweredProviderEffect, WorthQueryApplicationAttemptDenial> {
    lower(
        facts,
        unique,
        contracts,
        entity,
        id,
        fields
            .into_iter()
            .map(|(locator, value)| (locator, None, Some(value))),
    )
}

pub(super) fn optional(
    facts: &ObservedFactIndex<'_, impl FactSequence + ?Sized>,
    unique: &mut UniqueValueWrites<'_>,
    contracts: &AspectContracts<'_>,
    entity: &str,
    id: EntityId,
    fields: BTreeMap<AspectFieldLocator, WorthQueryApplicationOptionalFieldWrite>,
) -> Result<WorthQueryLoweredProviderEffect, WorthQueryApplicationAttemptDenial> {
    lower(
        facts,
        unique,
        contracts,
        entity,
        id,
        fields
            .into_iter()
            .map(|(locator, write)| (locator, Some(write.contract), write.value)),
    )
}

struct Group<'a> {
    contract: &'a AspectContract,
    absent: bool,
    selected: Vec<FieldKey>,
    sets: BTreeMap<FieldKey, AspectValue>,
    clears: Vec<FieldKey>,
}

fn lower<'a>(
    facts: &ObservedFactIndex<'_, impl FactSequence + ?Sized>,
    unique: &mut UniqueValueWrites<'_>,
    contracts: &AspectContracts<'a>,
    entity: &str,
    id: EntityId,
    fields: impl Iterator<
        Item = (
            AspectFieldLocator,
            Option<PortableAspectContractBasis>,
            Option<AspectValue>,
        ),
    >,
) -> Result<WorthQueryLoweredProviderEffect, WorthQueryApplicationAttemptDenial> {
    let mut groups = BTreeMap::<AspectKey, Group<'a>>::new();
    let mut steps = Vec::new();
    for (locator, authored, value) in fields {
        let target = facts.field_identity(id, &locator)?;
        let kind = facts.field_kind(id, &locator)?;
        let absent = facts.aspect_absent(id, &locator)?;
        let aspect = locator.aspect().aspect_key();
        let contract = contracts(entity, aspect).ok_or_else(progression_denial)?;
        let basis = PortableAspectContractBasis::from_contract(contract);
        if authored.as_ref().is_some_and(|authored| authored != &basis) {
            return Err(progression_denial());
        }
        let [field] = locator.field_path().fields() else {
            return Err(progression_denial());
        };
        if let Some(value) = &value {
            unique.admit(facts, kind, Some(id), &locator, value)?;
        }
        steps.push(effect_step(if value.is_some() {
            WorthQueryProvisionalEffectAction::Replace {
                target_identity: target.into(),
            }
        } else {
            WorthQueryProvisionalEffectAction::Retire {
                target_identity: target.into(),
            }
        })?);
        let group = groups.entry(aspect.clone()).or_insert_with(|| Group {
            contract,
            absent,
            selected: Vec::new(),
            sets: BTreeMap::new(),
            clears: Vec::new(),
        });
        if group.absent != absent {
            return Err(progression_denial());
        }
        group.selected.push(field.clone());
        if let Some(value) = value {
            group.sets.insert(field.clone(), value);
        } else {
            group.clears.push(field.clone());
        }
    }
    let mut operations = Vec::new();
    for group in groups.into_values() {
        let basis = PortableAspectContractBasis::from_contract(group.contract);
        if group.absent {
            if group.contract.absence() != AbsenceLaw::Optional
                || !matches!(group.contract.shape(), AspectShape::Struct(_))
            {
                return Err(progression_denial());
            }
            let mask = AspectMask::<MutationMask>::new(
                group
                    .selected
                    .iter()
                    .cloned()
                    .map(CanonicalFieldPath::single),
            );
            group
                .contract
                .admits_mutation_mask(&mask)
                .map_err(|_| progression_denial())?;
            if !group.clears.is_empty()
                && !matches!(
                    AuthoritativeRecordAspectPatch::field_level(
                        group.contract,
                        &mask,
                        std::iter::empty(),
                        group.clears
                    ),
                    worth_proof::TransitionOutcome::Success(_)
                )
            {
                return Err(progression_denial());
            }
            // Clears cannot manufacture a parent or erase another field.
            // The original sealed aspect-absence fact still guards commit.
            if group.sets.is_empty() {
                continue;
            }
            let value = StructAspectValue::new(group.sets).map_err(|_| progression_denial())?;
            let input = ContractValidationInput::Struct(value);
            // The foundational owner enforces every Required field and codec.
            if !matches!(
                worth_foundational::facade::validate_aspect_value(group.contract, input.clone()),
                worth_proof::TransitionOutcome::Success(_)
            ) {
                return Err(progression_denial());
            }
            operations.push(PortableAspectPatchOperation::SetWhole {
                basis,
                value: input,
            });
        } else {
            operations.push(PortableAspectPatchOperation::PatchFields {
                basis,
                selected_fields: group.selected,
                field_sets: group
                    .sets
                    .into_iter()
                    .map(|(k, v)| PortableAspectFieldSet::new(k, v))
                    .collect(),
                field_clears: group.clears,
            });
        }
    }
    if operations.is_empty() {
        return Ok(WorthQueryLoweredProviderEffect::ObservedNoop { steps });
    }
    mutation(
        steps,
        MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(
            ApplyEntityAspectPatchIntent {
                entity_id: id,
                aspect_patch: PortableRecordAspectPatch::new(operations),
            },
        )),
    )
}
