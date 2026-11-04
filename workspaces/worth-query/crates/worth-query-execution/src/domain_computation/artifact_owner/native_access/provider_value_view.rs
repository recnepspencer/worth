use worth_foundational::facade::{
    AspectShape, CanonicalBigInt, CanonicalDate, CanonicalDecimal, CanonicalF32, CanonicalF64,
    CanonicalRational, CanonicalTime, CanonicalTimestamp, CanonicalTimestampTz, ContentRefId,
    EntityId, InternedString, ScalarAspectType, StructAspectValue,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WorthQueryArtifactProviderValueView<'a> {
    Null,
    Bool(&'a bool),
    Int8(&'a i8),
    Int16(&'a i16),
    Int32(&'a i32),
    Int64(&'a i64),
    UInt8(&'a u8),
    UInt16(&'a u16),
    UInt32(&'a u32),
    UInt64(&'a u64),
    Float32(&'a CanonicalF32),
    Float64(&'a CanonicalF64),
    Decimal(&'a CanonicalDecimal),
    BigInt(&'a CanonicalBigInt),
    Rational(&'a CanonicalRational),
    String(&'a InternedString),
    Bytes(&'a ContentRefId),
    Uuid(&'a [u8; 16]),
    Date(&'a CanonicalDate),
    Time(&'a CanonicalTime),
    Timestamp(&'a CanonicalTimestamp),
    TimestampTz(&'a CanonicalTimestampTz),
    EntityRef(&'a EntityId),
    ContentRef(&'a ContentRefId),
    Struct(&'a StructAspectValue),
}

impl WorthQueryArtifactProviderValueView<'_> {
    pub(crate) fn physical_bytes(self) -> usize {
        match self {
            Self::Null => 0,
            Self::Bool(value) => std::mem::size_of_val(value),
            Self::Int8(value) => std::mem::size_of_val(value),
            Self::Int16(value) => std::mem::size_of_val(value),
            Self::Int32(value) => std::mem::size_of_val(value),
            Self::Int64(value) => std::mem::size_of_val(value),
            Self::UInt8(value) => std::mem::size_of_val(value),
            Self::UInt16(value) => std::mem::size_of_val(value),
            Self::UInt32(value) => std::mem::size_of_val(value),
            Self::UInt64(value) => std::mem::size_of_val(value),
            Self::Float32(value) => std::mem::size_of_val(value),
            Self::Float64(value) => std::mem::size_of_val(value),
            Self::Decimal(value) => std::mem::size_of_val(value).saturating_add(value.0.capacity()),
            Self::BigInt(value) => std::mem::size_of_val(value).saturating_add(value.0.capacity()),
            Self::Rational(value) => std::mem::size_of_val(value)
                .saturating_add(value.numerator.0.capacity())
                .saturating_add(value.denominator.0.capacity()),
            Self::String(value) => std::mem::size_of_val(value).saturating_add(match value {
                InternedString::Raw(value) => value.capacity(),
                InternedString::Symbol(_) => 0,
            }),
            Self::Bytes(value) => std::mem::size_of_val(value),
            Self::Uuid(value) => std::mem::size_of_val(value),
            Self::Date(value) => std::mem::size_of_val(value),
            Self::Time(value) => std::mem::size_of_val(value),
            Self::Timestamp(value) => std::mem::size_of_val(value),
            Self::TimestampTz(value) => std::mem::size_of_val(value),
            Self::EntityRef(value) => std::mem::size_of_val(value),
            Self::ContentRef(value) => std::mem::size_of_val(value),
            Self::Struct(value) => {
                std::mem::size_of_val(value).saturating_add(value.owned_allocation_capacity_bytes())
            }
        }
    }

    pub(crate) fn matches_shape(self, shape: &AspectShape) -> bool {
        use ScalarAspectType as Scalar;

        matches!(
            (self, shape),
            (Self::Null, AspectShape::Scalar(Scalar::Null))
                | (Self::Bool(_), AspectShape::Scalar(Scalar::Bool))
                | (Self::Int8(_), AspectShape::Scalar(Scalar::Int8))
                | (Self::Int16(_), AspectShape::Scalar(Scalar::Int16))
                | (Self::Int32(_), AspectShape::Scalar(Scalar::Int32))
                | (Self::Int64(_), AspectShape::Scalar(Scalar::Int64))
                | (Self::UInt8(_), AspectShape::Scalar(Scalar::UInt8))
                | (Self::UInt16(_), AspectShape::Scalar(Scalar::UInt16))
                | (Self::UInt32(_), AspectShape::Scalar(Scalar::UInt32))
                | (Self::UInt64(_), AspectShape::Scalar(Scalar::UInt64))
                | (Self::Float32(_), AspectShape::Scalar(Scalar::Float32))
                | (Self::Float64(_), AspectShape::Scalar(Scalar::Float64))
                | (Self::Decimal(_), AspectShape::Scalar(Scalar::Decimal))
                | (Self::BigInt(_), AspectShape::Scalar(Scalar::BigInt))
                | (Self::Rational(_), AspectShape::Scalar(Scalar::Rational))
                | (Self::String(_), AspectShape::Scalar(Scalar::String))
                | (Self::Bytes(_), AspectShape::Scalar(Scalar::Bytes))
                | (Self::Uuid(_), AspectShape::Scalar(Scalar::Uuid))
                | (Self::Date(_), AspectShape::Scalar(Scalar::Date))
                | (Self::Time(_), AspectShape::Scalar(Scalar::Time))
                | (Self::Timestamp(_), AspectShape::Scalar(Scalar::Timestamp))
                | (
                    Self::TimestampTz(_),
                    AspectShape::Scalar(Scalar::TimestampTz)
                )
                | (Self::EntityRef(_), AspectShape::Scalar(Scalar::EntityRef))
                | (Self::ContentRef(_), AspectShape::Scalar(Scalar::ContentRef))
                | (Self::Struct(_), AspectShape::Struct(_))
        )
    }
}

mod court_doors_for_test;
