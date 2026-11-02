use worth_foundational::facade::{
    AspectValue, CanonicalBigInt, CanonicalDate, CanonicalDecimal, CanonicalF32, CanonicalF64,
    CanonicalRational, CanonicalTime, CanonicalTimestamp, CanonicalTimestampTz, ContentRefId,
    EntityId, Generation, InternedString, LocalSlot, PartitionId, Symbol,
};

use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};

pub(super) fn write_aspect_value(
    output: &mut dyn BinaryEncodingSink,
    value: &AspectValue,
) -> Result<(), Denial> {
    match value {
        AspectValue::InlineBytes(value) => {
            output.u16(26)?;
            output.u32(value.wire_length())?;
            output.raw_bytes(value.as_bytes())
        }
        AspectValue::Null => output.u16(1),
        AspectValue::Bool(value) => {
            output.u16(2)?;
            write_bool(output, *value)
        }
        AspectValue::Int8(value) => tagged(output, 3, |output| output.i8(*value)),
        AspectValue::Int16(value) => tagged(output, 4, |output| output.i16(*value)),
        AspectValue::Int32(value) => tagged(output, 5, |output| output.i32(*value)),
        AspectValue::Int64(value) => tagged(output, 6, |output| output.i64(*value)),
        AspectValue::UInt8(value) => tagged(output, 7, |output| output.u8(*value)),
        AspectValue::UInt16(value) => tagged(output, 8, |output| output.u16(*value)),
        AspectValue::UInt32(value) => tagged(output, 9, |output| output.u32(*value)),
        AspectValue::UInt64(value) => tagged(output, 10, |output| output.u64(*value)),
        AspectValue::Float32(value) => tagged(output, 11, |output| output.u32(value.bits())),
        AspectValue::Float64(value) => tagged(output, 12, |output| output.u64(value.bits())),
        AspectValue::Decimal(value) => tagged(output, 13, |output| output.text(value.as_str())),
        AspectValue::BigInt(value) => tagged(output, 14, |output| output.text(value.as_str())),
        AspectValue::Rational(value) => {
            output.u16(15)?;
            output.text(value.numerator.as_str())?;
            output.text(value.denominator.as_str())
        }
        AspectValue::String(InternedString::Raw(value)) => {
            tagged(output, 16, |output| output.text(value))
        }
        AspectValue::String(InternedString::Symbol(value)) => {
            tagged(output, 17, |output| output.u32(value.0))
        }
        AspectValue::Bytes(value) => tagged(output, 18, |output| output.u64(value.0)),
        AspectValue::Uuid(value) => tagged(output, 19, |output| output.raw_bytes(value)),
        AspectValue::Date(value) => {
            tagged(output, 20, |output| output.i32(value.days_from_unix_epoch))
        }
        AspectValue::Time(value) => {
            tagged(output, 21, |output| output.u64(value.nanos_since_midnight))
        }
        AspectValue::Timestamp(value) => tagged(output, 22, |output| {
            output.i64(value.micros_since_unix_epoch)
        }),
        AspectValue::TimestampTz(value) => {
            output.u16(23)?;
            output.i64(value.utc_micros_since_unix_epoch)?;
            output.i32(value.offset_minutes)
        }
        AspectValue::EntityRef(value) => {
            output.u16(24)?;
            output.u32(value.partition_id.0)?;
            output.u64(value.local_slot.0)?;
            output.u32(value.generation.0)
        }
        AspectValue::ContentRef(value) => tagged(output, 25, |output| output.u64(value.0)),
    }
}

pub(super) fn decode_aspect_value(input: &mut BinaryInput<'_>) -> Result<AspectValue, Denial> {
    Ok(match input.u16()? {
        26 => {
            let length = input.u32()? as usize;
            worth_foundational::InlineBytes::admit_length(length)
                .map_err(|_| Denial::new(Kind::InvalidRecordLength))?;
            let value = worth_foundational::InlineBytes::try_new(input.take(length)?.to_vec())
                .map_err(|_| Denial::new(Kind::InvalidRecordLength))?;
            AspectValue::InlineBytes(value)
        }
        1 => AspectValue::Null,
        2 => AspectValue::Bool(decode_bool(input)?),
        3 => AspectValue::Int8(input.i8()?),
        4 => AspectValue::Int16(input.i16()?),
        5 => AspectValue::Int32(input.i32()?),
        6 => AspectValue::Int64(input.i64()?),
        7 => AspectValue::UInt8(input.u8()?),
        8 => AspectValue::UInt16(input.u16()?),
        9 => AspectValue::UInt32(input.u32()?),
        10 => AspectValue::UInt64(input.u64()?),
        11 => AspectValue::Float32(CanonicalF32(input.u32()?)),
        12 => AspectValue::Float64(CanonicalF64(input.u64()?)),
        13 => AspectValue::Decimal(CanonicalDecimal::new(input.text()?)),
        14 => AspectValue::BigInt(CanonicalBigInt::new(input.text()?)),
        15 => AspectValue::Rational(CanonicalRational {
            numerator: CanonicalBigInt::new(input.text()?),
            denominator: CanonicalBigInt::new(input.text()?),
        }),
        16 => AspectValue::String(InternedString::Raw(input.text()?.to_owned())),
        17 => AspectValue::String(InternedString::Symbol(Symbol(input.u32()?))),
        18 => AspectValue::Bytes(ContentRefId(input.u64()?)),
        19 => AspectValue::Uuid(input.array()?),
        20 => AspectValue::Date(CanonicalDate {
            days_from_unix_epoch: input.i32()?,
        }),
        21 => AspectValue::Time(CanonicalTime {
            nanos_since_midnight: input.u64()?,
        }),
        22 => AspectValue::Timestamp(CanonicalTimestamp {
            micros_since_unix_epoch: input.i64()?,
        }),
        23 => AspectValue::TimestampTz(CanonicalTimestampTz {
            utc_micros_since_unix_epoch: input.i64()?,
            offset_minutes: input.i32()?,
        }),
        24 => AspectValue::EntityRef(EntityId {
            partition_id: PartitionId(input.u32()?),
            local_slot: LocalSlot(input.u64()?),
            generation: Generation(input.u32()?),
        }),
        25 => AspectValue::ContentRef(ContentRefId(input.u64()?)),
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    })
}

