//! Grant and request values written in the grant record field's own type,
//! such as a closed `WorthQueryVocabulary` role, encoded with that field's
//! binding. A contract target and the request projecting onto it name one
//! typed value, so they cannot disagree.

use super::{
    ApplicationCapabilityEntitySelector, ApplicationCapabilityRequestProjectionDenial,
    ApplicationCapabilityValueBinding,
};
use crate::application_schema::fragment::RecordSelection;
use crate::application_schema::{
    ApplicationEncodedScalarValue, ApplicationSchemaDeclarationDenial,
};

/// The contract-side binding of `value` on `field`, as a target, status or
/// extent value.
pub fn application_capability_grant_value<Field: RecordSelection>(
    field: Field,
    value: impl Into<Field::Value>,
) -> Result<ApplicationCapabilityValueBinding, ApplicationSchemaDeclarationDenial> {
    let checked = field.checked()?;
    let encoded =
        ApplicationEncodedScalarValue::<Field::Binding>::try_new(value.into()).map_err(|_| {
            ApplicationSchemaDeclarationDenial::InvalidRecordField {
                field: checked.locus().clone(),
            }
        })?;
    ApplicationCapabilityValueBinding::new(field, encoded)
}

/// The request-side encoding of `value` on `field`, for a projection's
/// action or purpose.
pub fn application_capability_request_value<Field: RecordSelection>(
    field: &Field,
    value: impl Into<Field::Value>,
) -> Result<
    ApplicationEncodedScalarValue<Field::Binding>,
    ApplicationCapabilityRequestProjectionDenial,
> {
    ApplicationEncodedScalarValue::try_new(value.into())
        .map_err(|_| ApplicationCapabilityRequestProjectionDenial::input_variant(field.field()))
}

/// The request selector choosing the entity whose `field` equals `value`.
pub fn application_capability_selector<Schema, Field: RecordSelection>(
    field: Field,
    value: impl Into<Field::Value>,
) -> Result<
    ApplicationCapabilityEntitySelector<Schema, Field::Entity>,
    ApplicationCapabilityRequestProjectionDenial,
> {
    let encoded = application_capability_request_value(&field, value)?;
    ApplicationCapabilityEntitySelector::new(field, encoded)
}
