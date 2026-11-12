//! Lossless literal threshold payloads, never cached node results.
use super::{Denial, Read, Write};
use worth_foundational::facade::{
    AspectValue, CanonicalBigInt, CanonicalDate, CanonicalDecimal, CanonicalF32, CanonicalF64,
    CanonicalRational, CanonicalTime, CanonicalTimestamp, CanonicalTimestampTz, ContentRefId,
    EntityId, InlineBytes, InternedString, Symbol,
};

pub(super) fn write(out: &mut Write<'_>, value: &AspectValue) -> Result<(), Denial> {
    let tag = match value {
        AspectValue::Null => 0,
        AspectValue::Bool(_) => 1,
        AspectValue::Int8(_) => 2,
        AspectValue::Int16(_) => 3,
        AspectValue::Int32(_) => 4,
        AspectValue::Int64(_) => 5,
        AspectValue::UInt8(_) => 6,
        AspectValue::UInt16(_) => 7,
        AspectValue::UInt32(_) => 8,
        AspectValue::UInt64(_) => 9,
        AspectValue::Float32(_) => 10,
        AspectValue::Float64(_) => 11,
        AspectValue::Decimal(_) => 12,
        AspectValue::BigInt(_) => 13,
        AspectValue::Rational(_) => 14,
        AspectValue::String(_) => 15,
        AspectValue::Uuid(_) => 16,
        AspectValue::Date(_) => 17,
        AspectValue::Time(_) => 18,
        AspectValue::Timestamp(_) => 19,
        AspectValue::TimestampTz(_) => 20,
        AspectValue::EntityRef(_) => 21,
        AspectValue::ContentRef(_) => 22,
        AspectValue::InlineBytes(_) => 23,
    };
    out.byte(tag)?;
    match value {
        AspectValue::Null => Ok(()),
        AspectValue::Bool(value) => out.boolean(*value),
        AspectValue::Int8(value) => out.number(*value as u64),
        AspectValue::Int16(value) => out.number(*value as u64),
        AspectValue::Int32(value) => out.number(*value as u64),
        AspectValue::Int64(value) => out.number(*value as u64),
        AspectValue::UInt8(value) => out.number(*value as u64),
        AspectValue::UInt16(value) => out.number(*value as u64),
        AspectValue::UInt32(value) => out.number(*value as u64),
        AspectValue::UInt64(value) => out.number(*value),
        AspectValue::Float32(value) => out.small_number(value.bits()),
        AspectValue::Float64(value) => out.number(value.bits()),
        AspectValue::Decimal(value) => out.text(value.as_str()),
        AspectValue::BigInt(value) => out.text(value.as_str()),
        AspectValue::Rational(value) => {
            out.text(value.numerator.as_str())?;
            out.text(value.denominator.as_str())
        }
        AspectValue::String(value) => match value {
            InternedString::Raw(value) => {
                out.byte(0)?;
                out.text(value)
            }
            InternedString::Stored(value) => {
                out.byte(0)?;
                out.text(value.as_str())
            }
            InternedString::Symbol(symbol) => {
                out.byte(1)?;
                out.small_number(symbol.0)
            }
        },
        AspectValue::Uuid(value) => out.fixed(value),
        AspectValue::Date(value) => out.number(value.days_from_unix_epoch as u64),
        AspectValue::Time(value) => out.number(value.nanos_since_midnight),
        AspectValue::Timestamp(value) => out.number(value.micros_since_unix_epoch as u64),
        AspectValue::TimestampTz(value) => {
            out.number(value.utc_micros_since_unix_epoch as u64)?;
            out.number(value.offset_minutes as u64)
        }
        AspectValue::EntityRef(value) => {
            out.small_number(value.partition_id.0)?;
            out.number(value.local_slot.0)?;
            out.small_number(value.generation.0)
        }
        AspectValue::ContentRef(value) => out.fixed(&value.0),
        AspectValue::InlineBytes(value) => {
            out.count(value.as_bytes().len())?;
            out.fixed(value.as_bytes())
        }
    }
}
pub(super) fn read(input: &mut Read<'_>) -> Result<AspectValue, Denial> {
    Ok(match input.byte()? {
        0 => AspectValue::Null,
        1 => AspectValue::Bool(input.boolean()?),
        2 => AspectValue::Int8(
            (input.number()? as i64)
                .try_into()
                .map_err(|_| Denial::Malformed)?,
        ),
        3 => AspectValue::Int16(
            (input.number()? as i64)
                .try_into()
                .map_err(|_| Denial::Malformed)?,
        ),
        4 => AspectValue::Int32(
            (input.number()? as i64)
                .try_into()
                .map_err(|_| Denial::Malformed)?,
        ),
        5 => AspectValue::Int64(input.number()? as i64),
        6 => AspectValue::UInt8(input.number()?.try_into().map_err(|_| Denial::Malformed)?),
        7 => AspectValue::UInt16(input.number()?.try_into().map_err(|_| Denial::Malformed)?),
        8 => AspectValue::UInt32(input.number()?.try_into().map_err(|_| Denial::Malformed)?),
        9 => AspectValue::UInt64(input.number()?),
        10 => AspectValue::Float32(CanonicalF32(input.small_number()?)),
        11 => AspectValue::Float64(CanonicalF64(input.number()?)),
        12 => AspectValue::Decimal(CanonicalDecimal::new(input.text()?)),
        13 => AspectValue::BigInt(CanonicalBigInt::new(input.text()?)),
        14 => AspectValue::Rational(CanonicalRational {
            numerator: CanonicalBigInt::new(input.text()?),
            denominator: CanonicalBigInt::new(input.text()?),
        }),
        15 => AspectValue::String(match input.byte()? {
            0 => InternedString::Raw(input.text()?),
            1 => InternedString::Symbol(Symbol(input.small_number()?)),
            _ => return Err(Denial::Malformed),
        }),
        16 => AspectValue::Uuid(input.fixed()?),
        17 => AspectValue::Date(CanonicalDate {
            days_from_unix_epoch: (input.number()? as i64)
                .try_into()
                .map_err(|_| Denial::Malformed)?,
        }),
        18 => AspectValue::Time(CanonicalTime {
            nanos_since_midnight: input.number()?,
        }),
        19 => AspectValue::Timestamp(CanonicalTimestamp {
            micros_since_unix_epoch: input.number()? as i64,
        }),
        20 => AspectValue::TimestampTz(CanonicalTimestampTz {
            utc_micros_since_unix_epoch: input.number()? as i64,
            offset_minutes: (input.number()? as i64)
                .try_into()
                .map_err(|_| Denial::Malformed)?,
        }),
        21 => AspectValue::EntityRef(EntityId::new(
            worth_foundational::facade::PartitionId(input.small_number()?),
            input.number()?,
            input.small_number()?,
        )),
        22 => AspectValue::ContentRef(ContentRefId(input.fixed()?)),
        23 => {
            let materialize = input.materialize;
            let value = input.blob()?;
            AspectValue::InlineBytes(
                InlineBytes::try_new(if materialize {
                    value.to_vec()
                } else {
                    Vec::new()
                })
                .map_err(|_| Denial::SizeLimit)?,
            )
        }
        _ => return Err(Denial::Malformed),
    })
}
