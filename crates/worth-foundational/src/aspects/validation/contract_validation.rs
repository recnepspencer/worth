use worth_proof::{Artifact, TransitionOutcome};

use super::{
    ContractValidatedAspectArtifact, ContractValidatedAspectValue, ContractValidationDenial,
    ContractValidationInput,
};
use crate::aspects::contracts::{AspectContract, AspectShape, ReferenceAspectType};
use crate::aspects::structs::{FieldRequirement, StructAspectShape, StructAspectValue};
use crate::values::{AspectValue, ScalarAspectType};

pub fn validate_aspect_value(
    contract: &AspectContract,
    value: ContractValidationInput,
) -> TransitionOutcome<ContractValidatedAspectArtifact, ContractValidationDenial> {
    match (contract.shape(), value) {
        (AspectShape::Scalar(expected), ContractValidationInput::Scalar(value)) => {
            validate_scalar_family(contract, value, *expected)
        }
        (AspectShape::Struct(shape), ContractValidationInput::Struct(value)) => {
            validate_struct_value(contract, shape, value)
        }
        (AspectShape::Struct(_), ContractValidationInput::Scalar(_)) => {
            TransitionOutcome::denied(ContractValidationDenial::StructValueRequired)
        }
        (
            AspectShape::Reference(ReferenceAspectType::Entity),
            ContractValidationInput::Scalar(value),
        ) => validate_scalar_family(contract, value, ScalarAspectType::EntityRef),
        (AspectShape::Opaque(_), ContractValidationInput::Scalar(_)) => {
            TransitionOutcome::denied(ContractValidationDenial::ScalarValueRequired)
        }
        (_, ContractValidationInput::Struct(_)) => {
            TransitionOutcome::denied(ContractValidationDenial::ScalarValueRequired)
        }
    }
}

fn validate_struct_value(
    contract: &AspectContract,
    shape: &StructAspectShape,
    value: StructAspectValue,
) -> TransitionOutcome<ContractValidatedAspectArtifact, ContractValidationDenial> {
    if let Some(missing_field) = first_missing_required_field(shape, &value) {
        return TransitionOutcome::denied(ContractValidationDenial::MissingRequiredField(
            missing_field,
        ));
    }

    if let Some(denial) = first_struct_field_type_denial(shape, &value) {
        return TransitionOutcome::denied(denial);
    }

    TransitionOutcome::success(Artifact::new(ContractValidatedAspectValue::struct_value(
        contract.clone(),
        value,
    )))
}

fn first_missing_required_field(
    shape: &StructAspectShape,
    value: &StructAspectValue,
) -> Option<crate::aspects::structs::FieldKey> {
    shape
        .fields()
        .iter()
        .find(|field| {
            matches!(field.requirement(), FieldRequirement::Required)
                && value.get(field.key()).is_none()
        })
        .map(|field| field.key().clone())
}

fn first_struct_field_type_denial(
    shape: &StructAspectShape,
    value: &StructAspectValue,
) -> Option<ContractValidationDenial> {
    for (key, field_value) in value.fields() {
        let Some(field) = shape.field(key) else {
            return Some(ContractValidationDenial::UnknownField(key.clone()));
        };
        let found = field_value.value_family();
        if !field_value.has_canonical_representation() {
            return Some(ContractValidationDenial::NonCanonicalFieldValue {
                field: key.clone(),
                family: found,
            });
        }
        if found != field.value_type() {
            return Some(ContractValidationDenial::FieldTypeMismatch {
                field: key.clone(),
                expected: field.value_type(),
                found,
            });
        }
    }
    None
}

fn validate_scalar_family(
    contract: &AspectContract,
    value: AspectValue,
    expected: ScalarAspectType,
) -> TransitionOutcome<ContractValidatedAspectArtifact, ContractValidationDenial> {
    let found = value.value_family();
    if !value.has_canonical_representation() {
        return TransitionOutcome::denied(ContractValidationDenial::NonCanonicalScalarValue(found));
    }
    if found == expected {
        TransitionOutcome::success(Artifact::new(ContractValidatedAspectValue::scalar(
            contract.clone(),
            value,
        )))
    } else {
        TransitionOutcome::denied(ContractValidationDenial::ScalarTypeMismatch { expected, found })
    }
}
