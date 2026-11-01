//! Owned portable construction follows the published operations without trees.
use super::*;
use crate::publication::patch::data::{
    PublishedAuthoritativePatchOperation as Op, PublishedAuthoritativePatchValue as Value,
};
use std::alloc::Layout;

pub(super) fn quote(
    patch: &PublishedAuthoritativePatch,
    plan: &LoweredAspectContractPlan,
    creation: bool,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<AspectPatchStorageBounds, Stop> {
    let mut quote = AspectPatchStorageBounds {
        retained_bytes: backing::<PortableAspectPatchOperation>(
            patch.full_grammar_operation_count(),
        )?,
        transient_bytes: 0,
    };
    for op in patch.full_grammar_operations() {
        let key = op.aspect_key();
        let work = plan
            .executable_bindings
            .len()
            .checked_mul(
                key.as_str()
                    .len()
                    .checked_add(1)
                    .ok_or_else(super::super::memory_overflow)?,
            )
            .and_then(|n| n.checked_mul(3))
            .ok_or_else(super::super::memory_overflow)?;
        super::super::charge(context, work)?;
        if !plan
            .executable_bindings
            .iter()
            .any(|binding| binding.contract.key() == key)
        {
            return Err("output patch has no installed physical contract".into());
        }
    }
    for op in patch.full_grammar_operations() {
        if metadata_binding(binding(op)) {
            continue;
        }
        add(&mut quote.retained_bytes, op.aspect_key().as_str().len())?;
        match op {
            Op::WholeAspectSet { value, .. } => add(
                &mut quote.retained_bytes,
                match value {
                    Value::Scalar(value) => value.owned_allocation_capacity_bytes(),
                    Value::Struct(value) => value.owned_allocation_capacity_bytes(),
                },
            )?,
            Op::WholeAspectClear { .. } => {}
            Op::FieldLevelPatch {
                field_sets,
                field_clears,
                ..
            } => {
                if creation {
                    add(
                        &mut quote.retained_bytes,
                        backing::<(FieldKey, AspectValue)>(field_sets.len())?,
                    )?;
                    add(
                        &mut quote.transient_bytes,
                        backing::<(FieldKey, AspectValue)>(field_sets.len())?,
                    )?;
                } else {
                    let count = field_sets
                        .len()
                        .checked_add(field_clears.len())
                        .ok_or_else(super::super::memory_overflow)?;
                    add(&mut quote.retained_bytes, backing::<FieldKey>(count)?)?;
                    add(
                        &mut quote.retained_bytes,
                        backing::<PortableAspectFieldSet>(field_sets.len())?,
                    )?;
                    add(
                        &mut quote.retained_bytes,
                        backing::<FieldKey>(field_clears.len())?,
                    )?;
                    for key in field_sets.iter().map(|set| &set.field).chain(field_clears) {
                        add(
                            &mut quote.retained_bytes,
                            key.owned_allocation_capacity_bytes(),
                        )?;
                    }
                    for key in field_clears {
                        add(
                            &mut quote.retained_bytes,
                            key.owned_allocation_capacity_bytes(),
                        )?;
                    }
                }
                for set in field_sets {
                    add(
                        &mut quote.retained_bytes,
                        set.field.owned_allocation_capacity_bytes(),
                    )?;
                    add(
                        &mut quote.retained_bytes,
                        set.value.owned_allocation_capacity_bytes(),
                    )?;
                }
            }
        }
    }
    Ok(quote)
}

pub(super) fn build(
    patch: &PublishedAuthoritativePatch,
    plan: &LoweredAspectContractPlan,
    creation: bool,
) -> Result<PortableRecordAspectPatch, Stop> {
    let mut operations = Vec::with_capacity(patch.full_grammar_operation_count());
    for op in patch.full_grammar_operations() {
        let (identity, revision) = match op {
            Op::WholeAspectSet {
                aspect_identity,
                contract_revision,
                ..
            }
            | Op::WholeAspectClear {
                aspect_identity,
                contract_revision,
                ..
            }
            | Op::FieldLevelPatch {
                aspect_identity,
                contract_revision,
                ..
            } => (*aspect_identity, *contract_revision),
        };
        if !plan.executable_bindings.iter().any(|expected| {
            expected.contract.key() == op.aspect_key()
                && expected.contract.identity() == identity
                && expected.contract.revision() == revision
                && &expected.target == binding(op)
        }) {
            return Err("native output patch differs from installed physical contract".into());
        }
        if metadata_binding(binding(op)) {
            continue;
        }
        let basis = PortableAspectContractBasis::new(op.aspect_key().clone(), identity, revision);
        operations.push(match op {
            Op::WholeAspectSet { value, .. } => PortableAspectPatchOperation::SetWhole {
                basis,
                value: match value {
                    Value::Scalar(value) => ContractValidationInput::Scalar(value.clone()),
                    Value::Struct(value) => ContractValidationInput::Struct(value.clone()),
                },
            },
            Op::WholeAspectClear { .. } => PortableAspectPatchOperation::ClearWhole { basis },
            Op::FieldLevelPatch {
                field_sets,
                field_clears,
                ..
            } if creation => {
                if !field_clears.is_empty() {
                    return Err("native birth cannot clear absent fields".into());
                }
                let mut pairs = Vec::with_capacity(field_sets.len());
                pairs.extend(
                    field_sets
                        .iter()
                        .map(|set| (set.field.clone(), set.value.clone())),
                );
                pairs.sort_unstable_by(|left, right| left.0.cmp(&right.0));
                let value = StructAspectValue::from_sorted_fields(pairs.into_boxed_slice())
                    .map_err(|_| Stop::from("native birth repeats a struct field"))?;
                PortableAspectPatchOperation::SetWhole {
                    basis,
                    value: ContractValidationInput::Struct(value),
                }
            }
            Op::FieldLevelPatch {
                field_sets,
                field_clears,
                ..
            } => {
                let mut selected_fields = Vec::with_capacity(field_sets.len() + field_clears.len());
                selected_fields.extend(
                    field_sets
                        .iter()
                        .map(|set| set.field.clone())
                        .chain(field_clears.iter().cloned()),
                );
                let mut sets = Vec::with_capacity(field_sets.len());
                sets.extend(
                    field_sets.iter().map(|set| {
                        PortableAspectFieldSet::new(set.field.clone(), set.value.clone())
                    }),
                );
                let mut clears = Vec::with_capacity(field_clears.len());
                clears.extend(field_clears.iter().cloned());
                PortableAspectPatchOperation::PatchFields {
                    basis,
                    selected_fields,
                    field_sets: sets,
                    field_clears: clears,
                }
            }
        });
    }
    Ok(PortableRecordAspectPatch::new(operations))
}
fn binding(op: &Op) -> &AspectBinding {
    match op {
        Op::WholeAspectSet { binding, .. }
        | Op::WholeAspectClear { binding, .. }
        | Op::FieldLevelPatch { binding, .. } => binding,
    }
}
fn metadata_binding(binding: &AspectBinding) -> bool {
    matches!(
        binding,
        AspectBinding::LifecycleTransition
            | AspectBinding::StructuralRegion
            | AspectBinding::StructuralPartition
            | AspectBinding::StructuralFacet
    )
}
pub(super) fn backing<T>(count: usize) -> Result<usize, Stop> {
    Ok(Layout::array::<T>(count)
        .map_err(|_| super::super::memory_overflow())?
        .size())
}
pub(super) fn add(total: &mut usize, bytes: usize) -> Result<(), Stop> {
    *total = total
        .checked_add(bytes)
        .ok_or_else(super::super::memory_overflow)?;
    Ok(())
}
