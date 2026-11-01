use std::collections::{BTreeMap, BTreeSet};

use worth_proof::TransitionOutcome;

use crate::aspects::contracts::{AspectContract, AspectShape};
use crate::aspects::masks::{AspectMask, MutationMask};
use crate::aspects::patches::{
    AuthoritativePatchConstructionDenial, AuthoritativeRecordAspectPatch, FieldLevelAspectPatch,
};
use crate::aspects::structs::{FieldKey, FieldRequirement, StructAspectShape};
use crate::values::AspectValue;

impl AuthoritativeRecordAspectPatch {
    pub fn field_level(
        contract: &AspectContract,
        mask: &AspectMask<MutationMask>,
        field_sets: impl IntoIterator<Item = (FieldKey, AspectValue)>,
        field_clears: impl IntoIterator<Item = FieldKey>,
    ) -> TransitionOutcome<Self, AuthoritativePatchConstructionDenial> {
        match FieldLevelAspectPatch::new(contract, mask, field_sets, field_clears) {
            Ok(field_patch) => {
                let mut patch = Self::empty();
                patch.field_patches = Box::new([field_patch]);
                TransitionOutcome::success(patch)
            }
            Err(denial) => TransitionOutcome::denied(denial),
        }
    }
}

impl FieldLevelAspectPatch {
    fn new(
        contract: &AspectContract,
        mask: &AspectMask<MutationMask>,
        field_sets: impl IntoIterator<Item = (FieldKey, AspectValue)>,
        field_clears: impl IntoIterator<Item = FieldKey>,
    ) -> Result<Self, AuthoritativePatchConstructionDenial> {
        let shape = validate_contract_mask(contract, mask)?;

        let canonical_sets = canonical_field_sets(shape, mask, field_sets)?;
        let canonical_clears = canonical_field_clears(shape, mask, &canonical_sets, field_clears)?;

        if canonical_sets.is_empty() && canonical_clears.is_empty() {
            return Err(AuthoritativePatchConstructionDenial::EmptyFieldPatch);
        }

        Ok(Self {
            contract: contract.clone(),
            mask: mask.clone(),
            field_sets: canonical_sets
                .into_iter()
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            field_clears: canonical_clears
                .into_iter()
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        })
    }
}

fn canonical_field_sets(
    shape: &StructAspectShape,
    mask: &AspectMask<MutationMask>,
    field_sets: impl IntoIterator<Item = (FieldKey, AspectValue)>,
) -> Result<BTreeMap<FieldKey, AspectValue>, AuthoritativePatchConstructionDenial> {
    let mut canonical_sets = BTreeMap::new();
    for (field_key, field_value) in field_sets {
        validate_field_set(shape, mask, &field_key, &field_value)?;
        if canonical_sets
            .insert(field_key.clone(), field_value)
            .is_some()
        {
            return Err(AuthoritativePatchConstructionDenial::DuplicateFieldSet(
                field_key,
            ));
        }
    }
    Ok(canonical_sets)
}

fn canonical_field_clears(
    shape: &StructAspectShape,
    mask: &AspectMask<MutationMask>,
    canonical_sets: &BTreeMap<FieldKey, AspectValue>,
    field_clears: impl IntoIterator<Item = FieldKey>,
) -> Result<BTreeSet<FieldKey>, AuthoritativePatchConstructionDenial> {
    let mut canonical_clears = BTreeSet::new();
    for field_key in field_clears {
        validate_field_clear(shape, mask, &field_key)?;
        if !canonical_sets.contains_key(&field_key) {
            canonical_clears.insert(field_key);
        }
    }
    Ok(canonical_clears)
}

pub(super) fn validate_contract_mask<'a>(
    contract: &'a AspectContract,
    mask: &AspectMask<MutationMask>,
) -> Result<&'a StructAspectShape, AuthoritativePatchConstructionDenial> {
    let AspectShape::Struct(shape) = contract.shape() else {
        return Err(AuthoritativePatchConstructionDenial::FieldPatchRequiresStructAspect);
    };
    contract
        .admits_mutation_mask(mask)
        .map_err(AuthoritativePatchConstructionDenial::MaskNotAdmitted)?;
    if mask.is_whole_aspect() {
        return Err(AuthoritativePatchConstructionDenial::FieldPatchRequiresFieldMask);
    }
    Ok(shape)
}

fn validate_selected(
    mask: &AspectMask<MutationMask>,
    key: &FieldKey,
) -> Result<(), AuthoritativePatchConstructionDenial> {
    if mask
        .paths()
        .binary_search_by(|path| path.fields()[0].cmp(key))
        .is_err()
    {
        return Err(
            AuthoritativePatchConstructionDenial::FieldNotSelectedByMutationMask(key.clone()),
        );
    }
    Ok(())
}

pub(super) fn validate_field_set(
    shape: &StructAspectShape,
    mask: &AspectMask<MutationMask>,
    key: &FieldKey,
    value: &AspectValue,
) -> Result<(), AuthoritativePatchConstructionDenial> {
    validate_selected(mask, key)?;
    let field = shape
        .field(key)
        .ok_or_else(|| AuthoritativePatchConstructionDenial::UnknownField(key.clone()))?;
    let found = value.value_family();
    if found != field.value_type() {
        return Err(AuthoritativePatchConstructionDenial::FieldTypeMismatch {
            field: key.clone(),
            expected: field.value_type(),
            found,
        });
    }
    Ok(())
}

pub(super) fn validate_field_clear(
    shape: &StructAspectShape,
    mask: &AspectMask<MutationMask>,
    key: &FieldKey,
) -> Result<(), AuthoritativePatchConstructionDenial> {
    validate_selected(mask, key)?;
    let field = shape
        .field(key)
        .ok_or_else(|| AuthoritativePatchConstructionDenial::UnknownField(key.clone()))?;
    if matches!(field.requirement(), FieldRequirement::Required) {
        return Err(AuthoritativePatchConstructionDenial::RequiredFieldClearDenied(key.clone()));
    }
    Ok(())
}
