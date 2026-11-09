mod contract_validation;
mod validated_value;
mod validation_denial;
mod validation_input;

pub use contract_validation::validate_aspect_value;
pub use validated_value::{
    ContractValidatedAspectArtifact, ContractValidatedAspectValue, ContractValidatedAspectValueView,
};
pub use validation_denial::ContractValidationDenial;
pub use validation_input::ContractValidationInput;

#[cfg(test)]
pub(in crate::aspects) use validated_value::CloneObservation;

pub(crate) use contract_validation::validate_struct_field;
#[cfg(test)]
pub(in crate::aspects) use contract_validation::FieldValidationObservation;
