use crate::aspects::StructAspectValue;
use crate::canonicalization::basis::{
    CanonicalBasisValue, CanonicalFloatWidth, CanonicalIntegerWidth,
};
use crate::values::{AspectValue, EntityId};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CanonicalAspectValueIdentityBasis(String);

impl CanonicalAspectValueIdentityBasis {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub fn prepare_aspect_value_identity_basis(
    value: &AspectValue,
) -> CanonicalAspectValueIdentityBasis {
    CanonicalAspectValueIdentityBasis(
        crate::canonicalization::digest_slots::aspect_value_material(value),
    )
}

pub fn prepare_struct_aspect_value_identity_basis(
    value: &StructAspectValue,
) -> CanonicalAspectValueIdentityBasis {
    CanonicalAspectValueIdentityBasis(
        crate::canonicalization::digest_slots::struct_value_material(value),
    )
}

/// Lowers one Foundational scalar into its typed canonical-basis value.
///
/// This is representation only; canonical readiness and comparison authority
/// still require the corresponding Foundational preparation APIs.
pub fn canonical_basis_value_for_aspect_value(value: &AspectValue) -> CanonicalBasisValue {
    match value {
        AspectValue::Null => CanonicalBasisValue::Null,
        AspectValue::Bool(value) => CanonicalBasisValue::Bool(*value),
        AspectValue::Int8(value) => CanonicalBasisValue::SignedInteger {
            width: CanonicalIntegerWidth::Bits8,
            value: i128::from(*value),
        },
        AspectValue::Int16(value) => CanonicalBasisValue::SignedInteger {
            width: CanonicalIntegerWidth::Bits16,
            value: i128::from(*value),
        },
        AspectValue::Int32(value) => CanonicalBasisValue::SignedInteger {
            width: CanonicalIntegerWidth::Bits32,
            value: i128::from(*value),
        },
        AspectValue::Int64(value) => CanonicalBasisValue::SignedInteger {
            width: CanonicalIntegerWidth::Bits64,
            value: i128::from(*value),
        },
        AspectValue::UInt8(value) => CanonicalBasisValue::UnsignedInteger {
            width: CanonicalIntegerWidth::Bits8,
            value: u128::from(*value),
        },
        AspectValue::UInt16(value) => CanonicalBasisValue::UnsignedInteger {
            width: CanonicalIntegerWidth::Bits16,
            value: u128::from(*value),
        },
        AspectValue::UInt32(value) => CanonicalBasisValue::UnsignedInteger {
            width: CanonicalIntegerWidth::Bits32,
            value: u128::from(*value),
        },
        AspectValue::UInt64(value) => CanonicalBasisValue::UnsignedInteger {
            width: CanonicalIntegerWidth::Bits64,
            value: u128::from(*value),
        },
        AspectValue::Float32(value) => CanonicalBasisValue::FloatBits {
            width: CanonicalFloatWidth::Bits32,
            bits: u64::from(value.bits()),
        },
        AspectValue::Float64(value) => CanonicalBasisValue::FloatBits {
            width: CanonicalFloatWidth::Bits64,
            bits: value.bits(),
        },
        AspectValue::Decimal(value) => CanonicalBasisValue::DecimalText(value.as_str().into()),
        AspectValue::BigInt(value) => CanonicalBasisValue::BigIntText(value.as_str().into()),
        AspectValue::Rational(value) => CanonicalBasisValue::RationalText {
            numerator: value.numerator.as_str().into(),
            denominator: value.denominator.as_str().into(),
        },
        AspectValue::String(value) => CanonicalBasisValue::ExactText(value.clone()),
        AspectValue::Bytes(value) => CanonicalBasisValue::BytesRefId(value.0),
        AspectValue::Uuid(value) => CanonicalBasisValue::UuidBytes(*value),
        AspectValue::Date(value) => CanonicalBasisValue::DateDays(value.days_from_unix_epoch),
        AspectValue::Time(value) => CanonicalBasisValue::TimeNanos(value.nanos_since_midnight),
        AspectValue::Timestamp(value) => {
            CanonicalBasisValue::TimestampMicros(value.micros_since_unix_epoch)
        }
        AspectValue::TimestampTz(value) => CanonicalBasisValue::TimestampTz {
            utc_micros_since_unix_epoch: value.utc_micros_since_unix_epoch,
            offset_minutes: value.offset_minutes,
        },
        AspectValue::EntityRef(value) => entity_basis_value(value),
        AspectValue::ContentRef(value) => CanonicalBasisValue::ContentRefId(value.0),
    }
}

fn entity_basis_value(value: &EntityId) -> CanonicalBasisValue {
    CanonicalBasisValue::EntityRef {
        partition_id: value.partition_id.0,
        local_slot: value.local_slot.0,
        generation: value.generation.0,
    }
}
