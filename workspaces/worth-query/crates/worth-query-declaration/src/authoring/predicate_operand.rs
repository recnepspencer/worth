use worth_foundational::facade::{
    AspectValue, CanonicalBigInt, CanonicalDate, CanonicalDecimal, CanonicalF32, CanonicalF64,
    CanonicalRational, CanonicalTime, CanonicalTimestamp, CanonicalTimestampTz, ContentRefId,
    EntityId, InternedString, ScalarAspectType,
};

/// A proofless predicate-authoring carrier over exact Foundational value meaning.
///
/// Query owns the authoring role, while `AspectValue` remains the value algebra.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct WorthQueryPredicateOperand(AspectValue);

impl WorthQueryPredicateOperand {
    pub fn native(value: AspectValue) -> Self {
        Self(value)
    }

    pub fn as_native(&self) -> &AspectValue {
        &self.0
    }

    pub fn into_native(self) -> AspectValue {
        self.0
    }

    pub fn value_family(&self) -> ScalarAspectType {
        self.0.value_family()
    }

    pub fn string(value: impl Into<String>) -> Self {
        Self::interned_string(InternedString::Raw(value.into()))
    }

    pub fn interned_string(value: impl Into<InternedString>) -> Self {
        Self(AspectValue::String(value.into()))
    }

    pub fn null() -> Self {
        Self(AspectValue::Null)
    }

    pub fn int8(value: i8) -> Self {
        Self(AspectValue::Int8(value))
    }

    pub fn int16(value: i16) -> Self {
        Self(AspectValue::Int16(value))
    }

    pub fn int32(value: i32) -> Self {
        Self(AspectValue::Int32(value))
    }

    pub fn int64(value: i64) -> Self {
        Self(AspectValue::Int64(value))
    }

    pub fn uint8(value: u8) -> Self {
        Self(AspectValue::UInt8(value))
    }

    pub fn uint16(value: u16) -> Self {
        Self(AspectValue::UInt16(value))
    }

    pub fn uint32(value: u32) -> Self {
        Self(AspectValue::UInt32(value))
    }

    pub fn uint64(value: u64) -> Self {
        Self(AspectValue::UInt64(value))
    }

    pub fn float32(value: CanonicalF32) -> Self {
        Self(AspectValue::Float32(value))
    }

    pub fn float64(value: CanonicalF64) -> Self {
        Self(AspectValue::Float64(value))
    }

    pub fn decimal(value: CanonicalDecimal) -> Self {
        Self(AspectValue::Decimal(value))
    }

    pub fn big_int(value: CanonicalBigInt) -> Self {
        Self(AspectValue::BigInt(value))
    }

    pub fn rational(value: CanonicalRational) -> Self {
        Self(AspectValue::Rational(value))
    }

    pub fn uuid(value: [u8; 16]) -> Self {
        Self(AspectValue::Uuid(value))
    }

    pub fn date(value: CanonicalDate) -> Self {
        Self(AspectValue::Date(value))
    }

    pub fn time(value: CanonicalTime) -> Self {
        Self(AspectValue::Time(value))
    }

    pub fn timestamp(value: CanonicalTimestamp) -> Self {
        Self(AspectValue::Timestamp(value))
    }

    pub fn timestamp_tz(value: CanonicalTimestampTz) -> Self {
        Self(AspectValue::TimestampTz(value))
    }

    pub fn entity_ref(value: EntityId) -> Self {
        Self(AspectValue::EntityRef(value))
    }

    pub fn content_ref(value: ContentRefId) -> Self {
        Self(AspectValue::ContentRef(value))
    }

    pub fn boolean(value: bool) -> Self {
        Self(AspectValue::Bool(value))
    }
}

impl From<AspectValue> for WorthQueryPredicateOperand {
    fn from(value: AspectValue) -> Self {
        Self::native(value)
    }
}

impl From<String> for WorthQueryPredicateOperand {
    fn from(value: String) -> Self {
        Self::string(value)
    }
}

impl From<&str> for WorthQueryPredicateOperand {
    fn from(value: &str) -> Self {
        Self::string(value)
    }
}

macro_rules! native_operand_from {
    ($type:ty, $variant:ident) => {
        impl From<$type> for WorthQueryPredicateOperand {
            fn from(value: $type) -> Self {
                Self(AspectValue::$variant(value))
            }
        }
    };
}

native_operand_from!(bool, Bool);
native_operand_from!(i8, Int8);
native_operand_from!(i16, Int16);
native_operand_from!(i32, Int32);
native_operand_from!(i64, Int64);
native_operand_from!(u8, UInt8);
native_operand_from!(u16, UInt16);
native_operand_from!(u32, UInt32);
native_operand_from!(u64, UInt64);
native_operand_from!(CanonicalF32, Float32);
native_operand_from!(CanonicalF64, Float64);
native_operand_from!(CanonicalDecimal, Decimal);
native_operand_from!(CanonicalBigInt, BigInt);
native_operand_from!(CanonicalRational, Rational);
native_operand_from!([u8; 16], Uuid);
native_operand_from!(CanonicalDate, Date);
native_operand_from!(CanonicalTime, Time);
native_operand_from!(CanonicalTimestamp, Timestamp);
native_operand_from!(CanonicalTimestampTz, TimestampTz);
native_operand_from!(EntityId, EntityRef);
