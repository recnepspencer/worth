use crate::canonicalization::{CanonicalBasisValue, CanonicalFloatWidth, CanonicalIntegerWidth};
use crate::values::{AspectValue, InternedString};

#[derive(Clone, Copy)]
pub(super) enum CanonicalTextView<'a> {
    Raw(&'a str),
    Symbol(u32),
}

impl<'a> From<&'a InternedString> for CanonicalTextView<'a> {
    fn from(value: &'a InternedString) -> Self {
        match value {
            InternedString::Raw(value) => Self::Raw(value),
            InternedString::Symbol(value) => Self::Symbol(value.0),
        }
    }
}

/// Borrowed representation adapters share one downstream scalar grammar.
pub(super) enum CanonicalScalarView<'a> {
    Null,
    Bool(bool),
    SignedInteger {
        width: CanonicalIntegerWidth,
        value: i128,
    },
    UnsignedInteger {
        width: CanonicalIntegerWidth,
        value: u128,
    },
    FloatBits {
        width: CanonicalFloatWidth,
        bits: u64,
    },
    ExactText(CanonicalTextView<'a>),
    BytesDigest(&'a [u8]),
    DecimalText(CanonicalTextView<'a>),
    BigIntText(CanonicalTextView<'a>),
    RationalText {
        numerator: CanonicalTextView<'a>,
        denominator: CanonicalTextView<'a>,
    },
    BytesRefId(u64),
    ContentRefId(u64),
    EntityRef {
        partition_id: u32,
        local_slot: u64,
        generation: u32,
    },
    DateDays(i32),
    TimeNanos(u64),
    TimestampMicros(i64),
    TimestampTz {
        utc_micros_since_unix_epoch: i64,
        offset_minutes: i32,
    },
    UuidBytes(&'a [u8; 16]),
    NestedSequence(u32),
}

impl<'a> From<&'a CanonicalBasisValue> for CanonicalScalarView<'a> {
    fn from(value: &'a CanonicalBasisValue) -> Self {
        match value {
            CanonicalBasisValue::Null => Self::Null,
            CanonicalBasisValue::Bool(value) => Self::Bool(*value),
            CanonicalBasisValue::SignedInteger { width, value } => Self::SignedInteger {
                width: *width,
                value: *value,
            },
            CanonicalBasisValue::UnsignedInteger { width, value } => Self::UnsignedInteger {
                width: *width,
                value: *value,
            },
            CanonicalBasisValue::FloatBits { width, bits } => Self::FloatBits {
                width: *width,
                bits: *bits,
            },
            CanonicalBasisValue::ExactText(value) => Self::ExactText(value.into()),
            CanonicalBasisValue::BytesDigest(value) => Self::BytesDigest(value.bytes()),
            CanonicalBasisValue::DecimalText(value) => Self::DecimalText(value.into()),
            CanonicalBasisValue::BigIntText(value) => Self::BigIntText(value.into()),
            CanonicalBasisValue::RationalText {
                numerator,
                denominator,
            } => Self::RationalText {
                numerator: numerator.into(),
                denominator: denominator.into(),
            },
            CanonicalBasisValue::BytesRefId(value) => Self::BytesRefId(*value),
            CanonicalBasisValue::ContentRefId(value) => Self::ContentRefId(*value),
            CanonicalBasisValue::EntityRef {
                partition_id,
                local_slot,
                generation,
            } => Self::EntityRef {
                partition_id: *partition_id,
                local_slot: *local_slot,
                generation: *generation,
            },
            CanonicalBasisValue::DateDays(value) => Self::DateDays(*value),
            CanonicalBasisValue::TimeNanos(value) => Self::TimeNanos(*value),
            CanonicalBasisValue::TimestampMicros(value) => Self::TimestampMicros(*value),
            CanonicalBasisValue::TimestampTz {
                utc_micros_since_unix_epoch,
                offset_minutes,
            } => Self::TimestampTz {
                utc_micros_since_unix_epoch: *utc_micros_since_unix_epoch,
                offset_minutes: *offset_minutes,
            },
            CanonicalBasisValue::UuidBytes(value) => Self::UuidBytes(value),
            CanonicalBasisValue::NestedSequence(value) => Self::NestedSequence(*value),
        }
    }
}

impl<'a> From<&'a AspectValue> for CanonicalScalarView<'a> {
    fn from(value: &'a AspectValue) -> Self {
        use CanonicalIntegerWidth::{Bits16, Bits32, Bits64, Bits8};
        match value {
            AspectValue::Null => Self::Null,
            AspectValue::Bool(value) => Self::Bool(*value),
            AspectValue::Int8(value) => Self::SignedInteger {
                width: Bits8,
                value: i128::from(*value),
            },
            AspectValue::Int16(value) => Self::SignedInteger {
                width: Bits16,
                value: i128::from(*value),
            },
            AspectValue::Int32(value) => Self::SignedInteger {
                width: Bits32,
                value: i128::from(*value),
            },
            AspectValue::Int64(value) => Self::SignedInteger {
                width: Bits64,
                value: i128::from(*value),
            },
            AspectValue::UInt8(value) => Self::UnsignedInteger {
                width: Bits8,
                value: u128::from(*value),
            },
            AspectValue::UInt16(value) => Self::UnsignedInteger {
                width: Bits16,
                value: u128::from(*value),
            },
            AspectValue::UInt32(value) => Self::UnsignedInteger {
                width: Bits32,
                value: u128::from(*value),
            },
            AspectValue::UInt64(value) => Self::UnsignedInteger {
                width: Bits64,
                value: u128::from(*value),
            },
            AspectValue::Float32(value) => Self::FloatBits {
                width: CanonicalFloatWidth::Bits32,
                bits: u64::from(value.bits()),
            },
            AspectValue::Float64(value) => Self::FloatBits {
                width: CanonicalFloatWidth::Bits64,
                bits: value.bits(),
            },
            AspectValue::Decimal(value) => {
                Self::DecimalText(CanonicalTextView::Raw(value.as_str()))
            }
            AspectValue::BigInt(value) => Self::BigIntText(CanonicalTextView::Raw(value.as_str())),
            AspectValue::Rational(value) => Self::RationalText {
                numerator: CanonicalTextView::Raw(value.numerator.as_str()),
                denominator: CanonicalTextView::Raw(value.denominator.as_str()),
            },
            AspectValue::String(value) => Self::ExactText(value.into()),
            AspectValue::Bytes(value) => Self::BytesRefId(value.0),
            AspectValue::Uuid(value) => Self::UuidBytes(value),
            AspectValue::Date(value) => Self::DateDays(value.days_from_unix_epoch),
            AspectValue::Time(value) => Self::TimeNanos(value.nanos_since_midnight),
            AspectValue::Timestamp(value) => Self::TimestampMicros(value.micros_since_unix_epoch),
            AspectValue::TimestampTz(value) => Self::TimestampTz {
                utc_micros_since_unix_epoch: value.utc_micros_since_unix_epoch,
                offset_minutes: value.offset_minutes,
            },
            AspectValue::EntityRef(value) => Self::EntityRef {
                partition_id: value.partition_id.0,
                local_slot: value.local_slot.0,
                generation: value.generation.0,
            },
            AspectValue::ContentRef(value) => Self::ContentRefId(value.0),
        }
    }
}
