mod numeric_families;
mod primitive_families;
mod wrapper_families;

use serde_json::Value;

use super::JsonCompatibilityLoweringDenial;
use crate::locators::BoundarySourceLocator;
use crate::values::{AspectValue, ScalarAspectType};

pub(crate) use numeric_families::required_u64;

use numeric_families::lower_numeric_scalar;
use primitive_families::lower_primitive_scalar;
use wrapper_families::lower_wrapper_scalar;

pub(super) fn lower_json_scalar(
    source: &BoundarySourceLocator,
    value: &Value,
    expected: ScalarAspectType,
) -> Result<AspectValue, JsonCompatibilityLoweringDenial> {
    match expected {
        ScalarAspectType::Uuid => lower_wrapper_scalar(source, value, expected),
        ScalarAspectType::Null
        | ScalarAspectType::Bool
        | ScalarAspectType::String
        | ScalarAspectType::ContentRef => {
            if value.is_array() || value.is_object() {
                return Err(
                    JsonCompatibilityLoweringDenial::UnsupportedRecursiveDocument {
                        source: source.clone(),
                        expected,
                    },
                );
            }
            lower_primitive_scalar(source, value, expected)
        }
        ScalarAspectType::Int8
        | ScalarAspectType::Int16
        | ScalarAspectType::Int32
        | ScalarAspectType::Int64
        | ScalarAspectType::UInt8
        | ScalarAspectType::UInt16
        | ScalarAspectType::UInt32
        | ScalarAspectType::UInt64
        | ScalarAspectType::Float32
        | ScalarAspectType::Float64
        | ScalarAspectType::Date
        | ScalarAspectType::Time
        | ScalarAspectType::Timestamp => {
            if value.is_array() || value.is_object() {
                return Err(
                    JsonCompatibilityLoweringDenial::UnsupportedRecursiveDocument {
                        source: source.clone(),
                        expected,
                    },
                );
            }
            lower_numeric_scalar(source, value, expected)
        }
        ScalarAspectType::Decimal | ScalarAspectType::BigInt | ScalarAspectType::Rational => {
            if value.is_array() || value.is_object() {
                return Err(
                    JsonCompatibilityLoweringDenial::UnsupportedRecursiveDocument {
                        source: source.clone(),
                        expected,
                    },
                );
            }
            lower_wrapper_scalar(source, value, expected)
        }
        _ => Err(JsonCompatibilityLoweringDenial::UnsupportedScalarFamily {
            source: source.clone(),
            expected,
        }),
    }
}
