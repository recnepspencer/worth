use crate::portable_identity::WorthQueryPortableTypeIdentity;

use super::ApplicationValueValidationDenial;

/// Entry-owned identity and validation for a representation-neutral value.
///
/// Structured values are decomposed by their query, operation, or effect
/// binding. They are never coerced into one Foundational scalar.
pub trait ApplicationStructuredValueBinding: 'static {
    type Value: 'static;

    const IDENTITY_NAME: &'static str;
    const IDENTITY: WorthQueryPortableTypeIdentity =
        WorthQueryPortableTypeIdentity::declared(Self::IDENTITY_NAME);

    fn validate(value: &Self::Value) -> Result<(), ApplicationValueValidationDenial>;
}
