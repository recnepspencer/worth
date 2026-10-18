use super::references::{ContentRefId, EntityId};
use super::scalar_kind::ScalarAspectType;
use super::scalar_wrappers::{
    CanonicalBigInt, CanonicalDate, CanonicalDecimal, CanonicalF32, CanonicalF64,
    CanonicalRational, CanonicalTime, CanonicalTimestamp, CanonicalTimestampTz, InternedString,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AspectValue {
    Null,
    Bool(bool),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    Float32(CanonicalF32),
    Float64(CanonicalF64),
    Decimal(CanonicalDecimal),
    BigInt(CanonicalBigInt),
    Rational(CanonicalRational),
    String(InternedString),
    Bytes(ContentRefId),
    Uuid([u8; 16]),
    Date(CanonicalDate),
    Time(CanonicalTime),
    Timestamp(CanonicalTimestamp),
    TimestampTz(CanonicalTimestampTz),
    EntityRef(EntityId),
    ContentRef(ContentRefId),
}

impl AspectValue {
    pub fn value_family(&self) -> ScalarAspectType {
        match self {
            Self::Null => ScalarAspectType::Null,
            Self::Bool(_) => ScalarAspectType::Bool,
            Self::Int8(_) => ScalarAspectType::Int8,
            Self::Int16(_) => ScalarAspectType::Int16,
            Self::Int32(_) => ScalarAspectType::Int32,
            Self::Int64(_) => ScalarAspectType::Int64,
            Self::UInt8(_) => ScalarAspectType::UInt8,
            Self::UInt16(_) => ScalarAspectType::UInt16,
            Self::UInt32(_) => ScalarAspectType::UInt32,
            Self::UInt64(_) => ScalarAspectType::UInt64,
            Self::Float32(_) => ScalarAspectType::Float32,
            Self::Float64(_) => ScalarAspectType::Float64,
            Self::Decimal(_) => ScalarAspectType::Decimal,
            Self::BigInt(_) => ScalarAspectType::BigInt,
            Self::Rational(_) => ScalarAspectType::Rational,
            Self::String(_) => ScalarAspectType::String,
            Self::Bytes(_) => ScalarAspectType::Bytes,
            Self::Uuid(_) => ScalarAspectType::Uuid,
            Self::Date(_) => ScalarAspectType::Date,
            Self::Time(_) => ScalarAspectType::Time,
            Self::Timestamp(_) => ScalarAspectType::Timestamp,
            Self::TimestampTz(_) => ScalarAspectType::TimestampTz,
            Self::EntityRef(_) => ScalarAspectType::EntityRef,
            Self::ContentRef(_) => ScalarAspectType::ContentRef,
        }
    }

    /// Checks the borrowed scalar representation without allocating or normalizing it.
    pub fn has_canonical_representation(&self) -> bool {
        match self {
            Self::Float32(value) => value.is_canonical(),
            Self::Float64(value) => value.is_canonical(),
            Self::Decimal(value) => value.is_canonical(),
            Self::BigInt(value) => value.is_canonical(),
            Self::Rational(value) => value.is_canonical(),
            Self::Time(value) => value.is_canonical(),
            Self::TimestampTz(value) => value.is_canonical(),
            _ => true,
        }
    }

    /// Stable logical width of the native value, excluding allocator layout.
    pub fn semantic_byte_width(&self) -> usize {
        match self {
            Self::Null => 1,
            Self::Bool(_) | Self::Int8(_) | Self::UInt8(_) => 2,
            Self::Int16(_) | Self::UInt16(_) => 3,
            Self::Int32(_) | Self::UInt32(_) | Self::Float32(_) | Self::Date(_) => 5,
            Self::Int64(_)
            | Self::UInt64(_)
            | Self::Float64(_)
            | Self::Time(_)
            | Self::Timestamp(_)
            | Self::Bytes(_)
            | Self::ContentRef(_) => 9,
            Self::Decimal(value) => 1_usize.saturating_add(value.as_str().len()),
            Self::BigInt(value) => 1_usize.saturating_add(value.as_str().len()),
            Self::Rational(value) => 1_usize
                .saturating_add(value.numerator.as_str().len())
                .saturating_add(value.denominator.as_str().len()),
            Self::String(super::scalar_wrappers::InternedString::Raw(value)) => {
                1_usize.saturating_add(value.len())
            }
            Self::String(super::scalar_wrappers::InternedString::Symbol(_)) => 5,
            Self::Uuid(_) => 17,
            Self::TimestampTz(_) => 13,
            Self::EntityRef(_) => 17,
        }
    }

    /// Allocator capacity retained exclusively by this value, excluding its
    /// inline `AspectValue` storage.
    pub fn owned_allocation_capacity_bytes(&self) -> usize {
        match self {
            Self::Decimal(value) => value.0.capacity(),
            Self::BigInt(value) => value.0.capacity(),
            Self::Rational(value) => value
                .numerator
                .0
                .capacity()
                .saturating_add(value.denominator.0.capacity()),
            Self::String(InternedString::Raw(value)) => value.capacity(),
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests;
