use serde_json::Value;

use super::JsonCompatibilityLoweringDenial;
use crate::locators::BoundarySourceLocator;
use crate::values::{AspectValue, ContentRefId, ScalarAspectType};

pub(super) fn lower_primitive_scalar(
    source: &BoundarySourceLocator,
    value: &Value,
    expected: ScalarAspectType,
) -> Result<AspectValue, JsonCompatibilityLoweringDenial> {
    match expected {
        ScalarAspectType::Null if value.is_null() => Ok(AspectValue::Null),
        ScalarAspectType::Bool => value.as_bool().map(AspectValue::Bool).ok_or_else(|| {
            JsonCompatibilityLoweringDenial::JsonShapeNotAdmitted {
                source: source.clone(),
                expected: "boolean",
            }
        }),
        ScalarAspectType::String => value
            .as_str()
            .map(|value| AspectValue::String(value.into()))
            .ok_or_else(|| JsonCompatibilityLoweringDenial::JsonShapeNotAdmitted {
                source: source.clone(),
                expected: "string",
            }),
        ScalarAspectType::ContentRef => value
            .as_str()
            .and_then(ContentRefId::from_hex)
            .map(AspectValue::ContentRef)
            .ok_or_else(|| JsonCompatibilityLoweringDenial::JsonShapeNotAdmitted {
                source: source.clone(),
                expected: "lowercase 32-byte content digest",
            }),
        _ => Err(JsonCompatibilityLoweringDenial::UnsupportedScalarFamily {
            source: source.clone(),
            expected,
        }),
    }
}
