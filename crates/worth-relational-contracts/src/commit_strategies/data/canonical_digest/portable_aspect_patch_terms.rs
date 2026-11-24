use super::StrategyDigestBytes;
use worth_foundational::facade::{
    ContractValidationInput, PortableAspectContractBasis, PortableAspectPatchOperation,
    PortableRecordAspectPatch,
};

pub(super) fn write_portable_aspect_patch(
    bytes: &mut StrategyDigestBytes,
    patch: &PortableRecordAspectPatch,
) {
    bytes.usize(patch.operations().len());
    for operation in patch.operations() {
        write_operation(bytes, operation);
    }
}

fn write_operation(bytes: &mut StrategyDigestBytes, operation: &PortableAspectPatchOperation) {
    match operation {
        PortableAspectPatchOperation::SetWhole { basis, value } => {
            bytes.tag(1);
            write_basis(bytes, basis);
            write_validation_input(bytes, value);
        }
        PortableAspectPatchOperation::ClearWhole { basis } => {
            bytes.tag(2);
            write_basis(bytes, basis);
        }
        PortableAspectPatchOperation::PatchFields {
            basis,
            selected_fields,
            field_sets,
            field_clears,
        } => {
            bytes.tag(3);
            write_basis(bytes, basis);
            write_sorted_fields(bytes, selected_fields);
            let mut field_sets = field_sets.iter().collect::<Vec<_>>();
            field_sets.sort_by(|left, right| left.field().cmp(right.field()));
            bytes.usize(field_sets.len());
            for field_set in field_sets {
                bytes.string(field_set.field().as_str());
                write_aspect_value(bytes, field_set.value());
            }
            write_sorted_fields(bytes, field_clears);
        }
    }
}

fn write_basis(bytes: &mut StrategyDigestBytes, basis: &PortableAspectContractBasis) {
    bytes.string(basis.key().as_str());
    bytes.u64(basis.identity().0);
    bytes.u64(basis.revision().0);
}

fn write_validation_input(bytes: &mut StrategyDigestBytes, value: &ContractValidationInput) {
    match value {
        ContractValidationInput::Scalar(value) => {
            bytes.tag(1);
            write_aspect_value(bytes, value);
        }
        ContractValidationInput::Struct(value) => {
            bytes.tag(2);
            bytes.usize(value.fields().count());
            for (field, value) in value.fields() {
                bytes.string(field.as_str());
                write_aspect_value(bytes, value);
            }
        }
    }
}

fn write_sorted_fields(
    bytes: &mut StrategyDigestBytes,
    fields: &[worth_foundational::facade::FieldKey],
) {
    let mut fields = fields.iter().collect::<Vec<_>>();
    fields.sort();
    bytes.usize(fields.len());
    for field in fields {
        bytes.string(field.as_str());
    }
}

fn write_aspect_value(
    bytes: &mut StrategyDigestBytes,
    value: &worth_foundational::facade::AspectValue,
) {
    bytes.bytes(&crate::aspect_wire::encode_aspect_value(value));
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_foundational::facade::{
        AspectContractRevision, AspectIdentity, AspectKey, AspectValue, FieldKey,
        PortableAspectFieldSet,
    };

    #[test]
    fn operation_kind_and_target_granularity_are_digest_visible() {
        let basis = basis(1, 1);
        let set = patch(PortableAspectPatchOperation::SetWhole {
            basis: basis.clone(),
            value: ContractValidationInput::Scalar(text("value")),
        });
        let clear = patch(PortableAspectPatchOperation::ClearWhole {
            basis: basis.clone(),
        });
        let field = field("value");
        let fields = patch(PortableAspectPatchOperation::PatchFields {
            basis,
            selected_fields: vec![field.clone()],
            field_sets: vec![PortableAspectFieldSet::new(field, text("value"))],
            field_clears: Vec::new(),
        });

        assert_ne!(digest(&set), digest(&clear));
        assert_ne!(digest(&set), digest(&fields));
        assert_ne!(digest(&clear), digest(&fields));
    }

    #[test]
    fn contract_identity_revision_and_selected_mask_are_digest_visible() {
        let operation = |basis, selected_fields| PortableAspectPatchOperation::PatchFields {
            basis,
            selected_fields,
            field_sets: Vec::new(),
            field_clears: Vec::new(),
        };
        let base = patch(operation(basis(1, 1), vec![field("title")]));
        let identity = patch(operation(basis(2, 1), vec![field("title")]));
        let revision = patch(operation(basis(1, 2), vec![field("title")]));
        let mask = patch(operation(basis(1, 1), vec![field("status")]));

        assert_ne!(digest(&base), digest(&identity));
        assert_ne!(digest(&base), digest(&revision));
        assert_ne!(digest(&base), digest(&mask));
    }

    #[test]
    fn semantically_unordered_field_collections_have_one_digest() {
        let left = patch(PortableAspectPatchOperation::PatchFields {
            basis: basis(1, 1),
            selected_fields: vec![field("title"), field("status")],
            field_sets: vec![
                PortableAspectFieldSet::new(field("title"), text("T")),
                PortableAspectFieldSet::new(field("status"), text("S")),
            ],
            field_clears: Vec::new(),
        });
        let right = patch(PortableAspectPatchOperation::PatchFields {
            basis: basis(1, 1),
            selected_fields: vec![field("status"), field("title")],
            field_sets: vec![
                PortableAspectFieldSet::new(field("status"), text("S")),
                PortableAspectFieldSet::new(field("title"), text("T")),
            ],
            field_clears: Vec::new(),
        });

        assert_eq!(digest(&left), digest(&right));
    }

    fn digest(patch: &PortableRecordAspectPatch) -> [u8; 32] {
        StrategyDigestBytes::digest("portable-aspect-patch-test-v1", |bytes| {
            write_portable_aspect_patch(bytes, patch)
        })
    }

    fn patch(operation: PortableAspectPatchOperation) -> PortableRecordAspectPatch {
        PortableRecordAspectPatch::new([operation])
    }

    fn basis(identity: u64, revision: u64) -> PortableAspectContractBasis {
        PortableAspectContractBasis::new(
            AspectKey::new("summary").unwrap(),
            AspectIdentity(identity),
            AspectContractRevision(revision),
        )
    }

    fn field(value: &str) -> FieldKey {
        FieldKey::new(value).unwrap()
    }

    fn text(value: &str) -> AspectValue {
        AspectValue::String(value.into())
    }
}
