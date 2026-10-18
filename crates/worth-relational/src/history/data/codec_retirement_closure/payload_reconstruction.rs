//! Cold reconstruction uses Foundational's actual admitted patch application.
use super::{charge, reserve, CodecRetirementClosureStop as Stop};
use crate::publication::patch::data::{
    PublishedAuthoritativePatch as Patch, PublishedAuthoritativePatchOperation as Op,
    PublishedAuthoritativePatchValue as Value,
};
use crate::schema::data::LoweredAspectContractPlan;
use worth_execution::{ExecutionResourceLease, MapKernelContext};
use worth_foundational::facade::*;

pub(super) struct ReconstructedPayload {
    pub(super) state: Option<AuthoritativeRecordAspectState>,
    _funding: worth_execution::ExecutionMemoryReservation,
}

pub(super) fn apply(
    current: Option<&AuthoritativeRecordAspectState>,
    patch: &Patch,
    plan: &LoweredAspectContractPlan,
    creation: bool,
    lease: Option<&ExecutionResourceLease<'_>>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<ReconstructedPayload, Stop> {
    let prior_bytes = current
        .as_ref()
        .map_or(0, |state| state.owned_allocation_capacity_bytes());
    // Portable candidate, admitted patch, contract-validation scratch, result
    // map and field-application scratch may overlap. Eight complete copies of
    // reached payload/contract material cover those views and BTree node slack;
    // the old result keeps its independent ticket until this call returns.
    let mut contracts = 0usize;
    for operation in patch.full_grammar_operations() {
        let key = operation.aspect_key();
        charge(
            context,
            plan.executable_bindings
                .len()
                .saturating_mul(key.as_str().len().saturating_add(1))
                .saturating_mul(3),
        )?;
        let contract = plan
            .executable_bindings
            .iter()
            .find(|binding| binding.contract.key() == key)
            .ok_or("output patch has no installed physical contract")?;
        contracts = contracts
            .checked_add(std::mem::size_of::<AspectContract>())
            .and_then(|bytes| {
                bytes.checked_add(contract.contract.owned_allocation_capacity_bytes())
            })
            .ok_or_else(super::memory_overflow)?;
    }
    let bytes = (patch.owned_allocation_capacity_bytes() as usize)
        .checked_add(prior_bytes)
        .and_then(|n| n.checked_add(contracts))
        .and_then(|n| {
            n.checked_add(
                patch
                    .full_grammar_operation_count()
                    .checked_mul(std::mem::size_of::<PortableAspectPatchOperation>() * 4)?,
            )
        })
        .and_then(|n| n.checked_mul(8))
        .ok_or_else(super::memory_overflow)?;
    charge(context, bytes)?;
    let funding = reserve(lease, bytes)?;
    let mut operations = Vec::with_capacity(patch.full_grammar_operation_count());
    for operation in patch.full_grammar_operations() {
        let (key, identity, revision, binding) = match operation {
            Op::WholeAspectSet {
                aspect_key,
                aspect_identity,
                contract_revision,
                binding,
                ..
            }
            | Op::WholeAspectClear {
                aspect_key,
                aspect_identity,
                contract_revision,
                binding,
            }
            | Op::FieldLevelPatch {
                aspect_key,
                aspect_identity,
                contract_revision,
                binding,
                ..
            } => (aspect_key, *aspect_identity, *contract_revision, binding),
        };
        if !plan.executable_bindings.iter().any(|expected| {
            expected.contract.key() == key
                && expected.contract.identity() == identity
                && expected.contract.revision() == revision
                && &expected.target == binding
        }) {
            return Err("native output patch differs from installed physical contract".into());
        }
        // Lifecycle/structural operations describe native metadata transitions;
        // only field and endpoint bindings belong to this stored payload image.
        if matches!(
            binding,
            AspectBinding::LifecycleTransition
                | AspectBinding::StructuralRegion
                | AspectBinding::StructuralPartition
                | AspectBinding::StructuralFacet
        ) {
            continue;
        }
        let basis = PortableAspectContractBasis::new(key.clone(), identity, revision);
        operations.push(match operation {
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
                let value = StructAspectValue::new(
                    field_sets
                        .iter()
                        .map(|set| (set.field.clone(), set.value.clone())),
                )
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
            } => PortableAspectPatchOperation::PatchFields {
                basis,
                selected_fields: field_sets
                    .iter()
                    .map(|set| set.field.clone())
                    .chain(field_clears.iter().cloned())
                    .collect(),
                field_sets: field_sets
                    .iter()
                    .map(|set| PortableAspectFieldSet::new(set.field.clone(), set.value.clone()))
                    .collect(),
                field_clears: field_clears.clone(),
            },
        });
    }
    let patch = match readmit_portable_record_aspect_patch(
        PortableRecordAspectPatch::new(operations),
        if creation {
            PortablePatchReadmissionPurpose::RecordCreation
        } else {
            PortablePatchReadmissionPurpose::RecordMutation
        },
        &|key: &AspectKey| plan.contract_for(key),
    ) {
        worth_proof::TransitionOutcome::Success(patch) => patch,
        worth_proof::TransitionOutcome::Denied(_) => {
            return Err("native output patch readmission refused".into())
        }
    };
    match patch.apply_to_optional(current) {
        worth_proof::TransitionOutcome::Success(state) => Ok(ReconstructedPayload {
            state: state.map(|artifact| artifact.into_parts().into_parts().0),
            _funding: funding,
        }),
        worth_proof::TransitionOutcome::Denied(_) => {
            Err("native output patch application refused".into())
        }
    }
}
