use worth_foundational::facade::{
    CanonicalBigInt, CanonicalDate, CanonicalDecimal, CanonicalF32, CanonicalF64,
    CanonicalRational, CanonicalTime, CanonicalTimestamp, CanonicalTimestampTz, ContentRefId,
    EntityId, InternedString, ScalarAspectType, StructAspectValue,
};

use super::WorthQueryArtifactProviderValueView;

#[derive(Clone, Copy, Debug)]
pub enum WorthQueryArtifactProviderFieldSlice<'a> {
    Null(usize),
    Bool(&'a [bool]),
    Int8(&'a [i8]),
    Int16(&'a [i16]),
    Int32(&'a [i32]),
    Int64(&'a [i64]),
    UInt8(&'a [u8]),
    UInt16(&'a [u16]),
    UInt32(&'a [u32]),
    UInt64(&'a [u64]),
    Float32(&'a [CanonicalF32]),
    Float64(&'a [CanonicalF64]),
    Decimal(&'a [CanonicalDecimal]),
    BigInt(&'a [CanonicalBigInt]),
    Rational(&'a [CanonicalRational]),
    String(&'a [InternedString]),
    Bytes(&'a [ContentRefId]),
    Uuid(&'a [[u8; 16]]),
    Date(&'a [CanonicalDate]),
    Time(&'a [CanonicalTime]),
    Timestamp(&'a [CanonicalTimestamp]),
    TimestampTz(&'a [CanonicalTimestampTz]),
    EntityRef(&'a [EntityId]),
    ContentRef(&'a [ContentRefId]),
    Struct(&'a [StructAspectValue]),
}

impl<'a> WorthQueryArtifactProviderFieldSlice<'a> {
    pub fn len(self) -> usize {
        match self {
            Self::Null(len) => len,
            Self::Bool(values) => values.len(),
            Self::Int8(values) => values.len(),
            Self::Int16(values) => values.len(),
            Self::Int32(values) => values.len(),
            Self::Int64(values) => values.len(),
            Self::UInt8(values) => values.len(),
            Self::UInt16(values) => values.len(),
            Self::UInt32(values) => values.len(),
            Self::UInt64(values) => values.len(),
            Self::Float32(values) => values.len(),
            Self::Float64(values) => values.len(),
            Self::Decimal(values) => values.len(),
            Self::BigInt(values) => values.len(),
            Self::Rational(values) => values.len(),
            Self::String(values) => values.len(),
            Self::Bytes(values) => values.len(),
            Self::Uuid(values) => values.len(),
            Self::Date(values) => values.len(),
            Self::Time(values) => values.len(),
            Self::Timestamp(values) => values.len(),
            Self::TimestampTz(values) => values.len(),
            Self::EntityRef(values) => values.len(),
            Self::ContentRef(values) => values.len(),
            Self::Struct(values) => values.len(),
        }
    }

    pub fn is_empty(self) -> bool {
        self.len() == 0
    }

    pub(crate) fn physical_bytes(self) -> usize {
        match self {
            Self::Null(_) => 0,
            Self::Bool(values) => std::mem::size_of_val(values),
            Self::Int8(values) => std::mem::size_of_val(values),
            Self::Int16(values) => std::mem::size_of_val(values),
            Self::Int32(values) => std::mem::size_of_val(values),
            Self::Int64(values) => std::mem::size_of_val(values),
            Self::UInt8(values) => std::mem::size_of_val(values),
            Self::UInt16(values) => std::mem::size_of_val(values),
            Self::UInt32(values) => std::mem::size_of_val(values),
            Self::UInt64(values) => std::mem::size_of_val(values),
            Self::Float32(values) => std::mem::size_of_val(values),
            Self::Float64(values) => std::mem::size_of_val(values),
            Self::Decimal(values) => nested_bytes(values, |value| value.0.capacity()),
            Self::BigInt(values) => nested_bytes(values, |value| value.0.capacity()),
            Self::Rational(values) => nested_bytes(values, |value| {
                value
                    .numerator
                    .0
                    .capacity()
                    .saturating_add(value.denominator.0.capacity())
            }),
            Self::String(values) => nested_bytes(values, |value| match value {
                InternedString::Raw(value) => value.capacity(),
                InternedString::Symbol(_) => 0,
            }),
            Self::Bytes(values) => std::mem::size_of_val(values),
            Self::Uuid(values) => std::mem::size_of_val(values),
            Self::Date(values) => std::mem::size_of_val(values),
            Self::Time(values) => std::mem::size_of_val(values),
            Self::Timestamp(values) => std::mem::size_of_val(values),
            Self::TimestampTz(values) => std::mem::size_of_val(values),
            Self::EntityRef(values) => std::mem::size_of_val(values),
            Self::ContentRef(values) => std::mem::size_of_val(values),
            Self::Struct(values) => {
                nested_bytes(values, |value| value.owned_allocation_capacity_bytes())
            }
        }
    }

    pub(crate) fn value(self, row: usize) -> Option<WorthQueryArtifactProviderValueView<'a>> {
        Some(match self {
            Self::Null(len) if row < len => WorthQueryArtifactProviderValueView::Null,
            Self::Bool(values) => WorthQueryArtifactProviderValueView::Bool(values.get(row)?),
            Self::Int8(values) => WorthQueryArtifactProviderValueView::Int8(values.get(row)?),
            Self::Int16(values) => WorthQueryArtifactProviderValueView::Int16(values.get(row)?),
            Self::Int32(values) => WorthQueryArtifactProviderValueView::Int32(values.get(row)?),
            Self::Int64(values) => WorthQueryArtifactProviderValueView::Int64(values.get(row)?),
            Self::UInt8(values) => WorthQueryArtifactProviderValueView::UInt8(values.get(row)?),
            Self::UInt16(values) => WorthQueryArtifactProviderValueView::UInt16(values.get(row)?),
            Self::UInt32(values) => WorthQueryArtifactProviderValueView::UInt32(values.get(row)?),
            Self::UInt64(values) => WorthQueryArtifactProviderValueView::UInt64(values.get(row)?),
            Self::Float32(values) => WorthQueryArtifactProviderValueView::Float32(values.get(row)?),
            Self::Float64(values) => WorthQueryArtifactProviderValueView::Float64(values.get(row)?),
            Self::Decimal(values) => WorthQueryArtifactProviderValueView::Decimal(values.get(row)?),
            Self::BigInt(values) => WorthQueryArtifactProviderValueView::BigInt(values.get(row)?),
            Self::Rational(values) => {
                WorthQueryArtifactProviderValueView::Rational(values.get(row)?)
            }
            Self::String(values) => WorthQueryArtifactProviderValueView::String(values.get(row)?),
            Self::Bytes(values) => WorthQueryArtifactProviderValueView::Bytes(values.get(row)?),
            Self::Uuid(values) => WorthQueryArtifactProviderValueView::Uuid(values.get(row)?),
            Self::Date(values) => WorthQueryArtifactProviderValueView::Date(values.get(row)?),
            Self::Time(values) => WorthQueryArtifactProviderValueView::Time(values.get(row)?),
            Self::Timestamp(values) => {
                WorthQueryArtifactProviderValueView::Timestamp(values.get(row)?)
            }
            Self::TimestampTz(values) => {
                WorthQueryArtifactProviderValueView::TimestampTz(values.get(row)?)
            }
            Self::EntityRef(values) => {
                WorthQueryArtifactProviderValueView::EntityRef(values.get(row)?)
            }
            Self::ContentRef(values) => {
                WorthQueryArtifactProviderValueView::ContentRef(values.get(row)?)
            }
            Self::Struct(values) => WorthQueryArtifactProviderValueView::Struct(values.get(row)?),
            Self::Null(_) => return None,
        })
    }

    pub(crate) fn matches_scalar_family(self, family: ScalarAspectType) -> bool {
        matches!(
            (self, family),
            (Self::Null(_), ScalarAspectType::Null)
                | (Self::Bool(_), ScalarAspectType::Bool)
                | (Self::Int8(_), ScalarAspectType::Int8)
                | (Self::Int16(_), ScalarAspectType::Int16)
                | (Self::Int32(_), ScalarAspectType::Int32)
                | (Self::Int64(_), ScalarAspectType::Int64)
                | (Self::UInt8(_), ScalarAspectType::UInt8)
                | (Self::UInt16(_), ScalarAspectType::UInt16)
                | (Self::UInt32(_), ScalarAspectType::UInt32)
                | (Self::UInt64(_), ScalarAspectType::UInt64)
                | (Self::Float32(_), ScalarAspectType::Float32)
                | (Self::Float64(_), ScalarAspectType::Float64)
                | (Self::Decimal(_), ScalarAspectType::Decimal)
                | (Self::BigInt(_), ScalarAspectType::BigInt)
                | (Self::Rational(_), ScalarAspectType::Rational)
                | (Self::String(_), ScalarAspectType::String)
                | (Self::Bytes(_), ScalarAspectType::Bytes)
                | (Self::Uuid(_), ScalarAspectType::Uuid)
                | (Self::Date(_), ScalarAspectType::Date)
                | (Self::Time(_), ScalarAspectType::Time)
                | (Self::Timestamp(_), ScalarAspectType::Timestamp)
                | (Self::TimestampTz(_), ScalarAspectType::TimestampTz)
                | (Self::EntityRef(_), ScalarAspectType::EntityRef)
                | (Self::ContentRef(_), ScalarAspectType::ContentRef)
        )
    }

    pub(crate) fn is_struct(self) -> bool {
        matches!(self, Self::Struct(_))
    }
}

fn nested_bytes<T>(values: &[T], nested: impl Fn(&T) -> usize) -> usize {
    values
        .iter()
        .fold(std::mem::size_of_val(values), |total, value| {
            total.saturating_add(nested(value))
        })
}
