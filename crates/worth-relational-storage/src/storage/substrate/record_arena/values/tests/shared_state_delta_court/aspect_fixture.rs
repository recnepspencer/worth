//! Construct the authoritatively funded aspect states shared by the allocation cases.
use super::*;

pub(super) fn sample_contracts() -> (AspectContract, AspectContract) {
    let field = |name: &str| FieldKey::new(name).unwrap();
    let struct_contract = AspectContract::struct_aspect(
        AspectKey::new("profile").unwrap(),
        AspectIdentity(21),
        AspectContractRevision(1),
        StructAspectShape::new(["title", "notes"].map(|name| {
            FieldDeclaration::new(
                field(name),
                ScalarAspectType::String,
                FieldRequirement::Optional,
                AbsenceLaw::Optional,
                AspectEvolutionPolicy::AdditiveFieldsAllowed,
            )
            .unwrap()
        }))
        .unwrap(),
    );
    let scalar_contract = AspectContract::scalar(
        AspectKey::new("label").unwrap(),
        AspectIdentity(22),
        AspectContractRevision(1),
        ScalarAspectType::String,
    );
    (struct_contract, scalar_contract)
}

pub(super) fn build_state(
    struct_contract: &AspectContract,
    scalar_contract: &AspectContract,
    title: &str,
    stored_notes: StoredValueText,
    label: &str,
) -> AuthoritativeRecordAspectState {
    let field = |name: &str| FieldKey::new(name).unwrap();
    let TransitionOutcome::Success(struct_val) = validate_aspect_value(
        struct_contract,
        ContractValidationInput::Struct(
            StructAspectValue::new([
                (
                    field("title"),
                    AspectValue::String(InternedString::Raw(title.into())),
                ),
                (
                    field("notes"),
                    AspectValue::String(InternedString::Stored(stored_notes)),
                ),
            ])
            .unwrap(),
        ),
    ) else {
        panic!("admitted struct")
    };
    let TransitionOutcome::Success(scalar_val) = validate_aspect_value(
        scalar_contract,
        ContractValidationInput::Scalar(AspectValue::String(InternedString::Raw(label.into()))),
    ) else {
        panic!("admitted scalar")
    };
    let TransitionOutcome::Success(admitted) =
        admit_authoritative_record_aspect_state([struct_val, scalar_val])
    else {
        panic!("admitted state")
    };
    admitted.into_parts().into_parts().0
}
