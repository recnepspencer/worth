//! Lossless correspondence between native scalar facts and expression operands.
use super::{
    ExpressionDenial, ExpressionDenialDetail, ExpressionType, ExpressionValue, IntegerType,
};
use crate::values::{AspectValue, ScalarAspectType};

fn unsupported() -> ExpressionDenial {
    ExpressionDenial::new(ExpressionDenialDetail::InvalidValue(
        "native scalar has no lossless expression correspondence",
    ))
}

impl ExpressionType {
    /// The exact expression type of a native scalar family. Structured,
    /// contextual and unit-bearing interpretations require their own contract.
    pub fn from_scalar_family(family: ScalarAspectType) -> Result<Self, ExpressionDenial> {
        use ScalarAspectType as Scalar;
        Ok(match family {
            Scalar::Bool => Self::Bool,
            Scalar::Int8 => Self::Integer(IntegerType::Int8),
            Scalar::Int16 => Self::Integer(IntegerType::Int16),
            Scalar::Int32 => Self::Integer(IntegerType::Int32),
            Scalar::Int64 => Self::Integer(IntegerType::Int64),
            Scalar::UInt8 => Self::Integer(IntegerType::UInt8),
            Scalar::UInt16 => Self::Integer(IntegerType::UInt16),
            Scalar::UInt32 => Self::Integer(IntegerType::UInt32),
            Scalar::UInt64 => Self::Integer(IntegerType::UInt64),
            Scalar::Float32 => Self::Float32,
            Scalar::Float64 => Self::Float64,
            Scalar::Decimal => Self::Decimal,
            Scalar::String => Self::String,
            _ => return Err(unsupported()),
        })
    }
}

impl ExpressionValue {
    /// Interprets an actual native scalar without a domain projector. Presence
    /// and nominal/unit meaning remain the surrounding contract's concern.
    pub fn from_aspect_value(value: &AspectValue) -> Result<Self, ExpressionDenial> {
        use crate::values::InternedString;
        Ok(match value {
            AspectValue::Bool(value) => Self::bool(*value),
            AspectValue::Int8(value) => Self::integer(i128::from(*value)),
            AspectValue::Int16(value) => Self::integer(i128::from(*value)),
            AspectValue::Int32(value) => Self::integer(i128::from(*value)),
            AspectValue::Int64(value) => Self::integer(i128::from(*value)),
            AspectValue::UInt8(value) => Self::integer(i128::from(*value)),
            AspectValue::UInt16(value) => Self::integer(i128::from(*value)),
            AspectValue::UInt32(value) => Self::integer(i128::from(*value)),
            AspectValue::UInt64(value) => Self::integer(i128::from(*value)),
            AspectValue::Float32(value) => return Self::float32(*value),
            AspectValue::Float64(value) => return Self::float64(*value),
            AspectValue::Decimal(value) => return Self::decimal(value),
            AspectValue::String(InternedString::Raw(value)) => Self::string(value.as_str()),
            _ => return Err(unsupported()),
        })
    }
}
