use worth_foundational::facade::{
    AspectValue, CanonicalBigInt, ContentRefId, EntityId, InternedString,
};

use super::{encode_u32, tags};
use crate::aspect_wire::{CanonicalAspectOutput, CollectCanonicalAspect};

pub(crate) fn encode_length_prefixed_aspect_value(bytes: &mut Vec<u8>, value: &AspectValue) {
    let mut value_bytes = Vec::new();
    encode_aspect_value_body(&mut value_bytes, value);
    encode_u32(bytes, value_bytes.len() as u32);
    bytes.extend_from_slice(&value_bytes);
}

pub(crate) fn encode_aspect_value(value: &AspectValue) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(encoded_aspect_value_len(value).unwrap_or(0));
    encode_aspect_value_body(&mut bytes, value);
    bytes
}

/// Exact canonical key backing length before encoding allocates its Vec.
pub(crate) fn encoded_aspect_value_len(value: &AspectValue) -> Option<usize> {
    let text = |text: &str| {
        u32::try_from(text.len()).ok()?;
        4usize.checked_add(text.len())
    };
    match value {
        AspectValue::InlineBytes(value) => Some(5 + value.as_bytes().len()),
        AspectValue::Null => Some(1),
        AspectValue::Bool(_) | AspectValue::Int8(_) | AspectValue::UInt8(_) => Some(2),
        AspectValue::Int16(_) | AspectValue::UInt16(_) => Some(3),
        AspectValue::Int32(_)
        | AspectValue::UInt32(_)
        | AspectValue::Float32(_)
        | AspectValue::Date(_) => Some(5),
        AspectValue::Int64(_)
        | AspectValue::UInt64(_)
        | AspectValue::Float64(_)
        | AspectValue::Time(_)
        | AspectValue::Timestamp(_) => Some(9),
        AspectValue::ContentRef(_) => Some(33),
        AspectValue::String(InternedString::Raw(value)) => 2usize.checked_add(text(value)?),
        AspectValue::String(InternedString::Stored(value)) => {
            2usize.checked_add(text(value.as_str())?)
        }
        AspectValue::String(InternedString::Symbol(_)) => Some(6),
        AspectValue::Decimal(value) => 1usize.checked_add(text(value.as_str())?),
        AspectValue::BigInt(value) => 1usize.checked_add(text(value.as_str())?),
        AspectValue::Rational(value) => 1usize
            .checked_add(text(value.numerator.as_str())?)?
            .checked_add(text(value.denominator.as_str())?),
        AspectValue::Uuid(_) | AspectValue::EntityRef(_) => Some(17),
        AspectValue::TimestampTz(_) => Some(13),
    }
}

fn encode_aspect_value_body(bytes: &mut Vec<u8>, value: &AspectValue) {
    match write_aspect_value(&mut CollectCanonicalAspect(bytes), value) {
        Ok(()) => {}
        Err(never) => match never {},
    }
}

