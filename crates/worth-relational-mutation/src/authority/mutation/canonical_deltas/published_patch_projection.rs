use crate::contracts::publication::patch::data::published_authoritative_patch::{
    PublishedAuthoritativeFieldSet, PublishedAuthoritativePatch,
    PublishedAuthoritativePatchOperation, PublishedAuthoritativePatchOwnerPort as _,
    PublishedAuthoritativePatchValue,
};
use worth_foundational::facade::{
    AspectBinding, AspectKey, AuthoritativeRecordAspectPatch, ContractValidatedAspectValue,
    ContractValidatedAspectValueView, FieldLevelAspectPatch,
};

use super::data::AuthoritativePatchDeltaOperation;

pub(super) fn published_patch_from_authoritative_delta_operation(
    operation: &AuthoritativePatchDeltaOperation,
    binding: &AspectBinding,
) -> PublishedAuthoritativePatch {
    PublishedAuthoritativePatch::new(vec![
        published_operation_from_authoritative_delta_operation(operation, binding),
    ])
}

pub(in crate::internal_port::authority::mutation) fn published_patch_from_foundational_patch(
    patch: &AuthoritativeRecordAspectPatch,
    binding_for: impl Fn(&AspectKey) -> Option<AspectBinding>,
) -> Result<PublishedAuthoritativePatch, AspectKey> {
    let operations = patch
        .whole_aspect_sets()
        .map(|(aspect_key, value)| {
            Ok(PublishedAuthoritativePatchOperation::WholeAspectSet {
                aspect_key: aspect_key.clone(),
                aspect_identity: value.contract_identity(),
                contract_revision: value.contract_revision(),
                binding: binding_for(aspect_key).ok_or_else(|| aspect_key.clone())?,
                value: published_patch_value(value),
            })
        })
        .chain(
            patch
                .whole_aspect_clear_contracts()
                .map(|(aspect_key, contract)| {
                    Ok(PublishedAuthoritativePatchOperation::WholeAspectClear {
                        aspect_key: aspect_key.clone(),
                        aspect_identity: contract.identity(),
                        contract_revision: contract.revision(),
                        binding: binding_for(aspect_key).ok_or_else(|| aspect_key.clone())?,
                    })
                }),
        )
        .chain(patch.field_patches().map(|(aspect_key, field_patch)| {
            Ok(published_field_patch_operation(
                field_patch,
                binding_for(aspect_key).ok_or_else(|| aspect_key.clone())?,
            ))
        }))
        .collect::<Result<Vec<_>, AspectKey>>()?;
    Ok(PublishedAuthoritativePatch::new(operations))
}

fn published_operation_from_authoritative_delta_operation(
    operation: &AuthoritativePatchDeltaOperation,
    binding: &AspectBinding,
) -> PublishedAuthoritativePatchOperation {
    match operation {
        AuthoritativePatchDeltaOperation::WholeAspectSet { value } => {
            PublishedAuthoritativePatchOperation::WholeAspectSet {
                aspect_key: value.key().clone(),
                aspect_identity: value.contract_identity(),
                contract_revision: value.contract_revision(),
                binding: binding.clone(),
                value: published_patch_value(value),
            }
        }
        AuthoritativePatchDeltaOperation::WholeAspectClear { contract } => {
            PublishedAuthoritativePatchOperation::WholeAspectClear {
                aspect_key: contract.key().clone(),
                aspect_identity: contract.identity(),
                contract_revision: contract.revision(),
                binding: binding.clone(),
            }
        }
        AuthoritativePatchDeltaOperation::FieldLevelPatch { patch } => {
            published_field_patch_operation(patch, binding.clone())
        }
    }
}

fn published_field_patch_operation(
    patch: &FieldLevelAspectPatch,
    binding: AspectBinding,
) -> PublishedAuthoritativePatchOperation {
    PublishedAuthoritativePatchOperation::FieldLevelPatch {
        aspect_key: patch.key().clone(),
        aspect_identity: patch.contract().identity(),
        contract_revision: patch.contract().revision(),
        binding,
        field_sets: patch
            .field_sets()
            .map(|(field, value)| PublishedAuthoritativeFieldSet {
                field: field.clone(),
                value: value.clone(),
            })
            .collect(),
        field_clears: patch.field_clears().cloned().collect(),
    }
}

fn published_patch_value(value: &ContractValidatedAspectValue) -> PublishedAuthoritativePatchValue {
    match value.view() {
        ContractValidatedAspectValueView::Scalar(value) => {
            PublishedAuthoritativePatchValue::Scalar(value.clone())
        }
        ContractValidatedAspectValueView::Struct(value) => {
            PublishedAuthoritativePatchValue::Struct(value.clone())
        }
    }
}
