use crate::identities::CanonicalDigestId;
use crate::values::InternedString;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CanonicalIntegerWidth {
    Bits8,
    Bits16,
    Bits32,
    Bits64,
    Bits128,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CanonicalFloatWidth {
    Bits32,
    Bits64,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CanonicalBasisValue {
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
    ExactText(InternedString),
    BytesDigest(CanonicalDigestId),
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
}

impl CanonicalBasisValue {
    pub fn owned_allocation_capacity_bytes(&self) -> usize {
        match self {
            Self::ExactText(value) | Self::DecimalText(value) | Self::BigIntText(value) => {
                value.owned_allocation_capacity_bytes()
            }
            Self::RationalText {
                numerator,
                denominator,
            } => numerator
                .owned_allocation_capacity_bytes()
                .saturating_add(denominator.owned_allocation_capacity_bytes()),
            _ => 0,
        }
    }
}