pub(crate) fn write_aspect_value<O: CanonicalAspectOutput>(
    bytes: &mut O,
    value: &AspectValue,
) -> Result<(), O::Error> {
    match value {
        AspectValue::InlineBytes(value) => {
            bytes.byte(tags::INLINE_BYTES)?;
            bytes.length(value.as_bytes().len())?;
            bytes.bytes(value.as_bytes())?;
        }
        AspectValue::Null => bytes.byte(tags::NULL)?,
        AspectValue::Bool(value) => {
            bytes.byte(tags::BOOL)?;
            bytes.byte(u8::from(*value))?;
        }
        AspectValue::Int8(value) => {
            bytes.byte(tags::INT8)?;
            bytes.byte(*value as u8)?;
        }
        AspectValue::Int16(value) => {
            bytes.byte(tags::INT16)?;
            bytes.bytes(&value.to_le_bytes())?;
        }
        AspectValue::Int32(value) => {
            bytes.byte(tags::INT32)?;
            bytes.bytes(&value.to_le_bytes())?;
        }
        AspectValue::Int64(value) => {
            bytes.byte(tags::INT64)?;
            bytes.bytes(&value.to_le_bytes())?;
        }
        AspectValue::UInt8(value) => {
            bytes.byte(tags::UINT8)?;
            bytes.byte(*value)?;
        }
        AspectValue::UInt16(value) => {
            bytes.byte(tags::UINT16)?;
            bytes.bytes(&value.to_le_bytes())?;
        }
        AspectValue::UInt32(value) => {
            bytes.byte(tags::UINT32)?;
            bytes.bytes(&value.to_le_bytes())?;
        }
        AspectValue::UInt64(value) => {
            bytes.byte(tags::UINT64)?;
            bytes.bytes(&value.to_le_bytes())?;
        }
        AspectValue::Float32(value) => {
            bytes.byte(tags::FLOAT32)?;
            bytes.bytes(&value.bits().to_le_bytes())?;
        }
        AspectValue::Float64(value) => {
            bytes.byte(tags::FLOAT64)?;
            bytes.bytes(&value.bits().to_le_bytes())?;
        }
        AspectValue::String(value) => {
            bytes.byte(tags::STRING)?;
            encode_interned_string(bytes, value)?;
        }
        AspectValue::Decimal(value) => {
            bytes.byte(tags::DECIMAL)?;
            bytes.text(value.as_str())?;
        }
        AspectValue::BigInt(value) => {
            bytes.byte(tags::BIG_INT)?;
            encode_big_int(bytes, value)?;
        }
        AspectValue::Rational(value) => {
            bytes.byte(tags::RATIONAL)?;
            encode_big_int(bytes, &value.numerator)?;
            encode_big_int(bytes, &value.denominator)?;
        }
        AspectValue::Uuid(value) => {
            bytes.byte(tags::UUID)?;
            bytes.bytes(value)?;
        }
        AspectValue::Date(value) => {
            bytes.byte(tags::DATE)?;
            bytes.bytes(&value.days_from_unix_epoch.to_le_bytes())?;
        }
        AspectValue::Time(value) => {
            bytes.byte(tags::TIME)?;
            bytes.bytes(&value.nanos_since_midnight.to_le_bytes())?;
        }
        AspectValue::Timestamp(value) => {
            bytes.byte(tags::TIMESTAMP)?;
            bytes.bytes(&value.micros_since_unix_epoch.to_le_bytes())?;
        }
        AspectValue::TimestampTz(value) => {
            bytes.byte(tags::TIMESTAMP_TZ)?;
            bytes.bytes(&value.utc_micros_since_unix_epoch.to_le_bytes())?;
            bytes.bytes(&value.offset_minutes.to_le_bytes())?;
        }
        AspectValue::EntityRef(value) => {
            bytes.byte(tags::ENTITY_REF)?;
            encode_entity_id(bytes, *value)?;
        }
        AspectValue::ContentRef(value) => {
            bytes.byte(tags::CONTENT_REF)?;
            encode_content_ref(bytes, *value)?;
        }
    }
    Ok(())
}

fn encode_interned_string<O: CanonicalAspectOutput>(
    bytes: &mut O,
    value: &InternedString,
) -> Result<(), O::Error> {
    match value {
        InternedString::Raw(text) => {
            bytes.byte(tags::RAW_STRING)?;
            bytes.text(text)?;
        }
        InternedString::Stored(text) => {
            bytes.byte(tags::RAW_STRING)?;
            bytes.text(text.as_str())?;
        }
        InternedString::Symbol(symbol) => {
            bytes.byte(tags::SYMBOL_STRING)?;
            bytes.bytes(&symbol.0.to_le_bytes())?;
        }
    }
    Ok(())
}

fn encode_big_int<O: CanonicalAspectOutput>(
    bytes: &mut O,
    value: &CanonicalBigInt,
) -> Result<(), O::Error> {
    bytes.text(value.as_str())?;
    Ok(())
}

fn encode_content_ref<O: CanonicalAspectOutput>(
    bytes: &mut O,
    value: ContentRefId,
) -> Result<(), O::Error> {
    bytes.bytes(&value.0)?;
    Ok(())
}

fn encode_entity_id<O: CanonicalAspectOutput>(
    bytes: &mut O,
    value: EntityId,
) -> Result<(), O::Error> {
    bytes.bytes(&value.partition_id.0.to_le_bytes())?;
    bytes.bytes(&value.local_slot.0.to_le_bytes())?;
    bytes.bytes(&value.generation.0.to_le_bytes())?;
    Ok(())
}