pub(super) fn write_bool(output: &mut dyn BinaryEncodingSink, value: bool) -> Result<(), Denial> {
    output.u16(if value { 1 } else { 0 })
}

pub(super) fn decode_bool(input: &mut BinaryInput<'_>) -> Result<bool, Denial> {
    match input.u16()? {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(Denial::new(Kind::InvalidBooleanEncoding)),
    }
}

fn tagged(
    output: &mut dyn BinaryEncodingSink,
    tag: u16,
    write: impl FnOnce(&mut dyn BinaryEncodingSink) -> Result<(), Denial>,
) -> Result<(), Denial> {
    output.u16(tag)?;
    write(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary_output::BinaryOutput;

    #[test]
    fn every_aspect_value_family_round_trips_exactly() {
        let values = [
            AspectValue::Null,
            AspectValue::Bool(true),
            AspectValue::Int8(-8),
            AspectValue::Int16(-16),
            AspectValue::Int32(-32),
            AspectValue::Int64(-64),
            AspectValue::UInt8(8),
            AspectValue::UInt16(16),
            AspectValue::UInt32(32),
            AspectValue::UInt64(64),
            AspectValue::Float32(CanonicalF32(0x3f80_0000)),
            AspectValue::Float64(CanonicalF64(0x3ff0_0000_0000_0000)),
            AspectValue::Decimal(CanonicalDecimal::new("-12.50")),
            AspectValue::BigInt(CanonicalBigInt::new("12345678901234567890")),
            AspectValue::Rational(
                CanonicalRational::new(CanonicalBigInt::new("-7"), CanonicalBigInt::new("9"))
                    .expect("nonzero denominator"),
            ),
            AspectValue::String(InternedString::Raw("raw".to_owned())),
            AspectValue::String(InternedString::Symbol(Symbol(17))),
            AspectValue::Bytes(ContentRefId(18)),
            AspectValue::Uuid([19; 16]),
            AspectValue::Date(CanonicalDate {
                days_from_unix_epoch: -20,
            }),
            AspectValue::Time(CanonicalTime {
                nanos_since_midnight: 21,
            }),
            AspectValue::Timestamp(CanonicalTimestamp {
                micros_since_unix_epoch: -22,
            }),
            AspectValue::TimestampTz(CanonicalTimestampTz {
                utc_micros_since_unix_epoch: -23,
                offset_minutes: 90,
            }),
            AspectValue::EntityRef(EntityId {
                partition_id: PartitionId(24),
                local_slot: LocalSlot(25),
                generation: Generation(26),
            }),
            AspectValue::ContentRef(ContentRefId(27)),
            AspectValue::InlineBytes(
                worth_foundational::InlineBytes::try_new(vec![0, 128, 255]).unwrap(),
            ),
        ];

        for value in values {
            let mut output = BinaryOutput::with_capacity(64);
            write_aspect_value(&mut output, &value).expect("value encodes");
            let bytes = output.into_bytes();
            let mut input = BinaryInput::new(&bytes);

            assert_eq!(decode_aspect_value(&mut input), Ok(value));
            assert!(input.is_finished());
        }
    }

    #[test]
    fn aspect_value_decoder_rejects_unknown_tags_and_non_boolean_values() {
        let encoded_unknown = u16::MAX.to_be_bytes();
        let mut unknown = BinaryInput::new(&encoded_unknown);
        assert_eq!(
            decode_aspect_value(&mut unknown)
                .expect_err("unknown tag must fail")
                .kind(),
            Kind::UnsupportedRecordVariant
        );

        let encoded_non_boolean = [0, 2, 0, 2];
        let mut non_boolean = BinaryInput::new(&encoded_non_boolean);
        assert_eq!(
            decode_aspect_value(&mut non_boolean)
                .expect_err("non-boolean value must fail")
                .kind(),
            Kind::InvalidBooleanEncoding
        );
    }
}
