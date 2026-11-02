use serde::{Deserialize, Serialize};
use worth_foundational::facade::{
    CanonicalBasisValue, CanonicalDigestId, CanonicalFloatWidth, CanonicalIntegerWidth,
    InternedString,
};

#[derive(Serialize, Deserialize)]
pub(super) enum NativeValue {
    Null,
    Bool(bool),
    SignedInteger {
        width: NativeIntegerWidth,
        value: i128,
    },
    UnsignedInteger {
        width: NativeIntegerWidth,
        value: u128,
    },
    FloatBits {
        width: NativeFloatWidth,
        bits: u64,
    },
    ExactText(InternedString),
    BytesDigest([u8; 32]),
    DecimalText(InternedString),
    BigIntText(InternedString),
    RationalText {
        numerator: InternedString,
        denominator: InternedString,
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
    UuidBytes([u8; 16]),
    NestedSequence(u32),
    InlineBytes(worth_foundational::InlineBytes),
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub(super) enum NativeIntegerWidth {
    Bits8,
    Bits16,
    Bits32,
    Bits64,
    Bits128,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub(super) enum NativeFloatWidth {
    Bits32,
    Bits64,
}

impl TryFrom<&CanonicalBasisValue> for NativeValue {
    type Error = String;

    fn try_from(value: &CanonicalBasisValue) -> Result<Self, Self::Error> {
        Ok(match value {
            CanonicalBasisValue::InlineBytes(value) => Self::InlineBytes(value.clone()),
            CanonicalBasisValue::Null => Self::Null,
            CanonicalBasisValue::Bool(value) => Self::Bool(*value),
            CanonicalBasisValue::SignedInteger { width, value } => Self::SignedInteger {
                width: (*width).into(),
                value: *value,
            },
            CanonicalBasisValue::UnsignedInteger { width, value } => Self::UnsignedInteger {
                width: (*width).into(),
                value: *value,
            },
            CanonicalBasisValue::FloatBits { width, bits } => Self::FloatBits {
                width: (*width).into(),
                bits: *bits,
            },
            CanonicalBasisValue::ExactText(value) => Self::ExactText(value.clone()),
            CanonicalBasisValue::BytesDigest(value) => Self::BytesDigest(*value.bytes()),
            CanonicalBasisValue::DecimalText(value) => Self::DecimalText(value.clone()),
            CanonicalBasisValue::BigIntText(value) => Self::BigIntText(value.clone()),
            CanonicalBasisValue::RationalText {
                numerator,
                denominator,
            } => Self::RationalText {
                numerator: numerator.clone(),
                denominator: denominator.clone(),
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
            CanonicalBasisValue::UuidBytes(value) => Self::UuidBytes(*value),
            CanonicalBasisValue::NestedSequence(value) => Self::NestedSequence(*value),
        })
    }
}

impl TryFrom<NativeValue> for CanonicalBasisValue {
    type Error = String;

    fn try_from(value: NativeValue) -> Result<Self, Self::Error> {
        Ok(match value {
            NativeValue::InlineBytes(value) => Self::InlineBytes(value),
            NativeValue::Null => Self::Null,
            NativeValue::Bool(value) => Self::Bool(value),
            NativeValue::SignedInteger { width, value } => Self::SignedInteger {
                width: width.into(),
                value,
            },
            NativeValue::UnsignedInteger { width, value } => Self::UnsignedInteger {
                width: width.into(),
                value,
            },
            NativeValue::FloatBits { width, bits } => Self::FloatBits {
                width: width.into(),
                bits,
            },
            NativeValue::ExactText(value) => Self::ExactText(value),
            NativeValue::BytesDigest(value) => Self::BytesDigest(CanonicalDigestId::new(value)),
            NativeValue::DecimalText(value) => Self::DecimalText(value),
            NativeValue::BigIntText(value) => Self::BigIntText(value),
            NativeValue::RationalText {
                numerator,
                denominator,
            } => Self::RationalText {
                numerator,
                denominator,
            },
            NativeValue::BytesRefId(value) => Self::BytesRefId(value),
            NativeValue::ContentRefId(value) => Self::ContentRefId(value),
            NativeValue::EntityRef {
                partition_id,
                local_slot,
                generation,
            } => Self::EntityRef {
                partition_id,
                local_slot,
                generation,
            },
            NativeValue::DateDays(value) => Self::DateDays(value),
            NativeValue::TimeNanos(value) => Self::TimeNanos(value),
            NativeValue::TimestampMicros(value) => Self::TimestampMicros(value),
            NativeValue::TimestampTz {
                utc_micros_since_unix_epoch,
                offset_minutes,
            } => Self::TimestampTz {
                utc_micros_since_unix_epoch,
                offset_minutes,
            },
            NativeValue::UuidBytes(value) => Self::UuidBytes(value),
            NativeValue::NestedSequence(value) => Self::NestedSequence(value),
        })
    }
}

impl From<CanonicalIntegerWidth> for NativeIntegerWidth {
    fn from(width: CanonicalIntegerWidth) -> Self {
        match width {
            CanonicalIntegerWidth::Bits8 => Self::Bits8,
            CanonicalIntegerWidth::Bits16 => Self::Bits16,
            CanonicalIntegerWidth::Bits32 => Self::Bits32,
            CanonicalIntegerWidth::Bits64 => Self::Bits64,
            CanonicalIntegerWidth::Bits128 => Self::Bits128,
        }
    }
}

impl From<NativeIntegerWidth> for CanonicalIntegerWidth {
    fn from(width: NativeIntegerWidth) -> Self {
        match width {
            NativeIntegerWidth::Bits8 => Self::Bits8,
            NativeIntegerWidth::Bits16 => Self::Bits16,
            NativeIntegerWidth::Bits32 => Self::Bits32,
            NativeIntegerWidth::Bits64 => Self::Bits64,
            NativeIntegerWidth::Bits128 => Self::Bits128,
        }
    }
}

impl From<CanonicalFloatWidth> for NativeFloatWidth {
    fn from(width: CanonicalFloatWidth) -> Self {
        match width {
            CanonicalFloatWidth::Bits32 => Self::Bits32,
            CanonicalFloatWidth::Bits64 => Self::Bits64,
        }
    }
}

impl From<NativeFloatWidth> for CanonicalFloatWidth {
    fn from(width: NativeFloatWidth) -> Self {
        match width {
            NativeFloatWidth::Bits32 => Self::Bits32,
            NativeFloatWidth::Bits64 => Self::Bits64,
        }
    }
}
